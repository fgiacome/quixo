use std::fmt::Display;
use rand::{Rng, distr::{Distribution, StandardUniform}};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Player{
    X,
    O
}

impl Player {
    pub fn next(&self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X => write!(f, "X"),
            Self::O => write!(f, "O")
        }
    }
}

impl Distribution<Player> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Player {
        match rng.random_range(0..2) {
            0 => Player::X,
            _ => Player::O,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Shift {
    TOP,
    BOTTOM,
    LEFT,
    RIGHT
}

impl Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TOP => write!(f, "T"),
            Self::BOTTOM => write!(f, "B"),
            Self::LEFT => write!(f, "L"),
            Self::RIGHT => write!(f, "R"),
        }
    }
}

pub type Board = [[Option<Player>; 5]; 5];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Move {
    pub x: u8,
    pub y: u8,
    pub shift: Shift
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Error type for game-related errors.
pub enum GameError {
    InvalidMove,
    NoValidMoves,
}
impl Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::InvalidMove => write!(f, "invalid move"),
            GameError::NoValidMoves => write!(f, "no valid moves available"),
        }
    }
}
impl std::error::Error for GameError {}

type Result<T> = std::result::Result<T,GameError>;

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}, shift: {}", self.x, self.y, self.shift)
    }
}

impl Move {
    pub fn apply(&self, p: Player, b: &Board) -> Result<Board> {
        let (x, y) = (self.x as usize, self.y as usize);
        if !(x == 0 || x == 4 || y == 0 ||  y == 4) {
            return Err(GameError::InvalidMove) // Invalid move
        }
        if !match b[y][x] {
            Some(player) if player == p => true, // Valid move if the position is occupied by the player
            None => true, // Valid move if the position is empty
            _ => false, // Invalid move if the position is occupied by the other player
        } {
            return Err(GameError::InvalidMove); // Invalid move
        }
        let mut new_board = b.clone();
        match self.shift {
            Shift::TOP => {
                if self.y == 0 {
                    return Err(GameError::InvalidMove); // Cannot shift up if already at the top
                }
                for i in (1..=y).rev() {
                    new_board[i][x] = new_board[i - 1][x]; // Shift down
                }
                new_board[0][x] = Some(p); // Place the player piece on the top position
            },
            Shift::BOTTOM => {
                if self.y == 4 {
                    return Err(GameError::InvalidMove); // Cannot shift down if already at the bottom
                }
                for i in y..4 {
                    new_board[i][x] = new_board[i + 1][x]; // Shift up
                }
                new_board[4][x] = Some(p); // Place the player piece on the bottom position
            },
            Shift::LEFT => {
                if self.x == 0 {
                    return Err(GameError::InvalidMove); // Cannot shift left if already at the leftmost position
                }
                for i in (1..=x).rev() {
                    new_board[y][i] = new_board[y][i - 1]; // Shift right
                }
                new_board[y][0] = Some(p); // Place the player piece on the left position
            },
            Shift::RIGHT => {
                if self.x == 4 {
                    return Err(GameError::InvalidMove); // Cannot shift right if already at the rightmost position
                }
                for i in x..4 {
                    new_board[y][i] = new_board[y][i + 1]; // Shift left
                }
                new_board[y][4] = Some(p); // Place the player piece on the right position
            },
        }
        Ok(new_board)
    }
}

