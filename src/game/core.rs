use std::time::{Duration, Instant};

use rand::SeedableRng;

use crate::game::types::{GameSettings, Point, Snake};

pub(crate) struct Game {
    pub(crate) settings: GameSettings,
    pub(crate) snakes: Vec<Snake>,
    pub(crate) fruits: Vec<Point>,
    pub(crate) rng: rand::rngs::StdRng,
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
            rng: rand::rngs::StdRng::from_entropy(),
            start: Instant::now(),
            high_score: 0,
            quit_requested: false,
        };

        game.spawn_snakes();
        game.refill_fruits();
        game
    }

    fn is_finished(&self) -> bool {
        self.quit_requested
            || self.elapsed_seconds() >= self.settings.game_time_seconds
            || self.alive_snake_count() <= 1
    }

    pub(crate) fn get_is_finished(&self) -> bool {
        self.is_finished()
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

    pub(crate) fn get_current_tick_duration(&self) -> Duration {
        self.current_tick_duration()
    }

    pub(crate) fn player(&self) -> Option<&Snake> {
        self.snakes.iter().find(|s| s.is_player)
    }

    pub(crate) fn player_mut(&mut self) -> Option<&mut Snake> {
        self.snakes.iter_mut().find(|s| s.is_player)
    }

    pub(crate) fn alive_snake_count(&self) -> usize {
        self.snakes.iter().filter(|s| s.alive).count()
    }

    pub(crate) fn eliminate_snake(&mut self, index: usize) {
        if !self.snakes[index].alive {
            return;
        }
        self.snakes[index].alive = false;
        self.snakes[index].eliminated_at = Some(self.elapsed_seconds());
    }

}
