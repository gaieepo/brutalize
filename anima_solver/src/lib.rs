use arrayvec::ArrayVec;
use core::{fmt, num::ParseIntError};
use solver_common::{Direction, Vec2};

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Color {
    Red,
    Blue,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Passable,
    Impassable,
}

struct Goal {
    position: Vec2,
    color: Color,
}

pub struct Data {
    size: Vec2,
    tiles: Vec<Tile>,
    goals: Vec<Goal>,
}

impl Data {
    fn tile(&self, position: Vec2) -> Tile {
        if position.x < 0
            || position.x >= self.size.x
            || position.y < 0
            || position.y >= self.size.y
        {
            Tile::Impassable
        } else {
            self.tiles[(position.x + position.y * self.size.x) as usize]
        }
    }

    fn is_solved_by(&self, state: &State) -> bool {
        self.goals.iter().all(|g| {
            state
                .actors
                .iter()
                .any(|a| a.position == g.position && a.color == g.color)
        })
    }
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Actor {
    position: Vec2,
    color: Color,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct State {
    actors: ArrayVec<Actor, 8>,
}

impl State {
    fn transition(&self, data: &Data, direction: &Direction) -> State {
        let mut result = self.clone();

        for actor in result.actors.iter_mut() {
            let next_position = match actor.color {
                Color::Red => actor.position + direction.to_vec2(),
                Color::Blue => actor.position - direction.to_vec2(),
            };

            if data.tile(next_position) == Tile::Passable {
                actor.position = next_position;
            }
        }

        let mut done = false;
        while !done {
            done = true;
            for i in 0..result.actors.len() {
                for j in i + 1..result.actors.len() {
                    if result.actors[i].position == result.actors[j].position {
                        result.actors[i].position = self.actors[i].position;
                        result.actors[j].position = self.actors[j].position;
                        done = false;
                    }
                }
            }
        }

        result.actors.sort_unstable();

        result
    }
}

impl brutalize::State for State {
    type Data = Data;
    type Action = Direction;
    type Transitions = ArrayVec<(Self::Action, brutalize::Transition<Self>), 4>;
    type Heuristic = usize;

    fn transitions(&self, data: &Self::Data) -> Self::Transitions {
        let mut result = ArrayVec::new();
        for direction in [
            Direction::Right,
            Direction::Up,
            Direction::Left,
            Direction::Down,
        ]
        .iter()
        {
            let state = self.transition(data, direction);
            if data.is_solved_by(&state) {
                result.push((*direction, brutalize::Transition::Success));
            } else {
                result.push((*direction, brutalize::Transition::Indeterminate(state)));
            }
        }
        result
    }

    fn heuristic(&self, data: &Self::Data) -> Self::Heuristic {
        let mut max_distance = 0;

        for goal in data.goals.iter() {
            let mut min_distance = usize::MAX;
            for actor in self.actors.iter() {
                let d = (goal.position - actor.position).abs();
                min_distance = usize::min(min_distance, (d.x + d.y) as usize);
            }
            max_distance = usize::max(max_distance, min_distance);
        }

        max_distance
    }
}

#[derive(Debug)]
pub enum ParseError {
    NoRows,
    NoLineBreakAfterRows,
    UnevenRows {
        line_number: usize,
        data_width: usize,
        line_width: usize,
    },
    UnexpectedCharacter {
        line_number: usize,
        column_number: usize,
        character: char,
    },
    EmptyActorDefinition {
        line_number: usize,
    },
    InvalidActorColor {
        line_number: usize,
        color: String,
    },
    MissingActorX {
        line_number: usize,
    },
    MissingActorY {
        line_number: usize,
    },
    InvalidActorX {
        line_number: usize,
        parse_error: ParseIntError,
    },
    InvalidActorY {
        line_number: usize,
        parse_error: ParseIntError,
    },
}

impl brutalize_cli::State for State {
    type ParseError = ParseError;

    fn parse(s: &str) -> Result<(State, Data), ParseError> {
        let size_x = s.lines().next().ok_or(ParseError::NoRows)?.len();
        let size_y = s
            .lines()
            .enumerate()
            .find(|(_, l)| l.is_empty())
            .ok_or(ParseError::NoLineBreakAfterRows)?
            .0;

        let mut tiles = vec![Tile::Impassable; size_x * size_y as usize];
        let mut goals = Vec::new();
        let mut actors = ArrayVec::new();

        let mut lines = s.lines().enumerate();
        for y in (0..size_y).rev() {
            let (line_number, line) = lines.next().unwrap();

            if line.len() != size_x {
                return Err(ParseError::UnevenRows {
                    line_number,
                    data_width: size_x,
                    line_width: line.len(),
                });
            }

            for (x, c) in line.chars().enumerate() {
                let tile = match c {
                    '.' => Ok(Tile::Passable),
                    ' ' => Ok(Tile::Impassable),
                    'r' => {
                        goals.push(Goal {
                            position: Vec2::new(x as i32, y as i32),
                            color: Color::Red,
                        });
                        Ok(Tile::Passable)
                    }
                    'b' => {
                        goals.push(Goal {
                            position: Vec2::new(x as i32, y as i32),
                            color: Color::Blue,
                        });
                        Ok(Tile::Passable)
                    }
                    _ => Err(ParseError::UnexpectedCharacter {
                        line_number,
                        column_number: x + 1,
                        character: c,
                    }),
                }?;
                tiles[x + y * size_x] = tile;
            }
        }

        lines.next();

        for (line_number, line) in lines {
            let mut pieces = line.split(' ');
            let color = match pieces
                .next()
                .ok_or(ParseError::EmptyActorDefinition { line_number })?
            {
                "R" => Color::Red,
                "B" => Color::Blue,
                c => {
                    return Err(ParseError::InvalidActorColor {
                        line_number,
                        color: c.to_string(),
                    })
                }
            };
            let actor_x = pieces
                .next()
                .ok_or(ParseError::MissingActorX { line_number })?
                .parse()
                .map_err(|parse_error| ParseError::InvalidActorX {
                    line_number,
                    parse_error,
                })?;
            let actor_y = pieces
                .next()
                .ok_or(ParseError::MissingActorY { line_number })?
                .parse()
                .map_err(|parse_error| ParseError::InvalidActorY {
                    line_number,
                    parse_error,
                })?;

            actors.push(Actor {
                position: Vec2::new(actor_x, actor_y),
                color,
            });
        }

        Ok((
            State { actors },
            Data {
                size: Vec2::new(size_x as i32, size_y as i32),
                tiles,
                goals,
            },
        ))
    }

    fn display(&self, data: &Self::Data, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board_width = data.size.x + 2;
        let board_height = data.size.y + 2;
        let size = board_width * board_height;
        let mut board = vec![' '; size as usize];

        for y in 0..board_height {
            for x in 0..board_width {
                let index = x + y * board_width;
                let position = Vec2::new(x, y);
                board[index as usize] = match data.tile(position) {
                    Tile::Passable => '.',
                    Tile::Impassable => ' ',
                };
            }
        }

        for goal in data.goals.iter() {
            let index = goal.position.x + goal.position.y * board_width;
            board[index as usize] = match goal.color {
                Color::Red => 'r',
                Color::Blue => 'b',
            };
        }

        for actor in self.actors.iter() {
            let index = actor.position.x + actor.position.y * board_width;
            board[index as usize] = match actor.color {
                Color::Red => 'R',
                Color::Blue => 'B',
            };
        }

        for y in (0..board_height).rev() {
            let begin = y * board_width;
            let end = begin + board_width;
            for c in &board[begin as usize..end as usize] {
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solve_validate(initial_state: State, data: &Data, length: Option<usize>) {
        let solution = brutalize::solve(initial_state.clone(), data);

        if let Some(l) = length {
            assert_ne!(solution, None);
            let solution = solution.unwrap();
            assert_eq!(solution.len(), l);

            let mut state = initial_state.clone();
            for direction in solution.iter() {
                state = state.transition(data, direction);
            }

            assert!(data.is_solved_by(&state));
        } else {
            assert_eq!(solution, None);
        }
    }

    #[test]
    fn parse_solve_spiral() {
        const PUZZLE: &str = ".....\n.   .\n... .\n    .\nr....\n\nR 2 2";

        let (initial_state, data) = <State as brutalize_cli::State>::parse(PUZZLE).unwrap();
        solve_validate(initial_state, &data, Some(16));
    }

    #[test]
    fn solve_deadlock() {
        const PUZZLE: &str = " . \nbr.\n b \n\nR 1 1\nB 2 1\nB 1 2";

        let (initial_state, data) = <State as brutalize_cli::State>::parse(PUZZLE).unwrap();
        solve_validate(initial_state, &data, Some(6));
    }

    #[test]
    fn solve_square_dance() {
        const PUZZLE: &str = " ....\n.r.r.\n.. ..\n.r.r.\n.... \n\nR 2 1\nR 1 2\nR 3 2\nR 2 3";

        let (initial_state, data) = <State as brutalize_cli::State>::parse(PUZZLE).unwrap();
        solve_validate(initial_state, &data, Some(12));
    }

    #[test]
    fn solve_close_quarters() {
        const PUZZLE: &str = ".rb.\n.br.\n .. \n\nR 0 1\nB 0 2\nB 3 1\nR 3 2";

        let (initial_state, data) = <State as brutalize_cli::State>::parse(PUZZLE).unwrap();
        solve_validate(initial_state, &data, Some(11));
    }
}
