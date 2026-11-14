//! The rare little extras of each mood and of the nap: four per mood, each its own small scene that takes over the
//! whole screen for that status line.

use super::screens::{gear, C, ORANGE, PAPER, TEXT};
use super::*;

pub const EGGS: usize = 4;

const X0: f32 = 199.0;
const Y0: f32 = 121.0;
const W: f32 = 78.0;
const H: f32 = 60.0;

fn mix(a: Rgba, b: Rgba, k: f32) -> Rgba {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k.clamp(0.0, 1.0)) as u8;
    Rgba(m(a.0, b.0), m(a.1, b.1), m(a.2, b.2), m(a.3, b.3))
}

/// The whole screen in a vertical gradient from `top` to `bottom`.
fn sky(c: &mut Canvas, top: Rgba, bottom: Rgba) {
    for i in 0..12 {
        c.fill(&rrect(X0, Y0 + i as f32 * H / 12.0, W, H / 12.0 + 0.6, 0.0), mix(top, bottom, i as f32 / 11.0));
    }
}

/// Rolling ground from y down to the bottom edge.
fn ground(c: &mut Canvas, y: f32, amp: f32, phase: f32, col: Rgba) {
    let mut pts: Vec<(f32, f32)> = (0..=16).map(|i| {
        let x = X0 + W * i as f32 / 16.0;
        (x, y + amp * (i as f32 * 0.55 + phase).sin())
    }).collect();
    pts.push((X0 + W, Y0 + H));
    pts.push((X0, Y0 + H));
    c.fill(&poly(&pts), col);
}

fn stars(c: &mut Canvas, t: f32, n: i64, y_max: f32) {
    for i in 0..n {
        let x = X0 + 3.0 + (hash(i * 7 + 1) % 72) as f32;
        let y = Y0 + 3.0 + (hash(i * 11 + 2) % 1000) as f32 / 1000.0 * (y_max - Y0 - 3.0);
        let a = 0.35 + 0.65 * (0.5 + 0.5 * (t * 1.5 + i as f32 * 2.3).sin());
        c.fill(&circle(x, y, 0.5 + (hash(i + 5) % 3) as f32 * 0.3), WHITE.fade(a));
    }
}

fn cloud(c: &mut Canvas, x: f32, y: f32, s: f32, col: Rgba) {
    for (dx, dy, r) in [(-5.0, 1.0, 3.5), (0.0, -1.5, 5.0), (5.5, 0.5, 4.0)] {
        c.fill(&circle(x + dx * s, y + dy * s, r * s), col);
    }
    c.fill(&rrect(x - 8.0 * s, y, 17.0 * s, 4.5 * s, 2.2 * s), col);
}

fn note(c: &mut Canvas, x: f32, y: f32, col: Rgba) {
    c.fill(&oval(x - 2.2, y - 1.6, 4.4, 3.2), col);
    c.stroke(&line(x + 2.0, y, x + 2.0, y - 8.0), col, 1.0);
    c.stroke(&quad(x + 2.0, y - 8.0, x + 5.0, y - 6.5, x + 4.5, y - 3.5), col, 1.0);
}

/// A star with five points.
fn star5(c: &mut Canvas, x: f32, y: f32, r: f32, rot: f32, col: Rgba) {
    let pts: Vec<(f32, f32)> = (0..10).map(|i| {
        let a = rot - PI / 2.0 + i as f32 * PI / 5.0;
        let rr = if i % 2 == 0 { r } else { r * 0.45 };
        (x + a.cos() * rr, y + a.sin() * rr)
    }).collect();
    c.fill(&poly(&pts), col);
}

fn flame(c: &mut Canvas, x: f32, base: f32, h: f32, w: f32) {
    c.fill(&poly(&[(x - w, base), (x - w * 0.6, base - h * 0.45), (x, base - h), (x + w * 0.6, base - h * 0.45), (x + w, base)]), Rgba::rgb(240, 96, 60));
    c.fill(&poly(&[(x - w * 0.6, base), (x - w * 0.3, base - h * 0.35), (x, base - h * 0.7), (x + w * 0.3, base - h * 0.35), (x + w * 0.6, base)]), ORANGE);
    c.fill(&poly(&[(x - w * 0.3, base), (x, base - h * 0.35), (x + w * 0.3, base)]), SAND);
}

impl Scene {
    pub(super) fn mood_egg(&self, c: &mut Canvas, t: f32, mood: Option<Mood>, pick: usize) {
        match mood {
            Some(Mood::Waiting) => self.egg_waiting(c, t, pick),
            Some(Mood::Problem) => self.egg_problem(c, t, pick),
            Some(Mood::Testing) => self.egg_testing(c, t, pick),
            Some(Mood::Building) => self.egg_building(c, t, pick),
            Some(Mood::Researching) => self.egg_reading(c, t, pick),
            Some(Mood::Transfer(up)) => self.egg_transfer(c, t, pick, up),
            Some(Mood::Thinking) => self.egg_thinking(c, t, pick),
            None => self.egg_nap(c, t, pick),
            Some(Mood::Busy) => {}
        }
    }

