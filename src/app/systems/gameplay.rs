//! Gameplay lifecycle, input, and simulation update systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::app::components::{BoardDecor, Fruit, SnakeSegment, UIText};
use crate::app::state::{GameData, GameState, LastRoundResults, RoundRankEntry};
use crate::game::core::Game;
use crate::game::types::{DeathCause, Difficulty, Direction};

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
        player_death_cause: game.player_death_cause,
        best_combo: game.combo_count,
        near_miss_bonus: game.near_miss_bonus,
        quest_completed: game.quest_complete,
    }
}

/// Creates a new game model when entering `Playing`.
pub fn start_game(mut game_data: ResMut<GameData>) {
    if game_data.first_run && matches!(game_data.selected_difficulty, Difficulty::Normal) {
        game_data.selected_difficulty = Difficulty::Easy;
    }

    let settings = game_data.selected_difficulty.settings();
    if let Ok(mut game) = game_data.game.lock() {
        *game = Some(Game::new(settings));
    }

    if game_data.first_run {
        game_data.contextual_tip = "Tutorial run: stay away from walls and chain fruit for combo.".to_string();
    }
    game_data.first_run = false;
    game_data.last_update = 0.0;
}

/// Applies keyboard direction input and quit request.
pub fn game_input(mut game_data: ResMut<GameData>, keyboard: Res<ButtonInput<KeyCode>>) {
    let toggle_motion = keyboard.just_pressed(KeyCode::KeyM);
    let toggle_colorblind = keyboard.just_pressed(KeyCode::KeyC);
    let toggle_audio = keyboard.just_pressed(KeyCode::KeyV);

    if let Ok(mut game_opt) = game_data.game.lock() {
        if let Some(game) = game_opt.as_mut() {
            if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Up);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Down);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Left);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Right);
                }
            }
            if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Escape) {
                game.quit_requested = true;
                game.player_death_cause = Some(DeathCause::Quit);
            }
            if keyboard.just_pressed(KeyCode::ShiftLeft) || keyboard.just_pressed(KeyCode::ShiftRight) {
                game.trigger_sprint();
            }
            if keyboard.just_pressed(KeyCode::KeyP) {
                game.paused = !game.paused;
            }
            if keyboard.just_pressed(KeyCode::Space) {
                game.magnet_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(4));
            }
        }
    }

    if toggle_motion {
        game_data.accessibility.reduced_motion = !game_data.accessibility.reduced_motion;
    }
    if toggle_colorblind {
        game_data.accessibility.colorblind_mode = !game_data.accessibility.colorblind_mode;
    }
    if toggle_audio {
        game_data.accessibility.reduced_audio = !game_data.accessibility.reduced_audio;
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
            if game.paused {
                return;
            }

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
                game.tick_runtime_effects();

                advanced_tick = true;
                finished_round = game.get_is_finished();
            }

            if finished_round && game.remaining_seconds() == 0 {
                game.player_death_cause = game.player_death_cause.or(Some(DeathCause::TimeUp));
            }
        }
    }

    if advanced_tick {
        game_data.last_update = elapsed;
    }

    if finished_round {
        game_data.meta_progress.runs_completed += 1;
        let mut snapshot = None;
        if let Ok(game_opt) = game_data.game.lock() {
            if let Some(game) = game_opt.as_ref() {
                snapshot = Some((
                    game.quest_complete,
                    game.high_score,
                    game.daily_seed,
                    game.player_death_cause.unwrap_or(DeathCause::Unknown),
                ));
            }
        }

        if let Some((quest_complete, high_score, daily_seed, death_cause)) = snapshot {
            if quest_complete {
                game_data.meta_progress.total_quest_completions += 1;
            }
            let unlocks = high_score / 10;
            game_data.meta_progress.unlocked_trails = game_data.meta_progress.unlocked_trails.max(unlocks);
            game_data.meta_progress.daily_seed_label = format!("{}", daily_seed % 10_000);
            game_data.meta_progress.daily_best_score = game_data.meta_progress.daily_best_score.max(high_score);
            game_data.contextual_tip = match death_cause {
                DeathCause::Wall => "Tip: leave one cell of safety near borders before turning.".to_string(),
                DeathCause::SelfCollision => "Tip: widen your turns when your combo accelerates speed.".to_string(),
                DeathCause::HeadToHead => "Tip: only contest enemy heads when you are longer.".to_string(),
                DeathCause::EatenBySnake => "Tip: avoid crossing enemy bodies near the head.".to_string(),
                DeathCause::TimeUp => "Tip: prioritize combo chains to scale score faster.".to_string(),
                DeathCause::Quit => "Tip: press Shift for sprint bursts during escapes.".to_string(),
                DeathCause::Unknown => "Tip: collect fruit in clusters and keep center control.".to_string(),
            };
        }
        next_state.set(GameState::RoundSummary);
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
