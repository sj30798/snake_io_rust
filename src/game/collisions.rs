use std::collections::{HashMap, HashSet};

use crate::game::core::Game;
use crate::game::types::Point;

impl Game {
    pub(crate) fn resolve_head_to_head(&mut self) {
        let mut positions: HashMap<Point, Vec<usize>> = HashMap::new();
        for (i, snake) in self.snakes.iter().enumerate() {
            if !snake.alive {
                continue;
            }
            if let Some(h) = snake.head() {
                positions.entry(h).or_default().push(i);
            }
        }

        for indices in positions.into_values() {
            if indices.len() <= 1 {
                continue;
            }

            let mut strongest_len = 0usize;
            for i in &indices {
                strongest_len = strongest_len.max(self.snakes[*i].len());
            }

            let strongest_indices: Vec<usize> = indices
                .iter()
                .copied()
                .filter(|i| self.snakes[*i].len() == strongest_len)
                .collect();

            let player_wins_tie = strongest_indices.iter().any(|i| self.snakes[*i].is_player);

            if strongest_indices.len() > 1 && !player_wins_tie {
                for i in indices {
                    self.eliminate_snake(i);
                }
            } else {
                let winner = if player_wins_tie {
                    strongest_indices
                        .iter()
                        .copied()
                        .find(|i| self.snakes[*i].is_player)
                        .unwrap_or(strongest_indices[0])
                } else {
                    strongest_indices[0]
                };
                let mut growth = 0usize;
                for i in indices {
                    if i != winner {
                        growth += self.snakes[i].len();
                        self.eliminate_snake(i);
                    }
                }
                self.snakes[winner].pending_growth += growth;
            }
        }
    }

    pub(crate) fn resolve_wall_and_self_collisions(&mut self) {
        for i in 0..self.snakes.len() {
            if !self.snakes[i].alive {
                continue;
            }
            let head = match self.snakes[i].head() {
                Some(h) => h,
                None => continue,
            };

            if head.x < 0
                || head.x >= crate::game::types::WIDTH
                || head.y < 0
                || head.y >= crate::game::types::HEIGHT
            {
                self.eliminate_snake(i);
                continue;
            }

            let hit_self = self.snakes[i]
                .body
                .iter()
                .skip(1)
                .any(|segment| *segment == head);
            if hit_self {
                self.eliminate_snake(i);
            }
        }
    }

    pub(crate) fn resolve_snake_eating(&mut self) {
        let mut interactions = Vec::new();

        for eater_index in 0..self.snakes.len() {
            if !self.snakes[eater_index].alive {
                continue;
            }
            let eater_head = match self.snakes[eater_index].head() {
                Some(h) => h,
                None => continue,
            };

            for victim_index in 0..self.snakes.len() {
                if eater_index == victim_index || !self.snakes[victim_index].alive {
                    continue;
                }

                for (seg_index, segment) in
                    self.snakes[victim_index].body.iter().copied().enumerate()
                {
                    if segment == eater_head {
                        interactions.push((eater_index, victim_index, seg_index));
                        break;
                    }
                }
            }
        }

        for (eater, victim, seg_index) in interactions {
            if !self.snakes[eater].alive || !self.snakes[victim].alive {
                continue;
            }

            if seg_index == 0 {
                let victim_len = self.snakes[victim].len();
                self.eliminate_snake(victim);
                self.snakes[eater].pending_growth += victim_len;
            } else {
                let victim_len = self.snakes[victim].len();
                if seg_index >= victim_len {
                    continue;
                }
                let kept_len = seg_index.saturating_sub(1);
                let eaten_amount = victim_len - kept_len;
                self.snakes[victim].body.truncate(kept_len);
                if self.snakes[victim].body.is_empty() {
                    self.eliminate_snake(victim);
                }
                self.snakes[eater].pending_growth += eaten_amount;
            }
        }
    }

