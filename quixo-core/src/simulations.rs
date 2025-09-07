use rayon::prelude::*;
use crate::game::{Board, Player, random_game};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Result {
    pub wins_x: u32,
    pub wins_o: u32,
    pub draws: u32,
    pub total: u32,
}

pub fn parallel_simulation(b: Board, p: Player, n: u32) -> Result {
    let (wins_x, wins_o, draws) =    (0..n).into_par_iter()
        .map(|_| random_game(b, p))
        .fold(|| (0 as u32,0 as u32, 0 as u32), |(wins_x, wins_o, draws), game| {
            match game {
                Some(Player::X) => (wins_x + 1, wins_o, draws),
                Some(Player::O) => (wins_x, wins_o + 1, draws),
                None => (wins_x, wins_o, draws + 1),
            }
        })
        .reduce(|| (0, 0, 0), |(wins_x1, wins_o1, draws1), (wins_x2, wins_o2, draws2)| {
            (wins_x1 + wins_x2, wins_o1 + wins_o2, draws1 + draws2)
        });

    Result {
        wins_x,
        wins_o,
        draws,
        total: n,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::game::{Board, Player};

    #[test]
    fn test_parallel_simulation_from_empty_board() {
        let b: Board = [[None;5];5];
        let p = Player::X;
        let n = 100000;
        let result = parallel_simulation(b, p, n);
        
        assert_eq!(result.total, n);
        assert_eq!(result.wins_x + result.wins_o + result.draws, n);
        println!("{:#?}", result);
    }

    #[test]
    fn test_parallel_simulation_when_x_almost_won() {
        let b: Board = [
            [Some(Player::X), Some(Player::X), Some(Player::X), Some(Player::X), Some(Player::O)],
            [Some(Player::O), Some(Player::X), None, None, None],
            [None, Some(Player::X), None, None, None],
            [Some(Player::O), Some(Player::X), None, None, None],
            [None, None, Some(Player::O), None, None],
        ];
        let p = Player::X;
        let n = 100000;
        let result = parallel_simulation(b, p, n);
        
        assert_eq!(result.total, n);
        assert_eq!(result.wins_x + result.wins_o + result.draws, n);
        println!("{:#?}", result);
    }
}