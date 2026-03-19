use std::collections::VecDeque;

use crossterm::style::Color;

pub(crate) const WIDTH: i32 = 48;
pub(crate) const HEIGHT: i32 = 20;
pub(crate) const MAX_BOTS: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Point {
    pub(crate) fn add(self, d: Direction) -> Self {
        let (dx, dy) = d.delta();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub(crate) fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    pub(crate) fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

pub(crate) struct Snake {
    pub(crate) id: usize,
    pub(crate) body: VecDeque<Point>,
    pub(crate) dir: Direction,
    pub(crate) next_dir: Direction,
    pub(crate) pending_growth: usize,
    pub(crate) is_player: bool,
    pub(crate) alive: bool,
    pub(crate) eliminated_at: Option<u64>,
    pub(crate) color: Color,
}

impl Snake {
    pub(crate) fn len(&self) -> usize {
        self.body.len()
    }

    pub(crate) fn head(&self) -> Option<Point> {
        self.body.front().copied()
    }

    pub(crate) fn set_direction(&mut self, new_dir: Direction) {
        if new_dir != self.dir.opposite() {
            self.next_dir = new_dir;
        }
    }

    pub(crate) fn move_forward(&mut self) {
        if !self.alive {
            return;
        }
        self.dir = self.next_dir;
        if let Some(head) = self.head() {
            self.body.push_front(head.add(self.dir));
            if self.pending_growth > 0 {
                self.pending_growth -= 1;
            } else {
                self.body.pop_back();
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Clone, Copy)]
pub(crate) struct GameSettings {
    pub(crate) difficulty: Difficulty,
    pub(crate) fruit_count: usize,
    pub(crate) bot_count: usize,
    pub(crate) tick_ms: u64,
    pub(crate) game_time_seconds: u64,
    pub(crate) bot_aggression: f32,
}

impl GameSettings {
    pub(crate) fn label(self) -> &'static str {
        match self.difficulty {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }
}

impl Difficulty {
    pub(crate) fn settings(self) -> GameSettings {
        match self {
            Difficulty::Easy => GameSettings {
                difficulty: self,
                fruit_count: 10,
                bot_count: 2,
                tick_ms: 145,
                game_time_seconds: 75,
                bot_aggression: 0.55,
            },
            Difficulty::Normal => GameSettings {
                difficulty: self,
                fruit_count: 8,
                bot_count: 3,
                tick_ms: 120,
                game_time_seconds: 90,
                bot_aggression: 0.85,
            },
            Difficulty::Hard => GameSettings {
                difficulty: self,
                fruit_count: 6,
                bot_count: 5,
                tick_ms: 95,
                game_time_seconds: 100,
                bot_aggression: 1.25,
            },
        }
    }
}

pub(crate) fn in_bounds(p: Point) -> bool {
    p.x >= 0 && p.x < WIDTH && p.y >= 0 && p.y < HEIGHT
}

pub(crate) fn manhattan(a: Point, b: Point) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

pub(crate) fn make_snake(
    id: usize,
    start: Point,
    dir: Direction,
    is_player: bool,
    color: Color,
) -> Snake {
    let mut body = VecDeque::new();
    body.push_back(start);
    body.push_back(start.add(dir.opposite()));
    body.push_back(start.add(dir.opposite()).add(dir.opposite()));

    Snake {
        id,
        body,
        dir,
        next_dir: dir,
        pending_growth: 0,
        is_player,
        alive: true,
        eliminated_at: None,
        color,
    }
}

pub(crate) fn bot_color(index: usize) -> Color {
    match index % 6 {
        0 => Color::Red,
        1 => Color::Green,
        2 => Color::Magenta,
        3 => Color::Blue,
        4 => Color::DarkYellow,
        _ => Color::DarkCyan,
    }
}
