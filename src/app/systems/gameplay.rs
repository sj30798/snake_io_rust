//! Gameplay lifecycle, input, and simulation update systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::app::components::{BoardDecor, Fruit, SnakeSegment, UIText};
use crate::app::state::{GameData, GameState, LastRoundResults, RoundRankEntry};
use crate::game::core::Game;
use crate::game::types::Direction;

fn build_last_round_results(game: &Game) -> LastRoundResults {
    let mut entries: Vec<_> = game
        .snakes
        .iter()
        .map(|snake| {
            let name = if snake.is_player {
                "You".to_string()
            } else {
                format!("Bot {}", snake.id)
            };
            (snake.id, snake.is_player, snake.alive, snake.len(), name)
        })
        .collect();

    // Rank by score first, then alive status, then player priority, then stable id.
    entries.sort_by(|a, b| {
        b.3.cmp(&a.3)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.0.cmp(&b.0))
    });

    let ranked = entries
        .into_iter()
        .enumerate()
        .map(|(idx, (_, _, alive, score, name))| RoundRankEntry {
            rank: idx + 1,
            name,
            score,
            alive,
        })
        .collect();

    LastRoundResults {
        mode_label: game.settings.label().to_string(),
        high_score: game.high_score,
        time_left: game.remaining_seconds(),
        entries: ranked,
    }
}

/// Creates a new game model when entering `Playing`.
pub fn start_game(mut game_data: ResMut<GameData>) {
    let settings = game_data.selected_difficulty.settings();
    if let Ok(mut game) = game_data.game.lock() {
        *game = Some(Game::new(settings));
    }
    game_data.last_update = 0.0;
}

/// Applies keyboard direction input and quit request.
pub fn game_input(game_data: ResMut<GameData>, keyboard: Res<ButtonInput<KeyCode>>) {
    if let Ok(mut game_opt) = game_data.game.lock() {
        if let Some(game) = game_opt.as_mut() {
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Up);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Down);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowLeft) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Left);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Right);
                }
            }
            if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Escape) {
                game.quit_requested = true;
            }
        }
    }
}

/// Advances game simulation in fixed-ish steps and transitions to menu when round ends.
pub fn game_update(
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_seconds_f64();
    let last_update = game_data.last_update;
    let mut advanced_tick = false;
    let mut finished_round = false;

    if let Ok(mut game_opt) = game_data.game.lock() {
        if let Some(game) = game_opt.as_mut() {
            let tick_duration = game.get_current_tick_duration().as_secs_f64();

            if elapsed - last_update >= tick_duration {
                game.update_bot_directions();
                game.move_snakes();
                game.resolve_head_to_head();
                game.resolve_wall_and_self_collisions();
                game.resolve_snake_eating();
                game.resolve_cross_snake_overlaps();
                game.resolve_fruit_eating();
                game.update_high_score();
                game.refill_fruits();

                advanced_tick = true;
                finished_round = game.get_is_finished();
            }
        }
    }

    if advanced_tick {
        game_data.last_update = elapsed;
    }

    if finished_round {
        next_state.set(GameState::Menu);
    }
}

/// Clears game model and despawns all entities rendered in `Playing` state.
pub fn cleanup_game(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    board_decor: Query<Entity, With<BoardDecor>>,
    snake_segments: Query<Entity, With<SnakeSegment>>,
    fruits: Query<Entity, With<Fruit>>,
    ui_texts: Query<Entity, With<UIText>>,
) {
    let mut last_results = None;
    if let Ok(mut game_opt) = game_data.game.lock() {
        if let Some(game) = game_opt.as_ref() {
            last_results = Some(build_last_round_results(game));
        }
        *game_opt = None;
    }
    game_data.last_results = last_results;

    for entity in &board_decor {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &snake_segments {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &fruits {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &ui_texts {
        commands.entity(entity).despawn_recursive();
    }
}
