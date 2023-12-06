use arrayvec::ArrayVec;
use solver_common::{Direction, Vec2};
use std::{fmt, num::ParseIntError};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Empty,
    Ground,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Status {
    Solved,
    Unsolved,
    Failed,
}

pub struct Data {
    size: Vec2,
    tiles: Vec<Tile>,
    goal_position: Vec2,
}

impl Data {
    #[inline]
    fn size(&self) -> Vec2 {
        self.size
    }

    #[inline]
    fn tile(&self, position: Vec2) -> Tile {
        if position.x < 0
            || position.x >= self.size.x
            || position.y < 0
            || position.y >= self.size.y
        {
            Tile::Empty
        } else {
            let index = position.x + position.y * self.size.x;
            self.tiles[index as usize]
        }
    }

    #[inline]
    fn goal_position(&self) -> Vec2 {
        self.goal_position
    }

    #[inline]
    fn status_of(&self, state: &State) -> Status {
        if self.tile(state.player.position) == Tile::Empty {
            return Status::Failed;
        }

        let mut solved = true;
        if state.chest.position != self.goal_position() {
            solved = false;
        }
        if solved {
            Status::Solved
        } else {
            Status::Unsolved
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
struct Player {
    position: Vec2,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
struct Chest {
    position: Vec2,
}

impl Chest {
    #[inline]
    fn overlap(&self, position: Vec2) -> bool {
        position == self.position
    }

    #[inline]
    fn push(&mut self, direction: Direction) {
        self.position += direction.to_vec2();
    }
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Wall {
    position: Vec2,
}

impl Wall {
    #[inline]
    fn overlap(&self, position: Vec2) -> bool {
        position == self.position
    }

    #[inline]
    fn pull(&mut self, direction: Direction) {
        self.position += direction.to_vec2();
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct State {
    player: Player,
    chest: Chest,
    walls: ArrayVec<Wall, 32>,
}

impl State {
    #[inline]
    fn initial(start_position: Vec2, chest_position: Vec2, walls: ArrayVec<Wall, 32>) -> State {
        let mut result = State {
            player: Player {
                position: start_position,
            },
            chest: Chest {
                position: chest_position,
            },
            walls,
        };

        result.walls.sort_unstable();
        result
    }

    #[inline]
    fn try_strafe_player(&mut self, data: &Data, direction: Direction) -> bool {
        let old_player_position = self.player.position;
        let forward = direction.to_vec2();
        self.player.position += forward;

        // Try to move out of board
        if data.tile(self.player.position) == Tile::Empty {
            return false;
        }

        let backward = direction.reverse().to_vec2();
        let pull_position = old_player_position + backward;

        for wall in &mut self.walls {
            // Try to move into wall
            if wall.overlap(self.player.position) {
                return false;
            }
            // Pull wall
            if wall.overlap(pull_position) {
                wall.pull(direction);
            }
        }

        // Try to push chest
        if self.chest.overlap(self.player.position) {
            let behind_chest_position = self.player.position + forward;
            if data.tile(behind_chest_position) == Tile::Ground
                && !self
                    .walls
                    .iter()
                    .any(|wall| wall.overlap(behind_chest_position))
            {
                self.chest.push(direction);
            } else {
                return false;
            }
        }

        true
    }

    #[inline]
    fn transition(&self, data: &Data, direction: Direction) -> Option<State> {
        let mut result = self.clone();

        if !result.try_strafe_player(data, direction) {
            return None;
        }

        result.walls.sort_unstable();
        Some(result)
    }
}

impl brutalize::State for State {
    type Data = Data;
    type Action = Direction;
    type Transitions = ArrayVec<(Self::Action, brutalize::Transition<Self>), 4>;
    type Heuristic = usize;

    fn transitions(&self, data: &Data) -> Self::Transitions {
        let mut result = ArrayVec::new();
        for direction in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ]
        .iter()
        .cloned()
        {
            if let Some(state) = self.transition(data, direction) {
                match data.status_of(&state) {
                    Status::Solved => result.push((direction, brutalize::Transition::Success)),
                    Status::Unsolved => {
                        result.push((direction, brutalize::Transition::Indeterminate(state)))
                    }
                    Status::Failed => (),
                }
            }
        }
        result
    }

    fn heuristic(&self, data: &Self::Data) -> Self::Heuristic {
        let distance = (self.chest.position - data.goal_position).abs();
        distance.x as usize + distance.y as usize
    }
}

#[derive(Debug)]
pub enum ParseError {
    MissingCommand {
        line_number: usize,
    },
    InvalidCommand {
        line_number: usize,
        command: String,
    },
    PuzzleAlreadyDefined {
        line_number: usize,
    },
    MissingPuzzleSizeX {
        line_number: usize,
    },
    InvalidPuzzleSizeX {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingPuzzleSizeY {
        line_number: usize,
    },
    InvalidPuzzleSizeY {
        line_number: usize,
        parse_error: ParseIntError,
    },
    UnexpectedEndOfPuzzle {
        expected_lines: usize,
        found_lines: usize,
    },
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
    StartAlreadyDefined {
        line_number: usize,
    },
    MissingStartX {
        line_number: usize,
    },
    InvalidStartX {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingStartY {
        line_number: usize,
    },
    InvalidStartY {
        line_number: usize,
        parse_error: ParseIntError,
    },
    EndAlreadyDefined {
        line_number: usize,
    },
    MissingEndX {
        line_number: usize,
    },
    InvalidEndX {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingEndY {
        line_number: usize,
    },
    InvalidEndY {
        line_number: usize,
        parse_error: ParseIntError,
    },
    ChestAlreadyDefined {
        line_number: usize,
    },
    MissingChestX {
        line_number: usize,
    },
    InvalidChestX {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingChestY {
        line_number: usize,
    },
    InvalidChestY {
        line_number: usize,
        parse_error: ParseIntError,
    },
    WallsAlreadyDefined {
        line_number: usize,
    },
    MissingWallsCount {
        line_number: usize,
    },
    InvalidWallsCount {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingWallX {
        line_number: usize,
    },
    InvalidWallX {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingWallY {
        line_number: usize,
    },
    InvalidWallY {
        line_number: usize,
        parse_error: ParseIntError,
    },
    UnexpectedEndOfWalls {
        expected_lines: usize,
        found_lines: usize,
    },
    MissingPuzzle,
    MissingStart,
    MissingEnd,
    MissingChest,
    MissingWalls,
}

impl brutalize_cli::State for State {
    type ParseError = ParseError;

    fn parse(s: &str) -> Result<(State, Data), ParseError> {
        let mut puzzle = None;
        let mut start_pos = None;
        let mut end_pos = None;
        let mut chest_pos = None;
        let mut walls = None;

        let mut lines = s.lines().enumerate();
        while let Some((line_number, line)) = lines.next() {
            let mut pieces = line.split(' ');
            let command = pieces
                .next()
                .ok_or(ParseError::MissingCommand { line_number })?;
            match command {
                "puzzle" => {
                    if puzzle.is_some() {
                        return Err(ParseError::PuzzleAlreadyDefined { line_number });
                    }

                    let size_x = pieces
                        .next()
                        .ok_or(ParseError::MissingPuzzleSizeX { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidPuzzleSizeX {
                            line_number,
                            parse_error,
                        })?;
                    let size_y = pieces
                        .next()
                        .ok_or(ParseError::MissingPuzzleSizeY { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidPuzzleSizeY {
                            line_number,
                            parse_error,
                        })?;
                    let mut tiles = vec![Tile::Empty; size_x * size_y];

                    for y in (0..size_y).rev() {
                        let (line_number, line) =
                            lines.next().ok_or(ParseError::UnexpectedEndOfPuzzle {
                                expected_lines: size_y,
                                found_lines: y,
                            })?;

                        if line.len() != size_x {
                            return Err(ParseError::UnevenRows {
                                line_number,
                                data_width: size_x,
                                line_width: line.len(),
                            });
                        }

                        for (x, c) in line.chars().enumerate() {
                            let tile = match c {
                                '_' => Ok(Tile::Empty),
                                '.' => Ok(Tile::Ground),
                                _ => Err(ParseError::UnexpectedCharacter {
                                    line_number,
                                    column_number: x,
                                    character: c,
                                }),
                            }?;
                            tiles[x + y * size_x] = tile
                        }
                    }

                    puzzle = Some((Vec2::new(size_x as i32, size_y as i32), tiles));
                }
                "start" => {
                    if start_pos.is_some() {
                        return Err(ParseError::StartAlreadyDefined { line_number });
                    }

                    let start_x = pieces
                        .next()
                        .ok_or(ParseError::MissingStartX { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidStartX {
                            line_number,
                            parse_error,
                        })?;
                    let start_y = pieces
                        .next()
                        .ok_or(ParseError::MissingStartY { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidStartY {
                            line_number,
                            parse_error,
                        })?;

                    start_pos = Some(Vec2::new(start_x, start_y))
                }
                "end" => {
                    if end_pos.is_some() {
                        return Err(ParseError::EndAlreadyDefined { line_number });
                    }
                    let end_x = pieces
                        .next()
                        .ok_or(ParseError::MissingEndX { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidEndX {
                            line_number,
                            parse_error,
                        })?;
                    let end_y = pieces
                        .next()
                        .ok_or(ParseError::MissingEndY { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidEndY {
                            line_number,
                            parse_error,
                        })?;

                    end_pos = Some(Vec2::new(end_x, end_y))
                }
                "chest" => {
                    if chest_pos.is_some() {
                        return Err(ParseError::ChestAlreadyDefined { line_number });
                    }
                    let chest_x = pieces
                        .next()
                        .ok_or(ParseError::MissingChestX { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidChestX {
                            line_number,
                            parse_error,
                        })?;
                    let chest_y = pieces
                        .next()
                        .ok_or(ParseError::MissingChestY { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidChestY {
                            line_number,
                            parse_error,
                        })?;

                    chest_pos = Some(Vec2::new(chest_x, chest_y))
                }
                "walls" => {
                    if walls.is_some() {
                        return Err(ParseError::WallsAlreadyDefined { line_number });
                    }
                    let size = pieces
                        .next()
                        .ok_or(ParseError::MissingWallsCount { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidWallsCount {
                            line_number,
                            parse_error,
                        })?;

                    let mut read_walls = ArrayVec::new();
                    for i in 0..size {
                        let (line_number, line) =
                            lines.next().ok_or(ParseError::UnexpectedEndOfWalls {
                                expected_lines: size,
                                found_lines: i,
                            })?;

                        let mut pieces = line.split(' ');
                        let x = pieces
                            .next()
                            .ok_or(ParseError::MissingWallX { line_number })?
                            .parse()
                            .map_err(|parse_error| ParseError::InvalidWallX {
                                line_number,
                                parse_error,
                            })?;
                        let y = pieces
                            .next()
                            .ok_or(ParseError::MissingWallY { line_number })?
                            .parse()
                            .map_err(|parse_error| ParseError::InvalidWallY {
                                line_number,
                                parse_error,
                            })?;

                        read_walls.push(Wall {
                            position: Vec2::new(x, y),
                        });
                    }

                    walls = Some(read_walls);
                }
                command => {
                    return Err(ParseError::InvalidCommand {
                        line_number,
                        command: command.to_string(),
                    });
                }
            }
        }

        let (size, tiles) = puzzle.ok_or(ParseError::MissingPuzzle)?;
        let start_pos = start_pos.ok_or(ParseError::MissingStart)?;
        let end_pos = end_pos.ok_or(ParseError::MissingEnd)?;
        let chest_pos = chest_pos.ok_or(ParseError::MissingChest)?;
        let walls = walls.ok_or(ParseError::MissingWalls)?;

        // Log
        println!("Parsed data:");
        println!("Size: {:?} x {:?}", size.x, size.y);
        println!("Tiles:");
        for y in 0..size.y {
            for x in 0..size.x {
                let index = (x + y * size.x) as usize;
                print!("{:?} ", tiles[index]);
            }
            println!();
        }
        println!("Start position: {:?}", start_pos);
        println!("Goal position: {:?}", end_pos);
        println!("Chest position: {:?}", chest_pos);
        println!("Walls:");
        for wall in &walls {
            println!("Position: {:?}", wall.position);
        }

        let data = Data {
            size,
            tiles,
            goal_position: end_pos,
        };
        Ok((State::initial(start_pos, chest_pos, walls), data))
    }

    fn display(&self, data: &Self::Data, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board_width = data.size().x + 2;
        let board_height = data.size().y + 2;
        let size = board_width * board_height;
        let mut board = vec![' '; size as usize];

        for y in 0..board_height {
            for x in 0..board_width {
                let index = x + y * board_width;
                board[index as usize] = match data.tile(Vec2::new(x - 1, y - 1)) {
                    Tile::Empty => ' ',
                    Tile::Ground => '.',
                }
            }
        }

        // Add goal to the board
        let index = (data.goal_position.x + 1) + (data.goal_position.y + 1) * board_width;
        board[index as usize] = '*';

        for wall in self.walls.iter() {
            let index = (wall.position.x + 1) + (wall.position.y + 1) * board_width;
            board[index as usize] = '#';
        }

        // Add chest and player to the board
        let index = (self.chest.position.x + 1) + (self.chest.position.y + 1) * board_width;
        board[index as usize] = 'X';
        let index = (self.player.position.x + 1) + (self.player.position.y + 1) * board_width;
        board[index as usize] = 'P';

        for y in (0..board_height).rev() {
            let begin = y * board_width;
            let end = (y + 1) * board_width;
            for c in &board[begin as usize..end as usize] {
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
