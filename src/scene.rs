//! The kitten, its desk and its little computer.
//!
//! Everything is drawn in a 300 x 250 design space. `update` advances the animation
//! state (blinks, ear twitches, paw positions); `render` draws one frame from it.

use std::f32::consts::PI;

use tiny_skia::{Color, Pixmap};

use crate::canvas::*;

pub const W: f32 = 300.0;
pub const H: f32 = 250.0;

const CX: f32 = 105.0; // kitten centre line

// palette
const FUR: Rgba = Rgba::rgb(247, 178, 102);
const FUR_DARK: Rgba = Rgba::rgb(224, 136, 70);
const CREAM: Rgba = Rgba::rgb(255, 238, 216);
const PINK: Rgba = Rgba::rgb(255, 170, 186);
const PINK_DARK: Rgba = Rgba::rgb(226, 112, 134);
const BLUSH: Rgba = Rgba(255, 128, 150, 95);
const EYE: Rgba = Rgba::rgb(50, 34, 36);
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
}

impl Mood {
    fn of(s: &str) -> Mood {
        let word: String = s.chars().take_while(|c| c.is_alphabetic()).collect::<String>().to_lowercase();
        match word.as_str() {
            "waiting" | "wait" => Mood::Waiting,
            "failed" | "fail" | "error" | "problem" => Mood::Problem,
            _ => Mood::Busy,
        }
    }
}

/// Typing screens: scrolling code, progress bars, equalizer, gear, line graph.
const SCREENS: usize = 5;

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
}

#[derive(Clone, Copy)]
struct Paw {
    x: f32,
    lift: f32,
    key: Option<usize>,
    down: bool,
}

pub struct Scene {
    pub busy: bool,
    pub task: String,
    pub mood: Mood,
    /// which typing screen shows (0..SCREENS), picked at random per status line
    pub screen: usize,
    /// time (seconds) the current task started
    pub since: f64,
    /// time the last job finished (busy -> idle), for the "all done!" moment
    pub done_at: f64,
    rng: u64,
    next_blink: f64,
    next_twitch: f64,
    twitch_start: f64,
    twitch_ear: f32,
    lpx: f32,
    rpx: f32,
    last_t: f64,
    bold: Option<fontdue::Font>,
    regular: Option<fontdue::Font>,
}