    fn egg_waiting(&self, c: &mut Canvas, t: f32, pick: usize) {
        match pick {
            0 => {
                // high noon in the desert: a tumbleweed bounces past a cactus
                sky(c, Rgba::rgb(110, 80, 150), Rgba::rgb(255, 170, 110));
                c.fill(&circle(262.0, 158.0, 10.0), Rgba::rgb(255, 214, 140));
                ground(c, 168.0, 1.6, 0.0, Rgba::rgb(214, 150, 90));
                ground(c, 174.0, 1.2, 2.0, Rgba::rgb(190, 126, 76));
                let green = Rgba::rgb(70, 150, 96);
                c.fill(&rrect(213.0, 144.0, 6.0, 28.0, 3.0), green);
                c.fill(&rrect(207.0, 151.0, 4.5, 10.0, 2.2), green);
                c.fill(&rrect(207.0, 158.0, 8.0, 4.0, 2.0), green);
                c.fill(&rrect(221.0, 148.0, 4.5, 10.0, 2.2), green);
                c.fill(&rrect(217.0, 155.0, 8.0, 4.0, 2.0), green);
                let x = 280.0 - (t * 22.0) % 100.0;
                let hop = (t * 3.5).sin().abs();
                let y = 168.0 - 9.0 * hop;
                c.fill(&oval(x - 6.0 + hop, 172.0, 12.0 - 2.0 * hop, 2.5), INK.fade(0.25));
                let brown = Rgba::rgb(150, 104, 60);
                for k in 0..6 {
                    let a = -t * 7.0 + k as f32 * 1.05;
                    c.stroke(&circle(x + a.cos() * 1.6, y + a.sin() * 1.6, 4.8), brown.fade(0.85), 0.9);
                }
            }
            1 => {
                // a tea break: the pot pours, the cup steams, a "brb" card hangs on its string
                sky(c, Rgba::rgb(70, 58, 72), Rgba::rgb(96, 76, 86));
                c.fill(&rrect(X0, 167.0, W, 14.0, 0.0), Rgba::rgb(150, 104, 72));
                c.fill(&rrect(X0, 167.0, W, 2.0, 0.0), Rgba::rgb(190, 140, 100));
                let pour = smooth(((t * 0.35).fract() * 3.0).min(1.0)) * (1.0 - smooth((((t * 0.35).fract() - 0.66) * 3.0).clamp(0.0, 1.0)));
                let lift = 6.0 * pour;
                let pot = Rgba::rgb(214, 120, 150);
                let (px, py) = (222.0, 156.0 - lift);
                c.fill(&oval(px - 11.0, py - 8.0, 22.0, 16.0), pot);
                c.fill(&poly(&[(px + 9.0, py - 1.0), (px + 17.0, py - 7.0 - pour * 2.0), (px + 18.0, py - 5.0 - pour * 2.0), (px + 10.0, py + 3.0)]), pot);
                c.fill(&rrect(px - 5.0, py - 11.0, 10.0, 4.0, 2.0), Rgba::rgb(240, 160, 186));
                c.stroke(&oval(px - 18.0, py - 6.0, 10.0, 11.0), pot, 2.2);
                c.fill(&oval(px - 6.0, py - 5.0, 5.0, 3.0), WHITE.fade(0.4));
                if pour > 0.6 {
                    c.stroke(&quad(px + 18.0, py - 6.0, 252.0, py - 6.0, 251.0, 158.0), Rgba::rgb(170, 110, 70), 1.4);
                }
                c.fill(&rrect(244.0, 157.0, 14.0, 10.0, 2.5), CASE);
                c.stroke(&circle(260.0, 161.5, 2.8), CASE, 1.5);
                c.fill(&oval(245.0, 156.0, 12.0, 3.0), Rgba::rgb(170, 110, 70));
                for i in 0..2 {
                    let ph = (t * 0.6 + i as f32 * 0.5).fract();
                    let x = 248.0 + i as f32 * 5.0 + 1.5 * (ph * 8.0).sin();
                    c.stroke(&line(x, 153.0 - ph * 12.0, x + 0.6, 150.0 - ph * 12.0), WHITE.fade((ph * PI).sin() * 0.6), 1.3);
                }
                let sway = 1.2 * (t * 1.4).sin();
                c.stroke(&polyline(&[(256.0, Y0), (259.0 + sway, 129.0), (262.0, Y0)]), CASE.fade(0.6), 0.8);
                c.fill(&rrect(251.0 + sway, 129.0, 16.0, 10.0, 2.0), CASE);
                self.label(c, 6.5, 259.0 + sway, 136.5, "brb", Rgba::rgb(170, 90, 110));
            }
            2 => {
                // the elevator: doors open and shut, floors count up, music drifts out
                sky(c, Rgba::rgb(86, 84, 104), Rgba::rgb(110, 106, 126));
                c.fill(&rrect(214.0, 134.0, 48.0, 47.0, 1.0), Rgba::rgb(60, 58, 72));
                let k = (t * 0.25).fract();
                let open = smooth((k * 5.0).min(1.0)) * (1.0 - smooth(((k - 0.6) * 5.0).clamp(0.0, 1.0)));
                c.fill(&rrect(216.0, 136.0, 44.0, 45.0, 0.0), Rgba::rgb(240, 214, 150).fade(0.8));
                let half = 22.0 * (1.0 - open);
                let metal = Rgba::rgb(184, 190, 206);
                c.fill(&rrect(216.0, 136.0, half, 45.0, 0.0), metal);
                c.fill(&rrect(260.0 - half, 136.0, half, 45.0, 0.0), metal);
                c.stroke(&line(216.0 + half, 136.0, 216.0 + half, 181.0), Rgba::rgb(140, 146, 162), 0.8);
                c.fill(&rrect(228.0, 124.0, 20.0, 8.0, 2.0), INK);
                let floor = (t * 0.25) as i64 % 9 + 1;
                self.label(c, 6.5, 238.0, 130.5, &floor.to_string(), SAND);
                for i in 0..3 {
                    let ph = (t * 0.4 + i as f32 / 3.0).fract();
                    note(c, 238.0 + 12.0 * (ph * 5.0 + i as f32).sin(), 172.0 - 40.0 * ph, CODE[i].fade((ph * PI).sin()));
                }
            }
            _ => {
                // watching the grass grow, with a ruler to prove it
                sky(c, Rgba::rgb(120, 180, 230), Rgba::rgb(190, 224, 246));
                cloud(c, 222.0 + (t * 2.0) % 40.0, 132.0, 0.8, WHITE.fade(0.9));
                c.fill(&rrect(X0, 172.0, W, 9.0, 0.0), Rgba::rgb(130, 90, 60));
                let grow = (t * 0.06).fract();
                for i in 0..14 {
                    let x = 204.0 + i as f32 * 4.3;
                    let h = 4.0 + 18.0 * grow * (0.7 + 0.3 * ((hash(i) % 10) as f32 / 10.0));
                    let sway = 1.2 * (t * 1.5 + i as f32).sin();
                    c.stroke(&quad(x, 173.0, x + sway * 0.5, 173.0 - h * 0.5, x + sway, 173.0 - h), Rgba::rgb(90, 180, 90), 1.6);
                }
                c.fill(&rrect(264.0, 142.0, 6.0, 31.0, 1.0), SAND);
                for i in 0..7 {
                    let y = 170.0 - i as f32 * 4.5;
                    c.stroke(&line(264.0, y, 264.0 + if i % 2 == 0 { 3.5 } else { 2.0 }, y), INK.fade(0.6), 0.7);
                }
            }
        }
    }

