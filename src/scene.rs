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
    /// time (seconds) the current task started
    pub since: f64,
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
            since: 0.0,
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
        let s = s.trim();
        let busy = !s.is_empty() && !s.eq_ignore_ascii_case("idle");
        if busy && (!self.busy || s != self.task) {
            self.since = t;
        }
        self.busy = busy;
        self.task = if busy { s.to_string() } else { String::new() };
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

        let (hy, sway) = if busy {
            (126.0 + 1.0 * (tf * 2.0 * PI * 2.5).sin(), 8.0 * (tf * 2.4).sin())
        } else {
            (137.0 + 1.5 * (tf * 1.3).sin(), 3.0 * (tf * 0.8).sin())
        };

        // paws: busy = take turns tapping (about five taps a second), moving to a new key
        // while lifted; idle = folded on the keyboard under the chin
        let (mut l, mut r);
        if busy {
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
        let k = 1.0 - (-dt * if busy { 14.0 } else { 4.0 }).exp();
        if !busy || l.lift > 3.0 {
            self.lpx += (l.x - self.lpx) * k;
        }
        if !busy || r.lift > 3.0 {
            self.rpx += (r.x - self.rpx) * k;
        }
        l.x = self.lpx;
        r.x = self.rpx;
        l.down = busy && l.lift < 1.0;
        r.down = busy && r.lift < 1.0;

        // looks at the screen while typing, glances back at you every so often
        let look = if busy && (t % 9.0) > 1.3 { 2.5 } else { 0.0 };

        Pose { hy, sway, eyes_closed: !busy || blink, look, twitch, twitch_ear: self.twitch_ear, l, r }
    }

    pub fn render(&self, pix: &mut Pixmap, sc: f32, t: f64, p: &Pose) {
        pix.fill(Color::TRANSPARENT);
        let mut c = Canvas::new(pix, sc);
        let tf = t as f32;
        self.tail(&mut c, p);
        self.computer(&mut c, tf);
        self.body(&mut c, p);
        self.desk(&mut c);
        self.mug(&mut c, tf);
        self.keyboard(&mut c, p);
        self.arms(&mut c, p);
        self.head(&mut c, p);
        self.sparkles(&mut c, p);
        if !self.busy {
            self.zzz(&mut c, tf, p);
        }
        self.bubble(&mut c, t);
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

    fn computer(&self, c: &mut Canvas, t: f32) {
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
        let led = if self.busy {
            Rgba::rgb(110, 228, 140).fade(if (t * 3.0).fract() < 0.6 { 1.0 } else { 0.45 })
        } else {
            Rgba::rgb(255, 186, 84).fade(0.45 + 0.55 * (0.5 + 0.5 * (t * 1.5).sin()))
        };
        c.fill(&circle(277.0, 191.0, 2.8), led);
        c.stroke(&circle(277.0, 191.0, 2.8), INK.fade(0.6), 1.0);

        // screen
        let screen = rrect(199.0, 121.0, 78.0, 60.0, 7.0);
        c.fill(&screen, if self.busy { SCREEN_ON } else { SCREEN_OFF });
        c.clip(Some(&screen));
        if self.busy {
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
        for (paw, sx) in [(p.l, -24.0), (p.r, 24.0)] {
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
        for s in [-1.0f32, 1.0] {
            let tw = if s == p.twitch_ear { p.twitch } else { 0.0 };
            c.blob(
                &poly(&[(CX + s * 46.0, hy - 10.0), (CX + s * (37.0 + tw * 0.6), hy - 53.0 + tw), (CX + s * 11.0, hy - 34.0)]),
                FUR,
            );
            c.fill(
                &poly(&[(CX + s * 37.0, hy - 18.0), (CX + s * (34.5 + tw * 0.6), hy - 44.0 + tw), (CX + s * 19.0, hy - 32.0)]),
                PINK,
            );
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
            }
        }
        // cheeks, nose, mouth, whiskers
        c.fill(&oval(CX - 43.0, hy + 6.0, 16.0, 9.0), BLUSH);
        c.fill(&oval(CX + 27.0, hy + 6.0, 16.0, 9.0), BLUSH);
        let nose = poly(&[(CX - 4.5, hy + 6.0), (CX + 4.5, hy + 6.0), (CX, hy + 10.5)]);
        c.fill(&nose, PINK);
        c.stroke(&nose, PINK_DARK, 1.2);
        c.stroke(&quad(CX - 7.0, hy + 12.0, CX - 3.5, hy + 16.5, CX, hy + 12.0), INK, 1.5);
        c.stroke(&quad(CX, hy + 12.0, CX + 3.5, hy + 16.5, CX + 7.0, hy + 12.0), INK, 1.5);
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

    fn bubble(&self, c: &mut Canvas, t: f64) {
        let (Some(bold), Some(regular)) = (self.bold.as_ref(), self.regular.as_ref()) else { return };
        let (main, sub) = if self.busy {
            let m = ((t - self.since) / 60.0).max(0.0) as i64;
            let sub = match m {
                0 => "just started".to_string(),
                1..=59 => format!("{m} min so far"),
                _ => format!("{} h {} min so far", m / 60, m % 60),
            };
            (self.task.clone(), Some(sub))
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
        let fill = WHITE.fade(if self.busy { 0.98 } else { 0.86 });

        let body = rrect(bx, by, bw, bh, 11.0);
        c.fill(&body, fill);
        c.stroke(&body, INK, 1.8);
        let tip = (CX - 8.0, 82.0);
        c.fill(&poly(&[(CX - 13.0, bottom - 1.6), tip, (CX + 2.0, bottom - 1.6)]), fill);
        c.stroke(&line(CX - 13.0, bottom, tip.0, tip.1), INK, 1.8);
        c.stroke(&line(tip.0, tip.1, CX + 2.0, bottom), INK, 1.8);

        let col = if self.busy { INK } else { SUB };
        for (i, l) in lines.iter().enumerate() {
            c.text(bold, ms, bx + 12.0, by + 8.0 + 12.0 + i as f32 * lh, l, col);
        }
        if let Some(s) = sub {
            c.text(regular, ss, bx + 12.0, by + 8.0 + lines.len() as f32 * lh + 10.5, &s, SUB);
        }
    }
}