fn load_font(names: &[&str]) -> Option<fontdue::Font> {
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
            since: 0.0,
            done_at: -100.0,
            rng: 0x2545_F491_4F6C_DD1D,
            next_blink: 2.0,
            next_twitch: 4.0,
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
        let typing = busy && self.mood == Mood::Busy;
        let waiting = busy && self.mood == Mood::Waiting;
        let problem = busy && self.mood == Mood::Problem;

        let (hy, sway) = if typing {
            (126.0 + 1.0 * (tf * 2.0 * PI * 2.5).sin(), 8.0 * (tf * 2.4).sin())
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
            let tap = if beat < 0.6 { (beat * 2.0 * PI * 5.0).sin().max(0.0) } else { 0.0 };
            r = Paw { x: CX + 12.0, lift: 5.0 * tap, key: None, down: false };
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
        if self.lpx < 0.0 {
            self.lpx = l.x;
            self.rpx = r.x;
        }
        let k = 1.0 - (-dt * if typing || cheer.is_some() { 14.0 } else { 4.0 }).exp();
        if !typing || l.lift > 3.0 {
            self.lpx += (l.x - self.lpx) * k;
        }
        if !typing || r.lift > 3.0 {
            self.rpx += (r.x - self.rpx) * k;
        }
        l.x = self.lpx;
        r.x = self.rpx;
        l.down = typing && l.lift < 1.0;
        r.down = typing && r.lift < 1.0;

        // looks at the screen while typing and glances back at you every so often;
        // while waiting it looks off to the side now and then
        let look = if typing && (t % 9.0) > 1.3 {
            2.5
        } else if waiting && (t % 5.0) > 3.4 {
            // a glance over at the hourglass
            3.5
        } else {
            0.0
        };
        // heavy, bored eyelids while waiting (a little less during the glance)
        let lid = if waiting { if look > 0.0 { 0.25 } else { 0.5 } } else { 0.0 };

        Pose {
            hy,
            sway,
            // asleep, or squeezed shut with joy while cheering
            eyes_closed: !busy || blink,
            look,
            twitch,
            twitch_ear: self.twitch_ear,
            l,
            r,
            cheer,
            droop: if problem { 1.0 } else { 0.0 },
            lid,
        }
    }

    pub fn render(&self, pix: &mut Pixmap, sc: f32, t: f64, p: &Pose) {
        pix.fill(Color::TRANSPARENT);
        let mut c = Canvas::new(pix, sc);
        let tf = t as f32;
        self.tail(&mut c, p);
        self.computer(&mut c, tf, p);
        self.body(&mut c, p);
        self.desk(&mut c);
        self.mug(&mut c, tf);
        self.keyboard(&mut c, p);
        self.arms(&mut c, p);
        self.head(&mut c, p);
        if self.busy && self.mood == Mood::Waiting {
            // chin propped on an upright forearm: elbow on the keyboard, paw cupping the jaw,
            // drawn in front of the head
            let (px, py) = (CX - 33.0, p.hy + 12.0);
            let arm = line(CX - 38.0, KY + 4.0, px + 8.0, py + 9.0);
            c.stroke(&arm, INK, 14.5);
            c.stroke(&arm, FUR, 10.5);
            c.blob(&oval(px, py, 18.0, 15.0), FUR);
            // toes curl in towards the face
            for (dx, dy) in [(12.5, 3.5), (14.5, 7.0), (13.5, 10.5)] {
                c.stroke(&line(px + dx, py + dy, px + dx + 3.5, py + dy), INK.fade(0.8), 1.1);
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
        c.stroke(&path, FUR, 9.0);
        // darker tip
        let tip = polyline(&line[19..]);
        c.stroke(&tip, FUR_DARK, 9.0);
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
            Some(Mood::Busy) => GREEN.fade(if (t * 3.0).fract() < 0.6 { 1.0 } else { 0.45 }),
            Some(Mood::Waiting) => Rgba::rgb(255, 186, 84).fade(breathe(4.0)),
            Some(Mood::Problem) => RED.fade(if (t * 2.0).fract() < 0.5 { 1.0 } else { 0.3 }),
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
        } else if self.busy && self.screen != 0 {
            self.busy_screen(c, t);
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
        // glass glare
        c.fill(&poly(&[(203.0, 124.0), (226.0, 124.0), (208.0, 150.0), (203.0, 150.0)]), WHITE.fade(0.10));
        c.clip(None);
        c.stroke(&screen, INK, 1.6);
    }

    fn body(&self, c: &mut Canvas, p: &Pose) {
        let top = p.hy + 22.0;
        let h = 224.0 - top;
        c.blob(&oval(CX - 40.0, top, 80.0, h), FUR);
        c.fill(&oval(CX - 22.0, top + 14.0, 44.0, h - 12.0), CREAM);
    }

    fn desk(&self, c: &mut Canvas) {
        c.blob(&rrect(28.0, 222.0, 10.0, 30.0, 2.0), WOOD_FRONT);
        c.blob(&rrect(262.0, 222.0, 10.0, 30.0, 2.0), WOOD_FRONT);
        c.blob(&rrect(14.0, 215.0, 272.0, 10.0, 3.0), WOOD_FRONT);
        c.blob(&rrect(10.0, 208.0, 280.0, 9.0, 3.5), WOOD_TOP);
        c.stroke(&line(24.0, 212.5, 70.0, 212.5), WOOD_FRONT.fade(0.5), 1.0);
        c.stroke(&line(240.0, 212.0, 276.0, 212.0), WOOD_FRONT.fade(0.5), 1.0);
    }

    fn mug(&self, c: &mut Canvas, t: f32) {
        let handle = oval(181.0, 197.0, 8.0, 8.0);
        c.stroke(&handle, INK, 4.6);
        c.stroke(&handle, MUG, 2.2);
        c.blob(&rrect(173.0, 193.0, 12.0, 15.0, 3.0), MUG);
        c.fill(&oval(174.5, 193.5, 9.0, 3.0), Rgba::rgb(120, 76, 52));
        c.stroke(&line(175.0, 201.0, 183.0, 201.0), WHITE.fade(0.7), 1.6);
        if self.busy {
            for i in 0..2 {
                let ph = (t * 0.45 + i as f32 * 0.5).fract();
                let x0 = 177.0 + i as f32 * 4.0;
                let y0 = 190.0 - ph * 16.0;
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
        let chin = self.busy && self.mood == Mood::Waiting;
        for (paw, sx) in [(p.l, -24.0), (p.r, 24.0)] {
            if chin && sx < 0.0 {
                continue;
            }
            let py = rest_y - paw.lift;
            let arm = line(CX + sx, p.hy + 46.0, paw.x + PAW_W / 2.0, py + 5.0);
            c.stroke(&arm, INK, 14.5);
            c.stroke(&arm, FUR, 10.5);
            c.blob(&oval(paw.x, py, PAW_W, 13.0), FUR);
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
            c.blob(&poly(&[(CX + s * 46.0, hy - 10.0), tip, (CX + s * 11.0, hy - 34.0)]), FUR);
            c.fill(&poly(&[(CX + s * 37.0, hy - 18.0), inner, (CX + s * 19.0, hy - 32.0)]), PINK);
        }
        // head
        c.blob(&oval(CX - 51.0, hy - 40.0, 102.0, 80.0), FUR);
        // tabby stripes
        for (x0, x1) in [(0.0, 0.0), (-11.0, -9.0), (11.0, 9.0)] {
            c.stroke(&line(CX + x0, hy - 37.0, CX + x1, hy - 27.0), FUR_DARK, 3.4);
        }
        c.stroke(&quad(CX - 50.0, hy - 2.0, CX - 44.0, hy, CX - 40.0, hy - 6.0), FUR_DARK, 2.6);
        c.stroke(&quad(CX + 50.0, hy - 2.0, CX + 44.0, hy, CX + 40.0, hy - 6.0), FUR_DARK, 2.6);
        // muzzle
        c.fill(&oval(CX - 16.0, hy + 3.0, 32.0, 20.0), CREAM);
        // eyes
        for ex in [CX - 22.0, CX + 22.0] {
            let ey = hy - 3.0;
            if p.eyes_closed {
                c.stroke(&quad(ex - 7.5, ey + 1.0, ex, ey + 7.0, ex + 7.5, ey + 1.0), EYE, 2.6);
            } else {
                let x = ex + p.look;
                c.fill(&oval(x - 7.5, ey - 9.0, 15.0, 18.0), EYE);
                c.fill(&circle(x - 2.6, ey - 4.0, 3.3), WHITE);
                c.fill(&circle(x + 2.8, ey + 3.4, 1.6), WHITE);
                if p.lid > 0.0 {
                    // the upper lid covers the top of the eye
                    let edge = ey - 9.0 + 18.0 * p.lid;
                    c.fill(&rrect(x - 9.0, ey - 11.0, 18.0, edge - (ey - 11.0), 2.0), FUR);
                    c.stroke(&line(x - 8.0, edge, x + 8.0, edge), EYE, 2.2);
                }
            }
        }
        // cheeks, nose, mouth, whiskers
        c.fill(&oval(CX - 43.0, hy + 6.0, 16.0, 9.0), BLUSH);
        c.fill(&oval(CX + 27.0, hy + 6.0, 16.0, 9.0), BLUSH);
        let nose = poly(&[(CX - 4.5, hy + 6.0), (CX + 4.5, hy + 6.0), (CX, hy + 10.5)]);
        c.fill(&nose, PINK);
        c.stroke(&nose, PINK_DARK, 1.2);
        if p.droop > 0.5 {
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