    fn egg_problem(&self, c: &mut Canvas, t: f32, pick: usize) {
        let (cx, cy) = C;
        match pick {
            0 => {
                // the room is on fire and the mug is fine
                sky(c, Rgba::rgb(60, 40, 44), Rgba::rgb(120, 60, 46));
                for i in 0..4 {
                    let ph = (t * 0.2 + i as f32 / 4.0).fract();
                    c.fill(&circle(210.0 + i as f32 * 19.0 + 4.0 * (t + i as f32).sin(), 128.0 + 6.0 * ph, 7.0 + 3.0 * ph), Rgba::rgb(90, 80, 86).fade(0.6));
                }
                for (x, s) in [(204.0, 1.0), (213.0, 0.8), (263.0, 0.9), (272.0, 1.1)] {
                    flame(c, x, 181.0, (18.0 + 6.0 * (t * 8.0 + x).sin().abs()) * s, 5.0 * s);
                }
                c.fill(&rrect(222.0, 165.0, 32.0, 3.0, 1.0), Rgba::rgb(150, 104, 72));
                c.fill(&rrect(225.0, 168.0, 2.5, 13.0, 1.0), Rgba::rgb(150, 104, 72));
                c.fill(&rrect(249.0, 168.0, 2.5, 13.0, 1.0), Rgba::rgb(150, 104, 72));
                c.fill(&rrect(233.0, 156.0, 10.0, 9.0, 2.0), MUG);
                c.stroke(&circle(244.5, 160.5, 2.4), MUG, 1.3);
                self.label_on(c, 7.0, cx, 147.0, "this is fine", SAND);
            }
            1 => {
                // rubber duck debugging: the duck bobs in the bath and listens
                sky(c, Rgba::rgb(190, 214, 230), Rgba::rgb(170, 196, 214));
                for i in 0..6 {
                    c.stroke(&line(X0, Y0 + 8.0 + i as f32 * 8.0, X0 + W, Y0 + 8.0 + i as f32 * 8.0), WHITE.fade(0.4), 0.7);
                    c.stroke(&line(X0 + 6.0 + i as f32 * 13.0, Y0, X0 + 6.0 + i as f32 * 13.0, 160.0), WHITE.fade(0.4), 0.7);
                }
                c.fill(&rrect(201.0, 158.0, 74.0, 23.0, 8.0), WHITE);
                let wave: Vec<(f32, f32)> = (0..=16).map(|i| (204.0 + i as f32 * 4.3, 162.0 + 1.2 * (i as f32 * 0.9 + t * 3.0).sin())).collect();
                let mut water = wave.clone();
                water.push((273.0, 172.0));
                water.push((204.0, 172.0));
                c.fill(&poly(&water), Rgba::rgb(120, 180, 230));
                let (dx, dy) = (236.0, 157.0 + 1.2 * (t * 3.0).sin());
                c.fill(&oval(dx - 10.0, dy - 5.0, 20.0, 10.0), SAND);
                c.fill(&circle(dx + 7.0, dy - 7.0, 5.5), SAND);
                c.fill(&poly(&[(dx + 11.0, dy - 7.5), (dx + 16.0, dy - 6.0), (dx + 11.0, dy - 4.5)]), ORANGE);
                c.fill(&circle(dx + 8.5, dy - 8.5, 1.0), INK);
                c.fill(&oval(dx - 6.0, dy - 4.0, 8.0, 4.0), Rgba::rgb(240, 190, 70));
                c.fill(&circle(258.0, 135.0, 7.5), WHITE);
                c.fill(&poly(&[(252.0, 139.0), (249.0, 145.0), (256.0, 141.0)]), WHITE);
                self.label(c, 9.0, 258.0, 138.5, "?", Rgba::rgb(90, 130, 190));
                for i in 0..3 {
                    let ph = (t * 0.5 + i as f32 / 3.0).fract();
                    c.stroke(&circle(214.0 + i as f32 * 6.0, 160.0 - 30.0 * ph, 1.5 + ph * 1.5), WHITE.fade(1.0 - ph), 0.8);
                }
            }
            2 => {
                // the sad blue screen, counting up to restart
                c.fill(&rrect(X0, Y0, W, H, 0.0), Rgba::rgb(40, 104, 200));
                self.label(c, 20.0, 214.0, 145.0, ":(", WHITE);
                for i in 0..3 {
                    c.fill(&rrect(206.0, 152.0 + i as f32 * 5.0, 50.0 - i as f32 * 12.0, 2.0, 1.0), WHITE.fade(0.7));
                }
                let pct = ((t * 9.0) as i64 % 101).to_string() + "% complete";
                if let Some(font) = self.bold.as_ref() {
                    c.text(font, 6.0, 206.0, 174.0, &pct, WHITE);
                }
            }
            _ => {
                // a gremlin chews on the cables, sparks flying
                sky(c, Rgba::rgb(30, 30, 44), Rgba::rgb(44, 40, 58));
                c.stroke(&quad(X0, 140.0, 238.0, 175.0, X0 + W, 150.0), CODE[3], 3.0);
                c.stroke(&quad(X0, 168.0, 230.0, 150.0, X0 + W, 172.0), CODE[1], 3.0);
                let (gx, gy) = (240.0, 158.0);
                let green = Rgba::rgb(110, 190, 110);
                c.fill(&poly(&[(gx - 6.0, gy - 6.0), (gx - 16.0, gy - 14.0), (gx - 8.0, gy - 1.0)]), green);
                c.fill(&poly(&[(gx + 6.0, gy - 6.0), (gx + 16.0, gy - 14.0), (gx + 8.0, gy - 1.0)]), green);
                c.fill(&oval(gx - 9.0, gy - 8.0, 18.0, 17.0), green);
                c.fill(&circle(gx - 3.5, gy - 2.0, 2.6), SAND);
                c.fill(&circle(gx + 3.5, gy - 2.0, 2.6), SAND);
                c.fill(&circle(gx - 3.5, gy - 2.0, 1.1), INK);
                c.fill(&circle(gx + 3.5, gy - 2.0, 1.1), INK);
                let chew = (t * 8.0).sin().abs();
                c.fill(&oval(gx - 4.0, gy + 3.0, 8.0, 1.5 + 3.0 * chew), Rgba::rgb(60, 30, 40));
                c.fill(&poly(&[(gx - 2.5, gy + 3.0), (gx - 1.5, gy + 5.0), (gx - 0.5, gy + 3.0)]), WHITE);
                c.fill(&poly(&[(gx + 0.5, gy + 3.0), (gx + 1.5, gy + 5.0), (gx + 2.5, gy + 3.0)]), WHITE);
                for k in 0..5 {
                    let ph = (t * 3.0 + k as f32 / 5.0).fract();
                    let a = k as f32 * 1.3 + t;
                    let (sx, sy) = (gx + a.cos() * (6.0 + 12.0 * ph), gy + 6.0 + a.sin() * (4.0 + 8.0 * ph) - 6.0 * ph);
                    c.fill(&circle(sx, sy, 1.2 * (1.0 - ph) + 0.3), SAND.fade(1.0 - ph));
                }
                let _ = cy;
            }
        }
    }

