//! The kitten, its desk and its little computer.
//!
//! Everything is drawn in a 300 x 250 design space. `update` advances the animation
//! state (blinks, ear twitches, paw positions); `render` draws one frame from it.

use std::f32::consts::PI;

use tiny_skia::{Color, Path, Pixmap, Transform};

use crate::canvas::*;

pub const W: f32 = 300.0;
pub const H: f32 = 250.0;

const CX: f32 = 105.0; // kitten center line

/// A kitten's colors (right-click > Color; `coat=` in kitten-assistant.ini).
#[derive(Clone, Copy)]
pub struct Coat {
    pub name: &'static str,
    pub fur: Rgba,
    /// tabby stripes and the tail tip
    pub dark: Rgba,
    /// muzzle and tummy
    pub cream: Rgba,
    /// inner ears and nose
    pub pink: Rgba,
    pub eye: Rgba,
    pub stripes: bool,
    /// a patch over one ear and the forehead, one on the tummy side (calico, cow, tortie)
    pub patch: Option<Rgba>,
    /// patches alternate the patch color and `dark` (calico)
    pub tri: bool,
    /// the special coats' markings
    pub marks: Marks,
}

/// Markings drawn on the head and tummy sides (and the tail, for the rainbow).
#[derive(Clone, Copy)]
pub enum Marks {
    Plain,
    /// rainbow stripes and a rainbow-banded tail
    Rainbow,
    /// nebula glows and little stars
    Galaxy,
    Dots(Rgba),
    Hearts(Rgba),
    /// white clouds
    Clouds,
    /// seed dots and a leafy top
    Strawberry,
    /// a soft glow on the cheeks and tummy sides
    Blush(Rgba),
    /// see-through fur
    Ghost,
}

const RAINBOW: [Rgba; 6] = [
    Rgba::rgb(240, 84, 84),
    Rgba::rgb(250, 156, 60),
    Rgba::rgb(250, 214, 70),
    Rgba::rgb(110, 200, 110),
    Rgba::rgb(90, 156, 236),
    Rgba::rgb(160, 110, 220),
];

/// The galaxy coat's stars: (x, y, size) in -1..1 of the area's radii; size above 1.5 twinkles as a cross.
const HEAD_STARS: [(f32, f32, f32); 9] =
    [(-0.62, -0.55, 2.4), (-0.25, -0.85, 1.0), (0.18, -0.7, 1.6), (0.55, -0.45, 2.6), (0.78, -0.05, 1.0), (-0.82, 0.05, 1.2), (0.35, -0.1, 0.8), (-0.4, -0.25, 0.9), (0.05, -0.35, 2.0)];
const BODY_STARS: [(f32, f32, f32); 6] = [(-0.75, -0.3, 1.8), (-0.6, 0.35, 1.0), (0.7, -0.45, 1.0), (0.75, 0.2, 2.2), (-0.8, 0.0, 0.8), (0.6, 0.6, 1.2)];

const fn coat(name: &'static str, fur: (u8, u8, u8), dark: (u8, u8, u8), cream: (u8, u8, u8), stripes: bool) -> Coat {
    Coat {
        name,
        fur: Rgba::rgb(fur.0, fur.1, fur.2),
        dark: Rgba::rgb(dark.0, dark.1, dark.2),
        cream: Rgba::rgb(cream.0, cream.1, cream.2),
        pink: Rgba::rgb(255, 170, 186),
        eye: Rgba::rgb(50, 34, 36),
        stripes,
        patch: None,
        tri: false,
        marks: Marks::Plain,
    }
}

const fn with_marks(c: Coat, marks: Marks) -> Coat {
    Coat { marks, ..c }
}

const fn rgb(c: (u8, u8, u8)) -> Rgba {
    Rgba::rgb(c.0, c.1, c.2)
}

const fn with_patch(c: Coat, patch: (u8, u8, u8)) -> Coat {
    Coat { patch: Some(Rgba::rgb(patch.0, patch.1, patch.2)), ..c }
}

const fn with_pink(c: Coat, pink: (u8, u8, u8)) -> Coat {
    Coat { pink: Rgba::rgb(pink.0, pink.1, pink.2), ..c }
}

const fn with_eye(c: Coat, eye: (u8, u8, u8)) -> Coat {
    Coat { eye: Rgba::rgb(eye.0, eye.1, eye.2), ..c }
}

/// Every coat, the default first. The menu lists them in this order.
pub const COATS: [Coat; 32] = [
    coat("Orange tabby", (247, 178, 102), (224, 136, 70), (255, 238, 216), true),
    coat("Ginger", (238, 140, 60), (198, 96, 38), (255, 226, 196), true),
    coat("Cream tabby", (250, 222, 170), (226, 186, 126), (255, 246, 230), true),
    coat("Gray tabby", (168, 170, 180), (116, 118, 132), (236, 236, 242), true),
    coat("Silver tabby", (210, 214, 222), (140, 146, 162), (248, 248, 252), true),
    coat("Brown tabby", (178, 126, 84), (118, 78, 48), (236, 214, 190), true),
    coat("Golden", (240, 196, 92), (208, 152, 52), (255, 240, 200), true),
    with_eye(with_pink(coat("Black", (58, 55, 64), (40, 38, 46), (68, 65, 75), false), (210, 128, 148)), (226, 184, 72)),
    coat("White", (250, 248, 245), (222, 218, 214), (255, 255, 255), false),
    with_eye(with_pink(coat("Tuxedo", (66, 62, 72), (40, 38, 46), (250, 248, 245), false), (220, 136, 156)), (226, 184, 72)),
    coat("Russian blue", (140, 154, 178), (106, 118, 146), (196, 206, 224), false),
    coat("Chocolate", (132, 86, 60), (96, 60, 40), (212, 182, 158), false),
    coat("Cinnamon", (198, 122, 82), (156, 86, 54), (240, 210, 186), false),
    coat("Lilac", (198, 182, 198), (160, 142, 164), (242, 234, 242), false),
    coat("Fawn", (226, 196, 164), (196, 160, 124), (250, 238, 224), false),
    coat("Seal point", (238, 222, 200), (102, 74, 60), (250, 242, 230), false),
    with_eye(coat("Blue point", (236, 234, 238), (132, 142, 166), (250, 250, 252), false), (40, 64, 110)),
    Coat { tri: true, ..with_patch(coat("Calico", (250, 246, 240), (60, 54, 58), (255, 255, 255), false), (236, 146, 66)) },
    with_eye(with_patch(coat("Tortoiseshell", (74, 58, 54), (48, 38, 36), (120, 96, 84), false), (214, 128, 60)), (226, 184, 72)),
    with_patch(coat("Cow cat", (250, 248, 245), (52, 50, 56), (255, 255, 255), false), (52, 50, 56)),
    Coat { tri: true, ..with_patch(coat("Dilute calico", (246, 242, 238), (150, 150, 166), (255, 255, 255), false), (240, 200, 160)) },
    with_marks(coat("Cotton candy", (255, 190, 216), (240, 140, 186), (196, 226, 255), false), Marks::Hearts(rgb((255, 255, 255)))),
    with_marks(coat("Lavender", (200, 176, 240), (156, 128, 208), (242, 234, 252), true), Marks::Dots(rgb((255, 255, 255)))),
    with_marks(coat("Mint chip", (178, 234, 208), (120, 196, 162), (236, 252, 244), false), Marks::Dots(rgb((104, 70, 52)))),
    with_marks(coat("Sky", (140, 198, 246), (96, 160, 220), (236, 246, 255), false), Marks::Clouds),
    with_marks(coat("Lemon", (252, 230, 110), (232, 196, 52), (255, 250, 222), true), Marks::Dots(rgb((255, 250, 214)))),
    with_marks(coat("Peach", (255, 204, 172), (242, 160, 126), (255, 240, 228), false), Marks::Blush(Rgba(255, 140, 120, 90))),
    with_marks(with_pink(coat("Strawberry", (236, 76, 92), (196, 48, 64), (255, 214, 220), false), (255, 200, 210)), Marks::Strawberry),
    with_marks(coat("Grape", (148, 98, 192), (108, 64, 150), (226, 208, 244), false), Marks::Dots(rgb((214, 186, 248)))),
    with_marks(with_pink(with_eye(coat("Galaxy", (38, 36, 86), (24, 22, 62), (70, 62, 140), false), (240, 210, 110)), (200, 150, 236)), Marks::Galaxy),
    with_marks(Coat { fur: Rgba(236, 242, 255, 212), cream: Rgba(255, 255, 255, 224), ..with_eye(coat("Ghost", (236, 242, 255), (200, 210, 236), (255, 255, 255), false), (80, 110, 190)) }, Marks::Ghost),
    with_marks(coat("Rainbow", (255, 246, 240), (236, 110, 140), (255, 255, 255), false), Marks::Rainbow),
];

impl Coat {
    /// The coat called `name` (any case), else the default.
    pub fn named(name: &str) -> Coat {
        COATS.iter().find(|c| c.name.eq_ignore_ascii_case(name.trim())).copied().unwrap_or(COATS[0])
    }
}

// palette
const PINK_DARK: Rgba = Rgba::rgb(226, 112, 134);
const BLUSH: Rgba = Rgba(255, 128, 150, 95);
const WHITE: Rgba = Rgba::rgb(255, 255, 255);
const WOOD_TOP: Rgba = Rgba::rgb(216, 162, 110);
const WOOD_FRONT: Rgba = Rgba::rgb(178, 120, 78);
const CASE: Rgba = Rgba::rgb(240, 230, 210);
const CASE_SHADE: Rgba = Rgba::rgb(214, 202, 178);
const SCREEN_ON: Rgba = Rgba::rgb(36, 54, 66);
const SCREEN_OFF: Rgba = Rgba::rgb(42, 40, 60);
const SCREEN_BAD: Rgba = Rgba::rgb(66, 34, 42);
const GREEN: Rgba = Rgba::rgb(110, 228, 140);
const RED: Rgba = Rgba::rgb(240, 92, 92);
const SAND: Rgba = Rgba::rgb(255, 214, 102);
const CODE: [Rgba; 4] = [
    Rgba::rgb(126, 232, 170),
    Rgba::rgb(255, 160, 204),
    Rgba::rgb(255, 222, 122),
    Rgba::rgb(140, 202, 255),
];
const KB_BODY: Rgba = Rgba::rgb(110, 116, 150);
const KB_FRONT: Rgba = Rgba::rgb(82, 86, 116);
const KEY: Rgba = Rgba::rgb(240, 240, 250);
const KEY_SHADE: Rgba = Rgba::rgb(184, 186, 212);
const KEY_HOT: Rgba = Rgba::rgb(255, 214, 102);
const MUG: Rgba = Rgba::rgb(132, 202, 232);
const SUB: Rgba = Rgba::rgb(160, 112, 84);
const ZZZ: Rgba = Rgba::rgb(150, 138, 214);
const SPARK: Rgba = Rgba::rgb(255, 200, 70);

// keyboard layout
const KX: f32 = 40.0;
const KY: f32 = 188.0;
const KW: f32 = 130.0;
const KEY_PITCH: f32 = 11.5;
const KEY_W: f32 = 9.5;
const KEY_H: f32 = 3.6;
const PAW_W: f32 = 20.0;

fn key_x(col: usize) -> f32 {
    KX + 3.5 + col as f32 * KEY_PITCH
}

fn hash(n: i64) -> u32 {
    let mut v = (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    v ^= v >> 29;
    v = v.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    v ^= v >> 32;
    (v % 1000) as u32
}

/// What a busy status line asks for, from its first word.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    /// typing away
    Busy,
    /// `Waiting...`: sits back, taps a paw, hourglass on the screen
    Waiting,
    /// `Failed...` / `Error...`: ears down, red X on the screen
    Problem,
    /// `Testing...` / `Checking...`: plays a little game with a controller and jots notes, like a game tester
    Testing,
    /// `Building...` / `Compiling...`: types in a hard hat while a brick wall goes up on the screen
    Building,
    /// `Researching...` / `Reading...`: reading glasses on, leaning in to a page scrolling on the screen
    Researching,
    /// `Downloading...` / `Uploading...`: types while files fly between a cloud and the screen (true = up)
    Transfer(bool),
    /// `Thinking...` / `Planning...`: draws boxes and arrows on a whiteboard beside the desk, a checklist on the
    /// screen
    Thinking,
}

impl Mood {
    fn of(s: &str) -> Mood {
        let word: String = s.chars().take_while(|c| c.is_alphabetic()).collect::<String>().to_lowercase();
        match word.as_str() {
            "waiting" | "wait" => Mood::Waiting,
            "failed" | "fail" | "error" | "problem" => Mood::Problem,
            "testing" | "test" | "tests" | "playtesting" | "playtest" | "checking" | "check" | "verifying" | "verify"
            | "qa" | "autotest" | "regression" | "smoke" => Mood::Testing,
            "building" | "build" | "rebuilding" | "rebuild" | "compiling" | "compile" | "linking" | "packaging"
            | "bundling" => Mood::Building,
            "researching" | "research" | "reading" | "read" | "reviewing" | "review" | "analyzing" | "analyze"
            | "searching" | "search" | "investigating" | "investigate" | "studying" | "exploring" | "digging" => Mood::Researching,
            "downloading" | "download" | "fetching" | "fetch" | "pulling" | "pull" | "installing" | "install"
            | "syncing" | "sync" | "copying" | "copy" => Mood::Transfer(false),
            "uploading" | "upload" | "pushing" | "push" | "deploying" | "deploy" | "publishing" | "publish"
            | "backing" | "backup" => Mood::Transfer(true),
            "thinking" | "think" | "planning" | "plan" | "designing" | "design" | "brainstorming" | "pondering" => Mood::Thinking,
            _ => Mood::Busy,
        }
    }

    /// Moods that type on the keyboard.
    fn types(self) -> bool {
        matches!(self, Mood::Busy | Mood::Building | Mood::Transfer(_))
    }

    /// Moods that sit with the chin propped on a paw.
    fn chin(self) -> bool {
        matches!(self, Mood::Waiting)
    }
}

/// Seconds of a planning round, the first `PLAN_DRAW` of them drawing on the whiteboard.
const PLAN_ROUND: f64 = 8.5;
const PLAN_DRAW: f64 = 6.5;
/// The whiteboard on its easel left of the kitten: x, y, width, height.
const BOARD: (f32, f32, f32, f32) = (0.0, 100.0, 76.0, 90.0);
/// The part of the board the drawing uses: left of the kitten's head (x, y, width, height).
const BOARD_ART: (f32, f32, f32, f32) = (5.0, 108.0, 50.0, 72.0);

type Stroke = Vec<(f32, f32)>;

