use criterion::{black_box, criterion_group, criterion_main, Criterion};
use battlesnake::game_state::GameState;
use battlesnake::heuristic::{calculate_snake_control, calculate_control_percentages};

fn create_sample_game_state(size: usize, num_snakes: usize) -> GameState {
    let mut game = GameState::new(size, size);
    for i in 0..num_snakes {
        let start = i * (size * size / num_snakes);
        game.add_snake(
            format!("snake{}", i),
            vec![start, start + 1, start + 2],
            100,
        );
    }
    game.add_food(size * size / 2);
    game.add_hazard(size - 1);
    game.add_hazard(size * size - size);
    game
}

fn benchmark_snake_control(c: &mut Criterion) {
    let mut group = c.benchmark_group("Snake Control");
    
    for size in [11, 19].iter() {
        for &num_snakes in &[2, 4] {
            let game_state = create_sample_game_state(*size, num_snakes);
            
            group.bench_function(format!("control_{}x{}_{}snakes", size, size, num_snakes), |b| {
                b.iter(|| calculate_snake_control(black_box(&game_state)))
            });
            
            group.bench_function(format!("percentages_{}x{}_{}snakes", size, size, num_snakes), |b| {
                b.iter(|| calculate_control_percentages(black_box(&game_state)))
            });
        }
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_snake_control);
criterion_main!(benches);