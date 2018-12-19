#![feature(test)]
extern crate test;

use std::{
    env,
    fs,
    iter,
    num::ParseIntError,
};

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Color {
    Red,
    Blue,
}

struct Goal {
    position: (i32, i32),
    color: Color,
}

struct Data {
    size: (usize, usize),
    data: Vec<bool>,
    goals: Vec<Goal>,
}

impl Data {
    fn get(&self, position: (i32, i32)) -> bool {
        if position.0 < 0 || position.0 as usize >= self.size.0 || position.1 < 0 || position.1 as usize >= self.size.1 {
            false
        } else {
            self.data[position.0 as usize + position.1 as usize * self.size.0]
        }
    }

    fn is_solved_by(&self, state: &State) -> bool {
        self.goals.iter().all(|g| state.actors.iter().any(|a| a.position == g.position && a.color == g.color))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

impl Direction {
    fn horizontal(&self) -> i32 {
        match self {
            Direction::Right => 1,
            Direction::Left => -1,
            Direction::Up | Direction::Down => 0,
        }
    }

    fn vertical(&self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
            Direction::Right | Direction::Left => 0,
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Actor {
    position: (i32, i32),
    color: Color,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
struct State {
    actors: Vec<Actor>,
}

impl State {
    fn transition(&self, data: &Data, direction: &Direction) -> State {
        let mut result = self.clone();

        for actor in result.actors.iter_mut() {
            let next_position = match actor.color {
                Color::Red => (actor.position.0 + direction.horizontal(), actor.position.1 + direction.vertical()),
                Color::Blue => (actor.position.0 - direction.horizontal(), actor.position.1 - direction.vertical()),
            };

            if data.get(next_position) {
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
    type Transitions = Vec<(Self::Action, brutalize::Transition<Self>)>;
    type Heuristic = usize;

    fn transitions(&self, data: &Self::Data) -> Self::Transitions {
        let mut result = Vec::new();
        for direction in [Direction::Right, Direction::Up, Direction::Left, Direction::Down].iter() {
            let state = self.transition(data, direction);
            if data.is_solved_by(&state) {
                result.push((*direction, brutalize::Transition::Success));
            } else {
                result.push((*direction, brutalize::Transition::Indeterminate(state)));
            }
        }
        result
    }

    fn heuristic(&self, _data: &Self::Data) -> Self::Heuristic {
        0
    }
}

#[derive(Debug)]
enum ParseError {
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

fn parse(s: &str) -> Result<(State, Data), ParseError> {
    let width = s.lines().next().ok_or(ParseError::NoRows)?.len();
    let height = s.lines().enumerate().find(|(_, l)| l.len() == 0).ok_or(ParseError::NoLineBreakAfterRows)?.0;

    let mut data = iter::repeat(false).take(width * height).collect::<Vec<_>>();
    let mut goals = Vec::new();
    let mut actors = Vec::new();

    let mut lines = s.lines().enumerate();
    for _ in 0..height {
        let (line_number, line) = lines.next().unwrap();
        let y = height - line_number - 1;

        if line.len() != width {
            return Err(ParseError::UnevenRows {
                line_number: line_number + 1,
                data_width: width,
                line_width: line.len(),
            });
        }

        for (x, c) in line.chars().enumerate() {
            let value = match c {
                '.' => true,
                ' ' => false,
                'r' => {
                    goals.push(Goal {
                        position: (x as i32, y as i32),
                        color: Color::Red,
                    });
                    true
                },
                'b' => {
                    goals.push(Goal {
                        position: (x as i32, y as i32),
                        color: Color::Blue,
                    });
                    true
                },
                _ => return Err(ParseError::UnexpectedCharacter {
                    line_number: y + 1,
                    column_number: x + 1,
                    character: c,
                }),
            };
            data[x + y * width] = value;
        }
    }

    lines.next();

    while let Some((y, line)) = lines.next() {
        let mut pieces = line.split(' ');
        let color = match pieces.next().ok_or(ParseError::EmptyActorDefinition { line_number: y })? {
            "R" => Color::Red,
            "B" => Color::Blue,
            c => return Err(ParseError::InvalidActorColor { line_number: y + 1, color: c.to_string() }),
        };
        let actor_x = pieces.next().ok_or(ParseError::MissingActorX { line_number: y + 1 })?.parse::<i32>().map_err(|e| ParseError::InvalidActorX { line_number: y + 1, parse_error: e })?;
        let actor_y = pieces.next().ok_or(ParseError::MissingActorY { line_number: y + 1 })?.parse::<i32>().map_err(|e| ParseError::InvalidActorY { line_number: y + 1, parse_error: e })?;

        actors.push(Actor {
            position: (actor_x, actor_y),
            color,
        });
    }

    Ok((
        State {
            actors,
        },
        Data {
            size: (width, height),
            data,
            goals,
        },
    ))
}

fn main() {
    if let Some(path) = env::args().nth(1) {
        let contents = fs::read_to_string(path)
            .expect("Unable to read file");

        let (initial_state, data) = parse(contents.as_str()).expect("Failed to parse puzzle");
        println!("{:?}", brutalize::solve(initial_state.clone(), &data));
    } else {
        panic!("Usage: ./anima_solver <path>");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

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
        const SPIRAL: &str = ".....\n.   .\n... .\n    .\nr....\n\nR 2 2";

        let (initial_state, data) = parse(SPIRAL).unwrap();

        solve_validate(initial_state, &data, Some(16));
    }

    #[test]
    fn solve_deadlock() {
        let data = Data {
            size: (3, 3),
            data: vec![
                false, true, false,
                true, true, true,
                false, true, false,
            ],
            goals: vec![
                Goal { position: (0, 1), color: Color::Blue },
                Goal { position: (1, 0), color: Color::Blue },
                Goal { position: (1, 1), color: Color::Red },
            ],
        };
        let initial_state = State {
            actors: vec![
                Actor { position: (1, 2), color: Color::Blue },
                Actor { position: (2, 1), color: Color::Blue },
                Actor { position: (1, 1), color: Color::Red },
            ]
        };

        solve_validate(initial_state, &data, Some(6));
    }

    #[test]
    fn solve_square_dance() {
        let data = Data {
            size: (5, 5),
            data: vec![
                false, true, true, true, true,
                true, true, true, true, true,
                true, true, false, true, true,
                true, true, true, true, true,
                true, true, true, true, false,
            ],
            goals: vec![
                Goal { position: (1, 1), color: Color::Red },
                Goal { position: (3, 1), color: Color::Red },
                Goal { position: (1, 3), color: Color::Red },
                Goal { position: (3, 3), color: Color::Red },
            ],
        };
        let initial_state = State {
            actors: vec![
                Actor { position: (2, 1), color: Color::Red },
                Actor { position: (1, 2), color: Color::Red },
                Actor { position: (2, 3), color: Color::Red },
                Actor { position: (3, 2), color: Color::Red },
            ]
        };

        solve_validate(initial_state, &data, Some(12));
    }

    #[test]
    fn solve_close_quarters() {
        let data = Data {
            size: (4, 3),
            data: vec![
                false, true, true, false,
                true, true, true, true,
                true, true, true, true,
            ],
            goals: vec![
                Goal { position: (1, 1), color: Color::Blue },
                Goal { position: (1, 2), color: Color::Red },
                Goal { position: (2, 1), color: Color::Red },
                Goal { position: (2, 2), color: Color::Blue },
            ],
        };
        let initial_state = State {
            actors: vec![
                Actor { position: (0, 1), color: Color::Red },
                Actor { position: (0, 2), color: Color::Blue },
                Actor { position: (3, 1), color: Color::Blue },
                Actor { position: (3, 2), color: Color::Red },
            ]
        };

        solve_validate(initial_state, &data, Some(11));
    }

    #[bench]
    fn solve_free_radical(b: &mut Bencher) {
        const FREE_RADICAL: &str = " ....\n..r..\n.r.r.\n..r..\n.... \n\nR 1 3\nR 1 1\nB 2 2\nR 3 1\nR 3 3";

        let (initial_state, data) = parse(FREE_RADICAL).unwrap();

        b.iter(|| brutalize::solve(initial_state.clone(), &data));
    }
}
