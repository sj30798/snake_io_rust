//! Gameplay lifecycle, input, and simulation update systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::app::components::{BoardDecor, Fruit, SnakeSegment, UIText};
use crate::app::state::{GameData, GameState};
use crate::game::core::Game;
use crate::game::types::Direction;

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
    game_data: ResMut<GameData>,
    board_decor: Query<Entity, With<BoardDecor>>,
    snake_segments: Query<Entity, With<SnakeSegment>>,
    fruits: Query<Entity, With<Fruit>>,
    ui_texts: Query<Entity, With<UIText>>,
) {
    if let Ok(mut game) = game_data.game.lock() {
        *game = None;
    }

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