    fn egg_testing(&self, c: &mut Canvas, t: f32, pick: usize) {
        let (cx, _) = C;
        match pick {
            0 => {
                // the high score table, our name on top
                sky(c, Rgba::rgb(20, 18, 40), Rgba::rgb(40, 20, 60));
                let glow = mix(SAND, CODE[1], 0.5 + 0.5 * (t * 4.0).sin());
                self.label(c, 7.0, cx, 131.0, "HIGH SCORES", glow);
                let rows = [("1", "CAT", "99999"), ("2", "AAA", "45210"), ("3", "BOB", "31337"), ("4", "ZZZ", "12000")];
                if let Some(font) = self.bold.as_ref() {
                    for (i, (r, n, s)) in rows.iter().enumerate() {
                        let y = 143.0 + i as f32 * 9.0;
                        let col = if i == 0 { if (t * 3.0).fract() < 0.6 { SAND } else { WHITE } } else { CODE[3].fade(0.8) };
                        c.text(font, 6.5, 206.0, y, r, col);
                        c.text(font, 6.5, 216.0, y, n, col);
                        c.text(font, 6.5, 270.0 - measure(font, 6.5, s), y, s, col);
                    }
                }
            }
            1 => {
                // the famous cheat code, the pad lighting each press, then extra lives
                sky(c, Rgba::rgb(30, 34, 60), Rgba::rgb(46, 44, 80));
                let seq = [0usize, 0, 1, 1, 2, 3, 2, 3];
                let k = (t * 0.25).fract() * 10.0;
                let at = (k as usize).min(9);
                let (px, py) = (222.0, 148.0);
                c.fill(&rrect(px - 3.5, py - 11.0, 7.0, 22.0, 1.5), CASE_SHADE);
                c.fill(&rrect(px - 11.0, py - 3.5, 22.0, 7.0, 1.5), CASE_SHADE);
                if at < 8 {
                    let (dx, dy) = [(0.0, -7.0), (0.0, 7.0), (-7.0, 0.0), (7.0, 0.0)][seq[at]];
                    c.fill(&rrect(px + dx - 3.5, py + dy - 3.5, 7.0, 7.0, 1.5), SAND);
                }
                for (i, &d) in seq.iter().enumerate() {
                    let x = 206.0 + i as f32 * 8.5;
                    let (dx, dy) = [(0.0f32, -1.0f32), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)][d];
                    let col = if i < at { GREEN } else { WHITE.fade(0.3) };
                    let (tx, ty) = (x + dx * 3.0, 172.0 + dy * 3.0);
                    c.fill(&poly(&[(tx, ty), (x - dy * 2.8 - dx * 1.5, 172.0 + dx * 2.8 - dy * 1.5), (x + dy * 2.8 - dx * 1.5, 172.0 - dx * 2.8 - dy * 1.5)]), col);
                }
                for (bx, col) in [(250.0, RED), (262.0, CODE[3])] {
                    c.fill(&circle(bx, 151.0, 4.0), col);
                }
                if at >= 8 {
                    self.label_on(c, 7.5, cx, 136.0, "+30 LIVES", SAND);
                }
            }
            2 => {
                // a speedrun: the timer runs, the splits go green, then gold
                sky(c, Rgba::rgb(24, 30, 40), Rgba::rgb(30, 40, 54));
                let run = (t * 10.0) % 600.0;
                let time = format!("{}:{:04.1}", (run / 60.0) as i64, run % 60.0);
                if let Some(font) = self.bold.as_ref() {
                    for (i, (name, split)) in [("start", "-0.4"), ("boss", "-1.2"), ("end", "-2.7")].iter().enumerate() {
                        let y = 133.0 + i as f32 * 8.0;
                        c.text(font, 6.0, 206.0, y, name, TEXT);
                        let col = if i == 2 { SAND } else { GREEN };
                        c.text(font, 6.0, 270.0 - measure(font, 6.0, split), y, split, col);
                    }
                    let w = measure(font, 12.0, &time);
                    c.text(font, 12.0, 272.0 - w, 172.0, &time, GREEN);
                }
                if (t * 0.7).fract() < 0.4 {
                    star5(c, 213.0, 163.0, 5.5, t, SAND);
                    self.label(c, 6.5, 226.0, 166.0, "PB!", SAND);
                }
            }
            _ => {
                // a glitch: the game tears into colored bands, the hero doubles up, a bug is found
                sky(c, SCREEN_ON, SCREEN_ON);
                c.fill(&rrect(X0, 170.0, W, 11.0, 0.0), Rgba::rgb(64, 112, 82));
                let jitter = |k: i64| (hash(k + (t * 12.0) as i64) % 9) as f32 - 4.0;
                for (i, col) in [RED.fade(0.6), GREEN.fade(0.6), CODE[3].fade(0.6)].iter().enumerate() {
                    let off = jitter(i as i64) * 0.8;
                    c.fill(&rrect(226.0 + off + i as f32 * 2.0, 156.0, 12.0, 12.0, 2.5), *col);
                }
                for k in 0..4 {
                    let y = 124.0 + (hash(k * 3 + (t * 6.0) as i64) % 50) as f32;
                    c.fill(&rrect(X0, y, W, 2.0 + (hash(k) % 3) as f32, 0.0), CODE[(k % 4) as usize].fade(0.35));
                    c.fill(&rrect(X0 + jitter(k + 9).abs() * 6.0, y + 2.0, 20.0, 1.2, 0.0), WHITE.fade(0.4));
                }
                if (t * 2.0).fract() < 0.65 {
                    self.label_on(c, 7.5, cx, 138.0, "BUG FOUND", RED);
                }
            }
        }
    }

    fn egg_building(&self, c: &mut Canvas, t: f32, pick: usize) {
        match pick {
            0 => {
                // a rosette for the build: "works on my machine"
                sky(c, Rgba::rgb(36, 54, 66), Rgba::rgb(28, 44, 56));
                let (rx, ry) = (220.0, 147.0);
                let spin = t * 0.5;
                c.fill(&poly(&[(rx - 7.0, ry + 6.0), (rx - 12.0, ry + 24.0), (rx - 6.0, ry + 20.0), (rx - 2.0, ry + 25.0), (rx, ry + 8.0)]), RED);
                c.fill(&poly(&[(rx + 7.0, ry + 6.0), (rx + 12.0, ry + 24.0), (rx + 6.0, ry + 20.0), (rx + 2.0, ry + 25.0), (rx, ry + 8.0)]), Rgba::rgb(200, 70, 70));
                for k in 0..14 {
                    let a = spin + k as f32 / 14.0 * 2.0 * PI;
                    c.fill(&circle(rx + a.cos() * 12.0, ry + a.sin() * 12.0, 3.4), SAND);
                }
                c.fill(&circle(rx, ry, 12.0), SAND);
                c.fill(&circle(rx, ry, 8.5), Rgba::rgb(240, 190, 70));
                star5(c, rx, ry, 5.5, 0.0, WHITE.fade(0.9));
                let k = smooth(((t * 0.4).fract() * 4.0).min(1.0));
                self.label(c, 7.5, 254.0, 145.0, "works on", WHITE.fade(k));
                self.label(c, 7.5, 254.0, 155.0, "my machine", WHITE.fade(k));
            }
            1 => {
                // a digger scoops the dirt and tips it on the pile
                sky(c, Rgba::rgb(120, 180, 230), Rgba::rgb(200, 226, 240));
                cloud(c, 216.0, 132.0, 0.8, WHITE.fade(0.9));
                c.fill(&rrect(X0, 170.0, W, 11.0, 0.0), Rgba::rgb(150, 104, 66));
                let k = (t * 0.4).fract();
                let pile = 6.0 + 6.0 * ((t * 0.4) as i64 % 4) as f32 / 3.0;
                c.fill(&poly(&[(250.0, 170.0), (262.0, 170.0 - pile), (275.0, 170.0)]), Rgba::rgb(170, 120, 76));
                let (dx, dy) = (222.0, 162.0);
                c.fill(&rrect(dx - 13.0, dy + 4.0, 24.0, 6.0, 3.0), INK.fade(0.8));
                for wx in [dx - 9.0, dx - 1.0, dx + 7.0] {
                    c.fill(&circle(wx, dy + 7.0, 1.8), Rgba::rgb(110, 110, 120));
                }
                c.fill(&rrect(dx - 12.0, dy - 5.0, 20.0, 9.0, 2.0), SAND);
                c.fill(&rrect(dx - 10.0, dy - 13.0, 10.0, 9.0, 1.5), SAND);
                c.fill(&rrect(dx - 8.5, dy - 11.5, 7.0, 5.0, 1.0), Rgba::rgb(150, 200, 230));
                let swing = smooth((k * 2.0).min(1.0)) - smooth(((k - 0.5) * 2.0).clamp(0.0, 1.0));
                let a1 = -0.6 + 1.0 * swing;
                let (ex, ey) = (dx + 8.0 + 14.0 * a1.cos(), dy - 4.0 - 14.0 * a1.sin());
                let a2 = a1 - 1.4;
                let (bx, by) = (ex + 10.0 * a2.cos(), ey - 10.0 * a2.sin());
                let arm = Rgba::rgb(222, 164, 40);
                c.stroke(&line(dx + 6.0, dy - 3.0, ex, ey), arm, 3.0);
                c.stroke(&line(ex, ey, bx, by), arm, 2.6);
                c.fill(&circle(bx, by, 3.2), arm);
                if swing < 0.5 {
                    c.fill(&circle(bx, by - 1.0, 2.0), Rgba::rgb(170, 120, 76));
                } else if swing > 0.9 {
                    for i in 0..3 {
                        c.fill(&circle(bx + i as f32, by + 3.0 + i as f32 * 2.5, 1.0), Rgba::rgb(170, 120, 76));
                    }
                }
            }
            2 => {
                // a leaky pipe sprays, then a strip of tape stops it
                sky(c, Rgba::rgb(70, 76, 90), Rgba::rgb(86, 90, 104));
                let pipe = Rgba::rgb(160, 168, 184);
                c.fill(&rrect(X0, 143.0, W, 11.0, 0.0), pipe);
                c.fill(&rrect(X0, 144.5, W, 2.0, 0.0), WHITE.fade(0.4));
                for x in [214.0, 262.0] {
                    c.fill(&rrect(x - 3.0, 141.0, 6.0, 15.0, 1.5), Rgba::rgb(130, 136, 152));
                }
                let k = (t * 0.25).fract();
                let tape = smooth(((k - 0.5) * 4.0).clamp(0.0, 1.0));
                if tape < 1.0 {
                    for i in 0..6 {
                        let ph = (t * 2.0 + i as f32 / 6.0).fract();
                        let a = -PI / 2.0 + (i as f32 - 2.5) * 0.25;
                        let (x, y) = (238.0 + a.cos() * 20.0 * ph, 148.0 + a.sin() * 18.0 * ph + 30.0 * ph * ph);
                        c.fill(&circle(x, y, 1.3), Rgba::rgb(140, 200, 250).fade(1.0 - ph));
                    }
                    c.fill(&rrect(222.0, 176.0, 32.0, 3.0, 1.5), Rgba::rgb(140, 200, 250).fade(0.5));
                }
                if tape > 0.0 {
                    c.fill(&rrect(232.0, 141.0, 12.0, 15.0 * tape, 1.0), Rgba::rgb(200, 204, 214));
                    for i in 0..3 {
                        c.stroke(&line(233.0, 143.0 + i as f32 * 4.0, 243.0, 143.0 + i as f32 * 4.0), Rgba::rgb(170, 174, 186).fade(tape), 0.6);
                    }
                }
            }
            _ => {
                // a block tower sways while a block is eased out of the middle
                sky(c, Rgba::rgb(46, 54, 76), Rgba::rgb(36, 44, 62));
                c.fill(&rrect(X0, 176.0, W, 5.0, 0.0), Rgba::rgb(150, 104, 72));
                let wood = [Rgba::rgb(230, 190, 130), Rgba::rgb(210, 166, 108)];
                let sway = 0.35 * (t * 1.4).sin();
                let pull = (t * 0.3).fract();
                for row in 0..9 {
                    let y = 170.0 - row as f32 * 5.5;
                    let off = sway * row as f32;
                    if row % 2 == 0 {
                        c.fill(&rrect(226.0 + off, y, 24.0, 5.0, 0.8), wood[0]);
                        c.stroke(&line(234.0 + off, y, 234.0 + off, y + 5.0), wood[1], 0.6);
                        c.stroke(&line(242.0 + off, y, 242.0 + off, y + 5.0), wood[1], 0.6);
                    } else {
                        for b in 0..3 {
                            let mut x = 226.0 + off + b as f32 * 8.2;
                            if row == 5 && b == 1 {
                                x += 18.0 * pull;
                            }
                            c.fill(&rrect(x, y, 7.6, 5.0, 0.8), wood[1]);
                        }
                    }
                }
            }
        }
    }

    fn egg_reading(&self, c: &mut Canvas, t: f32, pick: usize) {
        let (cx, cy) = C;
        match pick {
            0 => {
                // a cat video got opened instead: the cat bobs, hearts float, the play bar runs
                sky(c, Rgba::rgb(24, 26, 36), Rgba::rgb(24, 26, 36));
                c.fill(&rrect(203.0, 124.0, 70.0, 44.0, 2.0), Rgba::rgb(110, 170, 210));
                c.fill(&rrect(203.0, 156.0, 70.0, 12.0, 0.0), Rgba::rgb(130, 190, 110));
                let bob = 1.5 * (t * 3.0).sin();
                let (fx, fy) = (238.0, 147.0 + bob);
                let fur = Rgba::rgb(255, 170, 90);
                c.fill(&poly(&[(fx - 10.0, fy - 2.0), (fx - 8.0, fy - 12.0), (fx - 3.0, fy - 7.0)]), fur);
                c.fill(&poly(&[(fx + 10.0, fy - 2.0), (fx + 8.0, fy - 12.0), (fx + 3.0, fy - 7.0)]), fur);
                c.fill(&oval(fx - 11.0, fy - 8.0, 22.0, 17.0), fur);
                c.fill(&oval(fx - 5.0, fy + 1.0, 10.0, 6.0), Rgba::rgb(255, 230, 200));
                c.fill(&circle(fx - 4.5, fy - 2.0, 1.6), INK);
                c.fill(&circle(fx + 4.5, fy - 2.0, 1.6), INK);
                c.fill(&poly(&[(fx - 1.2, fy + 1.0), (fx + 1.2, fy + 1.0), (fx, fy + 2.4)]), PINK_DARK);
                for i in 0..2 {
                    let ph = (t * 0.6 + i as f32 * 0.5).fract();
                    let (hx, hy) = (256.0 + i as f32 * 6.0, 152.0 - 24.0 * ph);
                    let a = (ph * PI).sin();
                    c.fill(&circle(hx - 1.4, hy, 1.8), PINK_DARK.fade(a));
                    c.fill(&circle(hx + 1.4, hy, 1.8), PINK_DARK.fade(a));
                    c.fill(&poly(&[(hx - 3.1, hy + 0.4), (hx + 3.1, hy + 0.4), (hx, hy + 3.6)]), PINK_DARK.fade(a));
                }
                c.fill(&rrect(203.0, 172.0, 70.0, 3.0, 1.5), WHITE.fade(0.2));
                let k = (t * 0.08).fract();
                c.fill(&rrect(203.0, 172.0, 70.0 * k, 3.0, 1.5), RED);
                c.fill(&circle(203.0 + 70.0 * k, 173.5, 2.4), RED);
            }
            1 => {
                // the answer, glowing in the middle of a turning galaxy
                sky(c, Rgba::rgb(14, 10, 34), Rgba::rgb(40, 20, 64));
                stars(c, t, 14, 181.0);
                for k in 0..36 {
                    let u = k as f32 / 36.0;
                    for arm in [0.0, PI] {
                        let a = arm + u * 5.0 + t * 0.3;
                        let r = 15.0 + 16.0 * u;
                        c.fill(&circle(cx + a.cos() * r, cy + a.sin() * r * 0.6, 1.4 - u), mix(CODE[1], CODE[3], u).fade(0.8 - u * 0.5));
                    }
                }
                let glow = 0.7 + 0.3 * (t * 2.0).sin();
                c.fill(&oval(cx - 14.0, cy - 9.0, 28.0, 18.0), SAND.fade(0.2 * glow));
                self.label(c, 15.0, cx, cy + 5.5, "42", SAND.fade(glow));
            }
            2 => {
                // down the rabbit hole: rings spin into the dark, a rabbit hops in
                sky(c, Rgba::rgb(70, 50, 90), Rgba::rgb(70, 50, 90));
                for k in 0..7 {
                    let r = 30.0 - k as f32 * 4.2;
                    let col = if k % 2 == 0 { Rgba::rgb(110, 80, 140) } else { Rgba::rgb(50, 34, 70) };
                    let wob = 1.5 * (t * 2.0 + k as f32).sin();
                    c.fill(&oval(cx - r * 1.3 + wob, cy - r * 0.8, r * 2.6, r * 1.6), col);
                }
                c.fill(&oval(cx - 4.0, cy - 2.5, 8.0, 5.0), INK);
                let ph = (t * 0.4).fract();
                let s = 1.0 - 0.85 * ph;
                let (rx, ry) = (cx + 24.0 * (1.0 - ph), cy - 14.0 * (ph * PI).sin() + 8.0 * (1.0 - ph));
                c.fill(&oval(rx - 5.0 * s, ry - 3.0 * s, 10.0 * s, 7.0 * s), WHITE);
                c.fill(&circle(rx - 4.0 * s, ry - 4.0 * s, 3.0 * s), WHITE);
                c.fill(&oval(rx - 5.5 * s, ry - 11.0 * s, 2.2 * s, 6.0 * s), WHITE);
                c.fill(&oval(rx - 3.0 * s, ry - 11.0 * s, 2.2 * s, 6.0 * s), WHITE);
            }
            _ => {
                // a coffee ring spreads on the page and it gets a TL;DR stamp
                sky(c, Rgba::rgb(36, 54, 66), Rgba::rgb(36, 54, 66));
                c.fill(&rrect(210.0, 123.0, 56.0, 58.0, 1.0), PAPER);
                for i in 0..7 {
                    c.fill(&rrect(215.0, 129.0 + i as f32 * 6.5, 30.0 + (hash(i + 70) % 16) as f32, 2.0, 1.0), TEXT);
                }
                let k = smooth(((t * 0.3).fract() * 2.0).min(1.0));
                c.stroke(&circle(252.0, 139.0, 4.0 + 7.0 * k), Rgba::rgb(150, 100, 60).fade(0.65), 2.0);
                c.fill(&circle(252.0, 139.0, 3.0 + 6.0 * k), Rgba::rgb(170, 120, 76).fade(0.2));
                if (t * 0.3).fract() > 0.55 {
                    c.stroke(&rrect(214.0, 160.0, 28.0, 12.0, 2.0), RED.fade(0.85), 1.6);
                    self.label(c, 7.0, 228.0, 169.0, "TL;DR", RED.fade(0.85));
                }
            }
        }
    }

    fn egg_transfer(&self, c: &mut Canvas, t: f32, pick: usize, up: bool) {
        let (cx, _) = C;
        match pick {
            0 => {
                // a carrier pigeon flaps across the sky with the letter
                sky(c, Rgba::rgb(110, 170, 230), Rgba::rgb(200, 226, 246));
                cloud(c, 212.0, 167.0, 1.0, WHITE.fade(0.9));
                cloud(c, 262.0, 134.0, 0.7, WHITE.fade(0.8));
                let dir = if up { 1.0 } else { -1.0 };
                let u = (t * 0.22).fract();
                let x = if up { 190.0 + 100.0 * u } else { 290.0 - 100.0 * u };
                let y = 150.0 + 3.0 * (t * 4.0).sin();
                let flap = (t * 12.0).sin();
                let body = Rgba::rgb(170, 176, 196);
                let wing = Rgba::rgb(140, 146, 170);
                c.fill(&poly(&[(x - dir * 9.0, y), (x - dir * 14.0, y - 3.0), (x - dir * 14.0, y + 3.0)]), wing);
                c.fill(&oval(x - 9.0, y - 4.0, 18.0, 8.0), body);
                c.fill(&circle(x + dir * 8.0, y - 3.0, 3.6), body);
                c.fill(&circle(x + dir * 8.0, y - 1.0, 1.6), Rgba::rgb(130, 190, 160));
                c.fill(&poly(&[(x + dir * 11.0, y - 3.6), (x + dir * 15.0, y - 2.5), (x + dir * 11.0, y - 1.6)]), ORANGE);
                c.fill(&circle(x + dir * 9.0, y - 4.0, 0.9), INK);
                c.fill(&poly(&[(x - dir * 4.0, y - 2.0), (x + dir * 4.0, y - 2.0), (x - dir * 1.0, y - 2.0 - 10.0 * flap)]), wing);
                c.fill(&rrect(x - 4.0, y + 4.0, 9.0, 6.0, 0.8), WHITE);
                c.stroke(&polyline(&[(x - 4.0, y + 4.0), (x + 0.5, y + 7.0), (x + 5.0, y + 4.0)]), TEXT, 0.7);
                c.fill(&circle(x + 0.5, y + 7.5, 1.0), RED);
            }
            1 => {
                // a floppy disk being written, the old way
                sky(c, Rgba::rgb(36, 54, 66), Rgba::rgb(30, 44, 56));
                let (fx, fy) = (222.0, 130.0);
                c.fill(&poly(&[(fx, fy), (fx + 28.0, fy), (fx + 32.0, fy + 4.0), (fx + 32.0, fy + 32.0), (fx, fy + 32.0)]), Rgba::rgb(60, 80, 150));
                let shutter = 3.0 * (t * 2.0).sin().max(0.0);
                c.fill(&rrect(fx + 7.0 + shutter, fy, 16.0, 11.0, 0.8), Rgba::rgb(190, 196, 210));
                c.fill(&rrect(fx + 17.0 + shutter, fy + 2.0, 4.0, 7.0, 0.6), Rgba::rgb(60, 80, 150));
                c.fill(&rrect(fx + 4.0, fy + 15.0, 24.0, 15.0, 1.0), WHITE);
                for i in 0..3 {
                    c.stroke(&line(fx + 6.0, fy + 19.0 + i as f32 * 3.5, fx + 26.0, fy + 19.0 + i as f32 * 3.5), TEXT.fade(0.6), 0.6);
                }
                let k = (t * 0.15).fract();
                c.fill(&rrect(207.0, 170.0, 62.0, 5.0, 2.5), WHITE.fade(0.12));
                c.fill(&rrect(207.0, 170.0, 4.0 + 58.0 * k, 5.0, 2.5), CODE[3]);
                let led = if (t * 6.0).fract() < 0.5 { GREEN } else { GREEN.fade(0.3) };
                c.fill(&circle(264.0, 133.0, 2.0), led);
            }
            2 => {
                // a dial-up modem sings, its lights chasing
                sky(c, Rgba::rgb(36, 40, 56), Rgba::rgb(44, 46, 64));
                c.fill(&rrect(212.0, 158.0, 52.0, 14.0, 3.0), CASE);
                c.fill(&rrect(212.0, 158.0, 52.0, 3.0, 1.5), WHITE.fade(0.4));
                for i in 0..6 {
                    let on = ((t * 8.0) as i64 + i) % 3 == 0;
                    c.fill(&circle(220.0 + i as f32 * 6.0, 166.0, 1.6), if on { CODE[(i % 4) as usize] } else { CASE_SHADE });
                }
                for k in 0..3 {
                    let ph = (t * 0.9 + k as f32 / 3.0).fract();
                    let pts: Vec<(f32, f32)> = (0..=20).map(|i| {
                        let x = 206.0 + i as f32 * 3.2;
                        (x, 150.0 - 20.0 * ph + 3.0 * (i as f32 * 1.2 + t * 12.0 + k as f32).sin())
                    }).collect();
                    c.stroke(&polyline(&pts), CODE[k].fade((1.0 - ph) * 0.8), 1.2);
                }
            }
            _ => {
                // capsules whoosh through the tubes
                sky(c, Rgba::rgb(46, 52, 70), Rgba::rgb(52, 58, 78));
                let tube = Rgba::rgb(150, 200, 220).fade(0.35);
                for y in [140.0, 162.0] {
                    c.fill(&rrect(X0, y - 5.0, W, 10.0, 5.0), tube);
                    c.stroke(&line(X0, y - 4.0, X0 + W, y - 4.0), WHITE.fade(0.35), 0.8);
                }
                for (i, y) in [140.0, 162.0].iter().enumerate() {
                    let ph = (t * 0.8 + i as f32 * 0.45).fract();
                    let dir = if up == (i == 0) { 1.0 } else { -1.0 };
                    let x = if dir > 0.0 { 190.0 + 100.0 * ph } else { 290.0 - 100.0 * ph };
                    c.fill(&rrect(x - 7.0, y - 3.5, 14.0, 7.0, 3.5), CODE[i * 2]);
                    c.fill(&rrect(x - 7.0, y - 3.5, 3.0, 7.0, 1.5), CASE);
                    for k in 1..4 {
                        c.stroke(&line(x - dir * (8.0 + k as f32 * 4.0), y - 1.5 + k as f32, x - dir * (11.0 + k as f32 * 4.0), y - 1.5 + k as f32), WHITE.fade(0.5 - k as f32 * 0.12), 0.8);
                    }
                }
                let _ = cx;
            }
        }
    }

    fn egg_thinking(&self, c: &mut Canvas, t: f32, pick: usize) {
        let (cx, cy) = C;
        match pick {
            0 => {
                // eureka in the bath: the water splashes over the edge
                sky(c, Rgba::rgb(196, 220, 230), Rgba::rgb(176, 200, 214));
                c.fill(&rrect(206.0, 156.0, 64.0, 20.0, 9.0), WHITE);
                c.fill(&rrect(212.0, 175.0, 4.0, 5.0, 1.5), Rgba::rgb(200, 170, 90));
                c.fill(&rrect(260.0, 175.0, 4.0, 5.0, 1.5), Rgba::rgb(200, 170, 90));
                c.fill(&rrect(209.0, 155.0, 58.0, 5.0, 2.5), Rgba::rgb(120, 180, 230));
                let k = (t * 0.6).fract();
                for i in 0..8 {
                    let a = PI + (i as f32 + 0.5) / 8.0 * PI;
                    let (x, y) = (cx + a.cos() * 26.0 * k, 155.0 + a.sin() * 20.0 * k + 26.0 * k * k);
                    c.fill(&circle(x, y, 1.6 * (1.0 - k) + 0.4), Rgba::rgb(120, 180, 230).fade(1.0 - k));
                }
                self.label_on(c, 9.0, cx, 138.0, "eureka!", Rgba::rgb(240, 200, 90));
            }
            1 => {
                // a galaxy brain: glowing, with little stars in orbit
                sky(c, Rgba::rgb(20, 14, 44), Rgba::rgb(50, 24, 80));
                stars(c, t, 10, 181.0);
                let glow = 0.4 + 0.3 * (t * 2.5).sin().abs();
                for r in [24.0, 19.0] {
                    c.fill(&circle(cx, cy, r), Rgba::rgb(170, 120, 255).fade(glow * 0.35));
                }
                c.fill(&oval(cx - 15.0, cy - 11.0, 30.0, 22.0), Rgba::rgb(255, 176, 206));
                c.stroke(&line(cx, cy - 10.0, cx, cy + 10.0), PINK_DARK, 1.2);
                for s in [-1.0f32, 1.0] {
                    c.stroke(&quad(cx + s * 3.0, cy - 6.0, cx + s * 10.0, cy - 7.0, cx + s * 7.0, cy - 1.0), PINK_DARK, 1.0);
                    c.stroke(&quad(cx + s * 4.0, cy + 1.0, cx + s * 12.0, cy + 1.0, cx + s * 9.0, cy + 7.0), PINK_DARK, 1.0);
                }
                for k in 0..3 {
                    let a = t * 1.5 + k as f32 * 2.1;
                    star5(c, cx + a.cos() * 27.0, cy + a.sin() * 12.0, 2.5, a, SAND);
                }
            }
            2 => {
                // a chess board, the knight hopping its L
                sky(c, Rgba::rgb(60, 50, 44), Rgba::rgb(50, 42, 36));
                let (bx, by, s) = (214.0, 127.0, 12.0);
                for r in 0..4 {
                    for q in 0..4 {
                        let col = if (r + q) % 2 == 0 { Rgba::rgb(236, 220, 190) } else { Rgba::rgb(170, 120, 80) };
                        c.fill(&rrect(bx + q as f32 * s, by + r as f32 * s, s, s, 0.0), col);
                    }
                }
                let squares = [(0.0f32, 3.0f32), (1.0, 1.0), (3.0, 0.0), (2.0, 2.0)];
                let k = (t * 0.5) as usize % 4;
                let u = smooth(((t * 0.5).fract() * 2.0).min(1.0));
                let (a, b) = (squares[k], squares[(k + 1) % 4]);
                let x = bx + (a.0 + (b.0 - a.0) * u) * s + s / 2.0;
                let y = by + (a.1 + (b.1 - a.1) * u) * s + s / 2.0 - 8.0 * (u * PI).sin();
                c.fill(&rrect(bx + b.0 * s, by + b.1 * s, s, s, 0.0), GREEN.fade(0.35));
                let ink = Rgba::rgb(40, 36, 44);
                c.fill(&poly(&[(x - 4.0, y + 5.0), (x + 4.0, y + 5.0), (x + 3.0, y - 1.0), (x + 1.0, y - 6.0), (x - 3.0, y - 3.0), (x - 4.5, y - 1.0), (x - 1.0, y - 1.5)]), ink);
                c.fill(&rrect(x - 5.0, y + 4.0, 10.0, 2.5, 1.0), ink);
                c.fill(&circle(x - 0.5, y - 3.5, 0.6), WHITE);
            }
            _ => {
                // the gears turning inside a head
                sky(c, Rgba::rgb(36, 54, 66), Rgba::rgb(30, 44, 56));
                let head = Rgba::rgb(110, 130, 160);
                c.fill(&poly(&[(222.0, 181.0), (222.0, 168.0), (216.0, 160.0), (216.0, 146.0), (222.0, 132.0), (236.0, 125.0), (252.0, 127.0), (262.0, 138.0), (264.0, 150.0), (270.0, 158.0), (264.0, 161.0), (264.0, 170.0), (254.0, 172.0), (252.0, 181.0)]), head);
                gear(c, 234.0, 145.0, 8.0, t * 1.6, SAND);
                gear(c, 251.0, 150.0, 5.5, -t * 1.6 * 8.0 / 5.5 + 0.3, CODE[1]);
                gear(c, 243.0, 132.0, 4.5, -t * 1.6 * 8.0 / 4.5, CODE[3]);
            }
        }
    }

    fn egg_nap(&self, c: &mut Canvas, t: f32, pick: usize) {
        let night = (Rgba::rgb(20, 20, 50), Rgba::rgb(46, 40, 80));
        match pick {
            0 => {
                // counting sheep over the fence on a moonlit hill
                sky(c, night.0, night.1);
                stars(c, t, 10, 160.0);
                c.fill(&circle(264.0, 134.0, 6.0), Rgba::rgb(255, 236, 170));
                ground(c, 170.0, 2.0, 0.5, Rgba::rgb(50, 90, 70));
                let wood = Rgba::rgb(170, 130, 90);
                for x in [234.0, 244.0] {
                    c.fill(&rrect(x, 160.0, 2.5, 13.0, 0.8), wood);
                }
                c.fill(&rrect(231.0, 163.0, 17.0, 2.0, 0.8), wood);
                c.fill(&rrect(231.0, 168.0, 17.0, 2.0, 0.8), wood);
                let ph = (t * 0.45).fract();
                let x = 206.0 + 64.0 * ph;
                let y = 166.0 - 16.0 * (ph * PI).sin();
                for (dx, dy, r) in [(-4.0, 0.0, 3.4), (0.0, -1.5, 3.8), (4.0, 0.0, 3.4), (0.0, 1.5, 3.4)] {
                    c.fill(&circle(x + dx, y + dy, r), WHITE);
                }
                c.fill(&oval(x + 5.0, y - 4.0, 6.0, 5.0), Rgba::rgb(70, 66, 86));
                c.stroke(&line(x - 3.0, y + 3.0, x - 4.0 - 2.0 * (ph * PI).sin(), y + 7.0), Rgba::rgb(70, 66, 86), 1.2);
                c.stroke(&line(x + 3.0, y + 3.0, x + 4.0 + 2.0 * (ph * PI).sin(), y + 7.0), Rgba::rgb(70, 66, 86), 1.2);
                let n = (t * 0.45) as i64 % 999 + 1;
                self.label(c, 8.0, 214.0, 136.0, &n.to_string(), Rgba::rgb(230, 226, 255));
            }
            1 => {
                // a shooting star crosses a starry sky, and a wish twinkles where it went
                sky(c, night.0, Rgba::rgb(70, 50, 100));
                stars(c, t, 22, 181.0);
                let ph = (t * 0.3).fract();
                if ph < 0.45 {
                    let k = ph / 0.45;
                    let (x, y) = (274.0 - 66.0 * k, 126.0 + 30.0 * k);
                    for i in 0..8 {
                        let u = i as f32 / 8.0;
                        c.fill(&circle(x + 18.0 * u, y - 8.2 * u, 1.6 * (1.0 - u)), WHITE.fade((1.0 - u) * (1.0 - k * 0.3)));
                    }
                } else {
                    let k = (ph - 0.45) / 0.55;
                    star5(c, 208.0, 156.0, 3.0 + 2.0 * (k * PI).sin(), t, SAND.fade((k * PI).sin()));
                }
                ground(c, 174.0, 1.5, 1.0, Rgba::rgb(24, 30, 44));
            }
            2 => {
                // a dream cloud with a fish swimming in it
                sky(c, night.0, night.1);
                stars(c, t, 8, 181.0);
                for (x, y, r) in [(214.0, 172.0, 1.6), (219.0, 166.0, 2.4)] {
                    c.fill(&circle(x, y, r), WHITE.fade(0.85));
                }
                for (dx, dy, r) in [(-14.0, 2.0, 9.0), (-4.0, -5.0, 11.0), (9.0, -3.0, 10.0), (17.0, 3.0, 8.0), (2.0, 5.0, 10.0)] {
                    c.fill(&circle(244.0 + dx, 146.0 + dy, r), WHITE.fade(0.92));
                }
                let fx = 244.0 + 9.0 * (t * 0.8).sin();
                let dir = if (t * 0.8).cos() > 0.0 { 1.0 } else { -1.0 };
                let fy = 147.0 + 1.5 * (t * 2.0).sin();
                c.fill(&poly(&[(fx - dir * 6.0, fy), (fx - dir * 11.0, fy - 4.0), (fx - dir * 11.0, fy + 4.0)]), ORANGE);
                c.fill(&oval(fx - 7.0, fy - 4.0, 14.0, 8.0), ORANGE);
                c.fill(&circle(fx + dir * 3.5, fy - 1.0, 1.0), INK);
            }
            _ => {
                // purring: soft waves rolling across the screen
                sky(c, night.0, night.1);
                for k in 0..4 {
                    let pts: Vec<(f32, f32)> = (0..=26).map(|i| {
                        let x = X0 + i as f32 * 3.0;
                        (x, 140.0 + k as f32 * 9.0 + 3.0 * (i as f32 * 0.45 - t * 2.0 + k as f32).sin())
                    }).collect();
                    c.stroke(&polyline(&pts), CODE[k].fade(0.6), 2.0);
                }
                let a = 0.5 + 0.5 * (t * 1.5).sin();
                self.label(c, 9.0, 238.0, 132.0, "prrr", Rgba::rgb(230, 226, 255).fade(a));
            }
        }
    }
}
