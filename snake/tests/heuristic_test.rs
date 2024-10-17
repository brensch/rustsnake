// File: tests/heuristic_test.rs

use battlesnake::heuristic::{calculate_control_percentages, calculate_snake_control};
use battlesnake::visualizer::{json_to_game_state, visualize_game_state};
use serde_json::json;

struct TestCase {
    name: &'static str,
    input: serde_json::Value,
    expected_control: Vec<i8>,
    expected_percentages: Vec<f32>,
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
            expected_control: vec![
                0, 0, 0, 0, -1, 0, 0, 0, -1, 1, 0, 0, -1, 1, 1, 0, -1, 1, 1, 1, -1, 1, 1, 1, 1,
            ],
            expected_percentages: vec![40.0, 40.0],
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
            expected_control: vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
            expected_percentages: vec![100.0],
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
            expected_control: vec![
                0, 0, 0, 0, -1, -1, -1, 0, 0, 0, 2, 2, -1, -1, 0, 0, 2, 2, 2, 2, -1, 0, 2, 2, 2, 2,
                2, 1, -1, 2, 2, 2, 2, 1, 1, -1, -1, 2, 2, 1, 1, 1, -1, -1, -1, 1, 1, 1, 1,
            ],
            expected_percentages: vec![20.408163, 20.408163, 34.69388],
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
                    // {
                    //     "id": "snake2",
                    //     "body": [4,5,6],
                    //     "health": 0
                    // },
                    {
                        "id": "snake3",
                        "body": [24, 25, 26],
                        "health": 100
                    }
                ],
                "food": [],
                "hazards": []
            }),
            expected_control: vec![
                0, 0, 0, 0, -1, -1, -1, 0, 0, 0, 2, 2, -1, -1, 0, 0, 2, 2, 2, 2, -1, 0, 2, 2, 2, 2,
                2, 1, -1, 2, 2, 2, 2, 1, 1, -1, -1, 2, 2, 1, 1, 1, -1, -1, -1, 1, 1, 1, 1,
            ],
            expected_percentages: vec![20.408163, 20.408163, 34.69388],
        },
        // Add more test cases as needed
    ]
}

fn visualize_control(control: &[i8], width: usize, height: usize) -> String {
    control
        .chunks(width)
        .map(|row| {
            row.iter()
                .map(|&c| match c {
                    -1 => '.',
                    0..=9 => std::char::from_digit(c as u32, 10).unwrap(),
                    _ => '#',
                })
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[test]
fn test_snake_control_calculation() {
    let test_cases = create_test_cases();

    for case in test_cases {
        let game_state = json_to_game_state(&case.input);

        println!("Test case: {}", case.name);
        println!("Initial game state:");
        println!("{}", visualize_game_state(&game_state));

        let control = calculate_snake_control(&game_state);
        println!("Calculated control:");
        println!(
            "{}",
            visualize_control(&control, game_state.width, game_state.height)
        );
        println!("Expected control:");
        println!(
            "{}",
            visualize_control(&case.expected_control, game_state.width, game_state.height)
        );

        assert_eq!(
            control, case.expected_control,
            "Test case '{}' failed for control calculation",
            case.name
        );

        let percentages = calculate_control_percentages(&game_state);
        println!("Calculated percentages: {:?}", percentages);
        println!("Expected percentages: {:?}", case.expected_percentages);

        assert_eq!(
            percentages.len(),
            case.expected_percentages.len(),
            "Test case '{}' failed: percentage count mismatch",
            case.name
        );

        for (actual, expected) in percentages.iter().zip(case.expected_percentages.iter()) {
            assert!(
                (actual - expected).abs() < 0.01,
                "Test case '{}' failed: percentage mismatch. Expected {}, got {}",
                case.name,
                expected,
                actual
            );
        }

        println!("\n");
    }
}
