// File: tests/mcts_test.rs

use battlesnake::game_state::{Direction, GameState};
use battlesnake::search::{Node, MCTS};
use battlesnake::tree::generate_most_visited_path_with_alternatives_html_tree;
use battlesnake::visualizer::{json_to_game_state, visualize_game_state};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
struct TestCase {
    name: &'static str,
    input: serde_json::Value,
    snake_id: &'static str,
    expected_move: Option<Direction>,
}

fn create_test_cases() -> Vec<TestCase> {
    vec![
        // TestCase {
        //     name: "Simple two snake scenario",
        //     input: json!({
        //         "width": 11,
        //         "height": 11,
        //         "snakes": [
        //             {
        //                 "id": "snake1",
        //                 "body": [0, 1, 2],
        //                 "health": 100
        //             },
        //             {
        //                 "id": "snake2",
        //                 "body": [24, 23, 22],
        //                 "health": 100
        //             }
        //         ],
        //         "food": [],
        //         "hazards": []
        //     }),
        //     snake_id: "snake1",
        //     expected_move: Some(Direction::Down),
        // },
        TestCase {
            name: "Simple two snake scenario",
            input: json!({
                "width": 11,
                "height": 11,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [3,2,1],
                        "health": 100
                    },
                    {
                        "id": "snake2",
                        "body": [5,6,7,8],
                        "health": 100
                    }
                ],
                "food": [],
                "hazards": []
            }),
            snake_id: "snake1",
            expected_move: Some(Direction::Down),
        },
        // Uncomment and adjust the following test cases as needed
        /*
        TestCase {
            name: "Single snake scenario",
            input: json!({
                "width": 3,
                "height": 3,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [4, 5, 6],
                        "health": 100
                    }
                ],
                "food": [],
                "hazards": []
            }),
            snake_id: "snake1",
            expected_move: Some(Direction::Up),
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
            snake_id: "snake1",
            expected_move: Some(Direction::Down),
        },
        */
    ]
}

#[test]
fn test_mcts_move_selection() {
    println!("starting tests");
    let test_cases = create_test_cases();

    for case in test_cases {
        let game_state = json_to_game_state(&case.input);

        println!("Test case: {}", case.name);
        println!("Initial game state:");
        println!("{}", visualize_game_state(&game_state));

        let mut mcts = MCTS::new(game_state.clone());
        let duration = Duration::from_millis(400); // Adjust as needed

        let root = mcts.run(duration, 12);

        // Find the longest path
        let longest_path = find_longest_path(&root);

        println!("Longest path in the MCTS tree (from root to leaf):");
        for (i, node) in longest_path.iter().enumerate() {
            let node_ref = node.lock().unwrap();
            println!("Step {}:", i);
            println!("{}", visualize_game_state(&node_ref.game_state));
            println!("Visits: {}", node_ref.visits);
            // println!("Value: {:?}", node_ref.value);
            println!("---");
        }

        // Get the best move for our snake
        let best_move = mcts.get_best_move_for_snake(case.snake_id);

        println!("Calculated best move: {:?}", best_move);
        println!("Expected move: {:?}", case.expected_move);

        if let Err(e) = generate_most_visited_path_with_alternatives_html_tree(&root) {
            eprintln!("Error generating move tree: {:?}", e);
        }

        // Since MCTS is stochastic, we'll check if the move is valid
        if let Some(our_snake_index) = game_state.snakes.iter().position(|s| s.id == case.snake_id)
        {
            if let Some(best_move) = best_move {
                let safe_moves = game_state.get_safe_moves(our_snake_index);
                assert!(
                    safe_moves.contains(&best_move),
                    "Test case '{}' failed: invalid move returned",
                    case.name
                );
            } else {
                // If no move is returned, our snake might have no safe moves
                let safe_moves = game_state.get_safe_moves(our_snake_index);
                assert!(
                    safe_moves.is_empty(),
                    "Test case '{}' failed: expected a move but none was returned",
                    case.name
                );
            }
        } else {
            // Our snake is not in the game state
            panic!(
                "Test case '{}' failed: snake '{}' not found in game state",
                case.name, case.snake_id
            );
        }

        println!("\n");
    }
}

// Function to find the longest path from the root to a leaf node
fn find_longest_path(node: &Arc<Mutex<Node>>) -> Vec<Arc<Mutex<Node>>> {
    let node_ref = node.lock().unwrap();
    if node_ref.children.is_empty() {
        return vec![Arc::clone(node)];
    }

    let mut max_path = Vec::new();

    // Iterate over the values (child nodes) of the HashMap
    for child in node_ref.children.values() {
        let path = find_longest_path(child);
        if path.len() > max_path.len() {
            max_path = path;
        }
    }

    drop(node_ref); // Explicitly drop the lock

    let mut full_path = vec![Arc::clone(node)];
    full_path.extend(max_path);
    full_path
}