pub const ALLOWED_MOVES: [Move; 44] = [
    Move { x: 0, y: 0, shift: Shift::BOTTOM },
    Move { x: 0, y: 0, shift: Shift::RIGHT },
    Move { x: 0, y: 1, shift: Shift::TOP },
    Move { x: 0, y: 1, shift: Shift::BOTTOM },
    Move { x: 0, y: 1, shift: Shift::RIGHT },
    Move { x: 0, y: 2, shift: Shift::TOP },
    Move { x: 0, y: 2, shift: Shift::BOTTOM },
    Move { x: 0, y: 2, shift: Shift::RIGHT },
    Move { x: 0, y: 3, shift: Shift::TOP },
    Move { x: 0, y: 3, shift: Shift::BOTTOM },
    Move { x: 0, y: 3, shift: Shift::RIGHT },
    Move { x: 0, y: 4, shift: Shift::TOP },
    Move { x: 0, y: 4, shift: Shift::RIGHT },
    Move { x: 1, y: 0, shift: Shift::BOTTOM},
    Move { x: 1, y: 0, shift: Shift::LEFT },
    Move { x: 1, y: 0, shift: Shift::RIGHT },
    // Move { x: 1, y: 1, shift: Shift::TOP },
    // Move { x: 1, y: 1, shift: Shift::BOTTOM },
    // Move { x: 1, y: 1, shift: Shift::LEFT },
    // Move { x: 1, y: 1, shift: Shift::RIGHT },
    // Move { x: 1, y: 2, shift: Shift::TOP },
    // Move { x: 1, y: 2, shift: Shift::BOTTOM },
    // Move { x: 1, y: 2, shift: Shift::LEFT },
    // Move { x: 1, y: 2, shift: Shift::RIGHT },
    // Move { x: 1, y: 3, shift: Shift::TOP },
    // Move { x: 1, y: 3, shift: Shift::BOTTOM },
    // Move { x: 1, y: 3, shift: Shift::LEFT },
    // Move { x: 1, y: 3, shift: Shift::RIGHT },
    Move { x: 1, y: 4, shift: Shift::TOP },
    Move { x: 1, y: 4, shift: Shift::LEFT },
    Move { x: 1, y: 4, shift: Shift::RIGHT },
    Move { x: 2, y: 0, shift: Shift::BOTTOM },
    Move { x: 2, y: 0, shift: Shift::LEFT },
    Move { x: 2, y: 0, shift: Shift::RIGHT },
    // Move { x: 2, y: 1, shift: Shift::TOP },
    // Move { x: 2, y: 1, shift: Shift::BOTTOM },
    // Move { x: 2, y: 1, shift: Shift::LEFT },
    // Move { x: 2, y: 1, shift: Shift::RIGHT },
    // Move { x: 2, y: 2, shift: Shift::TOP },
    // Move { x: 2, y: 2, shift: Shift::BOTTOM },
    // Move { x: 2, y: 2, shift: Shift::LEFT },
    // Move { x: 2, y: 2, shift: Shift::RIGHT },
    // Move { x: 2, y: 3, shift: Shift::TOP },
    // Move { x: 2, y: 3, shift: Shift::BOTTOM },
    // Move { x: 2, y: 3, shift: Shift::LEFT },
    // Move { x: 2, y: 3, shift: Shift::RIGHT },
    Move { x: 2, y: 4, shift: Shift::TOP },
    Move { x: 2, y: 4, shift: Shift::LEFT },
    Move { x: 2, y: 4, shift: Shift::RIGHT },
    Move { x: 3, y: 0, shift: Shift::BOTTOM },
    Move { x: 3, y: 0, shift: Shift::LEFT },
    Move { x: 3, y: 0, shift: Shift::RIGHT },
    // Move { x: 3, y: 1, shift: Shift::TOP },
    // Move { x: 3, y: 1, shift: Shift::BOTTOM },
    // Move { x: 3, y: 1, shift: Shift::LEFT },
    // Move { x: 3, y: 1, shift: Shift::RIGHT },
    // Move { x: 3, y: 2, shift: Shift::TOP },
    // Move { x: 3, y: 2, shift: Shift::BOTTOM },
    // Move { x: 3, y: 2, shift: Shift::LEFT },
    // Move { x: 3, y: 2, shift: Shift::RIGHT },
    // Move { x: 3, y: 3, shift: Shift::TOP },
    // Move { x: 3, y: 3, shift: Shift::BOTTOM },
    // Move { x: 3, y: 3, shift: Shift::LEFT }, 
    // Move { x: 3, y: 3, shift: Shift::RIGHT },
    Move { x: 3, y: 4, shift: Shift::TOP },
    Move { x: 3, y: 4, shift: Shift::LEFT },
    Move { x: 3, y: 4, shift: Shift::RIGHT },
    Move { x: 4, y: 0, shift: Shift::BOTTOM },
    Move { x: 4, y: 0, shift: Shift::LEFT },
    Move { x: 4, y: 1, shift: Shift::TOP },
    Move { x: 4, y: 1, shift: Shift::BOTTOM },
    Move { x: 4, y: 1, shift: Shift::LEFT },
    Move { x: 4, y: 2, shift: Shift::TOP },
    Move { x: 4, y: 2, shift: Shift::BOTTOM },
    Move { x: 4, y: 2, shift: Shift::LEFT },
    Move { x: 4, y: 3, shift: Shift::TOP },
    Move { x: 4, y: 3, shift: Shift::BOTTOM },
    Move { x: 4, y: 3, shift: Shift::LEFT },
    Move { x: 4, y: 4, shift: Shift::TOP },
    Move { x: 4, y: 4, shift: Shift::LEFT }
];

