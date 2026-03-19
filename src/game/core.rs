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

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::time::{Duration, Instant};

    use crossterm::style::Color;
    use rand::SeedableRng;

    use super::Game;
    use crate::game::types::{Difficulty, Direction, GameSettings, Point, Snake};

    fn test_settings() -> GameSettings {
        Difficulty::Normal.settings()
    }

    fn snake_at(id: usize, x: i32, y: i32, is_player: bool) -> Snake {
        let mut body = VecDeque::new();
        body.push_back(Point { x, y });
        body.push_back(Point { x: x - 1, y });
        body.push_back(Point { x: x - 2, y });
        Snake {
            id,
            body,
            dir: Direction::Right,
            next_dir: Direction::Right,
            pending_growth: 0,
            is_player,
            alive: true,
            eliminated_at: None,
            color: if is_player { Color::Cyan } else { Color::Red },
            skin: crate::game::types::generate_skin(id, is_player, id as u64),
        }
    }

    fn base_game() -> Game {
        Game {
            settings: test_settings(),
            snakes: vec![snake_at(0, 5, 5, true), snake_at(1, 20, 5, false)],
            fruits: Vec::new(),
            rng: rand::rngs::StdRng::seed_from_u64(7),
            start: Instant::now(),
            high_score: 0,
            quit_requested: false,
        }
    }

    #[test]
    fn finished_when_quit_requested() {
        let mut game = base_game();
        game.quit_requested = true;
        assert!(game.get_is_finished());
    }

    #[test]
    fn finished_when_one_snake_left() {
        let mut game = base_game();
        game.snakes[1].alive = false;
        assert!(game.get_is_finished());
    }

    #[test]
    fn finished_when_time_expires() {
        let mut game = base_game();
        game.start = Instant::now() - Duration::from_secs(game.settings.game_time_seconds + 1);
        assert!(game.get_is_finished());
    }

    #[test]
    fn tick_duration_decreases_as_player_grows() {
        let mut game = base_game();
        let base = game.get_current_tick_duration();

        game.snakes[0].body.push_back(Point { x: 2, y: 5 });
        game.snakes[0].body.push_back(Point { x: 1, y: 5 });
        game.snakes[0].body.push_back(Point { x: 0, y: 5 });
        let faster = game.get_current_tick_duration();

        assert!(faster <= base);
    }

    #[test]
    fn eliminate_marks_snake_dead_and_sets_time() {
        let mut game = base_game();
        game.eliminate_snake(1);
        assert!(!game.snakes[1].alive);
        assert!(game.snakes[1].eliminated_at.is_some());
    }
}
