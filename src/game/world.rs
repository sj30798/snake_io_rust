use crossterm::style::Color;
use rand::Rng;

use crate::game::core::Game;
use crate::game::types::{Direction, MAX_BOTS, Point, WIDTH, HEIGHT, bot_color, make_snake};

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
