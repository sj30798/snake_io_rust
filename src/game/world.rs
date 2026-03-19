use crossterm::style::Color;
use rand::Rng;

use crate::game::core::Game;
use crate::game::types::{bot_color, make_snake, Direction, Point, HEIGHT, MAX_BOTS, WIDTH};

impl Game {
    pub(crate) fn spawn_snakes(&mut self) {
        self.snakes.push(make_snake(
            0,
            Point {
                x: WIDTH / 4,
                y: HEIGHT / 2,
            },
            Direction::Right,
            true,
            Color::Cyan,
        ));

        let bot_count = self.settings.bot_count.min(MAX_BOTS);
        for i in 0..bot_count {
            let lane = i as i32 + 1;
            let y = lane * HEIGHT / (bot_count as i32 + 1);
            let x = if i % 2 == 0 {
                WIDTH * 3 / 4
            } else {
                WIDTH * 3 / 4 + 1
            };
            self.snakes.push(make_snake(
                i + 1,
                Point { x, y },
                Direction::Left,
                false,
                bot_color(i),
            ));
        }

        self.update_high_score();
    }

    pub(crate) fn move_snakes(&mut self) {
        for snake in &mut self.snakes {
            snake.move_forward();
        }
    }

    pub(crate) fn resolve_fruit_eating(&mut self) {
        let mut consumed = Vec::new();
        for (fi, fruit) in self.fruits.iter().copied().enumerate() {
            for snake in &mut self.snakes {
                if !snake.alive {
                    continue;
                }
                if snake.head().is_some_and(|h| h == fruit) {
                    snake.pending_growth += 1;
                    consumed.push(fi);
                    break;
                }
            }
        }

        consumed.sort_unstable();
        consumed.dedup();
        while let Some(index) = consumed.pop() {
            self.fruits.remove(index);
        }
    }

    pub(crate) fn refill_fruits(&mut self) {
        while self.fruits.len() < self.settings.fruit_count {
            if let Some(p) = self.random_empty_cell() {
                self.fruits.push(p);
            } else {
                break;
            }
        }
    }

    pub(crate) fn update_high_score(&mut self) {
        for snake in &self.snakes {
            self.high_score = self.high_score.max(snake.len());
        }
    }

    fn random_empty_cell(&mut self) -> Option<Point> {
        for _ in 0..500 {
            let p = Point {
                x: self.rng.gen_range(0..WIDTH),
                y: self.rng.gen_range(0..HEIGHT),
            };

            if self.fruits.contains(&p) {
                continue;
            }

            if self
                .snakes
                .iter()
                .filter(|s| s.alive)
                .any(|s| s.body.iter().any(|seg| *seg == p))
            {
                continue;
            }

            return Some(p);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::time::Instant;

    use crossterm::style::Color;
    use rand::SeedableRng;

    use crate::game::core::Game;
    use crate::game::types::{Direction, GameSettings, Point, Snake, HEIGHT, WIDTH};

    fn custom_settings(bot_count: usize, fruit_count: usize) -> GameSettings {
        GameSettings {
            difficulty: crate::game::types::Difficulty::Normal,
            fruit_count,
            bot_count,
            tick_ms: 120,
            game_time_seconds: 90,
            bot_aggression: 0.85,
        }
    }

    fn empty_game(settings: GameSettings) -> Game {
        Game {
            settings,
            snakes: Vec::new(),
            fruits: Vec::new(),
            rng: rand::rngs::StdRng::seed_from_u64(5),
            start: Instant::now(),
            high_score: 0,
            quit_requested: false,
        }
    }

    #[test]
    fn spawn_snakes_caps_bot_count() {
        let settings = custom_settings(99, 0);
        let mut game = empty_game(settings);

        game.spawn_snakes();

        // 1 player + MAX_BOTS
        assert_eq!(game.snakes.len(), 1 + crate::game::types::MAX_BOTS);
        assert!(game.snakes[0].is_player);
    }

    #[test]
    fn refill_fruits_reaches_target_when_space_available() {
        let settings = custom_settings(0, 8);
        let mut game = empty_game(settings);

        game.spawn_snakes();
        game.fruits.clear();
        game.refill_fruits();

        assert_eq!(game.fruits.len(), 8);
        for p in &game.fruits {
            assert!(p.x >= 0 && p.x < WIDTH && p.y >= 0 && p.y < HEIGHT);
        }
    }

    #[test]
    fn update_high_score_tracks_longest_snake() {
        let settings = custom_settings(0, 0);
        let mut game = empty_game(settings);
        let mut body = VecDeque::new();
        body.push_back(Point { x: 5, y: 5 });
        body.push_back(Point { x: 4, y: 5 });
        body.push_back(Point { x: 3, y: 5 });
        body.push_back(Point { x: 2, y: 5 });
        body.push_back(Point { x: 1, y: 5 });

        game.snakes = vec![Snake {
            id: 0,
            body,
            dir: Direction::Right,
            next_dir: Direction::Right,
            pending_growth: 0,
            is_player: true,
            alive: true,
            eliminated_at: None,
            color: Color::Cyan,
            skin: crate::game::types::generate_skin(0, true, 0),
        }];

        game.update_high_score();
        assert_eq!(game.high_score, 5);
    }
}
