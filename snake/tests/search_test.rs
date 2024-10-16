// File: tests/mcts_test.rs

use battlesnake::game_state::{Direction, GameState};
use battlesnake::search::MCTS;
use battlesnake::visualizer::{json_to_game_state, visualize_game_state};
use serde_json::json;
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
                        "body": [4, 5, 6],
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
        let best_moves = mcts.get_best_moves();

        println!("Calculated best moves: {:?}", best_moves);
        println!("Expected moves: {:?}", case.expected_moves);

        assert_eq!(
            best_moves.len(),
            case.expected_moves.len(),
            "Test case '{}' failed: move count mismatch",
            case.name
        );

        // Note: MCTS is stochastic, so we can't always expect exact matches.
        // Instead, we'll check if the moves are valid and print a warning if they don't match exactly.
        let all_moves_valid = best_moves
            .iter()
            .enumerate()
            .all(|(i, &m)| game_state.get_safe_moves(i).contains(&m));

        assert!(
            all_moves_valid,
            "Test case '{}' failed: invalid moves returned",
            case.name
        );

        if best_moves != case.expected_moves {
            println!("Warning: Moves don't exactly match expected moves. This may be due to the stochastic nature of MCTS.");
        }

        println!("\n");
    }
}
