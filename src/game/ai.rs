use std::cmp::Ordering;

use crate::game::core::Game;
use crate::game::types::{manhattan, BotPersona, Direction, Point, HEIGHT, WIDTH};

impl Game {
    pub(crate) fn update_bot_directions(&mut self) {
        for i in 0..self.snakes.len() {
            if self.snakes[i].is_player || !self.snakes[i].alive {
                continue;
            }

            let current_head = match self.snakes[i].head() {
                Some(h) => h,
                None => continue,
            };

            let mut candidates = [
                self.snakes[i].dir,
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ];

            candidates.sort_by(|a, b| {
                let sa = self.direction_score(i, current_head, *a);
                let sb = self.direction_score(i, current_head, *b);
                sb.partial_cmp(&sa).unwrap_or(Ordering::Equal)
            });

            for dir in candidates {
                if dir == self.snakes[i].dir.opposite() {
                    continue;
                }
                let next = current_head.add(dir);
                if !self.bot_can_step_into(i, next) {
                    continue;
                }
                self.snakes[i].set_direction(dir);
                break;
            }
        }
    }

    fn direction_score(&self, snake_index: usize, head: Point, dir: Direction) -> f32 {
        if dir == self.snakes[snake_index].dir.opposite() {
            return -10_000.0;
        }
        let next = head.add(dir);
        if next.x < 0 || next.x >= WIDTH || next.y < 0 || next.y >= HEIGHT {
            return -5_000.0;
        }

        let mut score = 0.0;
        if !self.bot_can_step_into(snake_index, next) {
            score -= 1_400.0;
        }

        if dir == self.snakes[snake_index].dir {
            score += 2.0;
        }

        if let Some(target) = self.closest_fruit(next) {
            let dist = manhattan(next, target) as f32;
            score += 62.0 / (1.0 + dist);
        }

        score += self.bot_hunt_score(snake_index, next);
        score += self.attack_cell_bonus(snake_index, next);
        score += self.local_open_space_score(snake_index, next);
        score += self.head_to_head_risk_score(snake_index, next);

        let center = Point {
            x: WIDTH / 2,
            y: HEIGHT / 2,
        };
        score += 10.0 / (1.0 + manhattan(next, center) as f32);
        // Small deterministic jitter avoids perfectly tied scores without mutating RNG state.
        let jitter_seed = (next.x * 31 + next.y * 17 + snake_index as i32 * 13) as f32;
        let dynamic_scale = 1.0 + (self.elapsed_progress() * 0.35);
        (score * dynamic_scale) + jitter_seed.sin().abs() * 0.001
    }

    fn head_to_head_risk_score(&self, snake_index: usize, next: Point) -> f32 {
        let my_len = self.snakes[snake_index].len();
        let mut score = 0.0;

        for (idx, snake) in self.snakes.iter().enumerate() {
            if idx == snake_index || !snake.alive {
                continue;
            }

            let Some(enemy_head) = snake.head() else {
                continue;
            };

            // If an enemy head is one move away from our candidate next cell,
            // that enemy can contest this cell on the next tick.
            let distance = manhattan(enemy_head, next);
            if distance == 1 {
                if snake.len() >= my_len {
                    score -= 120.0;
                } else {
                    score -= 24.0;
                }
            } else if distance == 2 && snake.len() >= my_len {
                // Mild caution zone for larger snakes nearby.
                score -= 6.0;
            }
        }

        score
    }

    fn bot_hunt_score(&self, snake_index: usize, next: Point) -> f32 {
        let my_len = self.snakes[snake_index].len() as f32;
        let mut score = 0.0;
        let persona = self.snakes[snake_index].persona;

        for (idx, snake) in self.snakes.iter().enumerate() {
            if idx == snake_index || !snake.alive {
                continue;
            }
            let Some(head) = snake.head() else {
                continue;
            };
            let distance = manhattan(next, head) as f32;

            if snake.len() + 1 < self.snakes[snake_index].len() {
                score += (22.0 * self.settings.bot_aggression) / (1.0 + distance);
            } else if snake.len() > my_len as usize {
                score -= 28.0 / (1.0 + distance);
            } else {
                score -= 6.0 / (1.0 + distance);
            }
        }

        if let Some(BotPersona::Aggressive) = persona {
            score *= 1.24;
        } else if let Some(BotPersona::Evasive) = persona {
            score *= 0.74;
        }

        score
    }

    fn attack_cell_bonus(&self, snake_index: usize, next: Point) -> f32 {
        let mut bonus = 0.0;
        let my_len = self.snakes[snake_index].len();

        for (idx, snake) in self.snakes.iter().enumerate() {
            if idx == snake_index || !snake.alive {
                continue;
            }

            for (seg_index, segment) in snake.body.iter().copied().enumerate() {
                if segment != next {
                    continue;
                }

                if seg_index == 0 {
                    if my_len > snake.len() {
                        bonus += 180.0 * self.settings.bot_aggression;
                    } else {
                        bonus -= 240.0;
                    }
                } else {
                    let bite_amount = (snake.len() - seg_index) as f32;
                    bonus += (30.0 + bite_amount * 8.0) * self.settings.bot_aggression;
                }
            }
        }

        bonus
    }

    fn local_open_space_score(&self, snake_index: usize, origin: Point) -> f32 {
        let neighbors = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        let mut safe_count = 0.0;
        for dir in neighbors {
            let p = origin.add(dir);
            if self.bot_can_step_into(snake_index, p) {
                safe_count += 1.0;
            }
        }
        let persona = self.snakes[snake_index].persona;
        if let Some(BotPersona::Evasive) = persona {
            safe_count * 6.4
        } else {
            safe_count * 4.0
        }
    }

    fn bot_can_step_into(&self, snake_index: usize, cell: Point) -> bool {
        if cell.x < 0 || cell.x >= WIDTH || cell.y < 0 || cell.y >= HEIGHT {
            return false;
        }

        let my_len = self.snakes[snake_index].len();

        for (idx, snake) in self.snakes.iter().enumerate() {
            if !snake.alive {
                continue;
            }

            for (seg_index, segment) in snake.body.iter().enumerate() {
                if *segment != cell {
                    continue;
                }

                if idx == snake_index {
                    if seg_index == snake.len().saturating_sub(1) && snake.pending_growth == 0 {
                        continue;
                    }
                    return false;
                }

                if seg_index == 0 {
                    return my_len > snake.len();
                }

                return true;
            }
        }

        true
    }

    fn closest_fruit(&self, from: Point) -> Option<Point> {
        self.fruits
            .iter()
            .copied()
            .min_by_key(|fruit| manhattan(from, *fruit))
    }

    fn elapsed_progress(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let total = self.settings.game_time_seconds.max(1) as f32;
        (elapsed / total).clamp(0.0, 1.0)
    }
}
