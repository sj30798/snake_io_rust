use std::thread;
use std::time::{Duration, Instant};

use crate::game::types::{GameSettings, Point, RoundSummary, Snake};

pub(crate) struct Game {
    pub(crate) settings: GameSettings,
    pub(crate) snakes: Vec<Snake>,
    pub(crate) fruits: Vec<Point>,
    pub(crate) rng: rand::rngs::ThreadRng,
    pub(crate) start: Instant,
    pub(crate) high_score: usize,
    pub(crate) quit_requested: bool,
}

impl Game {
    pub(crate) fn new(settings: GameSettings) -> Self {
        let mut game = Self {
            settings,
            snakes: Vec::new(),
            fruits: Vec::new(),
            rng: rand::rng(),
            start: Instant::now(),
            high_score: 0,
            quit_requested: false,
        };

        game.spawn_snakes();
        game.refill_fruits();
        game
    }

    pub(crate) fn run(&mut self) -> std::io::Result<RoundSummary> {
        while !self.is_finished() {
            let frame_start = Instant::now();
            self.process_input()?;
            self.update_bot_directions();
            self.move_snakes();
            self.resolve_head_to_head();
            self.resolve_wall_and_self_collisions();
            self.resolve_snake_eating();
            self.resolve_cross_snake_overlaps();
            self.resolve_fruit_eating();
            self.update_high_score();
            self.refill_fruits();
            self.render()?;

            let elapsed = frame_start.elapsed();
            let tick = self.current_tick_duration();
            if elapsed < tick {
                thread::sleep(tick - elapsed);
            }
        }

        let summary = self.build_summary();
        self.render_game_over(&summary)?;
        Ok(summary)
    }

    fn is_finished(&self) -> bool {
        self.quit_requested
            || self.elapsed_seconds() >= self.settings.game_time_seconds
            || !self.player().is_some_and(|p| p.alive)
    }

    fn elapsed_seconds(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    pub(crate) fn remaining_seconds(&self) -> u64 {
        self.settings
            .game_time_seconds
            .saturating_sub(self.elapsed_seconds())
    }

    fn current_tick_duration(&self) -> Duration {
        let base = self.settings.tick_ms;
        let min_tick = ((base as f32) * 0.55) as u64;
        let player_len = self.player().map_or(3usize, |p| p.len());
        let growth = player_len.saturating_sub(3) as u64;
        let reduction = growth / 2;
        let tick_ms = base.saturating_sub(reduction).max(min_tick);
        Duration::from_millis(tick_ms)
    }

    pub(crate) fn player(&self) -> Option<&Snake> {
        self.snakes.iter().find(|s| s.is_player)
    }

    pub(crate) fn build_summary(&self) -> RoundSummary {
        let winner = self
            .snakes
            .iter()
            .max_by_key(|s| s.len())
            .map(|s| format!("{} (length {})", s.name, s.len()))
            .unwrap_or_else(|| "No winner".to_string());

        let player_alive = self.player().is_some_and(|s| s.alive);
        let end_reason = if self.quit_requested {
            "Quit requested".to_string()
        } else if !player_alive {
            "Player collided and was eliminated".to_string()
        } else {
            "Timer ended".to_string()
        };

        RoundSummary { winner, end_reason }
    }
}
