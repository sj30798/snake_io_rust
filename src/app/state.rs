//! Shared game state and constants for the Bevy app.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::game::core::Game;
use crate::game::types::{DeathCause, Difficulty};

/// Visual size of a single board cell in world-space pixels.
pub const CELL_SIZE: f32 = 14.0;

/// Width of the right-side HUD panel in UI pixels.
pub const SIDE_PANEL_WIDTH: f32 = 290.0;

/// Top-level state machine for menu vs gameplay.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    /// Menu and difficulty selection screen.
    #[default]
    Menu,
    /// Active gameplay loop.
    Playing,
    /// Post-round summary screen shown after a game ends.
    RoundSummary,
}

/// Final standing row shown in menu after a round ends.
#[derive(Clone, Debug)]
pub struct RoundRankEntry {
    /// 1-based rank after sorting by score and tie-breakers.
    pub rank: usize,
    /// Display name, for example `You` or `Bot 2`.
    pub name: String,
    /// Score shown to the player.
    pub score: usize,
    /// Whether the snake survived when the round ended.
    pub alive: bool,
}

/// Snapshot of the previous completed round.
#[derive(Clone, Debug)]
pub struct LastRoundResults {
    /// Difficulty label for the completed round.
    pub mode_label: String,
    /// Highest player score reached during that round.
    pub high_score: usize,
    /// Remaining time when round ended.
    pub time_left: u64,
    /// Ranked entries including player and bots.
    pub entries: Vec<RoundRankEntry>,
    /// Player death reason for actionable retry tips.
    pub player_death_cause: Option<DeathCause>,
    /// Fruit streak count reached during the round.
    pub best_combo: usize,
    /// Near-miss bonus moments.
    pub near_miss_bonus: usize,
    /// True when session quest was completed.
    pub quest_completed: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct AccessibilitySettings {
    pub colorblind_mode: bool,
    pub reduced_motion: bool,
    pub reduced_audio: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            colorblind_mode: false,
            reduced_motion: false,
            reduced_audio: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MetaProgress {
    pub runs_completed: usize,
    pub total_quest_completions: usize,
    pub unlocked_trails: usize,
    pub daily_best_score: usize,
    pub daily_seed_label: String,
}

/// Runtime resource containing active game model and player configuration.
#[derive(Resource)]
pub struct GameData {
    /// Current game instance, created when entering `Playing`.
    pub game: Arc<Mutex<Option<Game>>>,
    /// Difficulty selected in menu.
    pub selected_difficulty: Difficulty,
    /// Last simulation timestamp used for fixed-step updates.
    pub last_update: f64,
    /// Results from the previous finished round, displayed in the menu.
    pub last_results: Option<LastRoundResults>,
    /// First-run helper for tutorial and gentle ramp.
    pub first_run: bool,
    /// Lightweight accessibility profile.
    pub accessibility: AccessibilitySettings,
    /// Long-term progression in current app session.
    pub meta_progress: MetaProgress,
    /// Short lived contextual tip shown in HUD.
    pub contextual_tip: String,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            game: Arc::new(Mutex::new(None)),
            selected_difficulty: Difficulty::Normal,
            last_update: 0.0,
            last_results: None,
            first_run: true,
            accessibility: AccessibilitySettings::default(),
            meta_progress: MetaProgress::default(),
            contextual_tip: "Collect fruit quickly to build combo.".to_string(),
        }
    }
}