    pub(crate) fn resolve_cross_snake_overlaps(&mut self) {
        for _ in 0..3 {
            let mut occupancy: HashMap<Point, Vec<(usize, usize)>> = HashMap::new();
            for (snake_index, snake) in self.snakes.iter().enumerate() {
                if !snake.alive {
                    continue;
                }
                for (seg_index, segment) in snake.body.iter().copied().enumerate() {
                    occupancy
                        .entry(segment)
                        .or_default()
                        .push((snake_index, seg_index));
                }
            }

            let mut had_overlap = false;
            for entries in occupancy.into_values() {
                let alive_distinct = entries
                    .iter()
                    .filter(|(idx, _)| self.snakes[*idx].alive)
                    .map(|(idx, _)| *idx)
                    .collect::<HashSet<_>>();

                if alive_distinct.len() <= 1 {
                    continue;
                }
                had_overlap = true;

                let has_head = entries
                    .iter()
                    .any(|(idx, seg)| self.snakes[*idx].alive && *seg == 0);

                let owner = entries
                    .iter()
                    .filter(|(idx, _)| self.snakes[*idx].alive)
                    .max_by_key(|(idx, seg)| {
                        let head_priority = if has_head && *seg == 0 {
                            1usize
                        } else {
                            0usize
                        };
                        (
                            head_priority,
                            self.snakes[*idx].len(),
                            usize::MAX - self.snakes[*idx].id,
                        )
                    })
                    .map(|(idx, _)| *idx);

                let Some(owner_index) = owner else {
                    continue;
                };

                for (snake_index, seg_index) in entries {
                    if snake_index == owner_index || !self.snakes[snake_index].alive {
                        continue;
                    }

                    if seg_index == 0 {
                        let victim_len = self.snakes[snake_index].len();
                        self.eliminate_snake(snake_index);
                        self.snakes[owner_index].pending_growth += victim_len;
                    } else {
                        let victim_len = self.snakes[snake_index].len();
                        let kept_len = seg_index.saturating_sub(1);
                        let eaten_amount = victim_len.saturating_sub(kept_len);
                        self.snakes[snake_index].body.truncate(kept_len);
                        if self.snakes[snake_index].body.is_empty() {
                            self.eliminate_snake(snake_index);
                        }
                        self.snakes[owner_index].pending_growth += eaten_amount;
                    }
                }
            }

            if !had_overlap {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::time::Instant;

    use crossterm::style::Color;
    use rand::SeedableRng;

    use crate::game::core::Game;
    use crate::game::types::{Difficulty, Direction, Point, Snake};

    fn snake_with_body(id: usize, body: Vec<Point>, is_player: bool) -> Snake {
        Snake {
            id,
            body: VecDeque::from(body),
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

    fn make_game(snakes: Vec<Snake>) -> Game {
        Game {
            settings: Difficulty::Normal.settings(),
            snakes,
            fruits: Vec::new(),
            rng: rand::rngs::StdRng::seed_from_u64(42),
            start: Instant::now(),
            high_score: 0,
            quit_requested: false,
        }
    }

    #[test]
    fn head_to_head_player_wins_tie() {
        let player = snake_with_body(
            0,
            vec![
                Point { x: 5, y: 5 },
                Point { x: 4, y: 5 },
                Point { x: 3, y: 5 },
            ],
            true,
        );
        let bot = snake_with_body(
            1,
            vec![
                Point { x: 5, y: 5 },
                Point { x: 6, y: 5 },
                Point { x: 7, y: 5 },
            ],
            false,
        );
        let mut game = make_game(vec![player, bot]);

        game.resolve_head_to_head();

        assert!(game.snakes[0].alive);
        assert!(!game.snakes[1].alive);
        assert!(game.snakes[0].pending_growth >= 3);
    }

    #[test]
    fn wall_collision_eliminates_snake() {
        let out_of_bounds = snake_with_body(
            0,
            vec![
                Point { x: -1, y: 0 },
                Point { x: 0, y: 0 },
                Point { x: 1, y: 0 },
            ],
            true,
        );
        let mut game = make_game(vec![out_of_bounds]);

        game.resolve_wall_and_self_collisions();

        assert!(!game.snakes[0].alive);
    }

    #[test]
    fn self_collision_eliminates_snake() {
        let self_hit = snake_with_body(
            0,
            vec![
                Point { x: 5, y: 5 },
                Point { x: 6, y: 5 },
                Point { x: 5, y: 5 },
            ],
            true,
        );
        let mut game = make_game(vec![self_hit]);

        game.resolve_wall_and_self_collisions();

        assert!(!game.snakes[0].alive);
    }

    #[test]
    fn eating_body_segment_truncates_victim_and_grows_eater() {
        let eater = snake_with_body(
            0,
            vec![
                Point { x: 9, y: 10 },
                Point { x: 8, y: 10 },
                Point { x: 7, y: 10 },
            ],
            true,
        );
        let victim = snake_with_body(
            1,
            vec![
                Point { x: 12, y: 10 },
                Point { x: 11, y: 10 },
                Point { x: 10, y: 10 },
                Point { x: 9, y: 10 },
            ],
            false,
        );
        let mut game = make_game(vec![eater, victim]);

        game.resolve_snake_eating();

        assert!(game.snakes[0].pending_growth > 0);
        assert!(game.snakes[1].len() < 4 || !game.snakes[1].alive);
    }
}
