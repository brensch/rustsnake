## Heuristic

write a function that takes a game state and computes the percentage of the board which each snake controls. the way i want you to do this is by finding the shortest path to each square from the current head of each snake. whichever snake is closest controls the square. i want you to return an array which represents which snake controls each piece. the index of the snake should represent the number at each index. if it's a draw it should be -1

the most important bit is that with every step you get away from each snake whilst calculating the ownership of each square, i want the tail to shorten by 1. this is because as all the snakes move, their tails retract so that area would become available to occupy.

speed and memory efficiency is extremely important for this test. i want you to emphasise speed so much that even if there's a tradeoff that leads to slight inaccuracies, if it's roughly correct then you should make that tradeoff.
