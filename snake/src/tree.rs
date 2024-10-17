// tree.rs

use crate::game_state::GameState;
use crate::search::Node;
use crate::visualizer::visualize_game_state;
use chrono::Utc;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize)]
pub struct TreeNode {
    pub id: String,
    pub visits: u32,
    pub avg_scores: Vec<f32>,
    pub ucb_values: Vec<f32>,
    pub is_most_visited: bool,
    pub children: Vec<TreeNode>,
    pub body: String,
    pub game_state: GameState,
}

impl TreeNode {
    fn from_node(node: &Arc<Mutex<Node>>, exploration_constant: f32) -> Self {
        let node_lock = node.lock().unwrap_or_else(|e| e.into_inner());
        let visits = node_lock.visits;
        let avg_scores = if node_lock.visits > 0 {
            node_lock
                .value
                .iter()
                .map(|v| *v / node_lock.visits as f32)
                .collect()
        } else {
            vec![0.0; node_lock.value.len()]
        };
        let ucb_values = calculate_ucb_values(&node_lock, exploration_constant);
        let id = format!("Node_{:p}", &*node_lock as *const _);
        let body = visualize_game_state(&node_lock.game_state);
        let game_state = node_lock.game_state.clone();
        let children = Vec::new(); // We'll populate this later

        TreeNode {
            id,
            visits,
            avg_scores,
            ucb_values,
            is_most_visited: false, // We'll set this during traversal
            children,
            body,
            game_state,
        }
    }
}

pub fn generate_most_visited_path_with_alternatives_html_tree(
    root_node: &Arc<Mutex<Node>>,
) -> Result<(), std::io::Error> {
    let tree_node = generate_tree_data(root_node);

    // Serialize to JSON
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S%.6f").to_string();
    let uuid = Uuid::new_v4().to_string();
    let file_name = format!("{}_{}", timestamp, uuid);
    let file_location = format!("visualiser/tree-data/{}.json", file_name);

    // Ensure the directory exists
    let path = Path::new(&file_location);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create the output file
    let mut file = File::create(&file_location)?;

    // Serialize tree_node to JSON and write to file
    let json_data = serde_json::to_string(&tree_node)?;
    file.write_all(json_data.as_bytes())?;

    println!(
        "Generated move tree: http://localhost:5173/trees/{}",
        file_name
    );

    Ok(())
}

fn generate_tree_data(root_node: &Arc<Mutex<Node>>) -> TreeNode {
    let mut root_tree_node = TreeNode::from_node(root_node, 1.414);

    traverse_and_build_tree(root_node, &mut root_tree_node);

    root_tree_node
}

fn traverse_and_build_tree(node: &Arc<Mutex<Node>>, tree_node: &mut TreeNode) {
    let node_lock = node.lock().unwrap_or_else(|e| e.into_inner());

    let mut children_nodes = node_lock.children.clone();

    // Sort children by visits descending
    children_nodes.sort_by(|a, b| {
        let a_visits = a.lock().unwrap_or_else(|e| e.into_inner()).visits;
        let b_visits = b.lock().unwrap_or_else(|e| e.into_inner()).visits;
        b_visits.cmp(&a_visits)
    });

    for (i, child_node) in children_nodes.iter().enumerate() {
        let mut child_tree_node = TreeNode::from_node(child_node, 1.414);

        // Mark the most visited child
        if i == 0 {
            child_tree_node.is_most_visited = true;
        }

        traverse_and_build_tree(child_node, &mut child_tree_node);
        tree_node.children.push(child_tree_node);
    }
}

fn calculate_ucb_values(node: &Node, exploration_constant: f32) -> Vec<f32> {
    let node_visits = node.visits as f32;
    let parent_visits = node.parent.upgrade().map_or(0, |p| {
        let p_lock = p.lock().unwrap_or_else(|e| e.into_inner());
        p_lock.visits
    }) as f32;

    let ln_parent_visits = if parent_visits > 0.0 {
        parent_visits.ln()
    } else {
        0.0
    };

    node.value
        .iter()
        .map(|&v| {
            if node_visits == 0.0 {
                f32::INFINITY
            } else {
                let avg_value = v / node_visits;
                let exploration = (2.0 * ln_parent_visits / node_visits).sqrt();
                avg_value + exploration_constant * exploration
            }
        })
        .collect()
}
