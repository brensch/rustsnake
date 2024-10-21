// File: tests/game_state_test.rs

use battlesnake::game_state::Direction;
use battlesnake::visualizer::{json_to_game_state, visualize_game_state};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug)]
struct TestCase {
    name: &'static str,
    initial_state: serde_json::Value,
    snake_moves: Vec<String>, // Updated to use Vec to represent moves in the order of snake indices
    expected_state: serde_json::Value,
}

fn create_test_cases() -> Vec<TestCase> {
    vec![
        // Test Case 1: Snake eats food and grows
        TestCase {
            name: "Snake eats food and grows",
            initial_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [12, 13, 14],
                        "health": 90
                    }
                ],
                "food": [7],
                "hazards": []
            }),
            snake_moves: vec!["up".to_string()], // Move for snake1
            expected_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [7, 12, 13, 13],
                        "health": 100
                    }
                ],
                "food": [],
                "hazards": []
            }),
        },
        // Test Case 2: Snake moves out of bounds and dies
        TestCase {
            name: "Snake moves out of bounds",
            initial_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [0, 1, 2],
                        "health": 90
                    }
                ],
                "food": [],
                "hazards": []
            }),
            snake_moves: vec!["up".to_string()], // Move for snake1
            expected_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [{
                    "id": "snake1",
                    "body": [usize::MAX, 0, 1],
                    "health": 0
                }],
                "food": [],
                "hazards": []
            }),
        },
        // Test Case 3: Head-on collision between snakes
        TestCase {
            name: "Head-on collision between snakes",
            initial_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [12, 13, 14],
                        "health": 90
                    },
                    {
                        "id": "snake2",
                        "body": [7, 6, 5],
                        "health": 90
                    }
                ],
                "food": [],
                "hazards": []
            }),
            snake_moves: vec!["up".to_string(), "down".to_string()], // Moves for snake1 and snake2
            expected_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [7, 12, 13],
                        "health": 0
                    },
                    {
                        "id": "snake2",
                        "body": [12, 7, 6],
                        "health": 0
                    }
                ],
                "food": [],
                "hazards": []
            }),
        },
        // Test Case 4: Snake runs out of health
        TestCase {
            name: "Snake runs out of health",
            initial_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [12, 13, 14],
                        "health": 1
                    }
                ],
                "food": [],
                "hazards": []
            }),
            snake_moves: vec!["left".to_string()], // Move for snake1
            expected_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [11, 12, 13],
                        "health": 0
                    }
                ],
                "food": [],
                "hazards": []
            }),
        },
        // Test Case 5: Snake collides with itself
        TestCase {
            name: "Snake collides with itself",
            initial_state: json!({
                "width": 7,
                "height": 7,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [24, 17, 10, 11, 18, 25],
                        "health": 90
                    }
                ],
                "food": [],
                "hazards": []
            }),
            snake_moves: vec!["right".to_string()], // Move for snake1
            expected_state: json!({
                "width": 7,
                "height": 7,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [ 25,24,17, 10, 11, 18 ],
                        "health": 89
                    }
                ],
                "food": [],
                "hazards": []
            }),
        },
        // Test Case 6: Head collision between snakes
        TestCase {
            name: "Head collision between snakes",
            initial_state: json!({
                "width": 7,
                "height": 7,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [2, 1, 0],
                        "health": 90
                    },
                    {
                        "id": "snake2",
                        "body": [4, 5, 6],
                        "health": 90
                    }
                ],
                "food": [],
                "hazards": []
            }),
            snake_moves: vec!["right".to_string(), "left".to_string()], // Moves for snake1 and snake2
            expected_state: json!({
                "width": 7,
                "height": 7,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [3, 2, 1],
                        "health": 0
                    },
                    {
                        "id": "snake2",
                        "body": [3, 4, 5],
                        "health": 0
                    }
                ],
                "food": [],
                "hazards": []
            }),
        },
        // Test Case 10: Snake collides with itself
        TestCase {
            name: "Snake collides with itself",
            initial_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [6, 7, 8, 13, 12, 11, 10],
                        "health": 90
                    }
                ],
                "food": [1], // Food at index 1
                "hazards": []
            }),
            snake_moves: vec!["down".to_string()], // Move for snake1
            expected_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [11, 6, 7, 8, 13, 12, 11],
                        "health": 0
                    }
                ],
                "food": [1],
                "hazards": []
            }),
        },
        // Test Case 11: Snake collides with another snake's body and dies
        TestCase {
            name: "Snake collides with another snake's body and dies",
            initial_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [12, 13, 14],
                        "health": 90
                    },
                    {
                        "id": "snake2",
                        "body": [7, 6, 5],
                        "health": 90
                    }
                ],
                "food": [],
                "hazards": []
            }),
            snake_moves: vec!["up".to_string(), "right".to_string()], // Moves for snake1 and snake2
            expected_state: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [7, 12, 13],
                        "health": 0 // Collides with snake2's body
                    },
                    {
                        "id": "snake2",
                        "body": [8, 7, 6],
                        "health": 89
                    }
                ],
                "food": [],
                "hazards": []
            }),
        },
    ]
}
#[test]
fn test_game_state_simulation() {
    let test_cases = create_test_cases();

    for case in test_cases {
        println!("Test Case: {}\n", case.name);

        // Convert initial state JSON to GameState
        let mut game_state = json_to_game_state(&case.initial_state);

        // Visualize initial state
        println!("Initial State:");
        println!("{}", visualize_game_state(&game_state));

        // Simulate moves based on the index of each snake
        for (index, move_direction) in case.snake_moves.iter().enumerate() {
            let direction = match move_direction.as_str() {
                "up" => Direction::Up,
                "down" => Direction::Down,
                "left" => Direction::Left,
                "right" => Direction::Right,
                _ => continue,
            };
            game_state.move_snake(index, direction); // Use snake index for the move
        }

        // Resolve collisions after moves
        game_state.resolve_collisions();

        // Convert expected state JSON to GameState
        let expected_game_state = json_to_game_state(&case.expected_state);

        // Visualize expected state
        println!("\nExpected State:");
        println!("{}", visualize_game_state(&expected_game_state));

        // Visualize actual state after simulation
        println!("\nActual State:");
        println!("{}", visualize_game_state(&game_state));

        // Compare actual state to expected state
        let actual_state_json =
            serde_json::to_value(&game_state).expect("Failed to serialize actual GameState");
        let expected_state_json = serde_json::to_value(&expected_game_state)
            .expect("Failed to serialize expected GameState");

        assert_eq!(
            actual_state_json, expected_state_json,
            "Test case '{}' failed",
            case.name
        );

        println!("----------------------------------------\n");
    }
}