pub fn find_available_moves(b: &Board, p: Player) -> ([Move; 80], usize) {
    let mut available_moves: [Move;80] = [Move { x: 0, y: 0, shift: Shift::TOP }; 80];
    let mut num_available_moves = 0;
    for mv in ALLOWED_MOVES.into_iter() {
        if b[mv.y as usize][mv.x as usize].is_none() || b[mv.y as usize][mv.x as usize] == Some(p) {
            available_moves[num_available_moves] = mv;
            num_available_moves += 1;
        }
    }
    (available_moves, num_available_moves)
}

/// Generate a random move for player p on the board b.
pub fn random_move(b: &Board, p: Player) -> Result<Move> {
    let mut rng = rand::rng();
    let (available_moves, num_available_moves) = find_available_moves(b, p);
    if num_available_moves == 0 {
        return Err(GameError::NoValidMoves); // No valid moves available
    }
    let random_index = rng.random_range(0..num_available_moves);
    Ok(available_moves[random_index])
}

/// Play a random game starting from the board b with the player p.
/// The game ends when one of the players has 5 in a row, column, or diagonal.
/// Return the winner player.
pub fn random_game(mut b: Board, mut player: Player) -> Option<Player> {
    loop {
        if let Some(winner_player) = winner(&b) {
            return Some(winner_player); // Return the winner if found
        }
        match random_move(&b, player) {
            Ok(mv) => {
                b = mv.apply(player, &b).unwrap();
                //println!("Player {} made a move: {}", player, mv);
                //print_board(&b);
                player = if player == Player::X { Player::O } else { Player::X }; 
            },
            Err(_) => {
                // No valid moves available, end the game
                return None; // Return the last player who made a move
            }
        }
    }
}

pub fn print_board(b: &Board) {
    for row in b.iter() {
        for cell in row.iter() {
            match cell {
                Some(player) => print!("{} ", player),
                None => print!(". "),
            }
        }
        println!();
    }
    println!();
}

pub fn winner(b: &Board) -> Option<Player> {
    // If a player has 5 in a row, column, or diagonal, return that player
    for i in 0..5 {
        // Check rows
        if let Some(player) = b[i][0] {
            if b[i].iter().all(|&cell| cell == Some(player)) {
                return Some(player);
            }
        }
        // Check columns
        if let Some(player) = b[0][i] {
            if b.iter().all(|row| row[i] == Some(player)) {
                return Some(player);
            }
        }
    }
    // Check diagonals
    if let Some(player) = b[0][0] {
        if (1..5).all(|i| b[i][i] == Some(player)) {
            return Some(player);
        }
    }
    if let Some(player) = b[0][4] {
        if (1..5).all(|i| b[i][4 - i] == Some(player)) {
            return Some(player);
        }
    }
    None

}


#[cfg(test)]
mod tests {
    use super::*;

    const B: Board = [
        [Some(Player::X), Some(Player::X), Some(Player::X), None, None],
        [Some(Player::O), Some(Player::X), None, None, None],
        [None, Some(Player::X), None, None, None],
        [Some(Player::O), Some(Player::X), None, None, None],
        [Some(Player::X), None, Some(Player::O), None, None],
    ];
    const B_WON: Board = [
        [Some(Player::X), Some(Player::X), Some(Player::X), None, None],
        [Some(Player::O), Some(Player::X), None, None, None],
        [None, Some(Player::X), None, None, None],
        [Some(Player::O), Some(Player::X), None, None, None],
        [Some(Player::X), Some(Player::X), Some(Player::O), None, None],
    ];

    #[test]
    fn test_random_game() {
        let board: Board = [[None; 5]; 5];
        let winner = random_game(board, Player::X);
        assert!(winner.is_none() || winner == Some(Player::X) || winner == Some(Player::O));
    }

    #[test]
    fn test_winner() {
        let mut board: Board = [[None; 5]; 5];
        board[0][0] = Some(Player::X);
        board[0][1] = Some(Player::X);
        board[0][2] = Some(Player::X);
        board[0][3] = Some(Player::X);
        board[0][4] = Some(Player::X);
        assert_eq!(winner(&board), Some(Player::X));

        board = [[None; 5]; 5];
        board[0][0] = Some(Player::O);
        board[1][1] = Some(Player::O);
        board[2][2] = Some(Player::O);
        board[3][3] = Some(Player::O);
        board[4][4] = Some(Player::O);
        assert_eq!(winner(&board), Some(Player::O));
    }

    #[test]
    fn test_move_x_almost_won() {
        let b = B;
        let b_new = B_WON;
        let p = Player::X;
        let m = Move {x: 1, y: 4, shift: Shift::LEFT};
        assert_eq!(m.apply(p, &b), Ok(b_new));
    }

}