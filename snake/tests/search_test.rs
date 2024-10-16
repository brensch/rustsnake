// File: tests/mcts_test.rs

use battlesnake::game_state::{Direction, GameState};
use battlesnake::search::{Node, MCTS};
use battlesnake::visualizer::{json_to_game_state, visualize_game_state};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct TestCase {
    name: &'static str,
    input: serde_json::Value,
    expected_moves: Vec<Direction>,
}

fn create_test_cases() -> Vec<TestCase> {
    vec![
        TestCase {
            name: "Simple two snake scenario",
            input: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [0, 1, 2],
                        "health": 100
                    },
                    {
                        "id": "snake2",
                        "body": [24, 23, 22],
                        "health": 100
                    }
                ],
                "food": [],
                "hazards": []
            }),
            expected_moves: vec![Direction::Down, Direction::Up],
        },
        TestCase {
            name: "Single snake scenario",
            input: json!({
                "width": 3,
                "height": 3,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [0, 1,   2],
                        "health": 100
                    }
                ],
                "food": [],
                "hazards": []
            }),
            expected_moves: vec![Direction::Up],
        },
        TestCase {
            name: "Three snake scenario",
            input: json!({
                "width": 7,
                "height": 7,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [0, 1, 2],
                        "health": 100
                    },
                    {
                        "id": "snake2",
                        "body": [48, 47, 46],
                        "health": 100
                    },
                    {
                        "id": "snake3",
                        "body": [24, 25, 26],
                        "health": 100
                    }
                ],
                "food": [],
                "hazards": []
            }),
            expected_moves: vec![Direction::Down, Direction::Up, Direction::Left],
        },
    ]
}

#[test]
fn test_mcts_move_selection() {
    let test_cases = create_test_cases();

    for case in test_cases {
        let game_state = json_to_game_state(&case.input);

        println!("Test case: {}", case.name);
        println!("Initial game state:");
        println!("{}", visualize_game_state(&game_state));

        let mut mcts = MCTS::new(game_state.clone());
        let duration = Duration::from_millis(100); // Adjust as needed
        let num_threads = 4; // Adjust as needed

        mcts.run(duration, num_threads);

        // Find the longest path
        let root = mcts.root.clone();
        let longest_path = find_longest_path(&root);

        println!("Longest path in the MCTS tree (from root to leaf):");
        for (i, node) in longest_path.iter().enumerate() {
            let node_lock = node.lock().unwrap_or_else(|e| e.into_inner());
            println!("Step {}:", i);
            println!("{}", visualize_game_state(&node_lock.game_state));
            println!("Moves: {:?}", node_lock.moves);
            println!("Visits: {}", node_lock.visits);
            println!("Value: {:?}", node_lock.value);
            println!("---");
        }

        // Get the best moves from the root
        let best_moves = mcts.get_best_moves();

        println!("Calculated best moves: {:?}", best_moves);
        println!("Expected moves: {:?}", case.expected_moves);

        assert_eq!(
            best_moves.len(),
            case.expected_moves.len(),
            "Test case '{}' failed: move count mismatch",
            case.name
        );

        // Since MCTS is stochastic, we'll check if the moves are valid
        let all_moves_valid = best_moves
            .iter()
            .enumerate()
            .all(|(i, &m)| game_state.get_safe_moves(i).contains(&m));

        assert!(
            all_moves_valid,
            "Test case '{}' failed: invalid moves returned",
            case.name
        );

        println!("\n");
    }
}

// Function to find the longest path from the root to a leaf node
fn find_longest_path(node: &Arc<Mutex<Node>>) -> Vec<Arc<Mutex<Node>>> {
    let node_lock = node.lock().unwrap_or_else(|e| e.into_inner());
    if node_lock.children.is_empty() {
        return vec![Arc::clone(node)];
    }

    let mut max_path = Vec::new();

    for child in &node_lock.children {
        let mut path = find_longest_path(child);
        if path.len() > max_path.len() {
            max_path = path;
        }
    }

    let mut full_path = vec![Arc::clone(node)];
    full_path.extend(max_path);
    full_path
}
