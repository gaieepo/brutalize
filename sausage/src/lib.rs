use arrayvec::ArrayVec;
use solver_common::{Direction, ParseDirectionError, Vec2};
use std::{fmt, num::ParseIntError, str::FromStr};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Empty,
    Ground,
    Grill,
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
    goal_orientation: Direction,
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
    fn goal_orientation(&self) -> Direction {
        self.goal_orientation
    }

    #[inline]
    fn status_of(&self, state: &State) -> Status {
        if self.tile(state.player.position) == Tile::Empty {
            return Status::Failed;
        }

        let mut solved = true;
        for sausage in state.sausages.iter() {
            if self.tile(sausage.position) == Tile::Empty
                && self.tile(sausage.end_position()) == Tile::Empty
            {
                return Status::Failed;
            }
            for cooked in &sausage.cooked {
                match cooked {
                    Cooked::Uncooked => solved = false,
                    Cooked::Cooked => (),
                    Cooked::Burned => return Status::Failed,
                }
            }
        }

        if state.player.position != self.goal_position()
            || state.player.orientation != self.goal_orientation()
        {
            solved = false
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
    orientation: Direction,
}

impl Player {
    #[inline]
    fn fork_position(&self) -> Vec2 {
        self.position + self.orientation.to_vec2()
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum SausageOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct ParseSausageOrientationError(String);

impl FromStr for SausageOrientation {
    type Err = ParseSausageOrientationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "horizontal" => Ok(SausageOrientation::Horizontal),
            "vertical" => Ok(SausageOrientation::Vertical),
            _ => Err(ParseSausageOrientationError(s.to_string())),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Cooked {
    Uncooked,
    Cooked,
    Burned,
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Sausage {
    position: Vec2,
    orientation: SausageOrientation,
    cooked: [Cooked; 4],
}

impl Sausage {
    #[inline]
    fn new(position: Vec2, orientation: SausageOrientation) -> Sausage {
        Sausage {
            position,
            orientation,
            cooked: [Cooked::Uncooked; 4],
        }
    }

    #[inline]
    fn roll(&mut self) {
        self.cooked.swap(0, 2);
        self.cooked.swap(1, 3);
    }

    #[inline]
    fn end_position(&self) -> Vec2 {
        match self.orientation {
            SausageOrientation::Horizontal => self.position + Direction::Right.to_vec2(),
            SausageOrientation::Vertical => self.position + Direction::Up.to_vec2(),
        }
    }

    #[inline]
    fn overlap(&self, position: Vec2) -> bool {
        (position == self.position) || (position == self.end_position())
    }

    #[inline]
    fn overlap_player(&self, player: &Player) -> bool {
        self.overlap(player.position) || self.overlap(player.fork_position())
    }

    #[inline]
    fn overlap_sausage(&self, sausage: &Sausage) -> bool {
        self.overlap(sausage.position) || self.overlap(sausage.end_position())
    }

    #[inline]
    fn cook(&mut self, index: usize) {
        self.cooked[index] = match self.cooked[index] {
            Cooked::Uncooked => Cooked::Cooked,
            _ => Cooked::Burned,
        };
    }

    #[inline]
    fn push(&mut self, direction: Direction, data: &Data) {
        self.position += direction.to_vec2();
        let rolled = match self.orientation {
            SausageOrientation::Horizontal => {
                direction == Direction::Up || direction == Direction::Down
            }
            SausageOrientation::Vertical => {
                direction == Direction::Left || direction == Direction::Right
            }
        };

        if rolled {
            self.roll();
        }

        let end_position = self.end_position();
        if data.tile(self.position) == Tile::Grill {
            self.cook(2);
        }
        if data.tile(end_position) == Tile::Grill {
            self.cook(3);
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct State {
    player: Player,
    sausages: ArrayVec<Sausage, 4>,
}

impl State {
    #[inline]
    fn initial(data: &Data, sausages: ArrayVec<Sausage, 4>) -> State {
        let mut result = State {
            player: Player {
                position: data.goal_position(),
                orientation: data.goal_orientation(),
            },
            sausages,
        };

        result.sausages.sort_unstable();
        result
    }

    #[inline]
    fn push_sausage(&mut self, sausage_index: usize, direction: Direction, data: &Data) {
        self.sausages[sausage_index].push(direction, data);

        for i in 0..self.sausages.len() {
            if i != sausage_index && self.sausages[sausage_index].overlap_sausage(&self.sausages[i])
            {
                self.push_sausage(i, direction, data);
            }
        }
    }

    #[inline]
    fn transition(&self, data: &Data, direction: &Direction) -> State {
        let mut result = self.clone();

        match direction.relative_to(self.player.orientation) {
            Direction::Up => {
                // Move player
                result.player.position += result.player.orientation.to_vec2();

                // Push sausages
                for i in 0..result.sausages.len() {
                    if result.sausages[i].overlap_player(&result.player) {
                        let direction = result.player.orientation;
                        result.push_sausage(i, direction, data);
                    }
                }

                // Get burned
                if data.tile(result.player.position) == Tile::Grill {
                    result.player.position -= result.player.orientation.to_vec2();
                }
            }
            Direction::Down => {
                // Move player
                result.player.position -= result.player.orientation.to_vec2();

                // Push sausages
                for i in 0..result.sausages.len() {
                    if result.sausages[i].overlap_player(&result.player) {
                        let direction = result.player.orientation.reverse();
                        result.push_sausage(i, direction, data);
                    }
                }

                // Get burned
                if data.tile(result.player.position) == Tile::Grill {
                    result.player.position += result.player.orientation.to_vec2();
                }
            }
            Direction::Left => {
                // Rotate player
                let from_orientation = result.player.orientation;
                result.player.orientation = from_orientation.rotate_ccw();

                let top = result.player.position
                    + result.player.orientation.to_vec2()
                    + from_orientation.to_vec2();
                let mid = result.player.position + result.player.orientation.to_vec2();

                // Push sausages
                for i in 0..result.sausages.len() {
                    if result.sausages[i].overlap(top) {
                        let direction = result.player.orientation;
                        result.push_sausage(i, direction, data);
                    } else if result.sausages[i].overlap(mid) {
                        let direction = result.player.orientation.rotate_ccw();
                        result.push_sausage(i, direction, data);
                    }
                }
            }
            Direction::Right => {
                // Rotate player
                let from_orientation = result.player.orientation;
                result.player.orientation = from_orientation.rotate_cw();

                let top = result.player.position
                    + result.player.orientation.to_vec2()
                    + from_orientation.to_vec2();
                let mid = result.player.position + result.player.orientation.to_vec2();

                // Push sausages
                for i in 0..result.sausages.len() {
                    if result.sausages[i].overlap(top) {
                        let direction = result.player.orientation;
                        result.push_sausage(i, direction, data);
                    } else if result.sausages[i].overlap(mid) {
                        let direction = result.player.orientation.rotate_cw();
                        result.push_sausage(i, direction, data);
                    }
                }
            }
        }

        result.sausages.sort_unstable();
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
            match data.status_of(&state) {
                Status::Solved => result.push((*direction, brutalize::Transition::Success)),
                Status::Unsolved => {
                    result.push((*direction, brutalize::Transition::Indeterminate(state)))
                }
                Status::Failed => (),
            }
        }
        result
    }

    fn heuristic(&self, data: &Self::Data) -> Self::Heuristic {
        let distance = (self.player.position - data.goal_position).abs();
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
    MissingStartOrientation {
        line_number: usize,
    },
    InvalidStartOrientation {
        line_number: usize,
        parse_error: ParseDirectionError,
    },
    SausagesAlreadyDefined {
        line_number: usize,
    },
    MissingSausagesCount {
        line_number: usize,
    },
    InvalidSausagesCount {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingSausageX {
        line_number: usize,
    },
    InvalidSausageX {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingSausageY {
        line_number: usize,
    },
    InvalidSausageY {
        line_number: usize,
        parse_error: ParseIntError,
    },
    MissingSausageOrientation {
        line_number: usize,
    },
    InvalidSausageOrientation {
        line_number: usize,
        parse_error: ParseSausageOrientationError,
    },
    UnexpectedEndOfSausages {
        expected_lines: usize,
        found_lines: usize,
    },
    MissingPuzzle,
    MissingStart,
    MissingSausages,
}

impl brutalize_cli::State for State {
    type ParseError = ParseError;

    fn parse(s: &str) -> Result<(State, Data), ParseError> {
        let mut puzzle = None;
        let mut start = None;
        let mut sausages = None;

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
                                ' ' => Ok(Tile::Empty),
                                '.' => Ok(Tile::Ground),
                                '#' => Ok(Tile::Grill),
                                _ => Err(ParseError::UnexpectedCharacter {
                                    line_number,
                                    column_number: x,
                                    character: c,
                                }),
                            }?;
                            tiles[x + y * size_x] = tile;
                        }
                    }

                    puzzle = Some((Vec2::new(size_x as i32, size_y as i32), tiles));
                }
                "start" => {
                    if start.is_some() {
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
                    let orientation = pieces
                        .next()
                        .ok_or(ParseError::MissingStartOrientation { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidStartOrientation {
                            line_number,
                            parse_error,
                        })?;

                    start = Some((Vec2::new(start_x, start_y), orientation));
                }
                "sausages" => {
                    if sausages.is_some() {
                        return Err(ParseError::SausagesAlreadyDefined { line_number });
                    }

                    let size = pieces
                        .next()
                        .ok_or(ParseError::MissingSausagesCount { line_number })?
                        .parse()
                        .map_err(|parse_error| ParseError::InvalidSausagesCount {
                            line_number,
                            parse_error,
                        })?;

                    let mut read_sausages = ArrayVec::new();
                    for i in 0..size {
                        let (line_number, line) =
                            lines.next().ok_or(ParseError::UnexpectedEndOfSausages {
                                expected_lines: size,
                                found_lines: i,
                            })?;

                        let mut pieces = line.split(' ');
                        let x = pieces
                            .next()
                            .ok_or(ParseError::MissingSausageX { line_number })?
                            .parse()
                            .map_err(|parse_error| ParseError::InvalidSausageX {
                                line_number,
                                parse_error,
                            })?;
                        let y = pieces
                            .next()
                            .ok_or(ParseError::MissingSausageY { line_number })?
                            .parse()
                            .map_err(|parse_error| ParseError::InvalidSausageY {
                                line_number,
                                parse_error,
                            })?;
                        let orientation = pieces
                            .next()
                            .ok_or(ParseError::MissingSausageOrientation { line_number })?
                            .parse()
                            .map_err(|parse_error| ParseError::InvalidSausageOrientation {
                                line_number,
                                parse_error,
                            })?;

                        read_sausages.push(Sausage::new(Vec2::new(x, y), orientation));
                    }

                    sausages = Some(read_sausages);
                }
                command => {
                    return Err(ParseError::InvalidCommand {
                        line_number,
                        command: command.to_string(),
                    })
                }
            }
        }

        let (size, tiles) = puzzle.ok_or(ParseError::MissingPuzzle)?;
        let (goal_position, goal_orientation) = start.ok_or(ParseError::MissingStart)?;
        let sausages = sausages.ok_or(ParseError::MissingSausages)?;

        let data = Data {
            size,
            tiles,
            goal_position,
            goal_orientation,
        };

        Ok((State::initial(&data, sausages), data))
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
                    Tile::Grill => '#',
                }
            }
        }

        for sausage in self.sausages.iter() {
            let index = (sausage.position.x + 1) + (sausage.position.y + 1) * board_width;
            board[index as usize] = 'S';
            let end_position = sausage.end_position();
            let index = (end_position.x + 1) + (end_position.y + 1) * board_width;
            board[index as usize] = 's';
        }

        let index = (self.player.position.x + 1) + (self.player.position.y + 1) * board_width;
        board[index as usize] = 'P';
        let fork_position = self.player.fork_position();
        let index = (fork_position.x + 1) + (fork_position.y + 1) * board_width;
        board[index as usize] = 'F';

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
