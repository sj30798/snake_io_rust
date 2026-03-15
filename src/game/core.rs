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
            || self.alive_snake_count() <= 1
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

    fn rank_key_for(&self, index: usize, clock_ended: bool) -> (u8, u64, usize, usize) {
        let snake = &self.snakes[index];
        if snake.alive {
            if clock_ended {
                // Survived to clock end: rank by length first.
                (2, 0, snake.len(), usize::MAX - snake.id)
            } else {
                // Last snake alive before timeout.
                (3, 0, snake.len(), usize::MAX - snake.id)
            }
        } else {
            // Eliminated before end: rank by survival time, then length.
            (
                1,
                snake.eliminated_at.unwrap_or(0),
                snake.len(),
                usize::MAX - snake.id,
            )
        }
    }

    pub(crate) fn ranked_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.snakes.len()).collect();
        let clock_ended = self.elapsed_seconds() >= self.settings.game_time_seconds;
        indices.sort_by(|a, b| {
            let ka = self.rank_key_for(*a, clock_ended);
            let kb = self.rank_key_for(*b, clock_ended);
            kb.cmp(&ka)
        });
        indices
    }

    pub(crate) fn build_summary(&self) -> RoundSummary {
        let ranked = self.ranked_indices();
        let winner = ranked
            .first()
            .map(|i| {
                let s = &self.snakes[*i];
                format!("{} (length {})", s.name, s.len())
            })
            .unwrap_or_else(|| "No winner".to_string());

        let end_reason = if self.quit_requested {
            "Quit requested".to_string()
        } else if self.elapsed_seconds() >= self.settings.game_time_seconds {
            "Timer ended".to_string()
        } else {
            "Only one snake remaining".to_string()
        };

        RoundSummary { winner, end_reason }
    }
}