/// One whiteboard drawing, picked and varied by `seed`: a flowchart, bar chart, rising line graph, mind map,
/// roadmap with a flag, pie chart, rocket, Venn diagram or light bulb. Coordinates are pixels in the board's
/// drawing area (`BOARD_ART`, 50 x 72), strokes in drawing order. Also returns the flowchart's box centers (for its
/// easter egg), empty for the other drawings.
fn plan_drawing(seed: u32) -> (Vec<Stroke>, Vec<(f32, f32)>) {
    let mut anchors = Vec::new();
    let r = |i: i64| (hash(seed as i64 * 977 + i) % 1000) as f32 / 1000.0;
    let arc = |cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32| -> Stroke {
        (0..=28).map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / 28.0;
            (cx + rx * a.cos(), cy + ry * a.sin())
        }).collect()
    };
    let rect = |x0: f32, y0: f32, x1: f32, y1: f32| vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)];
    let arrow = |out: &mut Vec<Stroke>, a: (f32, f32), b: (f32, f32)| {
        out.push(vec![a, b]);
        let ang = (b.1 - a.1).atan2(b.0 - a.0);
        let head = |s: f32| (b.0 - 4.5 * (ang + s).cos(), b.1 - 4.5 * (ang + s).sin());
        out.push(vec![head(0.45), b, head(-0.45)]);
    };
    let axes = || vec![(4.0, 4.0), (4.0, 66.0), (48.0, 66.0)];
    let mut out: Vec<Stroke> = Vec::new();
    match hash(seed as i64) % 9 {
        0 => {
            // flowchart: a chain of boxes, nudged left and right, joined by arrows
            let n = 3 + (r(1) * 2.0) as usize;
            let step = 68.0 / n as f32;
            let boxes: Vec<(f32, f32, f32)> = (0..n).map(|i| (25.0 + (r(10 + i as i64) - 0.5) * 14.0, 2.0 + i as f32 * step, 26.0 + r(20 + i as i64) * 10.0)).collect();
            let h = step * 0.45;
            for (i, &(cx, y, w)) in boxes.iter().enumerate() {
                anchors.push((cx, y + h / 2.0));
                out.push(rect(cx - w / 2.0, y, cx + w / 2.0, y + h));
                if let Some(&(nx, ny, _)) = boxes.get(i + 1) {
                    arrow(&mut out, (cx, y + h), (nx, ny));
                }
            }
        }
        1 => {
            // bar chart with a trend arrow over the bars
            out.push(axes());
            let n = 3 + (r(1) * 3.0) as usize;
            let w = 40.0 / n as f32;
            let mut tops = Vec::new();
            for i in 0..n {
                let x = 7.0 + i as f32 * w;
                let h = 10.0 + r(30 + i as i64) * 46.0;
                out.push(vec![(x, 66.0), (x, 66.0 - h), (x + w * 0.62, 66.0 - h), (x + w * 0.62, 66.0)]);
                tops.push((x + w * 0.31, 66.0 - h - 5.0));
            }
            if r(2) > 0.4 {
                arrow(&mut out, (8.0, 8.0 + r(3) * 6.0 + 30.0), (46.0, 6.0));
            }
        }
        2 => {
            // a line graph that climbs, ending in an arrow
            out.push(axes());
            let n = 5 + (r(1) * 3.0) as usize;
            let pts: Vec<(f32, f32)> = (0..n).map(|i| {
                let u = i as f32 / (n - 1) as f32;
                (8.0 + u * 36.0, 60.0 - u * 46.0 + (r(40 + i as i64) - 0.5) * 16.0)
            }).collect();
            let last = pts[n - 1];
            let before = pts[n - 2];
            out.push(pts);
            arrow(&mut out, before, (last.0 + 3.0, last.1 - 3.0));
        }
        3 => {
            // mind map: an idea in the middle, branches out to little bubbles
            let (cx, cy) = (25.0, 36.0);
            out.push(arc(cx, cy, 10.0, 6.5, 0.0, 2.0 * PI));
            let n = 4 + (r(1) * 3.0) as usize;
            let spin = r(2) * PI;
            for i in 0..n {
                let a = spin + i as f32 / n as f32 * 2.0 * PI;
                let d = 19.0 + r(50 + i as i64) * 6.0;
                let (nx, ny) = ((cx + a.cos() * d).clamp(5.0, 45.0), (cy + a.sin() * d * 1.25).clamp(5.0, 67.0));
                out.push(vec![(cx + a.cos() * 10.0, cy + a.sin() * 6.5), (nx - a.cos() * 4.0, ny - a.sin() * 4.0)]);
                out.push(arc(nx, ny, 4.0, 4.0, 0.0, 2.0 * PI));
            }
        }
        4 => {
            // roadmap: a winding road of milestones up to a flag
            let pts: Vec<(f32, f32)> = (0..4).map(|i| (if i % 2 == 0 { 12.0 } else { 38.0 } + (r(60 + i) - 0.5) * 8.0, 64.0 - i as f32 * 17.0)).collect();
            out.push(pts.clone());
            for &(x, y) in &pts[..3] {
                out.push(arc(x, y, 3.0, 3.0, 0.0, 2.0 * PI));
            }
            let (fx, fy) = pts[3];
            out.push(vec![(fx, fy), (fx, fy - 14.0)]);
            out.push(vec![(fx, fy - 14.0), (fx + 10.0, fy - 10.5), (fx, fy - 7.0)]);
        }
        5 => {
            // pie chart with a slice pulled out
            let (cx, cy, rr) = (25.0, 36.0, 18.0);
            out.push(arc(cx, cy, rr, rr, 0.0, 2.0 * PI));
            let n = 3 + (r(1) * 2.0) as usize;
            let mut a = r(2) * 2.0 * PI;
            for i in 0..n {
                out.push(vec![(cx, cy), (cx + a.cos() * rr, cy + a.sin() * rr)]);
                a += (0.6 + r(70 + i as i64) * 1.2) * PI / n as f32 * 2.0 * 0.6;
            }
        }
        6 => {
            // a rocket, ready for launch
            let tilt = (r(1) - 0.5) * 6.0;
            out.push(vec![(25.0 + tilt, 4.0), (33.0, 22.0), (33.0, 48.0), (17.0, 48.0), (17.0, 22.0), (25.0 + tilt, 4.0)]);
            out.push(arc(25.0, 25.0, 4.0, 4.0, 0.0, 2.0 * PI));
            out.push(vec![(17.0, 38.0), (10.0, 52.0), (17.0, 48.0)]);
            out.push(vec![(33.0, 38.0), (40.0, 52.0), (33.0, 48.0)]);
            out.push(vec![(20.0, 50.0), (21.5, 60.0), (25.0, 54.0), (28.5, 64.0), (30.0, 50.0)]);
        }
        7 => {
            // Venn diagram, two or three circles
            let three = r(1) > 0.5;
            let rr = 13.0;
            out.push(arc(18.0, 30.0, rr, rr, 0.0, 2.0 * PI));
            out.push(arc(32.0, 30.0, rr, rr, 0.0, 2.0 * PI));
            if three {
                out.push(arc(25.0, 44.0, rr, rr, 0.0, 2.0 * PI));
            }
            out.push(vec![(23.0, 34.0 + if three { 3.0 } else { 0.0 }), (25.0, 36.5 + if three { 3.0 } else { 0.0 }), (28.0, 31.0 + if three { 3.0 } else { 0.0 })]);
        }
        _ => {
            // a light bulb idea, rays and all
            let (cx, cy) = (25.0, 28.0);
            out.push(arc(cx, cy, 14.0, 14.0, 0.75 * PI, 2.25 * PI));
            out.push(vec![(cx - 9.9, cy + 9.9), (cx - 6.0, cy + 20.0), (cx + 6.0, cy + 20.0), (cx + 9.9, cy + 9.9)]);
            out.push(vec![(cx - 6.0, cy + 24.0), (cx + 6.0, cy + 24.0)]);
            out.push(vec![(cx - 4.0, cy + 28.0), (cx + 4.0, cy + 28.0)]);
            out.push(vec![(cx - 4.0, cy + 6.0), (cx - 2.0, cy), (cx, cy + 6.0), (cx + 2.0, cy), (cx + 4.0, cy + 6.0)]);
            for i in 0..5 {
                let a = PI * (1.1 + 0.2 * i as f32) + (r(80 + i) - 0.5) * 0.1;
                out.push(vec![(cx + a.cos() * 18.0, cy + a.sin() * 18.0), (cx + a.cos() * 23.0, cy + a.sin() * 23.0)]);
            }
        }
    }
    (out, anchors)
}

/// Whether this round's whiteboard drawing gets its easter egg (about 6 rounds in 10).
fn plan_egg(seed: u32) -> bool {
    hash(seed as i64 * 31 + 7) % 10 < 6
}

/// The whiteboard drawing for `seed`, drawn `k` (0..1) of the way, and where the marker's tip is (board pixels).
fn plan_strokes(k: f32, seed: u32) -> (Vec<Stroke>, (f32, f32)) {
    let (ax, ay, ..) = BOARD_ART;
    let all: Vec<Stroke> = plan_drawing(seed).0.into_iter().map(|s| s.into_iter().map(|(x, y)| (ax + x, ay + y)).collect()).collect();
    let seg = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    let total: f32 = all.iter().flat_map(|s| s.windows(2).map(|w| seg(w[0], w[1]))).sum();
    let mut left = total * k.clamp(0.0, 1.0);
    let mut out = Vec::new();
    let mut tip = all[0][0];
    for s in &all {
        if left <= 0.0 {
            break;
        }
        let mut part = vec![s[0]];
        for w in s.windows(2) {
            let d = seg(w[0], w[1]);
            if left >= d {
                part.push(w[1]);
                left -= d;
            } else {
                let u = left / d;
                part.push((w[0].0 + (w[1].0 - w[0].0) * u, w[0].1 + (w[1].1 - w[0].1) * u));
                left = 0.0;
                break;
            }
        }
        tip = *part.last().unwrap_or(&s[0]);
        out.push(part);
    }
    (out, tip)
}

/// Pixel sprites for the little extras: `X` takes the given color, `K` ink.
const INVADER_A: [&str; 8] =
    ["..X.....X..", "...X...X...", "..XXXXXXX..", ".XX.XXX.XX.", "XXXXXXXXXXX", "X.XXXXXXX.X", "X.X.....X.X", "...XX.XX..."];
const INVADER_B: [&str; 8] =
    ["..X.....X..", "X..X...X..X", "X.XXXXXXX.X", "XXX.XXX.XXX", "XXXXXXXXXXX", ".XXXXXXXXX.", "..X.....X..", ".X.......X."];
const PIXEL_CAT: [&str; 5] = ["X...X", "XXXXX", "XKXKX", "XXXXX", ".X.X."];

fn pixels(c: &mut Canvas, rows: &[&str], x: f32, y: f32, px: f32, col: Rgba) {
    for (r, line) in rows.iter().enumerate() {
        for (k, ch) in line.chars().enumerate() {
            let fill = match ch {
                'X' => col,
                'K' => INK,
                _ => continue,
            };
            c.fill(&rrect(x + k as f32 * px, y + r as f32 * px, px, px, 0.0), fill);
        }
    }
}

/// The point `u` (0..1) of the way along a polyline.
fn along(pts: &[(f32, f32)], u: f32) -> (f32, f32) {
    let seg = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    let total: f32 = pts.windows(2).map(|w| seg(w[0], w[1])).sum();
    let mut left = total * u.clamp(0.0, 1.0);
    for w in pts.windows(2) {
        let d = seg(w[0], w[1]);
        if left <= d && d > 0.0 {
            let k = left / d;
            return (w[0].0 + (w[1].0 - w[0].0) * k, w[0].1 + (w[1].1 - w[0].1) * k);
        }
        left -= d;
    }
    pts[pts.len() - 1]
}

/// Seconds of a testing round: playing, then jotting notes for the last `NOTES_SECS`.
const TEST_ROUND: f64 = 6.5;
const NOTES_SECS: f64 = 2.2;
/// Where the notepad lies on the desk.
const PAD: (f32, f32) = (12.0, 192.0);
/// The gamepad's center while testing: held up in front of the chest.
const PAD_Y: f32 = 183.0;
/// The end of the right grip, in the pad's own frame (the left one mirrors it); the paws hold these.
const GRIP: (f32, f32) = (30.0, 12.0);

/// A point of the gamepad (its own frame, center 0,0) on the canvas, bobbing a little with the game.
fn pad_point(t: f32, x: f32, y: f32) -> (f32, f32) {
    (CX + x, PAD_Y + y + 0.8 * (t * 3.2).sin())
}

/// Typing screens: scrolling code, progress bars, equalizer, gear, line graph.
const SCREENS: usize = 5;

/// Seconds a coffee sip takes.
const SIP_SECS: f64 = 2.4;
/// Seconds an empty-mug refill takes: lift, nothing there, annoyed, kettle from off the picture, pour, pleased.
const REFILL_SECS: f64 = 7.0;

/// Where the right paw holds the empty mug out for the kettle (mug center), in front of the chest on the left.
const REFILL_HOLD: (f32, f32) = (78.0, 176.0);

fn lerp2(a: (f32, f32), b: (f32, f32), k: f32) -> (f32, f32) {
    let k = smooth(k);
    (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k)
}

/// The mug during a refill (0..1): up to the mouth, a look inside, over to the left for the kettle, held there while
/// the kettle comes and goes, then back home.
fn refill_mug(p: f32, hy: f32) -> (f32, f32) {
    if p < 0.20 {
        mug_at(p / 0.20 * 0.40, hy)
    } else if p < 0.30 {
        mug_at(0.5, hy)
    } else if p < 0.40 {
        lerp2(mug_at(0.5, hy), REFILL_HOLD, (p - 0.30) / 0.10)
    } else if p < 0.90 {
        REFILL_HOLD
    } else if p < 0.96 {
        lerp2(REFILL_HOLD, MUG_HOME, (p - 0.90) / 0.06)
    } else {
        MUG_HOME
    }
}
/// Center of the mug on the desk.
const MUG_HOME: (f32, f32) = (179.0, 200.0);
/// How much bigger the mug is drawn than its outline numbers.
const MUG_SCALE: f32 = 1.35;

