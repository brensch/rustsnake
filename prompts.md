## Heuristic

write a function that takes a game state and computes the percentage of the board which each snake controls. the way i want you to do this is by finding the shortest path to each square from the current head of each snake. whichever snake is closest controls the square. i want you to return an array which represents which snake controls each piece. the index of the snake should represent the number at each index. if it's a draw it should be -1

the most important bit is that with every step you get away from each snake whilst calculating the ownership of each square, i want the tail to shorten by 1. this is because as all the snakes move, their tails retract so that area would become available to occupy.

speed and memory efficiency is extremely important for this test. i want you to emphasise speed so much that even if there's a tradeoff that leads to slight inaccuracies, if it's roughly correct then you should make that tradeoff.

## mcts

can you please implement mas mcts as a rust module 'mcts'. the difference is i want you to replace the rollout with a static heuristic. it is:

pub fn calculate_control_percentages(game_state: &GameState) -> Vec<f32>

so you should only calculate each node's value once, and should store it. do everything necessary for the simultaneous aspect of the mcts. the most important bit of the simultaneous moves is that the snakes can potentially die if they move into a square a longer snake can attack. they don't know which square the opponent will move into though. if you do back and forth taking turns a snake moving earlier can think it is safe not realising another snake will move at the same time and kill it.

i want you to have an input to the top level function that specifies how long the search should run for. all threads should stop once that happens.

ideally, i would like to make this use as many threads as the system has available. if you can have the number of threads to use as an input that would be good also. if you think making this multithreaded is too tricky, leave that bit out for now. if you do go for concurrency, discuss the concurrency model you used.

the output should be the full node tree. i want to eventually cache the tree and visualise it etc.
