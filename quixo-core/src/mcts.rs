use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::mpsc;
use crate::simulations::{Result, parallel_simulation};
use crate::game::{find_available_moves, winner, Board, Move, Player};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameState {
    pub board: Board,
    pub player: Player,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MCTSNode {
    // pub game_state: GameState,
    pub visits: u32,
    pub x_wins: u32,
    pub o_wins: u32
}

impl MCTSNode {
    pub fn new() -> Self {
        MCTSNode {
            visits: 0,
            x_wins: 0,
            o_wins: 0,
        }
    }
}

type NodeTable = HashMap<GameState, MCTSNode>;

fn find_child_states(current_state: GameState, available_moves: &[Move]) -> ([GameState;80], usize) {
    let mut child_states = [GameState{board: [[None;5];5], player: Player::X}; 80];
    let mut len = 0;
    for (_, &m) in available_moves.iter().enumerate() {
        child_states[len] = GameState{board: m.apply(current_state.player, &current_state.board).unwrap(), player: current_state.player.next()};
        len += 1;
    }
    (child_states, len)
}

fn calculate_ucb_scores(node_table: &NodeTable, parent_state: GameState, child_states: &[GameState]) -> ([f64; 80], usize) {
    let mut scores: [f64; 80] = [0.0;80];
    let mut len: usize = 0;
    let mut n_visits: u32 = 0;
    child_states.iter()
        .map(|s| node_table.get(s))
        .map(|s| s.map_or((0,0,0), |n| (n.visits, n.x_wins, n.o_wins)))
        .for_each(|(visits, x_wins, o_wins)| {
            n_visits += visits;
            let wins = match parent_state.player {
                Player::X => x_wins,
                Player::O => o_wins
            };
            scores[len] = match visits {
                0 => std::f64::INFINITY,
                _ => (wins as f64) / (visits as f64) * (1.0 / visits as f64).sqrt()
            };
            len += 1;
        });
    if n_visits > 1 {
        for i in 0..len {
            scores[i] = scores[i] * (2.0 * (n_visits as f64).log10()).sqrt();
        }
    }
    (scores, len)
}

fn simulation(current_state: GameState, n: u32) -> Result {
    if let Some(p ) = winner(&current_state.board) {
        // println!("rolling out from a winning state");
        return match p {
            Player::X => Result { wins_x: n, wins_o: 0, draws: 0, total: n },
            Player::O => Result { wins_x: 0, wins_o: n, draws: 0, total: n }
        }
    }
    parallel_simulation(current_state.board, current_state.player, n)
}

fn one_search(
    node_table: &mut NodeTable,
    root_state: GameState,
    num_simulations: u32
) {
    let mut current_state = root_state;
    let mut traversed_states: HashSet<GameState> = HashSet::new();
    loop {
        traversed_states.insert(current_state);
        if let None = node_table.get(&current_state) {
            node_table.insert(current_state, MCTSNode::new());
        }
        if let Some(_) = winner(&current_state.board) {
            // end traversal if a player has already won
            break;
        }
        let available_moves = find_available_moves(&current_state.board, current_state.player);
        if available_moves.1 == 0 {
            // end traversal if there are not moves available
            break;
        }
        let child_states = find_child_states(current_state, &available_moves.0[0..available_moves.1]);
        let scores = calculate_ucb_scores(node_table, current_state, &child_states.0[0..child_states.1]);
        // assert!(scores.1 == child_states.1);
        let (max_score_i, &max_score) = scores.0[0..scores.1].iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap();
        // println!("scores {:?}", scores);
        // println!("chose {} with score {}", max_score_i, max_score);
        if max_score == std::f64::INFINITY {
            // end traversal if a child wasn't visited
            // add it to the node table and traversed node set
            current_state = child_states.0[max_score_i];
            traversed_states.insert(current_state);
            node_table.insert(current_state, MCTSNode::new());
            break;
        }
        if traversed_states.contains(&child_states.0[max_score_i]) {
            // end traversal in case of a loop
            break;
        }
        current_state = child_states.0[max_score_i];
    }
    // println!("traversal ended at level {}", level);
    let result = simulation(current_state, num_simulations);
    // assert!(result.total == 1000);
    for state in traversed_states {
        let node = node_table.get_mut(&state).expect("visited node not found in table");
        node.visits += result.total;
        node.x_wins += result.wins_x;
        node.o_wins += result.wins_o;
    }
}

fn best_move(
    node_table: &NodeTable,
    root_state: GameState
) -> Option<Move> {
    let available_moves = find_available_moves(&root_state.board, root_state.player);
    available_moves.0
        .into_iter()
        .take(available_moves.1)
        .max_by(|m1, m2| {
            let state1 = m1.apply(root_state.player, &root_state.board).unwrap();
            let state2 = m2.apply(root_state.player, &root_state.board).unwrap();
            let node1 = node_table.get(&GameState { board: state1, player: root_state.player.next() });
            let node2 = node_table.get(&GameState { board: state2, player: root_state.player.next() });
            let visits = |n: Option<&MCTSNode>| -> u32 {
                match n {
                    Some(n) => n.visits,
                    None => 0
                }
            };
            // println!("node: {}, visits: {}, x_wins: {}; node: {}, visits: {}, x_wins: {}", m1, visits(node1), node1.unwrap().x_wins, m2, visits(node2), node2.unwrap().x_wins);
            visits(node1).cmp(&visits(node2))
        })
}

pub fn mcts(
    root: GameState,
    iterations: u32,
    sim_per_iter: u32,
    progress_channel: Option<mpsc::Sender<(u32, Option<Move>)>>
) -> Option<Move> {
    let mut node_table: NodeTable = HashMap::new();
    node_table.insert(root, MCTSNode::new());

    for i in 0..iterations {
        one_search(&mut node_table, root, sim_per_iter);
        if let Some(c) = &progress_channel && i % 10 == 0 {
            let _ = c.send((i, best_move(&node_table, root)));
        }
    } 
    best_move(&node_table, root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Board, Player, Move, Shift};
    use crate::simulations::Result;
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
    fn test_mcts_x_almost_won() {
        let b = B;
        let p = Player::X;
        let root = GameState { board: b, player: p };
        let best_move = mcts(root, 100, 1000, None);
        let winning_move = vec![
            Move{x: 1, y: 4, shift: Shift::TOP},
            Move{x: 1, y: 4, shift: Shift::LEFT},
            Move{x: 3, y: 4, shift: Shift::LEFT},
            Move{x: 4, y: 4, shift: Shift::LEFT},
        ];
        assert!(winning_move.contains(&best_move.unwrap()));
    }

    #[test]
    fn test_simulation() {
        let b = B_WON;
        let result = simulation(GameState { board: b, player: Player::O }, 2000);
        assert_eq!(Result{wins_x: 2000, wins_o: 0, draws: 0, total: 2000}, result );
    }

    #[test]
    fn test_one_search() {
        let mut node_table = NodeTable::new();
        for _ in 0..44 {
            one_search(&mut node_table, GameState { board: B, player: Player::X }, 1000);
        }
        let winning_state = GameState{board: B_WON, player: Player::O};
        let winning_node = node_table.get(&winning_state).expect("winning state not in node table");
        assert!(winning_node.x_wins >= 1000);
        assert!(winning_node.o_wins == 0);
        assert!(winning_node.visits == winning_node.x_wins);
    }

    #[test]
    fn test_get_children() {
        let available_moves = find_available_moves(&B, Player::X);
        let children = find_child_states(GameState { board: B, player: Player::X }, &available_moves.0[0..available_moves.1]);
        let n: Vec<u32> = children.0.iter().take(children.1)
            .map(|&c| c==GameState{board: B_WON, player: Player::O})
            .map(|c| c as u32)
            .collect();
        println!("{:?}", n);
        assert!(n.iter().sum::<u32>() == 2);
        assert!(children.1 == 35);
        println!("Distinct children: {}", (children.0[0..children.1])
            .into_iter()
            .map(|&s| s)
            .collect::<HashSet<GameState>>().iter().count());
    }

}