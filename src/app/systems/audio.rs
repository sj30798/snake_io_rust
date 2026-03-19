//! Gameplay audio feedback systems.

use std::path::PathBuf;

use bevy::audio::{PlaybackMode, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::app::state::GameData;

const PICKUP_SFX: &str = "audio/pickup.ogg";
const NEAR_MISS_SFX: &str = "audio/near_miss.ogg";
const COMBO_SFX: &str = "audio/combo.ogg";
const HIT_SFX: &str = "audio/hit.ogg";
const AMBIENCE_CALM_SFX: &str = "audio/ambience_calm.ogg";
const AMBIENCE_TENSION_SFX: &str = "audio/ambience_tension.ogg";
const AMBIENCE_CLUTCH_SFX: &str = "audio/ambience_clutch.ogg";

#[derive(Clone, Copy, PartialEq, Eq)]
enum IntensityTier {
    Calm,
    Tension,
    Clutch,
}

#[derive(Component)]
pub(crate) struct AmbienceLoop;

#[derive(Resource, Default)]
pub struct GameplayAudioAssets {
    pickup: Option<Handle<AudioSource>>,
    near_miss: Option<Handle<AudioSource>>,
    combo: Option<Handle<AudioSource>>,
    hit: Option<Handle<AudioSource>>,
    ambience_calm: Option<Handle<AudioSource>>,
    ambience_tension: Option<Handle<AudioSource>>,
    ambience_clutch: Option<Handle<AudioSource>>,
}

#[derive(Resource, Default)]
pub struct GameplayAudioTracker {
    pub last_total_fruits: usize,
    pub last_near_miss_bonus: usize,
    pub last_combo_count: usize,
    pub player_alive_last_tick: bool,
    last_intensity_tier: Option<IntensityTier>,
}

fn maybe_load_audio(asset_server: &AssetServer, relative_path: &str) -> Option<Handle<AudioSource>> {
    let mut full_path = PathBuf::from("assets");
    full_path.push(relative_path.replace('/', "\\"));
    if full_path.exists() {
        Some(asset_server.load(relative_path.to_string()))
    } else {
        None
    }
}

pub fn load_gameplay_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameplayAudioAssets {
        pickup: maybe_load_audio(&asset_server, PICKUP_SFX),
        near_miss: maybe_load_audio(&asset_server, NEAR_MISS_SFX),
        combo: maybe_load_audio(&asset_server, COMBO_SFX),
        hit: maybe_load_audio(&asset_server, HIT_SFX),
        ambience_calm: maybe_load_audio(&asset_server, AMBIENCE_CALM_SFX),
        ambience_tension: maybe_load_audio(&asset_server, AMBIENCE_TENSION_SFX),
        ambience_clutch: maybe_load_audio(&asset_server, AMBIENCE_CLUTCH_SFX),
    });
    commands.insert_resource(GameplayAudioTracker::default());
}

fn spawn_sfx(commands: &mut Commands, handle: &Handle<AudioSource>, volume: f32) {
    commands.spawn(AudioBundle {
        source: handle.clone(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::new(volume),
            ..default()
        },
    });
}

fn spawn_ambience_loop(commands: &mut Commands, handle: &Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioBundle {
            source: handle.clone(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(volume),
                ..default()
            },
        },
        AmbienceLoop,
    ));
}

fn compute_intensity_tier(
    combo_count: usize,
    near_miss_bonus: usize,
    remaining_seconds: u64,
    alive_snakes: usize,
) -> IntensityTier {
    if remaining_seconds <= 18 || combo_count >= 5 || alive_snakes <= 2 {
        IntensityTier::Clutch
    } else if remaining_seconds <= 40 || combo_count >= 3 || near_miss_bonus >= 2 {
        IntensityTier::Tension
    } else {
        IntensityTier::Calm
    }
}

fn tier_handle<'a>(tier: IntensityTier, assets: &'a GameplayAudioAssets) -> Option<&'a Handle<AudioSource>> {
    match tier {
        IntensityTier::Calm => assets.ambience_calm.as_ref(),
        IntensityTier::Tension => assets.ambience_tension.as_ref(),
        IntensityTier::Clutch => assets.ambience_clutch.as_ref(),
    }
}

fn despawn_ambience(commands: &mut Commands, ambience_loops: &Query<Entity, With<AmbienceLoop>>) {
    for entity in ambience_loops {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn gameplay_audio_feedback(
    game_data: Res<GameData>,
    assets: Res<GameplayAudioAssets>,
    mut tracker: ResMut<GameplayAudioTracker>,
    mut commands: Commands,
    ambience_loops: Query<Entity, With<AmbienceLoop>>,
) {
    if let Ok(game_opt) = game_data.game.lock() {
        if let Some(game) = game_opt.as_ref() {
            let reduced_audio = game_data.accessibility.reduced_audio;
            let master = if reduced_audio { 0.30 } else { 1.0 };

            let intensity_tier = compute_intensity_tier(
                game.combo_count,
                game.near_miss_bonus,
                game.remaining_seconds(),
                game.alive_snake_count(),
            );

            if tracker.last_intensity_tier != Some(intensity_tier) {
                despawn_ambience(&mut commands, &ambience_loops);
                if let Some(handle) = tier_handle(intensity_tier, &assets) {
                    let ambience_volume = match intensity_tier {
                        IntensityTier::Calm => 0.18,
                        IntensityTier::Tension => 0.23,
                        IntensityTier::Clutch => 0.28,
                    } * master;
                    spawn_ambience_loop(&mut commands, handle, ambience_volume);
                }
                tracker.last_intensity_tier = Some(intensity_tier);
            }

            if game.total_fruits_eaten > tracker.last_total_fruits {
                if let Some(pickup) = assets.pickup.as_ref() {
                    spawn_sfx(&mut commands, pickup, 0.38 * master);
                }
                tracker.last_total_fruits = game.total_fruits_eaten;
            }

            if game.near_miss_bonus > tracker.last_near_miss_bonus {
                if let Some(near_miss) = assets.near_miss.as_ref() {
                    spawn_sfx(&mut commands, near_miss, 0.52 * master);
                }
                tracker.last_near_miss_bonus = game.near_miss_bonus;
            }

            // Combo layer triggers only on meaningful streak bumps.
            if game.combo_count > tracker.last_combo_count {
                let combo_threshold_hit = game.combo_count >= 3 && game.combo_count % 2 == 1;
                if combo_threshold_hit {
                    if let Some(combo) = assets.combo.as_ref() {
                        let combo_vol = (0.46 + (game.combo_count as f32 * 0.03)).min(0.76) * master;
                        spawn_sfx(&mut commands, combo, combo_vol);
                    }
                }
                tracker.last_combo_count = game.combo_count;
            }

            let player_alive = game.player().is_some_and(|p| p.alive);
            if tracker.player_alive_last_tick && !player_alive {
                if let Some(hit) = assets.hit.as_ref() {
                    spawn_sfx(&mut commands, hit, 0.68 * master);
                }
            }
            tracker.player_alive_last_tick = player_alive;

            return;
        }
    }

    // No active game: reset tracker so next round starts fresh.
    despawn_ambience(&mut commands, &ambience_loops);
    *tracker = GameplayAudioTracker::default();
}
