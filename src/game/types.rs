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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PatternType {
    Solid,
    Stripes,
    Dots,
    Gradient,
    Scales,
}

#[derive(Clone, Copy)]
pub(crate) struct RGB {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
}

impl RGB {
    pub(crate) fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone)]
pub(crate) struct Skin {
    pub(crate) base_color: RGB,
    pub(crate) pattern_type: PatternType,
    pub(crate) pattern_color: RGB,
    pub(crate) eye_style: EyeStyle,
    #[allow(dead_code)]
    pub(crate) seed: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EyeStyle {
    Round,
    Oval,
    Angular,
    None,
}

pub(crate) fn generate_skin(index: usize, is_player: bool, seed: u64) -> Skin {
    let pattern_variants = [
        PatternType::Solid,
        PatternType::Stripes,
        PatternType::Dots,
        PatternType::Gradient,
        PatternType::Scales,
    ];

    let pattern_idx = (index + (seed as usize)) % pattern_variants.len();
    let pattern_type = pattern_variants[pattern_idx];

    let base_colors: Vec<RGB> = if is_player {
        vec![
            RGB::new(0.0, 1.0, 1.0),   // Cyan
            RGB::new(0.0, 0.8, 1.0),   // Light blue
            RGB::new(0.2, 0.9, 1.0),   // Sky blue
        ]
    } else {
        vec![
            RGB::new(1.0, 0.2, 0.2),   // Red
            RGB::new(0.2, 1.0, 0.2),   // Green
            RGB::new(1.0, 0.0, 1.0),   // Magenta
            RGB::new(0.2, 0.6, 1.0),   // Blue
            RGB::new(1.0, 0.8, 0.0),   // Yellow
            RGB::new(1.0, 0.4, 0.0),   // Orange
        ]
    };

    let color_idx = index % base_colors.len();
    let base_color = base_colors[color_idx];

    let pattern_color = match pattern_type {
        PatternType::Solid => base_color,
        PatternType::Stripes => RGB::new(
            (base_color.r * 0.7).max(0.3),
            (base_color.g * 0.7).max(0.3),
            (base_color.b * 0.7).max(0.3),
        ),
        PatternType::Dots => RGB::new(
            (base_color.r + 0.3).min(1.0),
            (base_color.g + 0.3).min(1.0),
            (base_color.b + 0.3).min(1.0),
        ),
        PatternType::Gradient => RGB::new(
            (base_color.r * 1.3).min(1.0),
            (base_color.g * 1.3).min(1.0),
            (base_color.b * 1.3).min(1.0),
        ),
        PatternType::Scales => RGB::new(
            (base_color.r * 0.85).max(0.1),
            (base_color.g * 0.85).max(0.1),
            (base_color.b * 0.85).max(0.1),
        ),
    };

    let eye_styles = [
        EyeStyle::Round,
        EyeStyle::Oval,
        EyeStyle::Angular,
        EyeStyle::None,
    ];
    let eye_idx = (index + (seed as usize) / 7) % eye_styles.len();
    let eye_style = eye_styles[eye_idx];

    Skin {
        base_color,
        pattern_type,
        pattern_color,
        eye_style,
        seed,
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
    #[allow(dead_code)]
    pub(crate) color: Color,
    pub(crate) skin: Skin,
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

    let skin = generate_skin(id, is_player, id as u64);

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
        skin,
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

#[cfg(test)]
mod tests {
    use crossterm::style::Color;

    use super::{bot_color, make_snake, manhattan, Difficulty, Direction, Point};

    #[test]
    fn point_add_uses_direction_delta() {
        let p = Point { x: 10, y: 5 };
        let moved = p.add(Direction::Left);
        assert_eq!(moved.x, 9);
        assert_eq!(moved.y, 5);
    }

    #[test]
    fn opposite_direction_is_inverse() {
        assert!(matches!(Direction::Up.opposite(), Direction::Down));
        assert!(matches!(Direction::Down.opposite(), Direction::Up));
        assert!(matches!(Direction::Left.opposite(), Direction::Right));
        assert!(matches!(Direction::Right.opposite(), Direction::Left));
    }

    #[test]
    fn snake_rejects_immediate_reverse_turn() {
        let mut snake = make_snake(0, Point { x: 4, y: 4 }, Direction::Right, true, Color::Cyan);
        snake.set_direction(Direction::Left);
        assert!(matches!(snake.next_dir, Direction::Right));
    }

    #[test]
    fn snake_moves_and_grows_when_pending_growth() {
        let mut snake = make_snake(0, Point { x: 4, y: 4 }, Direction::Right, true, Color::Cyan);
        let initial_len = snake.len();
        snake.pending_growth = 1;
        snake.move_forward();
        assert_eq!(snake.len(), initial_len + 1);
        assert_eq!(snake.pending_growth, 0);
    }

    #[test]
    fn difficulty_profiles_are_ordered_by_speed_and_bots() {
        let easy = Difficulty::Easy.settings();
        let normal = Difficulty::Normal.settings();
        let hard = Difficulty::Hard.settings();

        assert!(easy.tick_ms > normal.tick_ms);
        assert!(normal.tick_ms > hard.tick_ms);
        assert!(easy.bot_count < normal.bot_count);
        assert!(normal.bot_count < hard.bot_count);
    }

    #[test]
    fn manhattan_distance_is_correct() {
        let a = Point { x: 1, y: 2 };
        let b = Point { x: 4, y: -2 };
        assert_eq!(manhattan(a, b), 7);
    }

    #[test]
    fn bot_color_cycles_every_six() {
        assert_eq!(bot_color(0), bot_color(6));
        assert_eq!(bot_color(1), bot_color(7));
    }
}
