// File: tests/visualizer_test.rs

use battlesnake::game_state::{GameState, Snake, Position};
use battlesnake::visualizer::{json_to_game_state, visualize_game_state};
use serde_json::json;

struct TestCase {
    name: &'static str,
    input: serde_json::Value,
    expected_output: &'static str,
}

fn create_test_cases() -> Vec<TestCase> {
    vec![
        TestCase {
            name: "Initial game state",
            input: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [12, 13, 14],
                        "health": 100
                    },
                    {
                        "id": "snake2",
                        "body": [20, 21],
                        "health": 100
                    }
                ],
                "food": [7],
                "hazards": [18]
            }),
            expected_output: "\
.....
..*..
..Aaa
...!.
Bb...",
        },
        TestCase {
            name: "After snake move",
            input: json!({
                "width": 5,
                "height": 5,
                "snakes": [
                    {
                        "id": "snake1",
                        "body": [7, 12, 13],
                        "health": 99
                    },
                    {
                        "id": "snake2",
                        "body": [20, 21],
                        "health": 100
                    }
                ],
                "food": [7],
                "hazards": [18]
            }),
            expected_output: "\
.....
..A..
..aa.
...!.
Bb...",
        },
        TestCase {
            name: "Empty board",
            input: json!({
                "width": 3,
                "height": 3,
                "snakes": [],
                "food": [],
                "hazards": []
            }),
            expected_output: "\
...
...
...",
        },
        TestCase {
            name: "Board with only food",
            input: json!({
                "width": 3,
                "height": 3,
                "snakes": [],
                "food": [4],
                "hazards": []
            }),
            expected_output: "\
...
.*.
...",
        },
        // Add more test cases here
    ]
}



#[test]
fn test_visualize_game_state() {
    let test_cases = create_test_cases();
    
    for case in test_cases {
        let game_state = json_to_game_state(&case.input);
        let visual = visualize_game_state(&game_state);
        assert_eq!(visual, case.expected_output, "Test case '{}' failed", case.name);
    }
}