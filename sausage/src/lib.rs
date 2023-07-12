use arrayvec::ArrayVec;
use solver_common::{Direction, ParseDirectionError, Vec2};
use std::{fmt, num::ParseIntError, str::FromStr};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Empty,
    Ground,
    Grill,
    Wall,
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
            if !sausage.overlap(state.player.fork_position())
                && self.tile(sausage.position) == Tile::Empty
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
    fn end_offset(&self) -> Vec2 {
        match self.orientation {
            SausageOrientation::Horizontal => Direction::Right.to_vec2(),
            SausageOrientation::Vertical => Direction::Up.to_vec2(),
        }
    }

    #[inline]
    fn end_position(&self) -> Vec2 {
        self.position + self.end_offset()
    }

    #[inline]
    fn overlap(&self, position: Vec2) -> bool {
        (position == self.position) || (position == self.end_position())
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
    fn push(&mut self, direction: Direction, data: &Data, can_roll: bool) {
        self.position += direction.to_vec2();
        if can_roll {
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
        }

        if data.tile(self.position) == Tile::Grill {
            self.cook(2);
        }
        if data.tile(self.end_position()) == Tile::Grill {
            self.cook(3);
        }
    }

    #[inline]
    fn is_in_wall(&self, data: &Data) -> bool {
        data.tile(self.position) == Tile::Wall || data.tile(self.end_position()) == Tile::Wall
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
    fn try_move_sausage(&mut self, sausage_index: usize, direction: Direction, data: &Data, can_roll: bool) -> bool {
        self.sausages[sausage_index].push(direction, data, can_roll);
        if self.sausages[sausage_index].is_in_wall(data) {
            return false;
        }

        for i in (0..self.sausages.len()).filter(|&i| i != sausage_index) {
            if self.sausages[sausage_index].overlap_sausage(&self.sausages[i]) {
                if !self.try_move_sausage(i, direction, data, true) {
                    return false;
                }
            }
        }

        true
    }

    #[inline]
    fn try_strafe_player(&mut self, data: &Data, direction: Direction) -> bool {
        let old_fork_position = self.player.fork_position();

        // Move player
        let forward = direction.to_vec2();
        self.player.position += forward;

        // No invalid moves
        let player_in_wall = data.tile(self.player.position) == Tile::Wall;
        let fork_in_wall = data.tile(self.player.fork_position()) == Tile::Wall;
        if player_in_wall || fork_in_wall {
            return false;
        }

        // Push sausages
        let mut impaled = None;
        for i in 0..self.sausages.len() {
            if self.sausages[i].overlap(old_fork_position) {
                // Impaled sausages always move with the player
                let original_sausages = self.sausages.clone();
                if !self.try_move_sausage(i, direction, data, false) {
                     if direction != self.player.orientation.reverse() {
                        // If the player isn't moving backwards and the impaled
                        // sausage cannot move, then the move cannot be done.
                        return false;
                    } else {
                        // If the player is moving backwards and the impaled
                        // sausage cannot move, then the impaled sausage does
                        // not move.
                        self.sausages = original_sausages;
                        impaled = None;
                    }
                } else {
                    impaled = Some(i);
                }
            } else if self.sausages[i].overlap(self.player.position) {
                if !self.try_move_sausage(i, direction, data, true) {
                    // If the player cannot push a sausage out of the way, then
                    // the move cannot be done.
                    return false;
                }
            } else if self.sausages[i].overlap(self.player.fork_position()) {
                let original_sausages = self.sausages.clone();
                if !self.try_move_sausage(i, direction, data, true) {
                    if direction != self.player.orientation {
                        // If the fork isn't moving forward and cannot push a
                        // sausage out of the way, then the move cannot be done.
                        return false;
                    } else {
                        // If the fork is moving forward and cannot push a
                        // sausage out of the way, then the sausages don't move
                        // and the fork impales a sausage.
                        self.sausages = original_sausages;
                        impaled = Some(i);
                    }
                }
            }
        }

        // Get burned
        if data.tile(self.player.position) == Tile::Grill {
            self.player.position -= forward;
            if let Some(impaled) = impaled {
                let original_sausages = self.sausages.clone();
                if !self.try_move_sausage(impaled, direction.reverse(), data, false) {
                    // If the impaled sausage can't move back with us, then it
                    // does not move.
                    self.sausages = original_sausages;
                }
            }
        }

        true
    }

    #[inline]
    fn try_rotate_player(&mut self, data: &Data, direction: Direction) -> bool {
        // Rotate player
        let original_orientation = self.player.orientation;
        self.player.orientation = direction;

        let mid = self.player.fork_position();
        let top = mid + original_orientation.to_vec2();

        // No invalid moves
        if data.tile(top) == Tile::Wall {
            return false;
        }

        // Push top sausages
        if let Some(i) = self.sausages.iter().position(|sausage| sausage.overlap(top)) {
            let direction = self.player.orientation;
            if !self.try_move_sausage(i, direction, data, true) {
                // If the top sausage can't be moved then the move cannot be
                // done.
                return false;
            }
        }

        // If the mid tile is a wall then we can't do a full turn but we can do
        // a half turn.
        if data.tile(mid) == Tile::Wall {
            self.player.orientation = original_orientation;
            return true;
        }

        // Push mid sausages
        if let Some(i) = self.sausages.iter().position(|sausage| sausage.overlap(mid)) {
            let original_sausages = self.sausages.clone();
            let direction = original_orientation.reverse();
            if !self.try_move_sausage(i, direction, data, true) {
                // If the mid sausage can't be moved then the top sausage move
                // still happens and the player unrotates.
                self.player.orientation = original_orientation;
                self.sausages = original_sausages;
            }
        }

        true
    }

    #[inline]
    fn transition(&self, data: &Data, direction: Direction) -> Option<State> {
        let mut result = self.clone();

        let is_impaled = self.sausages.iter().any(|s| s.overlap(self.player.fork_position()));
        let moving_forward = direction == self.player.orientation;
        let moving_backward = direction == self.player.orientation.reverse();
        if is_impaled || moving_forward || moving_backward {
            if !result.try_strafe_player(data, direction) {
                return None;
            }
        } else {
            if !result.try_rotate_player(data, direction) {
                return None;
            }
        }

        result.sausages.sort_unstable();
        Some(result)
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
                                'X' => Ok(Tile::Wall),
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
                    Tile::Wall => 'X',
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

#[cfg(test)]
mod tests {
    use brutalize_cli::State as _;
    use solver_common::{Direction, Vec2};
    use crate::{State, Sausage, SausageOrientation, Cooked, Player};

    macro_rules! lines {
        ($($line:expr)*) => {
            concat!(
                $($line, "\n",)*
            )
        }
    }

    macro_rules! arrayvec {
        ($($el:expr),* $(,)?) => {{
            let mut result = arrayvec::ArrayVec::new();
            $(result.push($el);)*
            result
        }}
    }

    #[test]
    fn strafe_roll_two() {
        const PUZZLE: &'static str = lines![
            "puzzle 5 5"
            "....."
            "....."
            "....."
            "....."
            "....."
            "start 0 0 right"
            "sausages 2"
            "2 0 vertical"
            "3 1 vertical"
        ];

        let (state, data) = State::parse(PUZZLE).unwrap();
        assert_eq!(
            state.transition(&data, Direction::Right),
            Some(State {
                player: Player {
                    position: Vec2::new(1, 0),
                    orientation: Direction::Right,
                },
                sausages: arrayvec![
                    Sausage {
                        position: Vec2::new(3, 0),
                        orientation: SausageOrientation::Vertical,
                        cooked: [Cooked::Uncooked; 4],
                    },
                    Sausage {
                        position: Vec2::new(4, 1),
                        orientation: SausageOrientation::Vertical,
                        cooked: [Cooked::Uncooked; 4],
                    },
                ],
            })
        )
    }

    #[test]
    fn turn_roll_two() {
        const PUZZLE: &'static str = lines![
            "puzzle 5 5"
            "....."
            "....."
            "....."
            "....."
            "....."
            "start 0 1 up"
            "sausages 2"
            "1 2 vertical"
            "1 1 horizontal"
        ];

        let (state, data) = State::parse(PUZZLE).unwrap();
        assert_eq!(
            state.transition(&data, Direction::Right),
            Some(State {
                player: Player {
                    position: Vec2::new(0, 1),
                    orientation: Direction::Right,
                },
                sausages: arrayvec![
                    Sausage {
                        position: Vec2::new(1, 0),
                        orientation: SausageOrientation::Horizontal,
                        cooked: [Cooked::Uncooked; 4],
                    },
                    Sausage {
                        position: Vec2::new(2, 2),
                        orientation: SausageOrientation::Vertical,
                        cooked: [Cooked::Uncooked; 4],
                    },
                ],
            })
        )
    }

    #[test]
    fn half_turn_roll() {
        const PUZZLE: &'static str = lines![
            "puzzle 3 3"
            "..."
            "..."
            ".X."
            "start 0 0 up"
            "sausages 1"
            "1 1 vertical"
        ];

        let (state, data) = State::parse(PUZZLE).unwrap();
        assert_eq!(
            state.transition(&data, Direction::Right),
            Some(State {
                player: Player {
                    position: Vec2::new(0, 0),
                    orientation: Direction::Up,
                },
                sausages: arrayvec![
                    Sausage {
                        position: Vec2::new(2, 1),
                        orientation: SausageOrientation::Vertical,
                        cooked: [Cooked::Uncooked; 4],
                    },
                ],
            })
        )
    }
}