/// A light muzzle / tummy over patches: solid in the middle with a soft edge (`grow` 1.0 keeps the
/// soft edge inside the given oval), so the patches around it fade into it.
fn soft_cream(c: &mut Canvas, x: f32, y: f32, w: f32, h: f32, col: Rgba, grow: f32) {
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    let rings = [(1.0, 0.22), (0.96, 0.32), (0.92, 0.45), (0.88, 0.65), (0.84, 1.0)];
    for (k, a) in rings.map(|(k, a)| (k * grow, a)) {
        c.fill(&oval(cx - w * k / 2.0, cy - h * k / 2.0, w * k, h * k), col.fade(a));
    }
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Mug center `p` (0..1) of the way through a sip: on the desk, up to the mouth, a sip, back down.
fn mug_at(p: f32, hy: f32) -> (f32, f32) {
    let a = if p < 0.15 {
        0.0
    } else if p < 0.35 {
        (p - 0.15) / 0.2
    } else if p < 0.68 {
        1.0
    } else if p < 0.88 {
        1.0 - (p - 0.68) / 0.2
    } else {
        0.0
    };
    // straight up first, then across to the mouth, so the mug clears the keyboard's corner
    let up = smooth(a / 0.55);
    let across = smooth((a - 0.35) / 0.65);
    // the mug's middle beside the mouth, so its tipped rim meets the open mouth's corner without hiding it
    let mouth = (CX + 20.0, hy + 21.0);
    (MUG_HOME.0 + (mouth.0 - MUG_HOME.0) * across, MUG_HOME.1 + (mouth.1 - MUG_HOME.1) * up)
}

/// Seconds the "all done!" moment lasts after busy turns idle.
const CHEER_SECS: f64 = 2.5;

/// Per-frame pose worked out by `update`.
pub struct Pose {
    hy: f32,
    sway: f32,
    eyes_closed: bool,
    look: f32,
    twitch: f32,
    twitch_ear: f32,
    l: Paw,
    r: Paw,
    /// 0..1 through the "all done!" moment, None outside it
    cheer: Option<f32>,
    /// 0 = ears up, 1 = ears down
    droop: f32,
    /// 0 = eyes wide open, 1 = lids all the way down
    lid: f32,
    /// 0..1 through a coffee sip, None outside one
    sip: Option<f32>,
    /// where the mug's center is
    mug: (f32, f32),
    /// 0..1 through jotting notes while testing, None while playing (or another mood)
    notes: Option<f32>,
    /// 0..1 of the whiteboard drawn while planning (1 = done, held until the round ends), None in other moods
    plan: Option<(f32, u32)>,
    /// 0..1 through an empty-mug refill, None outside one
    refill: Option<f32>,
    /// the kettle in the paw during a refill: the paw's center and the kettle's tilt
    kettle: Option<(f32, f32, f32)>,
    /// how full the mug is (0 empty .. 1)
    fill: f32,
    /// the annoyed mark by the head (0 or 1)
    anger: f32,
    /// how far into a mouthful (0..1): the mouth opens and the mug tips toward it
    sipping: f32,
}

#[derive(Clone, Copy)]
struct Paw {
    x: f32,
    lift: f32,
    key: Option<usize>,
    down: bool,
}

pub struct Scene {
    pub coat: Coat,
    /// rolls where the patches sit and how they look (a new one on every color pick)
    pub patch_seed: u32,
    pub busy: bool,
    pub task: String,
    pub mood: Mood,
    /// which typing screen shows (0..SCREENS), picked at random per status line
    pub screen: usize,
    /// this status line's screen shows its rare little extra
    pub egg: bool,
    /// time (seconds) the current task started
    pub since: f64,
    /// time the last job finished (busy -> idle), for the "all done!" moment
    pub done_at: f64,
    rng: u64,
    next_blink: f64,
    next_twitch: f64,
    /// when the next coffee sip starts, and when the current one started
    pub next_sip: f64,
    pub sip_start: f64,
    /// when the current empty-mug refill started
    pub refill_start: f64,
    twitch_start: f64,
    twitch_ear: f32,
    lpx: f32,
    rpx: f32,
    last_t: f64,
    bold: Option<fontdue::Font>,
    regular: Option<fontdue::Font>,
}

pub(crate) fn load_font(names: &[&str]) -> Option<fontdue::Font> {
    let dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    names.iter().find_map(|n| {
        let bytes = std::fs::read(format!("{dir}\\Fonts\\{n}")).ok()?;
        fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).ok()
    })
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            busy: false,
            task: String::new(),
            mood: Mood::Busy,
            screen: 0,
            egg: false,
            since: 0.0,
            done_at: -100.0,
            rng: 0x2545_F491_4F6C_DD1D,
            next_blink: 2.0,
            next_twitch: 4.0,
            next_sip: 10.0,
            coat: COATS[0],
            patch_seed: 1,
            sip_start: -100.0,
            refill_start: -100.0,
            twitch_start: -10.0,
            twitch_ear: 1.0,
            lpx: -1.0,
            rpx: -1.0,
            last_t: 0.0,
            // Windows ships these; the kitten still draws without text if none load.
            bold: load_font(&["seguisb.ttf", "segoeuib.ttf", "arialbd.ttf"]),
            regular: load_font(&["segoeui.ttf", "arial.ttf"]),
        }
    }

    /// The kitten's head in `coat` (patches rolled by `seed`), eyes open, for the menu's color tiles: drawn at
    /// `sc` with the middle of the face at pixel (`cx`, `cy`).
    pub fn portrait(&mut self, coat: Coat, seed: u32, pix: &mut Pixmap, sc: f32, cx: f32, cy: f32) {
        self.coat = coat;
        self.patch_seed = seed;
        let paw = Paw { x: 0.0, lift: 0.0, key: None, down: false };
        let p = Pose {
            hy: 0.0,
            sway: 0.0,
            eyes_closed: false,
            look: 0.0,
            twitch: 0.0,
            twitch_ear: 1.0,
            l: paw,
            r: paw,
            cheer: None,
            droop: 0.0,
            lid: 0.0,
            sip: None,
            mug: MUG_HOME,
            notes: None,
            plan: None,
            refill: None,
            kettle: None,
            fill: 1.0,
            anger: 0.0,
            sipping: 0.0,
        };
        let mut c = Canvas::new_at(pix, sc, cx - CX * sc, cy + 6.0 * sc);
        self.head(&mut c, &p);
    }

    /// First line of the status file: text = busy with that, empty or "idle" = nap.
    pub fn set_status(&mut self, s: &str, t: f64) {
        self.set_status_aged(s, t, 0.0);
    }

    /// Like `set_status`, but new busy text counts as already `age` seconds old.
    pub fn set_status_aged(&mut self, s: &str, t: f64, age: f64) {
        let s = s.trim();
        let busy = !s.is_empty() && !s.eq_ignore_ascii_case("idle");
        if busy && (!self.busy || s != self.task) {
            self.since = t - age.max(0.0);
            // a new screen for the new line, never the same one twice in a row
            let pick = (self.rand() * (SCREENS - 1) as f64) as usize;
            self.screen = (self.screen + 1 + pick) % SCREENS;
            self.egg = self.rand() < 0.1;
        }
        // a job that ends without a problem gets a little celebration
        if self.busy && !busy && self.mood != Mood::Problem {
            self.done_at = t;
        }
        if busy {
            self.done_at = -100.0;
            self.mood = Mood::of(s);
        }
        self.busy = busy;
        self.task = if busy { s.to_string() } else { String::new() };
    }

    fn cheer(&self, t: f64) -> Option<f32> {
        let p = (t - self.done_at) / CHEER_SECS;
        (!self.busy && (0.0..1.0).contains(&p)).then_some(p as f32)
    }

    fn rand(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn update(&mut self, t: f64) -> Pose {
        let dt = (t - self.last_t).clamp(0.0, 0.2) as f32;
        self.last_t = t;
        let busy = self.busy;
        let tf = t as f32;

        // blinks every few seconds
        if t > self.next_blink + 0.13 {
            self.next_blink = t + 2.5 + self.rand() * 4.0;
        }
        let blink = t >= self.next_blink && t <= self.next_blink + 0.13;

        // an ear flicks now and then
        if t > self.next_twitch {
            self.twitch_start = t;
            self.twitch_ear = if self.rand() < 0.5 { -1.0 } else { 1.0 };
            self.next_twitch = t + 4.0 + self.rand() * 6.0;
        }
        let tw = ((t - self.twitch_start) / 0.22) as f32;
        let twitch = if (0.0..1.0).contains(&tw) { (tw * PI).sin() * 4.0 } else { 0.0 };

        let cheer = self.cheer(t);
        let typing = busy && self.mood.types();
        let waiting = busy && self.mood.chin();
        let problem = busy && self.mood == Mood::Problem;
        let testing = busy && self.mood == Mood::Testing;
        let researching = busy && self.mood == Mood::Researching;
        let round = (t - self.since).max(0.0) % TEST_ROUND;
        let notes = (testing && round > TEST_ROUND - NOTES_SECS).then(|| ((round - (TEST_ROUND - NOTES_SECS)) / NOTES_SECS) as f32);
        let planning = busy && self.mood == Mood::Thinking;
        let plan = planning.then(|| {
            let into = (t - self.since).max(0.0);
            let k = ((into % PLAN_ROUND) / PLAN_DRAW).min(1.0) as f32;
            // a new drawing every round, different for every status line
            (k, hash((into / PLAN_ROUND).floor() as i64 * 131 + (self.since * 7.0) as i64))
        });

        let (hy, sway) = if typing {
            (126.0 + 1.0 * (tf * 2.0 * PI * 2.5).sin(), 8.0 * (tf * 2.4).sin())
        } else if testing {
            // leaning in to the game, bobbing along; head down over the notepad
            if notes.is_some() { (131.0, -6.0 + 2.0 * (tf * 3.0).sin()) } else { (127.0 + 1.0 * (tf * 3.2).sin(), 5.0 * (tf * 1.8).sin()) }
        } else if researching {
            // leaning in to read
            (130.0 + 0.5 * (tf * 0.9).sin(), 2.0 * (tf * 0.7).sin())
        } else if planning {
            (128.0 + 0.6 * (tf * 1.4).sin(), -4.0 + 2.0 * (tf * 0.8).sin())
        } else if waiting {
            // slumps a little, tail swishing slowly
            (131.0 + 0.8 * (tf * 1.2).sin(), 11.0 * (tf * 0.7).sin())
        } else if problem {
            (133.0 + 0.6 * tf.sin(), 2.0 * (tf * 0.6).sin())
        } else if cheer.is_some() {
            // happy little hops
            (124.0 - 5.0 * (tf * 2.0 * PI * 1.6).sin().abs(), 0.0)
        } else {
            (137.0 + 1.5 * (tf * 1.3).sin(), 3.0 * (tf * 0.8).sin())
        };

        // paws: busy = take turns tapping (about five taps a second), moving to a new key
        // while lifted; waiting = one paw taps the desk; cheering = both paws up;
        // otherwise folded on the keyboard under the chin
        let (mut l, mut r);
        if waiting {
            // chin propped on one paw, the other drumming: three quick taps, then a pause
            l = Paw { x: CX - 30.0, lift: 181.0 - (hy + 17.0), key: None, down: false };
            let beat = tf % 1.6;
            // drumming while waiting; a thinking kitten keeps the other paw still
            let tap = if self.mood == Mood::Waiting && beat < 0.6 { (beat * 2.0 * PI * 5.0).sin().max(0.0) } else { 0.0 };
            r = Paw { x: CX + 12.0, lift: 5.0 * tap, key: None, down: false };
        } else if testing {
            // both paws on the controller's grips, thumbs mashing; the left one leaves to write notes
            let mash = |k: f32| 1.4 * (tf * 2.0 * PI * k).sin().max(0.0);
            // the paws close over the grip ends and follow the pad as it rocks (a paw's center sits 10 right of and
            // 6.5 below its corner)
            let hold = |s: f32, m: f32| {
                let (gx, gy) = pad_point(tf, s * GRIP.0, GRIP.1);
                Paw { x: gx - PAW_W / 2.0, lift: 181.0 - (gy - 6.5) + m, key: None, down: false }
            };
            r = hold(1.0, mash(4.3));
            l = match notes {
                Some(n) => {
                    // the pencil runs along a line, then hops back to the start of the next one (its tip sits 4
                    // right of and 10 below the paw's corner)
                    let along = ((n * 3.0).fract() * 1.15).min(1.0);
                    let row = (n * 3.0).floor().min(2.0);
                    Paw { x: PAD.0 - 1.0 + 18.0 * along, lift: 181.0 - (PAD.1 - 6.0 + 4.0 * row), key: None, down: false }
                }
                None => hold(-1.0, mash(3.7)),
            };
        } else if researching {
            // paws folded on the keyboard while it reads
            l = Paw { x: CX - 29.0, lift: 0.0, key: None, down: false };
            r = Paw { x: CX + 9.0, lift: 0.0, key: None, down: false };
        } else if let Some((k, seed)) = plan.filter(|(k, _)| *k < 1.0) {
            // the left paw draws on the whiteboard, the marker's tip 7 left of and 8 above the paw's center
            let (tx, ty) = plan_strokes(k, seed).1;
            l = Paw { x: tx + 7.0 - PAW_W / 2.0, lift: 181.0 - (ty + 8.0 - 6.5), key: None, down: false };
            r = Paw { x: CX + 9.0, lift: 0.0, key: None, down: false };
        } else if cheer.is_some() {
            let wave = 3.0 * (tf * 2.0 * PI * 1.6).sin();
            l = Paw { x: CX - 50.0, lift: 46.0 + wave, key: None, down: false };
            r = Paw { x: CX + 30.0, lift: 46.0 - wave, key: None, down: false };
        } else if typing {
            let phase = t * 5.0;
            let s = (std::f64::consts::PI * phase).sin() as f32;
            let cycle = (phase / 2.0).floor() as i64;
            let lk = 1 + (hash(cycle) % 4) as usize;
            let rk = 6 + (hash(cycle + 99) % 4) as usize;
            l = Paw { x: key_x(lk) + KEY_W / 2.0 - PAW_W / 2.0, lift: 9.0 * s.max(0.0), key: Some(lk), down: false };
            r = Paw { x: key_x(rk) + KEY_W / 2.0 - PAW_W / 2.0, lift: 9.0 * (-s).max(0.0), key: Some(rk), down: false };
        } else {
            l = Paw { x: CX - 29.0, lift: 0.0, key: None, down: false };
            r = Paw { x: CX + 9.0, lift: 0.0, key: None, down: false };
        }
        // a rare sip of coffee: a short break, then straight back to the mood (not with both paws on a controller)
        let hands_free = !testing;
        if busy && t > self.next_sip {
            if hands_free {
                // now and then the mug turns out to be empty: a refill instead of a sip
                if self.rand() < 0.25 {
                    self.refill_start = t;
                } else {
                    self.sip_start = t;
                }
            }
            self.next_sip = t + 15.0 + self.rand() * 20.0;
        }
        // a refill ends with a sip of the fresh coffee
        let after = ((t - self.refill_start - REFILL_SECS) / SIP_SECS) as f32;
        let sp = if (0.0..1.0).contains(&after) { after } else { ((t - self.sip_start) / SIP_SECS) as f32 };
        let sip = (busy && hands_free && (0.0..1.0).contains(&sp)).then_some(sp);
        let rp = ((t - self.refill_start) / REFILL_SECS) as f32;
        let refill = (busy && hands_free && (0.0..1.0).contains(&rp)).then_some(rp);
        let mug = match refill {
            Some(p) => refill_mug(p, hy),
            None => sip.map_or(MUG_HOME, |p| mug_at(p, hy)),
        };
        if (sip.is_some() || refill.is_some()) && typing {
            // the typing paw rests while the other one holds the mug
            l = Paw { x: CX - 29.0, lift: 0.0, key: None, down: false };
        }
        if self.lpx < 0.0 {
            self.lpx = l.x;
            self.rpx = r.x;
        }
        let k = 1.0 - (-dt * if typing || testing || planning || cheer.is_some() { 14.0 } else { 4.0 }).exp();
        if !typing || l.lift > 3.0 {
            self.lpx += (l.x - self.lpx) * k;
        }
        if !typing || r.lift > 3.0 {
            self.rpx += (r.x - self.rpx) * k;
        }
        l.x = self.lpx;
        r.x = self.rpx;
        if let Some(p) = sip {
            // reach for the mug, hold it while it travels, let go and come back
            let w = smooth(if p < 0.15 { p / 0.15 } else if p > 0.88 { (1.0 - p) / 0.12 } else { 1.0 });
            let (gx, gy) = (mug.0 - PAW_W / 2.0 + 2.0, mug.1 + 3.0);
            let py = 181.0 + (gy - 181.0) * w;
            r = Paw { x: r.x + (gx - r.x) * w, lift: 181.0 - py, key: None, down: false };
            self.rpx = r.x;
        }
        // the refill: the right paw keeps hold of the mug (up to the mouth, then out to the left for the kettle, then
        // home); the left paw reaches out of the picture on the left, comes back with the kettle, pours, takes it
        // away and comes back to rest (paw centers)
        let (mut kettle, mut fill, mut anger) = (None, 1.0, 0.0);
        if let Some(p) = refill {
            let grip = |m: (f32, f32)| (m.0 + 2.0, m.1 + 9.5);
            let center = |w: &Paw| (w.x + PAW_W / 2.0, 181.0 - w.lift + 6.5);
            let (rest_r, rest_l) = (center(&r), center(&l));
            let (off, pour) = ((-30.0, 118.0), (39.0, 138.0));
            let rc = if p < 0.04 {
                lerp2(rest_r, grip(mug), p / 0.04)
            } else if p < 0.96 {
                grip(mug)
            } else {
                lerp2(grip(MUG_HOME), rest_r, (p - 0.96) / 0.04)
            };
            r = Paw { x: rc.0 - PAW_W / 2.0, lift: 181.0 - (rc.1 - 6.5), key: None, down: false };
            self.rpx = r.x;
            if p >= 0.40 {
                let lc = if p < 0.52 {
                    lerp2(rest_l, off, (p - 0.40) / 0.12)
                } else if p < 0.62 {
                    lerp2(off, pour, (p - 0.52) / 0.10)
                } else if p < 0.80 {
                    (pour.0, pour.1 + 0.8 * ((p - 0.62) * 40.0).sin())
                } else if p < 0.90 {
                    lerp2(pour, off, (p - 0.80) / 0.10)
                } else {
                    lerp2(off, rest_l, (p - 0.90) / 0.10)
                };
                l = Paw { x: lc.0 - PAW_W / 2.0, lift: 181.0 - (lc.1 - 6.5), key: None, down: false };
                self.lpx = l.x;
                if (0.52..0.90).contains(&p) {
                    let tilt = if p < 0.62 {
                        0.0
                    } else if p < 0.68 {
                        0.55 * smooth((p - 0.62) / 0.06)
                    } else if p < 0.74 {
                        0.55
                    } else if p < 0.80 {
                        0.55 * (1.0 - smooth((p - 0.74) / 0.06))
                    } else {
                        0.0
                    };
                    kettle = Some((lc.0, lc.1, tilt));
                }
            }
            fill = if p < 0.66 { 0.0 } else { ((p - 0.66) / 0.10).min(1.0) };
            anger = if (0.26..0.55).contains(&p) { 1.0 } else { 0.0 };
        }
        l.down = typing && sip.is_none() && refill.is_none() && l.lift < 1.0;
        r.down = typing && sip.is_none() && refill.is_none() && r.lift < 1.0;

        // looks at the screen while typing and glances back at you every so often;
        // while waiting it looks off to the side now and then
        let drinking = sip.is_some_and(|p| (0.38..0.66).contains(&p));
        let look = if sip.is_some() || refill.is_some() {
            0.0
        } else if typing && (t % 9.0) > 1.3 {
            2.5
        } else if testing {
            // eyes on the game, then down at the notepad
            if notes.is_some() { -3.0 } else { 3.0 }
        } else if researching {
            3.5
        } else if planning {
            // at the whiteboard
            -3.5
        } else if waiting && (t % 5.0) > 3.4 {
            // a glance over at the hourglass
            3.5
        } else {
            0.0
        };
        // heavy, bored eyelids while waiting (a little less during the glance); half-lidded over the notes
        let lid = if waiting && self.mood == Mood::Waiting {
            if look > 0.0 { 0.25 } else { 0.5 }
        } else if notes.is_some() {
            0.35
        } else if refill.is_some_and(|p| (0.30..0.58).contains(&p)) {
            // a glare at the empty mug
            0.4
        } else if researching {
            // reading closely over the glasses
            0.2
        } else {
            0.0
        };
        // content again once the mug is full
        let pleased = refill.is_some_and(|p| p > 0.9);
        // the mouthful: the mug tips and the mouth opens (on the refill's first try too, before it finds the mug empty)
        let bump = |x: f32, a: f32, b: f32, c: f32, d: f32| if x < b { smooth((x - a) / (b - a)) } else if x < c { 1.0 } else { 1.0 - smooth((x - c) / (d - c)) };
        let sipping = sip.map_or(0.0, |p| bump(p, 0.34, 0.42, 0.60, 0.68)).max(refill.map_or(0.0, |p| 0.8 * bump(p, 0.16, 0.20, 0.25, 0.29)));

        Pose {
            hy,
            sway,
            // asleep, or squeezed shut with joy while cheering
            eyes_closed: !busy || blink || drinking || pleased,
            look,
            twitch,
            twitch_ear: self.twitch_ear,
            l,
            r,
            cheer,
            droop: if problem { 1.0 } else { 0.7 * anger },
            lid: if drinking { 0.0 } else { lid },
            sip,
            mug,
            notes,
            plan,
            refill,
            kettle,
            fill,
            anger,
            sipping,
        }
    }

    pub fn render(&self, pix: &mut Pixmap, sc: f32, t: f64, p: &Pose) {
        pix.fill(Color::TRANSPARENT);
        let mut c = Canvas::new(pix, sc);
        let tf = t as f32;
        self.tail(&mut c, p);
        if let Some((k, seed)) = p.plan {
            self.whiteboard(&mut c, k, seed, tf);
        }
        self.computer(&mut c, tf, p);
        self.body(&mut c, p);
        self.desk(&mut c);
        let testing = self.busy && self.mood == Mood::Testing;
        if testing {
            self.notepad(&mut c, tf, p);
        }
        // the mug sits on the desk unless a paw holds it
        let mug_held = p.sip.is_some() || p.refill.is_some_and(|r| r < 0.96);
        if !mug_held {
            self.mug(&mut c, tf, MUG_HOME, p.fill, 0.0);
        }
        // the keyboard is put away while testing: the gamepad is held in front of the arms, the paws wrapped over
        // its grips
        if !testing {
            self.keyboard(&mut c, p);
        }
        self.arms(&mut c, p);
        if testing {
            self.controller(&mut c, tf, p);
            let holding = [Some(p.r), p.notes.is_none().then_some(p.l)];
            for (paw, s) in holding.into_iter().zip([1.0f32, -1.0]) {
                let Some(paw) = paw else { continue };
                // the paw wraps around the grip: its long side lies across the grip, the toes curl in against the
                // controller (same center as the paw the pose placed)
                let (cx, cy) = (paw.x + PAW_W / 2.0, 181.0 - paw.lift + 6.5);
                let (gx, gy) = (s * (GRIP.0 - 19.0), GRIP.1 - 2.0);
                let len = (gx * gx + gy * gy).sqrt();
                let (dx, dy) = (gx / len, gy / len);
                let (lx, ly) = (-dy, dx);
                let (rx, ry) = (11.5, 8.5);
                let shape: Vec<(f32, f32)> = (0..32).map(|i| {
                    let u = i as f32 / 32.0 * 2.0 * PI;
                    (cx + rx * u.cos() * lx + ry * u.sin() * dx, cy + rx * u.cos() * ly + ry * u.sin() * dy)
                }).collect();
                c.blob(&poly(&shape), self.coat.fur);
                for k in [-4.5f32, 0.0, 4.5] {
                    let (bx, by) = (cx + k * lx, cy + k * ly);
                    c.stroke(&line(bx - 4.5 * dx, by - 4.5 * dy, bx - 8.0 * dx, by - 8.0 * dy), INK.fade(0.8), 1.1);
                }
            }
        }
        if let Some(n) = p.notes {
            // the pencil in the writing paw, its tip on the paper
            let (x, y) = (p.l.x + 4.0, 181.0 - p.l.lift + 10.0);
            let wob = 1.2 * (n * 40.0).sin();
            c.stroke(&line(x - 7.0, y - 13.0 + wob, x, y), INK, 4.2);
            c.stroke(&line(x - 7.0, y - 13.0 + wob, x - 1.2, y - 2.0), SAND, 2.4);
            c.fill(&circle(x, y, 1.1), INK);
        }
        self.head(&mut c, p);
        if self.busy && self.mood == Mood::Building {
            self.hard_hat(&mut c, p);
        }
        if self.busy && self.mood == Mood::Researching {
            self.glasses(&mut c, p);
        }
        if let Some((k, seed)) = p.plan.filter(|(k, _)| *k < 1.0) {
            // the marker in the drawing paw, its tip on the board (put down while the finished drawing is admired)
            let (tx, ty) = plan_strokes(k, seed).1;
            let (px, py) = (p.l.x + PAW_W / 2.0, 181.0 - p.l.lift + 6.5);
            c.stroke(&line(px, py, tx, ty), INK, 5.0);
            c.stroke(&line(px, py, tx + 1.5, ty + 1.7), Rgba::rgb(60, 110, 200), 3.0);
        }
        if self.busy && self.mood.chin() {
            // chin propped on an upright forearm: elbow on the keyboard, paw cupping the jaw,
            // drawn in front of the head
            let (px, py) = (CX - 33.0, p.hy + 12.0);
            let arm = line(CX - 38.0, KY + 4.0, px + 8.0, py + 9.0);
            c.stroke(&arm, INK, 14.5);
            c.stroke(&arm, self.coat.fur, 10.5);
            c.blob(&oval(px, py, 18.0, 15.0), self.coat.fur);
            // toes curl in towards the face
            for (dx, dy) in [(12.5, 3.5), (14.5, 7.0), (13.5, 10.5)] {
                c.stroke(&line(px + dx, py + dy, px + dx + 3.5, py + dy), INK.fade(0.8), 1.1);
            }
        }
        if p.refill.is_some() {
            // the left arm goes behind the mug the right paw holds out (its paw and the kettle come after it)
            let py = 181.0 - p.l.lift;
            let arm = line(CX - 24.0, p.hy + 46.0, p.l.x + PAW_W / 2.0, py + 5.0);
            c.stroke(&arm, INK, 14.5);
            c.stroke(&arm, self.coat.fur, 10.5);
        }
        if p.sip.is_some() || p.refill.is_some() {
            // the arm, paw and mug in front of the head; the paw holds the mug from below
            let py = 181.0 - p.r.lift;
            let arm = line(CX + 24.0, p.hy + 46.0, p.r.x + PAW_W / 2.0, py + 5.0);
            c.stroke(&arm, INK, 14.5);
            c.stroke(&arm, self.coat.fur, 10.5);
            c.blob(&oval(p.r.x, py, PAW_W, 13.0), self.coat.fur);
            if mug_held {
                self.mug(&mut c, tf, p.mug, p.fill, p.sipping);
            }
        }
        if p.refill.is_some() {
            // the kettle and the left paw over everything, so it pours from above
            let py = 181.0 - p.l.lift;
            if let Some(k) = p.kettle {
                self.kettle(&mut c, k, p.mug);
            }
            c.blob(&oval(p.l.x, py, PAW_W, 13.0), self.coat.fur);
        }
        if p.anger > 0.0 {
            // the annoyed mark: four little brackets, pulsing
            let (ax, ay) = (CX + 40.0, p.hy - 32.0);
            let s = 1.0 + 0.18 * (tf * 12.0).sin();
            for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                c.stroke(&quad(ax + sx * 1.6, ay + sy * 6.0 * s, ax + sx * 1.6, ay + sy * 1.6, ax + sx * 6.0 * s, ay + sy * 1.6), RED, 2.2);
            }
        }
        self.sparkles(&mut c, p);
        if let Some(k) = p.cheer {
            self.confetti(&mut c, tf, k, p);
        } else if self.busy && self.mood == Mood::Waiting {
            // "..." beside the head, the dots lighting up one by one
            let step = (tf * 2.2) as i64 % 4;
            for i in 0..3 {
                let a = if (i as i64) < step { 1.0 } else { 0.25 };
                c.fill(&circle(CX + 58.0 + i as f32 * 9.0, p.hy - 30.0, 2.8), SUB.fade(a));
            }
        } else if !self.busy {
            self.zzz(&mut c, tf, p);
        }
        self.bubble(&mut c, t, p);
    }

    /// The typing screens other than scrolling code (screen 0), picked at random per status line.
    fn busy_screen(&self, c: &mut Canvas, t: f32) {
        match self.screen {
            1 => {
                // three progress bars filling at their own pace
                for i in 0..3 {
                    let y = 133.0 + i as f32 * 15.0;
                    let k = (t * (0.11 + 0.07 * i as f32) + i as f32 * 0.37).fract();
                    c.fill(&rrect(207.0, y, 62.0, 7.0, 3.5), WHITE.fade(0.12));
                    c.fill(&rrect(207.0, y, 6.0 + 56.0 * k, 7.0, 3.5), CODE[i]);
                }
            }
            2 => {
                // an equalizer
                for i in 0..8 {
                    let f = i as f32;
                    let h = 5.0 + 32.0 * (t * (2.1 + 0.37 * f) + f * 1.7).sin().abs();
                    c.fill(&rrect(206.0 + f * 8.4, 172.0 - h, 6.0, h, 2.0), CODE[i % 4]);
                }
            }
            3 => {
                // a spinning gear and loading dots
                let (x, y, rot) = (238.0, 146.0, t * 1.6);
                for k in 0..8 {
                    let a = rot + k as f32 / 8.0 * 2.0 * PI;
                    c.stroke(&line(x + a.cos() * 9.0, y + a.sin() * 9.0, x + a.cos() * 16.0, y + a.sin() * 16.0), CODE[1], 5.0);
                }
                c.fill(&circle(x, y, 11.5), CODE[1]);
                c.fill(&circle(x, y, 4.5), SCREEN_ON);
                let step = (t * 3.0) as i64 % 4;
                for i in 0..3 {
                    let a = if (i as i64) < step { 1.0 } else { 0.25 };
                    c.fill(&circle(229.0 + i as f32 * 9.0, 172.0, 2.4), CODE[0].fade(a));
                }
            }
            _ => {
                // a line graph scrolling past
                for gy in [136.0, 151.0, 166.0] {
                    c.stroke(&line(203.0, gy, 273.0, gy), WHITE.fade(0.08), 1.0);
                }
                let pts: Vec<(f32, f32)> = (0..=20)
                    .map(|i| {
                        let x = 203.0 + i as f32 * 3.5;
                        let u = i as f32 * 0.45 + t * 1.8;
                        (x, 152.0 - 9.0 * u.sin() - 5.0 * (u * 2.3 + 1.0).sin())
                    })
                    .collect();
                c.stroke(&polyline(&pts), CODE[2], 2.2);
                let (lx, ly) = pts[pts.len() - 1];
                c.fill(&circle(lx, ly, 2.6), CODE[0]);
            }
        }
    }

    /// The screens of the activity moods (testing, building, researching, transferring, thinking).
    fn mood_screen(&self, c: &mut Canvas, t: f32, mood: Mood, p: &Pose) {
        match mood {
            Mood::Testing => {
                // a little platformer: the hero hops over blocks; it freezes under a pause sign while notes are taken
                let g = t - p.notes.map_or(0.0, |n| n * NOTES_SECS as f32);
                c.fill(&rrect(199.0, 170.0, 78.0, 11.0, 0.0), Rgba::rgb(64, 112, 82));
                c.stroke(&line(199.0, 170.0, 277.0, 170.0), GREEN.fade(0.8), 1.4);
                const HOP: f32 = 1.1;
                const SPEED: f32 = 40.0;
                let n = (g / HOP).floor();
                for k in -1..=2 {
                    let ox = 216.0 + SPEED * ((n + k as f32 + 0.5) * HOP - g);
                    c.fill(&rrect(ox - 3.5, 162.0, 7.0, 8.0, 1.5), RED.fade(0.9));
                }
                let jump = ((g / HOP).fract() * PI).sin();
                let py = 160.0 - 17.0 * jump;
                c.fill(&rrect(211.0, py, 10.0, 10.0, 2.5), GREEN);
                c.fill(&circle(218.5, py + 3.5, 1.3), SCREEN_ON);
                for i in 0..3 {
                    c.fill(&circle(206.0 + i as f32 * 6.0, 128.0, 2.0), PINK_DARK);
                }
                // the pass count ticks up in the corner
                let passes = ((t / HOP) as i64 % 9) as usize;
                for i in 0..passes {
                    c.fill(&rrect(268.0 - i as f32 * 4.0, 126.0, 2.5, 5.0, 1.0), SAND);
                }
                if p.notes.is_some() {
                    c.fill(&rrect(199.0, 121.0, 78.0, 60.0, 0.0), INK.fade(0.35));
                    c.fill(&rrect(231.0, 140.0, 5.0, 16.0, 1.5), WHITE);
                    c.fill(&rrect(240.0, 140.0, 5.0, 16.0, 1.5), WHITE);
                }
            }
            Mood::Building => {
                // a brick wall goes up course by course; the next brick drops into place
                c.stroke(&line(203.0, 176.5, 273.0, 176.5), CASE.fade(0.5), 1.4);
                let step = t * 3.5;
                let placed = (step as usize) % 31;
                let fall = 1.0 - step.fract();
                for i in 0..placed.min(25) + 1 {
                    if i >= 25 {
                        break;
                    }
                    let (row, col) = (i / 5, i % 5);
                    let x = 206.0 + col as f32 * 13.0 + if row % 2 == 1 { 6.5 } else { 0.0 };
                    let mut y = 169.0 - row as f32 * 7.5;
                    if i == placed {
                        y -= fall * 34.0;
                    }
                    let w = if row % 2 == 1 && col == 4 { 6.0 } else { 12.0 };
                    c.fill(&rrect(x, y, w, 6.5, 1.0), Rgba::rgb(206, 96, 66));
                    c.fill(&rrect(x, y, w, 1.6, 0.8), Rgba::rgb(232, 138, 104));
                }
            }
            Mood::Researching => {
                // a page scrolls up slowly; the line passing the reading spot gets highlighted as it is read, and
                // stays highlighted
                c.fill(&rrect(211.0, 121.0, 54.0, 60.0, 0.0), Rgba::rgb(236, 232, 220));
                let (lh, read_y) = (7.0, 150.0);
                let off = t * 3.5;
                let first = (off / lh).floor() as i64;
                for i in -1..10i64 {
                    let n = first + i;
                    let y = 124.0 + i as f32 * lh - (off - first as f32 * lh);
                    if !(123.0..178.0).contains(&y) {
                        continue;
                    }
                    let len = 24.0 + (hash(n + 3) % 18) as f32;
                    let marked = hash(n * 5 + 1) % 3 == 0;
                    if marked && y <= read_y {
                        let k = ((read_y - y) / 5.0).clamp(0.0, 1.0);
                        c.fill(&rrect(215.0, y - 2.4, (len + 4.0) * k, 5.0, 2.0), SAND.fade(0.8));
                    }
                    c.fill(&rrect(217.0, y - 1.0, len, 2.2, 1.1), Rgba::rgb(120, 116, 130));
                }
            }
            Mood::Transfer(up) => {
                // files fly between a cloud and a tray; a progress bar fills underneath
                for (x, y, r) in [(229.0, 137.0, 6.5), (238.0, 132.0, 8.5), (248.0, 137.0, 6.5)] {
                    c.fill(&circle(x, y, r), WHITE.fade(0.92));
                }
                c.fill(&rrect(223.0, 135.0, 31.0, 9.0, 4.5), WHITE.fade(0.92));
                c.fill(&rrect(222.0, 165.0, 33.0, 5.0, 2.0), CASE);
                for i in 0..2 {
                    let ph = (t * 0.75 + i as f32 / 2.0).fract();
                    let y = if up { 160.0 - 16.0 * ph } else { 146.0 + 16.0 * ph };
                    let a = (ph * PI).sin();
                    c.fill(&rrect(234.5, y - 5.0, 8.0, 10.0, 1.6), CODE[i].fade(a));
                    c.fill(&rrect(236.5, y - 2.0, 4.0, 1.2, 0.6), SCREEN_ON.fade(a));
                }
                let (ax, ay) = (263.0, 155.0);
                let d = if up { -1.0 } else { 1.0 };
                c.stroke(&polyline(&[(ax - 4.0, ay - 2.0 * d), (ax, ay + 2.0 * d), (ax + 4.0, ay - 2.0 * d)]), GREEN, 2.0);
                c.fill(&rrect(207.0, 174.0, 62.0, 4.0, 2.0), WHITE.fade(0.12));
                c.fill(&rrect(207.0, 174.0, 4.0 + 58.0 * (t * 0.12).fract(), 4.0, 2.0), GREEN);
            }
            Mood::Thinking => {
                // a checklist: an item is ticked with every box the board gets
                let done = p.plan.map_or(0, |(k, _)| (k * 5.0).floor() as usize).min(4);
                let _ = t;
                for i in 0..4 {
                    let y = 133.0 + i as f32 * 11.0;
                    c.stroke(&rrect(207.0, y - 4.0, 8.0, 8.0, 1.5), CASE.fade(0.8), 1.4);
                    let len = 30.0 + (hash(i as i64 + 11) % 16) as f32;
                    c.fill(&rrect(220.0, y - 1.5, len, 3.0, 1.5), WHITE.fade(if i < done { 0.3 } else { 0.55 }));
                    if i < done {
                        c.stroke(&polyline(&[(208.5, y), (211.0, y + 2.5), (215.5, y - 3.5)]), GREEN, 2.0);
                    }
                }
            }
            _ => {}
        }
    }

    /// The notepad beside the keyboard while testing: last round's notes, or the lines being written now.
    fn notepad(&self, c: &mut Canvas, _t: f32, p: &Pose) {
        let (x, y) = PAD;
        c.blob(&rrect(x, y, 26.0, 17.0, 2.0), Rgba::rgb(252, 248, 234));
        c.stroke(&line(x + 4.0, y + 1.5, x + 4.0, y + 15.5), RED.fade(0.45), 0.9);
        let written = p.notes.map_or(3.0, |n| n * 3.0);
        for row in 0..3 {
            let k = (written - row as f32).clamp(0.0, 1.0);
            let k = (k * 1.15).min(1.0);
            if k <= 0.0 {
                continue;
            }
            let ly = y + 4.0 + row as f32 * 4.0;
            let len = 17.0 * k;
            let pts: Vec<(f32, f32)> = (0..=8).map(|i| {
                let u = i as f32 / 8.0;
                (x + 6.0 + len * u, ly + 0.7 * (u * 9.0 + row as f32).sin())
            }).collect();
            c.stroke(&polyline(&pts), Rgba::rgb(70, 82, 140), 1.0);
        }
    }

    /// A gamepad held up at the chest with its face tipped toward the kitten (the keyboard is put away while
    /// testing): from our side we see its underside and front edge, the triggers and shoulder buttons along the
    /// top, and the grips coming down into the paws. The trigger being pressed sinks in and lights up.
    fn controller(&self, c: &mut Canvas, t: f32, p: &Pose) {
        let (shell, under, light) = (Rgba::rgb(96, 104, 140), Rgba::rgb(78, 84, 116), Rgba::rgb(150, 158, 196));
        let at = |x: f32, y: f32| pad_point(t, x, y);
        let seg = |x1: f32, y1: f32, x2: f32, y2: f32| {
            let (a, b) = (at(x1, y1), at(x2, y2));
            line(a.0, a.1, b.0, b.1)
        };
        let hot = if p.notes.is_some() { usize::MAX } else { (hash((t * 4.0) as i64) % 3) as usize };
        for (i, s) in [-1.0f32, 1.0].into_iter().enumerate() {
            let sink = if hot == i { 1.6 } else { 0.0 };
            let (x, y) = at(s * 21.0, -12.0 + sink);
            c.blob(&rrect(x - 8.0, y - 3.5, 16.0, 8.0, 3.5), if hot == i { light } else { shell });
        }
        for s in [-1.0f32, 1.0] {
            // narrower than the paw that closes over its end, so nothing of it shows around the paw
            let grip = seg(s * 19.0, 2.0, s * GRIP.0, GRIP.1);
            c.stroke(&grip, INK, 15.0);
            c.stroke(&grip, under, 11.6);
        }
        let body = seg(-27.0, 0.0, 27.0, 0.0);
        c.stroke(&body, INK, 22.0);
        c.stroke(&body, under, 18.4);
        // the front edge: a bumper either side of the cable port, the shell's seam below
        for s in [-1.0f32, 1.0] {
            c.stroke(&seg(s * 12.0, -6.5, s * 30.0, -6.5), shell, 4.6);
            c.stroke(&seg(s * 13.0, -7.4, s * 29.0, -7.4), light.fade(0.8), 1.2);
        }
        let (px, py) = at(0.0, -6.5);
        c.fill(&rrect(px - 3.5, py - 1.6, 7.0, 3.2, 1.2), INK.fade(0.8));
        c.stroke(&seg(-24.0, 1.5, 24.0, 1.5), light.fade(0.35), 1.0);
    }

    /// A yellow hard hat while building.
    fn hard_hat(&self, c: &mut Canvas, p: &Pose) {
        let hy = p.hy;
        let (hat, shade) = (Rgba::rgb(255, 204, 64), Rgba::rgb(222, 164, 40));
        c.blob(&oval(CX - 32.0, hy - 60.0, 64.0, 44.0), hat);
        c.stroke(&line(CX, hy - 59.0, CX, hy - 36.0), shade, 4.0);
        c.blob(&rrect(CX - 44.0, hy - 37.0, 88.0, 7.0, 3.5), hat);
        c.stroke(&line(CX - 40.0, hy - 32.0, CX + 40.0, hy - 32.0), shade.fade(0.7), 1.2);
    }

    /// Round reading glasses over the eyes while researching.
    fn glasses(&self, c: &mut Canvas, p: &Pose) {
        let ey = p.hy - 3.0;
        let frame = Rgba::rgb(70, 52, 40);
        for ex in [CX - 22.0, CX + 22.0] {
            c.fill(&circle(ex, ey, 11.5), WHITE.fade(0.14));
            c.stroke(&circle(ex, ey, 11.5), frame, 2.4);
            c.stroke(&quad(ex - 6.5, ey - 5.0, ex - 5.0, ey - 8.0, ex - 1.0, ey - 8.5), WHITE.fade(0.7), 1.3);
        }
        c.stroke(&quad(CX - 10.5, ey - 2.0, CX, ey - 6.0, CX + 10.5, ey - 2.0), frame, 2.2);
        for s in [-1.0f32, 1.0] {
            c.stroke(&line(CX + s * 33.5, ey - 3.0, CX + s * 46.0, ey - 7.0), frame, 2.2);
        }
    }

    /// The whiteboard on its easel beside the desk, with the marker strokes drawn k (0..1) of the way.
    fn whiteboard(&self, c: &mut Canvas, k: f32, seed: u32, t: f32) {
        let (x, y, w, h) = BOARD;
        // an easel on the floor behind the desk: its legs run down behind the desk (drawn later, over them)
        for (top, foot) in [(x + 12.0, x + 8.0), (x + w - 12.0, x + w - 8.0)] {
            c.stroke(&line(top, y + h - 2.0, foot, H), INK, 4.6);
            c.stroke(&line(top, y + h - 2.0, foot, H), WOOD_FRONT, 2.6);
        }
        c.blob(&rrect(x, y, w, h, 3.0), Rgba::rgb(250, 250, 246));
        c.stroke(&rrect(x + 1.5, y + 1.5, w - 3.0, h - 3.0, 2.0), CASE_SHADE, 1.6);
        c.blob(&rrect(x + 3.0, y + h - 1.0, w - 6.0, 4.0, 1.5), CASE_SHADE);
        for s in plan_strokes(k, seed).0 {
            if s.len() > 1 {
                c.stroke(&polyline(&s), Rgba::rgb(60, 110, 200), 2.0);
            }
        }
        if k >= 1.0 && (self.egg || plan_egg(seed)) {
            self.board_egg(c, seed, t);
        }
    }

    /// The finished drawing's little extra: a small animated scene in the drawing's own style.
    fn board_egg(&self, c: &mut Canvas, seed: u32, t: f32) {
        let (ax, ay, ..) = BOARD_ART;
        let red = Rgba::rgb(214, 70, 70);
        let blue = Rgba::rgb(60, 110, 200);
        let (drawing, anchors) = plan_drawing(seed);
        match hash(seed as i64) % 9 {
            0 => {
                // underpants gnomes: step 2 is "?", the last step is the money
                if let [.., (x1, y1), (x2, y2)] = anchors[..] {
                    if let Some(font) = self.bold.as_ref() {
                        for i in 0..3 {
                            let bob = 1.5 * (t * 4.0 + i as f32 * 1.4).sin();
                            c.text(font, 7.0, ax + x1 - 7.0 + i as f32 * 5.0, ay + y1 + 2.6 + bob, "?", red);
                        }
                    }
                    let (bx, by) = (ax + x2, ay + y2 + 1.0);
                    c.blob(&oval(bx - 6.0, by - 4.5, 12.0, 10.0), SAND);
                    c.blob(&poly(&[(bx - 2.5, by - 4.0), (bx - 4.0, by - 8.0), (bx + 4.0, by - 8.0), (bx + 2.5, by - 4.0)]), SAND);
                    c.stroke(&line(bx - 3.0, by - 4.5, bx + 3.0, by - 4.5), WOOD_FRONT, 1.2);
                    if let Some(font) = self.bold.as_ref() {
                        c.text(font, 6.5, bx - 1.8, by + 3.0, "$", WOOD_FRONT);
                    }
                    for (i, (dx, dy)) in [(-9.0f32, -6.0f32), (8.0, -8.0), (9.0, 3.0)].into_iter().enumerate() {
                        let s = 1.5 + 1.5 * (t * 5.0 + i as f32 * 2.1).sin().max(0.0);
                        c.stroke(&line(bx + dx - s, by + dy, bx + dx + s, by + dy), SPARK, 1.0);
                        c.stroke(&line(bx + dx, by + dy - s, bx + dx, by + dy + s), SPARK, 1.0);
                    }
                }
            }
            1 => {
                // a space invader patrols over the bars and drops a bomb now and then
                let x = ax + 25.0 + 13.0 * (t * 1.2).sin();
                let y = ay + 4.0;
                let frame = if (t * 2.5) as i64 % 2 == 0 { &INVADER_A } else { &INVADER_B };
                pixels(c, frame, x - 5.5, y, 1.0, Rgba::rgb(70, 170, 90));
                let drop = (t * 0.8).fract();
                if drop < 0.7 {
                    let bx = ax + 25.0 + 13.0 * ((t - drop * 1.25) * 1.2).sin();
                    c.fill(&rrect(bx - 0.6, y + 9.0 + drop * 70.0, 1.2, 3.0, 0.4), red);
                }
            }
            2 => {
                // a Nokia snake slithers along the graph to an apple at its end
                if let Some(pts) = drawing.get(1) {
                    let pts: Vec<(f32, f32)> = pts.iter().map(|&(x, y)| (ax + x, ay + y)).collect();
                    let end = pts[pts.len() - 1];
                    c.fill(&circle(end.0 + 2.0, end.1 - 4.0, 2.4), red);
                    c.stroke(&line(end.0 + 2.0, end.1 - 6.2, end.0 + 3.4, end.1 - 7.6), Rgba::rgb(80, 160, 80), 1.0);
                    let head = (t * 0.22).fract();
                    for k in 0..7 {
                        let u = (head - k as f32 * 0.035).max(0.0);
                        let (x, y) = along(&pts, u);
                        let s = if k == 0 { 3.0 } else { 2.4 };
                        c.fill(&rrect(x - s / 2.0, y - s / 2.0, s, s, 0.6), Rgba::rgb(80, 150, 70));
                        if k == 0 {
                            c.fill(&circle(x + 0.5, y - 0.5, 0.5), INK);
                        }
                    }
                }
            }
            3 => {
                // a tiny Tetris well in the corner: a piece drops in and the row flashes as it clears
                let (wx, wy, cell) = (ax + 37.0, ay + 50.0, 3.4);
                c.stroke(&polyline(&[(wx, wy), (wx, wy + 5.0 * cell), (wx + 3.0 * cell, wy + 5.0 * cell), (wx + 3.0 * cell, wy)]), blue, 1.0);
                let ph = (t / 2.2).fract();
                let flash = ph > 0.78;
                let row_col = if flash && (t * 12.0) as i64 % 2 == 0 { WHITE } else { CODE[2] };
                for col in [0usize, 2] {
                    c.fill(&rrect(wx + col as f32 * cell + 0.3, wy + 4.0 * cell + 0.3, cell - 0.6, cell - 0.6, 0.6), row_col);
                }
                let drop = ((ph / 0.78).min(1.0) * 4.0).floor();
                for (dx, dy) in [(1.0f32, 0.0f32), (1.0, -1.0), (0.0, -1.0), (2.0, -1.0)] {
                    let y = wy + (drop + dy) * cell;
                    if y >= wy - 0.1 {
                        c.fill(&rrect(wx + dx * cell + 0.3, y + 0.3, cell - 0.6, cell - 0.6, 0.6), if flash && dy == 0.0 { row_col } else { CODE[3] });
                    }
                }
            }
            4 => {
                // a pixel kitten hops along the milestones, a coin popping at each, then slides down the flag pole
                if let Some(road) = drawing.first() {
                    let road: Vec<(f32, f32)> = road.iter().map(|&(x, y)| (ax + x, ay + y)).collect();
                    let ph = (t / 4.5).fract() * 4.0;
                    let leg = (ph.floor() as usize).min(3);
                    let u = ph.fract();
                    let (x, y) = if leg < 3 {
                        let (a, b) = (road[leg], road[leg + 1]);
                        (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u - 9.0 * (u * PI).sin())
                    } else {
                        let top = road[3];
                        (top.0 - 2.0, top.1 - 14.0 + 14.0 * u)
                    };
                    pixels(c, &PIXEL_CAT, x - 3.0, y - 6.0, 1.2, Rgba::rgb(240, 150, 60));
                    if leg < 3 && u < 0.5 {
                        let (cx, cy) = road[leg];
                        let w = 2.6 * (u * 30.0).cos().abs() + 0.4;
                        c.fill(&oval(cx - w / 2.0, cy - 8.0 - u * 10.0, w, 3.6), SAND.fade(1.0 - u * 2.0));
                    }
                }
            }
            5 => {
                // the pie chart turns into a chomper, a little ghost close behind
                let (cx, cy, r) = (ax + 25.0, ay + 36.0, 18.0);
                let mouth = 0.05 + 0.22 * (t * 6.0).sin().abs();
                let mut pts = vec![(cx, cy)];
                pts.extend((0..=36).map(|i| {
                    let a = mouth * PI + (2.0 - 2.0 * mouth) * PI * i as f32 / 36.0;
                    (cx + r * a.cos(), cy + r * a.sin())
                }));
                c.fill(&poly(&pts), SAND.fade(0.85));
                c.fill(&circle(cx + 3.0, cy - 9.0, 1.8), INK);
                let gx = ax + 8.0 + 3.0 * (t * 2.0).sin();
                let gy = ay + 62.0;
                let ghost = Rgba::rgb(240, 130, 190);
                let mut g = vec![(gx - 4.5, gy + 5.0)];
                g.extend((0..=12).map(|i| {
                    let a = PI + PI * i as f32 / 12.0;
                    (gx + 4.5 * a.cos(), gy + 4.5 * a.sin())
                }));
                g.extend([(gx + 4.5, gy + 5.0), (gx + 3.0, gy + 3.5), (gx + 1.5, gy + 5.0), (gx, gy + 3.5), (gx - 1.5, gy + 5.0), (gx - 3.0, gy + 3.5)]);
                c.fill(&poly(&g), ghost);
                for ex in [gx - 1.8, gx + 1.8] {
                    c.fill(&circle(ex, gy - 0.5, 1.2), WHITE);
                    c.fill(&circle(ex + 0.4, gy - 0.3, 0.5), INK);
                }
            }
            6 => {
                // a three-eyed alien peeks out of the porthole; the flame flickers
                let (px, py) = (ax + 25.0, ay + 25.0);
                c.fill(&circle(px, py, 3.6), Rgba::rgb(40, 50, 80));
                c.fill(&circle(px, py + 0.8, 2.8), Rgba::rgb(110, 200, 110));
                let blink = (t * 0.7).fract() > 0.92;
                for ex in [px - 1.4, px, px + 1.4] {
                    if blink {
                        c.stroke(&line(ex - 0.5, py, ex + 0.5, py), INK, 0.5);
                    } else {
                        c.fill(&circle(ex, py, 0.55), INK);
                    }
                }
                let f = 0.8 + 0.3 * (t * 17.0).sin();
                c.fill(&poly(&[(ax + 20.0, ay + 50.0), (ax + 25.0, ay + 50.0 + 13.0 * f), (ax + 30.0, ay + 50.0)]), Rgba::rgb(255, 150, 60).fade(0.85));
                c.fill(&poly(&[(ax + 22.5, ay + 50.0), (ax + 25.0, ay + 50.0 + 7.0 * f), (ax + 27.5, ay + 50.0)]), SAND);
            }
            7 => {
                // a little invader dancing in the overlap
                let three = drawing.iter().filter(|s| s.len() > 20).count() > 2;
                let frame = if (t * 3.0) as i64 % 2 == 0 { &INVADER_A } else { &INVADER_B };
                pixels(c, frame, ax + 25.0 - 6.6, ay + if three { 34.0 } else { 26.0 }, 1.2, Rgba::rgb(70, 170, 90));
            }
            _ => {
                // "Lumos": a wand taps the bulb and it lights up
                let (cx, cy) = (ax + 25.0, ay + 28.0);
                let tap = (t * 1.3).fract();
                let lit = tap > 0.25;
                if lit {
                    let glow = 0.35 + 0.15 * (t * 6.0).sin();
                    c.fill(&circle(cx, cy, 13.0), SAND.fade(glow));
                    for i in 0..4 {
                        let a = t * 2.0 + i as f32 * PI / 2.0;
                        let (sx, sy) = (cx + a.cos() * 17.0, cy + a.sin() * 17.0);
                        c.stroke(&line(sx - 1.6, sy, sx + 1.6, sy), SPARK, 1.0);
                        c.stroke(&line(sx, sy - 1.6, sx, sy + 1.6), SPARK, 1.0);
                    }
                }
                let reach = if tap < 0.25 { tap / 0.25 } else { 1.0 };
                let (tx, ty) = (cx + 15.0 - 4.0 * reach, cy + 10.0 - 3.0 * reach);
                c.stroke(&line(tx, ty, tx + 9.0, ty + 10.0), WOOD_FRONT, 2.0);
                c.fill(&circle(tx, ty, 1.1), WHITE);
            }
        }
    }

    /// A typing screen's rare little extra, drawn over it.
    fn screen_egg(&self, c: &mut Canvas, t: f32) {
        match self.screen {
            0 => {
                // digital rain
                c.fill(&rrect(199.0, 121.0, 78.0, 60.0, 0.0), Rgba::rgb(8, 20, 12).fade(0.9));
                for col in 0..11 {
                    let x = 202.0 + col as f32 * 7.0;
                    let speed = 18.0 + (hash(col + 5) % 20) as f32;
                    let head = 118.0 + ((t * speed + (hash(col) % 60) as f32) % 80.0);
                    for k in 0..6 {
                        let y = head - k as f32 * 4.5;
                        let a = if k == 0 { 1.0 } else { 0.75 - k as f32 * 0.12 };
                        let col_rgb = if k == 0 { Rgba::rgb(200, 255, 210) } else { GREEN };
                        c.fill(&rrect(x, y, 2.4, 3.2, 0.6), col_rgb.fade(a));
                    }
                }
            }
            1 => {
                // a spinning plumbob floats above the bars
                let (x, y) = (262.0, 129.0 + 1.5 * (t * 2.0).sin());
                let w = 5.0 * (t * 2.2).cos();
                let green = Rgba::rgb(110, 230, 110);
                c.fill(&poly(&[(x, y - 7.0), (x + w, y), (x, y + 2.0), (x - w, y)]), green);
                c.fill(&poly(&[(x, y + 2.0), (x + w, y), (x, y + 9.0), (x - w, y)]), Rgba::rgb(70, 180, 80));
                c.stroke(&line(x, y - 7.0, x + w * 0.5, y - 1.0), WHITE.fade(0.6), 0.8);
            }
            2 => {
                // a pop-tart cat streaks across with its rainbow
                let x = 190.0 + (t * 26.0) % 110.0;
                let y = 133.0 + 2.0 * (t * 6.0).sin();
                let rainbow = [Rgba::rgb(240, 80, 80), Rgba::rgb(250, 160, 60), SAND, GREEN, CODE[1], Rgba::rgb(170, 110, 230)];
                for (i, col) in rainbow.iter().enumerate() {
                    let mut sx = x - 3.0;
                    let mut step = 0;
                    while sx > 195.0 {
                        let wob = if step % 2 == 0 { 0.0 } else { 1.0 };
                        c.fill(&rrect(sx - 6.0, y - 4.0 + i as f32 * 1.6 + wob, 6.0, 1.6, 0.0), *col);
                        sx -= 6.0;
                        step += 1;
                    }
                }
                c.fill(&rrect(x - 3.0, y - 5.0, 12.0, 10.0, 2.0), Rgba::rgb(240, 200, 150));
                c.fill(&rrect(x - 1.5, y - 3.5, 9.0, 7.0, 1.5), Rgba::rgb(250, 150, 200));
                for (dx, dy) in [(0.5f32, -1.5f32), (3.5, 1.0), (6.0, -2.0)] {
                    c.fill(&rrect(x + dx, y + dy, 1.0, 1.0, 0.0), Rgba::rgb(230, 60, 120));
                }
                c.fill(&rrect(x + 6.0, y - 2.0, 7.0, 6.0, 2.0), Rgba::rgb(150, 150, 160));
                c.fill(&poly(&[(x + 6.5, y - 1.5), (x + 7.5, y - 4.0), (x + 9.0, y - 1.5)]), Rgba::rgb(150, 150, 160));
                c.fill(&poly(&[(x + 10.0, y - 1.5), (x + 11.5, y - 4.0), (x + 12.5, y - 1.5)]), Rgba::rgb(150, 150, 160));
                c.fill(&circle(x + 8.2, y + 0.5, 0.6), INK);
                c.fill(&circle(x + 11.2, y + 0.5, 0.6), INK);
            }
            3 => {
                // two portals and a companion cube passing through
                c.fill(&rrect(199.0, 121.0, 78.0, 60.0, 0.0), SCREEN_ON.fade(0.85));
                let (bx, ox, y) = (212.0, 264.0, 151.0);
                let ph = (t * 0.6).fract();
                let cx = bx + (ox - bx) * ph;
                let s = 9.0;
                c.fill(&rrect(cx - s / 2.0, y - s / 2.0, s, s, 1.8), Rgba::rgb(190, 194, 210));
                c.stroke(&rrect(cx - s / 2.0, y - s / 2.0, s, s, 1.8), Rgba::rgb(120, 124, 140), 1.0);
                c.fill(&circle(cx - 1.0, y - 0.6, 1.3), Rgba::rgb(240, 110, 160));
                c.fill(&circle(cx + 1.0, y - 0.6, 1.3), Rgba::rgb(240, 110, 160));
                c.fill(&poly(&[(cx - 2.2, y - 0.2), (cx, y + 2.3), (cx + 2.2, y - 0.2)]), Rgba::rgb(240, 110, 160));
                for (x, col) in [(bx, Rgba::rgb(80, 160, 255)), (ox, Rgba::rgb(255, 150, 50))] {
                    c.stroke(&oval(x - 4.0, y - 11.0, 8.0, 22.0), col, 2.4);
                    c.fill(&oval(x - 2.8, y - 9.5, 5.6, 19.0), col.fade(0.25));
                }
            }
            _ => {
                // a rocket rides the graph line up to the moon
                c.fill(&circle(268.0, 129.0, 5.0), Rgba::rgb(255, 230, 150));
                c.fill(&circle(270.5, 127.0, 4.6), SCREEN_ON);
                let ph = (t * 0.25).fract();
                let x = 203.0 + 66.0 * ph;
                let yat = |x: f32| {
                    let u = (x - 203.0) / 3.5 * 0.45 + t * 1.8;
                    152.0 - 9.0 * u.sin() - 5.0 * (u * 2.3 + 1.0).sin() - 20.0 * ph * ph
                };
                let (y, ahead) = (yat(x), yat(x + 2.0));
                let ang = (ahead - y).atan2(2.0);
                let (ca, sa) = (ang.cos(), ang.sin());
                let at = |px: f32, py: f32| (x + px * ca - py * sa, y + px * sa + py * ca);
                let body: Vec<(f32, f32)> = [(6.0, 0.0), (2.0, -2.4), (-4.0, -2.4), (-4.0, 2.4), (2.0, 2.4)].iter().map(|&(px, py)| at(px, py)).collect();
                let flame: Vec<(f32, f32)> = [(-4.0, -1.6), (-8.5 - 1.5 * (t * 20.0).sin(), 0.0), (-4.0, 1.6)].iter().map(|&(px, py)| at(px, py)).collect();
                c.fill(&poly(&flame), Rgba::rgb(255, 150, 60));
                c.fill(&poly(&body), WHITE);
                let (wx, wy) = at(1.0, 0.0);
                c.fill(&circle(wx, wy, 1.1), CODE[1]);
            }
        }
    }

    /// Twinkling stars around the head during the "all done!" moment.
    fn confetti(&self, c: &mut Canvas, t: f32, k: f32, p: &Pose) {
        let fade = if k > 0.8 { (1.0 - k) / 0.2 } else { 1.0 };
        for i in 0..6 {
            // an arc above the head, clear of the face
            let a = PI * (1.05 + 0.9 * i as f32 / 5.0);
            let r = 64.0 + 5.0 * (t * 5.0 + i as f32).sin();
            let (x, y) = (CX + a.cos() * r, p.hy - 14.0 + a.sin() * r * 0.75);
            let s = 3.0 + 1.5 * (t * 7.0 + i as f32 * 1.3).sin().abs();
            let col = if i % 2 == 0 { SPARK } else { PINK_DARK }.fade(fade);
            c.stroke(&line(x - s, y, x + s, y), col, 1.8);
            c.stroke(&line(x, y - s, x, y + s), col, 1.8);
        }
    }

    fn tail(&self, c: &mut Canvas, p: &Pose) {
        let pts = [
            (CX - 26.0, 204.0),
            (CX - 62.0, 208.0),
            (CX - 80.0 + p.sway * 0.5, 176.0),
            (CX - 66.0 + p.sway, 148.0),
        ];
        let line: Vec<(f32, f32)> = (0..=24).map(|i| bezier(pts, i as f32 / 24.0)).collect();
        let path = polyline(&line);
        c.stroke(&path, INK, 13.0);
        c.stroke(&path, self.coat.fur, 9.0);
        if matches!(self.coat.marks, Marks::Rainbow) {
            // bands along the tail, red at the tip
            for (i, col) in RAINBOW.iter().enumerate() {
                let from = 24 - (i + 1) * 4;
                c.stroke(&polyline(&line[from..=from + 4]), *col, 9.0);
            }
        } else {
            // darker tip
            let tip = polyline(&line[19..]);
            c.stroke(&tip, self.coat.dark, 9.0);
        }
    }

    fn computer(&self, c: &mut Canvas, t: f32, p: &Pose) {
        let mood = if self.busy { Some(self.mood) } else { None };
        // stand
        c.blob(&rrect(229.0, 196.0, 18.0, 9.0, 2.0), CASE_SHADE);
        c.blob(&rrect(213.0, 202.0, 50.0, 8.0, 4.0), CASE);
        // case
        c.blob(&rrect(190.0, 113.0, 96.0, 85.0, 12.0), CASE);
        c.fill(&rrect(193.0, 186.0, 90.0, 9.0, 4.0), CASE_SHADE.fade(0.6));
        // vents and power light
        for i in 0..3 {
            let y = 188.0 + i as f32 * 3.0;
            c.stroke(&line(200.0, y, 214.0, y), INK.fade(0.35), 1.2);
        }
        let breathe = |speed: f32| 0.45 + 0.55 * (0.5 + 0.5 * (t * speed).sin());
        let led = match mood {
            Some(Mood::Busy | Mood::Building | Mood::Transfer(_)) => GREEN.fade(if (t * 3.0).fract() < 0.6 { 1.0 } else { 0.45 }),
            Some(Mood::Waiting | Mood::Thinking) => Rgba::rgb(255, 186, 84).fade(breathe(4.0)),
            Some(Mood::Problem) => RED.fade(if (t * 2.0).fract() < 0.5 { 1.0 } else { 0.3 }),
            Some(Mood::Testing | Mood::Researching) => CODE[1].fade(breathe(3.0)),
            None if p.cheer.is_some() => GREEN,
            None => Rgba::rgb(255, 186, 84).fade(breathe(1.5)),
        };
        c.fill(&circle(277.0, 191.0, 2.8), led);
        c.stroke(&circle(277.0, 191.0, 2.8), INK.fade(0.6), 1.0);

        // screen
        let screen = rrect(199.0, 121.0, 78.0, 60.0, 7.0);
        let bg = match mood {
            Some(Mood::Problem) => SCREEN_BAD,
            Some(_) => SCREEN_ON,
            None if p.cheer.is_some() => SCREEN_ON,
            None => SCREEN_OFF,
        };
        c.fill(&screen, bg);
        c.clip(Some(&screen));
        if mood == Some(Mood::Waiting) {
            // an hourglass: the sand runs down, then the glass turns over (the full bottom bulb
            // becomes the top one, so the next run starts exactly where the turn ends)
            const RUN: f32 = 2.4;
            const TURN: f32 = 0.6;
            let ph = t % (RUN + TURN);
            let k = (ph / (RUN - 0.2)).min(1.0);
            let turn = ((ph - RUN) / TURN).clamp(0.0, 1.0);
            let ang = PI * turn * turn * (3.0 - 2.0 * turn);
            let (x, mid, h, hw) = (238.0, 151.0, 14.0, 12.0);
            let (ca, sa) = (ang.cos(), ang.sin());
            let rot = |px: f32, py: f32| (x + (px - x) * ca - (py - mid) * sa, mid + (px - x) * sa + (py - mid) * ca);
            let shape = |pts: &[(f32, f32)]| poly(&pts.iter().map(|&(px, py)| rot(px, py)).collect::<Vec<_>>());
            let seg = |x1: f32, y1: f32, x2: f32, y2: f32| {
                let (a, b) = (rot(x1, y1), rot(x2, y2));
                line(a.0, a.1, b.0, b.1)
            };
            let left = 1.0 - k;
            c.fill(&shape(&[(x - hw * left, mid - h * left), (x + hw * left, mid - h * left), (x, mid)]), SAND);
            c.fill(&shape(&[(x - hw, mid + h), (x + hw, mid + h), (x, mid + h - h * k)]), SAND);
            if k < 1.0 {
                c.stroke(&seg(x, mid, x, mid + h - h * k), SAND, 1.2);
            }
            let frame = CASE;
            c.stroke(&shape(&[(x - hw, mid - h), (x + hw, mid - h), (x, mid)]), frame, 1.6);
            c.stroke(&shape(&[(x, mid), (x + hw, mid + h), (x - hw, mid + h)]), frame, 1.6);
            c.stroke(&seg(x - hw - 3.0, mid - h - 1.5, x + hw + 3.0, mid - h - 1.5), frame, 2.6);
            c.stroke(&seg(x - hw - 3.0, mid + h + 1.5, x + hw + 3.0, mid + h + 1.5), frame, 2.6);
        } else if mood == Some(Mood::Problem) {
            // a blinking red X
            let a = if (t * 2.0).fract() < 0.5 { 1.0 } else { 0.55 };
            c.stroke(&line(226.0, 139.0, 250.0, 163.0), RED.fade(a), 5.0);
            c.stroke(&line(250.0, 139.0, 226.0, 163.0), RED.fade(a), 5.0);
        } else if mood.is_none() && p.cheer.is_some() {
            // a big green tick
            c.stroke(&polyline(&[(224.0, 151.0), (234.0, 161.0), (254.0, 139.0)]), GREEN, 5.0);
        } else if let Some(m @ (Mood::Testing | Mood::Building | Mood::Researching | Mood::Transfer(_) | Mood::Thinking)) = mood {
            self.mood_screen(c, t, m, p);
        } else if self.busy && self.screen != 0 {
            self.busy_screen(c, t);
            if self.egg {
                self.screen_egg(c, t);
            }
        } else if self.busy {
            // code scrolls up as it is typed
            let lh = 8.0;
            let off = t * 12.0;
            let first = (off / lh).floor() as i64;
            let frac = off - first as f32 * lh;
            for i in 0..7i64 {
                let n = first + i;
                let y = 125.0 + i as f32 * lh - frac;
                let mut x = 204.0 + (hash(n + 7) % 3) as f32 * 6.0;
                let toks = 1 + hash(n + 1) % 3;
                let lens: Vec<f32> = (0..toks).map(|k| 6.0 + (hash(n * 7 + k as i64) % 18) as f32).collect();
                let total: f32 = lens.iter().sum::<f32>() + 4.0 * (toks as f32 - 1.0);
                let budget = if i == 6 { total * (frac / lh) } else { total };
                let mut used = 0.0;
                for (k, len) in lens.iter().enumerate() {
                    let len = len.min(budget - used);
                    if len <= 0.5 {
                        break;
                    }
                    let col = CODE[(hash(n * 3 + k as i64) % 4) as usize];
                    c.fill(&rrect(x, y, len, 3.6, 1.8), col);
                    x += len + 4.0;
                    used += len + 4.0;
                }
                if i == 6 && (t * 2.5) as i64 % 2 == 0 {
                    c.fill(&rrect(x - 2.0, y - 1.0, 4.0, 6.0, 1.0), CODE[0]);
                }
            }
        } else {
            // sleepy screen: a moon and twinkling stars
            c.fill(&circle(256.0, 137.0, 7.5), Rgba::rgb(255, 230, 150));
            c.fill(&circle(260.5, 133.5, 7.0), SCREEN_OFF);
            let stars = [(212.0, 131.0), (228.0, 147.0), (241.0, 168.0), (266.0, 163.0), (216.0, 170.0), (250.0, 152.0)];
            for (i, (x, y)) in stars.iter().enumerate() {
                let a = 0.3 + 0.7 * (0.5 + 0.5 * (t * 1.7 + i as f32 * 1.9).sin());
                let col = Rgba::rgb(230, 226, 255).fade(a);
                c.stroke(&line(x - 2.2, *y, x + 2.2, *y), col, 1.2);
                c.stroke(&line(*x, y - 2.2, *x, y + 2.2), col, 1.2);
            }
        }
        if self.busy && self.egg && self.mood == Mood::Busy && self.screen == 0 {
            self.screen_egg(c, t);
        }
        // glass glare
        c.fill(&poly(&[(203.0, 124.0), (226.0, 124.0), (208.0, 150.0), (203.0, 150.0)]), WHITE.fade(0.10));
        c.clip(None);
        c.stroke(&screen, INK, 1.6);
    }

    fn body(&self, c: &mut Canvas, p: &Pose) {
        let top = p.hy + 22.0;
        let h = 224.0 - top;
        c.blob(&oval(CX - 40.0, top, 80.0, h), self.coat.fur);
        // patches run over the tummy (drawn first), markings sit beside it
        let tummy = oval(CX - 22.0, top + 14.0, 44.0, h - 12.0);
        let patched = self.coat.patch.is_some();
        if patched {
            c.fill(&tummy, self.coat.cream);
        }
        if self.coat.patch.is_some() {
            let body = oval(CX - 40.0, top, 80.0, h);
            c.clip(Some(&body));
            for (shape, col) in self.patches(CX, top + h * 0.5, 40.0, h * 0.5, 1, 2, 7) {
                self.soft_patch(c, &shape, col);
            }
            c.clip(None);
            c.stroke(&body, INK, OUTLINE);
        }
        self.marks_on(c, CX, top + h * 0.5, 36.0, h * 0.45, &BODY_STARS);
        if patched {
            // the patches show through the tummy, lighter
            soft_cream(c, CX - 22.0, top + 14.0, 44.0, h - 12.0, self.coat.cream, 1.0);
        } else {
            c.fill(&tummy, self.coat.cream);
        }
    }

    /// One patch with a soft edge: lighter bands just outside its outline, so it melts into the fur
    fn soft_patch(&self, c: &mut Canvas, shape: &[(f32, f32)], col: Rgba) {
        let n = shape.len() as f32;
        let (mx, my) = shape.iter().fold((0.0, 0.0), |(x, y), p| (x + p.0 / n, y + p.1 / n));
        for (k, a) in [(1.1, 0.35), (1.05, 0.5), (1.0, 1.0)] {
            let ring: Vec<(f32, f32)> = shape.iter().map(|p| (mx + (p.0 - mx) * k, my + (p.1 - my) * k)).collect();
            c.fill(&poly(&ring), col.fade(a));
        }
    }

    /// The coat's patches inside the ellipse at (`cx`, `cy`) with radii `rx`, `ry`: `min`..=`max`
    /// lumpy blobs placed, sized, stretched and turned from `patch_seed` (`salt` keeps the head
    /// and the body apart), each point kept just inside the ellipse so none spills past the outline.
    fn patches(&self, cx: f32, cy: f32, rx: f32, ry: f32, min: u32, max: u32, salt: u32) -> Vec<(Vec<(f32, f32)>, Rgba)> {
        let Some(main) = self.coat.patch else { return Vec::new() };
        let mut state = self.patch_seed.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B) | 1;
        let mut rnd = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state >> 8) as f32 / (1u32 << 24) as f32
        };
        let n = min + (rnd() * (max - min + 1) as f32) as u32;
        (0..n)
            .map(|i| {
                let (px, py) = (cx + (rnd() * 1.4 - 0.7) * rx, cy + (rnd() * 1.4 - 0.7) * ry);
                let (a, b) = (rx * (0.22 + rnd() * 0.3), ry * (0.22 + rnd() * 0.3));
                let turn = rnd() * PI;
                let (w1, w2, k1, k2) = (rnd() * 0.12, rnd() * 0.06, 2.0 + (rnd() * 3.0).floor(), 4.0 + (rnd() * 3.0).floor());
                let ph = rnd() * 6.28;
                let shape = (0..36)
                    .map(|j| {
                        let t = j as f32 / 36.0 * 2.0 * PI;
                        let lump = 1.0 + w1 * (k1 * t + ph).sin() + w2 * (k2 * t - ph).cos();
                        let (ex, ey) = (a * lump * t.cos(), b * lump * t.sin());
                        let (mut x, mut y) = (px + ex * turn.cos() - ey * turn.sin(), py + ex * turn.sin() + ey * turn.cos());
                        let r = ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2);
                        if r > 1.08 * 1.08 {
                            let s = 1.08 / r.sqrt();
                            (x, y) = (cx + (x - cx) * s, cy + (y - cy) * s);
                        }
                        (x, y)
                    })
                    .collect();
                let col = if self.coat.tri && i % 2 == 1 { self.coat.dark } else { main };
                (shape, col)
            })
            .collect()
    }

    /// The coat's markings inside the ellipse at (`cx`, `cy`) with radii `rx`, `ry`; `spots` are
    /// (x, y, size) in -1..1 of the radii.
    fn marks_on(&self, c: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, spots: &[(f32, f32, f32)]) {
        let at = |x: f32, y: f32| (cx + x * rx, cy + y * ry);
        match self.coat.marks {
            Marks::Galaxy => self.starry(c, cx, cy, rx, ry, spots),
            Marks::Dots(col) => {
                for &(x, y, s) in spots {
                    let (px, py) = at(x, y);
                    c.fill(&circle(px, py, 1.6 + s * 0.7), col);
                }
            }
            Marks::Hearts(col) => {
                for &(x, y, s) in spots.iter().filter(|s| s.2 >= 1.0) {
                    let (px, py) = at(x, y);
                    let r = 1.4 + s * 0.45;
                    c.fill(&circle(px - r * 0.7, py, r), col);
                    c.fill(&circle(px + r * 0.7, py, r), col);
                    c.fill(&poly(&[(px - r * 1.6, py + r * 0.3), (px + r * 1.6, py + r * 0.3), (px, py + r * 2.4)]), col);
                }
            }
            Marks::Clouds => {
                for &(x, y, s) in spots.iter().filter(|s| s.2 >= 1.6) {
                    let (px, py) = at(x, y);
                    let r = 2.6 + s;
                    for (dx, dy, k) in [(-1.0f32, 0.3f32, 0.8f32), (0.0, -0.2, 1.0), (1.0, 0.3, 0.8)] {
                        c.fill(&circle(px + dx * r, py + dy * r, r * k), Rgba(255, 255, 255, 235));
                    }
                }
            }
            Marks::Strawberry => {
                for &(x, y, s) in spots {
                    let (px, py) = at(x, y);
                    c.fill(&oval(px - 1.1, py - 1.7, 2.2, 3.4 + s * 0.3), Rgba(255, 226, 120, 255));
                }
            }
            Marks::Blush(col) => {
                c.fill(&oval(cx - rx * 0.95, cy + ry * 0.05, rx * 0.7, ry * 0.75), col);
                c.fill(&oval(cx + rx * 0.25, cy + ry * 0.05, rx * 0.7, ry * 0.75), col);
            }
            Marks::Plain | Marks::Rainbow | Marks::Ghost => {}
        }
    }

    /// Nebula glows and stars inside the ellipse at (`cx`, `cy`) with radii `rx`, `ry`; `stars` are
    /// (x, y, size) in -1..1 of the radii.
    fn starry(&self, c: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, stars: &[(f32, f32, f32)]) {
        c.fill(&oval(cx - rx * 0.9, cy - ry * 0.7, rx * 1.1, ry * 0.9), Rgba(150, 90, 220, 70));
        c.fill(&oval(cx + rx * 0.05, cy - ry * 0.1, rx * 0.8, ry * 0.75), Rgba(236, 110, 190, 45));
        for &(x, y, s) in stars {
            let (sx, sy) = (cx + x * rx, cy + y * ry);
            if s > 1.5 {
                c.stroke(&line(sx - s, sy, sx + s, sy), Rgba(255, 244, 200, 230), 1.1);
                c.stroke(&line(sx, sy - s, sx, sy + s), Rgba(255, 244, 200, 230), 1.1);
            }
            c.fill(&circle(sx, sy, 0.7 + s * 0.25), Rgba(255, 255, 255, 220));
        }
    }

    fn desk(&self, c: &mut Canvas) {
        c.blob(&rrect(28.0, 222.0, 10.0, 30.0, 2.0), WOOD_FRONT);
        c.blob(&rrect(262.0, 222.0, 10.0, 30.0, 2.0), WOOD_FRONT);
        c.blob(&rrect(14.0, 215.0, 272.0, 10.0, 3.0), WOOD_FRONT);
        c.blob(&rrect(10.0, 208.0, 280.0, 9.0, 3.5), WOOD_TOP);
        c.stroke(&line(24.0, 212.5, 70.0, 212.5), WOOD_FRONT.fade(0.5), 1.0);
        c.stroke(&line(240.0, 212.0, 276.0, 212.0), WOOD_FRONT.fade(0.5), 1.0);
    }

    /// The kettle in the left paw, held by its handle at (hx, hy), spout to the right, tipped by 	ilt (radians,
    /// positive pours to the right), with the stream into the mug at mug while it is tipped far enough.
    fn kettle(&self, c: &mut Canvas, (hx, hy, tilt): (f32, f32, f32), mug: (f32, f32)) {
        let (s, k) = tilt.sin_cos();
        // laid out spout-left and mirrored; drawn half again as big as its outline numbers, so it reads at the
        // kitten's size
        let at = |x: f32, y: f32| (hx + 1.5 * (-x * k - y * s), hy + 1.5 * (-x * s + y * k));
        let red = Rgba::rgb(222, 96, 84);
        let body: Vec<(f32, f32)> = (0..28).map(|i| {
            let a = i as f32 / 28.0 * 2.0 * PI;
            at(-13.0 + 11.5 * a.cos(), 6.0 + 9.0 * a.sin())
        }).collect();
        if tilt > 0.4 {
            let tip = at(-29.0, -2.0);
            let (mx, my) = (mug.0, mug.1 - 12.0);
            c.stroke(&quad(tip.0, tip.1, tip.0 + 1.5, (tip.1 + my) / 2.0, mx, my), Rgba::rgb(120, 76, 52), 2.4);
        }
        c.blob(&poly(&[at(-21.0, 3.0), at(-30.0, -3.5), at(-28.0, -5.0), at(-19.0, -1.0)]), red);
        c.blob(&poly(&body), red);
        c.fill(&poly(&[at(-20.0, 1.0), at(-16.0, -1.5), at(-12.0, -0.5), at(-17.0, 4.0)]), WHITE.fade(0.35));
        c.blob(&poly(&[at(-18.0, -2.0), at(-8.0, -2.0), at(-9.0, -4.5), at(-17.0, -4.5)]), Rgba::rgb(200, 80, 72));
        let knob = at(-13.0, -6.0);
        c.blob(&circle(knob.0, knob.1, 1.8), Rgba::rgb(60, 60, 70));
        let (a, b, e) = (at(-4.0, 0.5), at(2.0, -2.0), at(-2.5, 11.0));
        c.stroke(&quad(a.0, a.1, b.0, b.1 + 6.0, e.0, e.1), INK, 4.4);
        c.stroke(&quad(a.0, a.1, b.0, b.1 + 6.0, e.0, e.1), Rgba::rgb(60, 60, 70), 2.4);
    }

    /// The mug at `at`, `fill` full (0 shows its empty inside), tipped toward the mouth by `tip` (0..1).
    fn mug(&self, c: &mut Canvas, t: f32, at: (f32, f32), fill: f32, tip: f32) {
        let (dx, dy) = (at.0 - MUG_HOME.0, at.1 - MUG_HOME.1);
        // tipping turns it about its base so the rim leans left, to the mouth
        let turn = Transform::from_rotate_at(-35.0 * tip, 179.0 + dx, 208.0 + dy);
        let tp = |p: Path| if tip > 0.0 { p.clone().transform(turn).unwrap_or(p) } else { p };
        // drawn MUG_SCALE times its outline numbers, grown from the middle of its bottom (so it still sits on the desk)
        let s = MUG_SCALE;
        let m = |x: f32, y: f32| (179.0 + dx + (x - 179.0) * s, 208.0 + dy + (y - 208.0) * s);
        let (hx, hy) = m(181.0, 197.0);
        let handle = tp(oval(hx, hy, 8.0 * s, 8.0 * s));
        c.stroke(&handle, INK, 4.6 * s);
        c.stroke(&handle, MUG, 2.2 * s);
        let (bx, by) = m(173.0, 193.0);
        c.blob(&tp(rrect(bx, by, 12.0 * s, 15.0 * s, 3.0 * s)), MUG);
        let (ix, iy) = m(174.5, 193.5);
        c.fill(&tp(oval(ix, iy, 9.0 * s, 3.0 * s)), Rgba::rgb(86, 150, 182));
        c.fill(&tp(oval(ix, iy, 9.0 * s, 3.0 * s)), Rgba::rgb(120, 76, 52).fade(fill));
        let (l0, l1) = (m(175.0, 201.0), m(183.0, 201.0));
        c.stroke(&tp(line(l0.0, l0.1, l1.0, l1.1)), WHITE.fade(0.7), 1.6 * s);
        // steam rises only while a full mug sits on the desk
        if self.busy && dx == 0.0 && dy == 0.0 && fill >= 1.0 {
            for i in 0..2 {
                let ph = (t * 0.45 + i as f32 * 0.5).fract();
                let (x0, y0) = m(177.0 + i as f32 * 4.0, 190.0);
                let y0 = y0 - ph * 18.0;
                let a = (ph * PI).sin() * 0.7;
                let wob = 2.5 * (ph * 6.0 + i as f32).sin();
                c.stroke(&quad(x0, y0 + 6.0, x0 + wob, y0 + 3.0, x0, y0), WHITE.fade(a), 1.8);
            }
        }
    }

    fn keyboard(&self, c: &mut Canvas, p: &Pose) {
        // body with a front edge, three rows of keys and a space bar
        c.blob(&rrect(KX + 1.0, KY + 12.0, KW - 2.0, 9.0, 3.0), KB_FRONT);
        c.blob(&rrect(KX, KY, KW, 16.0, 4.0), KB_BODY);
        let hot = |col: usize| {
            (p.l.down && p.l.key == Some(col)) || (p.r.down && p.r.key == Some(col))
        };
        let key = |c: &mut Canvas, x: f32, y: f32, w: f32, pressed: bool| {
            if pressed {
                c.fill(&rrect(x, y + 0.9, w, KEY_H, 1.3), KEY_HOT);
            } else {
                c.fill(&rrect(x, y + 1.0, w, KEY_H, 1.3), KEY_SHADE);
                c.fill(&rrect(x, y, w, KEY_H, 1.3), KEY);
            }
        };
        for col in 0..11 {
            key(c, key_x(col), KY + 2.3, KEY_W, hot(col));
        }
        for col in 0..10 {
            key(c, key_x(col) + KEY_PITCH / 2.0, KY + 6.8, KEY_W, false);
        }
        let y3 = KY + 11.3;
        key(c, key_x(0), y3, KEY_W, false);
        key(c, key_x(1), y3, KEY_W, false);
        key(c, key_x(2), y3, key_x(8) + KEY_W - key_x(2), false);
        key(c, key_x(9), y3, KEY_W, false);
        key(c, key_x(10), y3, KEY_W, false);
    }

    fn arms(&self, c: &mut Canvas, p: &Pose) {
        let rest_y = if self.busy { 181.0 } else { 179.0 };
        // while waiting the left arm props the chin and is drawn after the head
        let chin = self.busy && self.mood.chin();
        for (paw, sx) in [(p.l, -24.0), (p.r, 24.0)] {
            // the sipping arm is drawn after the head too, so it does not slip under the face
            if p.refill.is_some() || (chin && sx < 0.0) || (p.sip.is_some() && sx > 0.0) {
                continue;
            }
            let py = rest_y - paw.lift;
            let arm = line(CX + sx, p.hy + 46.0, paw.x + PAW_W / 2.0, py + 5.0);
            c.stroke(&arm, INK, 14.5);
            c.stroke(&arm, self.coat.fur, 10.5);
            c.blob(&oval(paw.x, py, PAW_W, 13.0), self.coat.fur);
            // toes point down at the keys, so the toe lines sit at the front edge
            for tx in [6.5, 10.0, 13.5] {
                c.stroke(&line(paw.x + tx, py + 12.0, paw.x + tx, py + 8.6), INK.fade(0.8), 1.1);
            }
        }
    }

    fn head(&self, c: &mut Canvas, p: &Pose) {
        let hy = p.hy;
        // ears (one flicks now and then)
        // ears droop sideways when something went wrong
        let d = p.droop;
        let mix = |a: f32, b: f32| a + (b - a) * d;
        for s in [-1.0f32, 1.0] {
            let tw = if s == p.twitch_ear { p.twitch } else { 0.0 };
            let tip = (CX + s * mix(37.0 + tw * 0.6, 60.0), hy + mix(-53.0 + tw, -30.0));
            let inner = (CX + s * mix(34.5 + tw * 0.6, 52.0), hy + mix(-44.0 + tw, -28.0));
            // which ears take a patch color is rolled with the patches
            let pick = (self.patch_seed.wrapping_mul(0x2545_F491) >> if s < 0.0 { 20 } else { 26 }) & 3;
            let ear = match (self.coat.patch, pick) {
                (Some(p), 0) => p,
                (Some(_), 1) if self.coat.tri => self.coat.dark,
                _ => self.coat.fur,
            };
            c.blob(&poly(&[(CX + s * 46.0, hy - 10.0), tip, (CX + s * 11.0, hy - 34.0)]), ear);
            c.fill(&poly(&[(CX + s * 37.0, hy - 18.0), inner, (CX + s * 19.0, hy - 32.0)]), self.coat.pink);
        }
        // head
        c.blob(&oval(CX - 51.0, hy - 40.0, 102.0, 80.0), self.coat.fur);
        // patches run over the muzzle like real markings (it is drawn first for patched coats)
        let muzzle = oval(CX - 16.0, hy + 3.0, 32.0, 20.0);
        if self.coat.patch.is_some() {
            c.fill(&muzzle, self.coat.cream);
        }
        if self.coat.patch.is_some() {
            // clipped to the head, so a patch runs right up to the outline (drawn again over it)
            let head = oval(CX - 51.0, hy - 40.0, 102.0, 80.0);
            c.clip(Some(&head));
            for (shape, col) in self.patches(CX, hy, 51.0, 40.0, 1, 3, 1) {
                self.soft_patch(c, &shape, col);
            }
            c.clip(None);
            c.stroke(&head, INK, OUTLINE);
        }
        self.marks_on(c, CX, hy - 6.0, 46.0, 32.0, &HEAD_STARS);
        if matches!(self.coat.marks, Marks::Strawberry) {
            let leaf = Rgba::rgb(96, 186, 92);
            for (dx, tip) in [(-9.0f32, (-17.0f32, -43.0f32)), (0.0, (0.0, -47.0)), (9.0, (17.0, -43.0))] {
                c.blob(&poly(&[(CX + dx - 5.0, hy - 37.0), (CX + tip.0, hy + tip.1), (CX + dx + 5.0, hy - 37.0)]), leaf);
            }
        }
        if matches!(self.coat.marks, Marks::Rainbow) {
            for (i, x) in [-24.0f32, -12.0, 0.0, 12.0, 24.0].into_iter().enumerate() {
                c.stroke(&line(CX + x, hy - 37.0 + x.abs() * 0.12, CX + x * 0.85, hy - 25.0), RAINBOW[i], 4.0);
            }
            for (i, dy) in [-6.0f32, 0.0].into_iter().enumerate() {
                c.stroke(&quad(CX - 50.0, hy + dy, CX - 44.0, hy + 2.0 + dy, CX - 39.0, hy - 4.0 + dy), RAINBOW[4 + i], 2.8);
                c.stroke(&quad(CX + 50.0, hy + dy, CX + 44.0, hy + 2.0 + dy, CX + 39.0, hy - 4.0 + dy), RAINBOW[i], 2.8);
            }
        } else if self.coat.stripes {
            for (x0, x1) in [(0.0, 0.0), (-11.0, -9.0), (11.0, 9.0)] {
                c.stroke(&line(CX + x0, hy - 37.0, CX + x1, hy - 27.0), self.coat.dark, 3.4);
            }
            c.stroke(&quad(CX - 50.0, hy - 2.0, CX - 44.0, hy, CX - 40.0, hy - 6.0), self.coat.dark, 2.6);
            c.stroke(&quad(CX + 50.0, hy - 2.0, CX + 44.0, hy, CX + 40.0, hy - 6.0), self.coat.dark, 2.6);
        }
        // on patched coats the patches show through the muzzle, lighter
        if self.coat.patch.is_some() {
            soft_cream(c, CX - 16.0, hy + 3.0, 32.0, 20.0, self.coat.cream, 1.18);
        } else {
            c.fill(&muzzle, self.coat.cream);
        }
        // eyes
        for ex in [CX - 22.0, CX + 22.0] {
            let ey = hy - 3.0;
            if p.eyes_closed {
                c.stroke(&quad(ex - 7.5, ey + 1.0, ex, ey + 7.0, ex + 7.5, ey + 1.0), self.coat.eye, 2.6);
            } else {
                let x = ex + p.look;
                if p.lid > 0.0 {
                    // the upper lid hides the top of the eye: only the part below it is drawn,
                    // so markings around the eye stay visible
                    let edge = ey - 9.0 + 18.0 * p.lid;
                    let open: Vec<(f32, f32)> = (0..32)
                        .map(|i| {
                            let a = i as f32 / 32.0 * 2.0 * PI;
                            (x + 7.5 * a.cos(), (ey + 9.0 * a.sin()).max(edge))
                        })
                        .collect();
                    c.fill(&poly(&open), self.coat.eye);
                    if ey - 4.0 > edge + 2.0 {
                        c.fill(&circle(x - 2.6, ey - 4.0, 3.3), WHITE);
                    }
                    if ey + 3.4 > edge + 1.0 {
                        c.fill(&circle(x + 2.8, ey + 3.4, 1.6), WHITE);
                    }
                    c.stroke(&line(x - 8.0, edge, x + 8.0, edge), self.coat.eye, 2.2);
                } else {
                    c.fill(&oval(x - 7.5, ey - 9.0, 15.0, 18.0), self.coat.eye);
                    c.fill(&circle(x - 2.6, ey - 4.0, 3.3), WHITE);
                    c.fill(&circle(x + 2.8, ey + 3.4, 1.6), WHITE);
                }
            }
        }
        // cheeks, nose, mouth, whiskers
        c.fill(&oval(CX - 43.0, hy + 6.0, 16.0, 9.0), BLUSH);
        c.fill(&oval(CX + 27.0, hy + 6.0, 16.0, 9.0), BLUSH);
        let nose = poly(&[(CX - 4.5, hy + 6.0), (CX + 4.5, hy + 6.0), (CX, hy + 10.5)]);
        c.fill(&nose, self.coat.pink);
        c.stroke(&nose, PINK_DARK, 1.2);
        if p.sipping > 0.15 {
            // open for the mouthful
            let h = 1.5 + 4.5 * p.sipping;
            c.blob(&oval(CX - 3.5, hy + 12.0, 7.0, h), Rgba::rgb(150, 60, 70));
        } else if p.droop > 0.5 {
            // a small worried frown
            c.stroke(&quad(CX - 6.0, hy + 16.0, CX, hy + 11.5, CX + 6.0, hy + 16.0), INK, 1.5);
        } else {
            c.stroke(&quad(CX - 7.0, hy + 12.0, CX - 3.5, hy + 16.5, CX, hy + 12.0), INK, 1.5);
            c.stroke(&quad(CX, hy + 12.0, CX + 3.5, hy + 16.5, CX + 7.0, hy + 12.0), INK, 1.5);
        }
        for s in [-1.0f32, 1.0] {
            c.stroke(&line(CX + s * 31.0, hy + 9.0, CX + s * 58.0, hy + 5.0), INK.fade(0.6), 1.1);
            c.stroke(&line(CX + s * 31.0, hy + 13.0, CX + s * 57.0, hy + 16.0), INK.fade(0.6), 1.1);
        }
    }

    fn sparkles(&self, c: &mut Canvas, p: &Pose) {
        for paw in [p.l, p.r] {
            if !paw.down {
                continue;
            }
            let strength = (1.0 - paw.lift).clamp(0.0, 1.0);
            let col = SPARK.fade(strength);
            let x = paw.x + PAW_W / 2.0;
            let y = 181.0;
            c.stroke(&line(x - 13.0, y + 3.0, x - 17.0, y - 1.0), col, 1.5);
            c.stroke(&line(x + 13.0, y + 3.0, x + 17.0, y - 1.0), col, 1.5);
            c.stroke(&line(x, y - 4.0, x, y - 8.0), col, 1.5);
        }
    }

    fn zzz(&self, c: &mut Canvas, t: f32, p: &Pose) {
        let Some(font) = self.bold.as_ref() else { return };
        for i in 0..3 {
            let ph = (t * 0.3 + i as f32 / 3.0).fract();
            let x = CX + 46.0 + ph * 30.0 + 3.0 * (ph * 6.0).sin();
            let y = p.hy - 30.0 - ph * 46.0;
            let size = 11.0 + ph * 9.0;
            c.text(font, size, x, y, "z", ZZZ.fade((ph * PI).sin()));
        }
    }

    fn bubble(&self, c: &mut Canvas, t: f64, p: &Pose) {
        let (Some(bold), Some(regular)) = (self.bold.as_ref(), self.regular.as_ref()) else { return };
        let (main, sub) = if self.busy {
            let m = ((t - self.since) / 60.0).max(0.0) as i64;
            let (start, so_far) = if self.mood == Mood::Waiting { ("just started", "waiting") } else { ("just started", "so far") };
            let sub = match m {
                0 => start.to_string(),
                1..=59 => format!("{m} min {so_far}"),
                _ => format!("{} h {} min {so_far}", m / 60, m % 60),
            };
            (self.task.clone(), Some(sub))
        } else if p.cheer.is_some() {
            ("all done!".to_string(), None)
        } else {
            ("napping... no jobs right now".to_string(), None)
        };
        let (ms, ss, lh) = (12.5, 10.5, 15.5);
        let lines = wrap(bold, ms, &main, 214.0, 2);
        let tw = lines.iter().map(|l| measure(bold, ms, l)).fold(0.0, f32::max);
        let sw = sub.as_ref().map(|s| measure(regular, ss, s)).unwrap_or(0.0);
        let bw = (tw.max(sw) + 24.0).clamp(96.0, 240.0);
        let bh = 8.0 + lines.len() as f32 * lh + if sub.is_some() { 13.0 } else { 0.0 } + 7.0;
        let bottom = 68.0;
        let by = bottom - bh;
        let bx = (CX - bw * 0.32).clamp(4.0, W - 4.0 - bw);
        let fill = WHITE.fade(if self.busy || p.cheer.is_some() { 0.98 } else { 0.86 });

        let body = rrect(bx, by, bw, bh, 11.0);
        c.fill(&body, fill);
        c.stroke(&body, INK, 1.8);
        let tip = (CX - 8.0, 82.0);
        c.fill(&poly(&[(CX - 13.0, bottom - 1.6), tip, (CX + 2.0, bottom - 1.6)]), fill);
        c.stroke(&line(CX - 13.0, bottom, tip.0, tip.1), INK, 1.8);
        c.stroke(&line(tip.0, tip.1, CX + 2.0, bottom), INK, 1.8);

        let col = if self.busy || p.cheer.is_some() { INK } else { SUB };
        for (i, l) in lines.iter().enumerate() {
            c.text(bold, ms, bx + 12.0, by + 8.0 + 12.0 + i as f32 * lh, l, col);
        }
        if let Some(s) = sub {
            c.text(regular, ss, bx + 12.0, by + 8.0 + lines.len() as f32 * lh + 10.5, &s, SUB);
        }
    }
}
