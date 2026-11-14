//! Screens 1..4 of each mood and of the nap (look 0 is the mood's first screen, drawn in scene.rs; 5..7 are in
//! more.rs). A new status line picks one at random, never the one that mood showed last time.

use super::*;

pub(super) const C: (f32, f32) = (238.0, 151.0);
pub(super) const PAPER: Rgba = Rgba::rgb(236, 232, 220);
pub(super) const TEXT: Rgba = Rgba::rgb(150, 156, 172);
pub(super) const ORANGE: Rgba = Rgba::rgb(255, 150, 70);

pub(super) fn arc_pts(x: f32, y: f32, r: f32, a0: f32, a1: f32) -> Vec<(f32, f32)> {
    (0..=32).map(|i| {
        let a = a0 + (a1 - a0) * i as f32 / 32.0;
        (x + r * a.cos(), y + r * a.sin())
    }).collect()
}

/// v bouncing between 0 and span.
pub(super) fn bounce(v: f32, span: f32) -> f32 {
    let q = (v / span).rem_euclid(2.0);
    if q < 1.0 { q * span } else { (2.0 - q) * span }
}

pub(super) fn gear(c: &mut Canvas, x: f32, y: f32, r: f32, rot: f32, col: Rgba) {
    let teeth = (r * 0.8) as usize;
    for k in 0..teeth {
        let a = rot + k as f32 / teeth as f32 * 2.0 * PI;
        c.stroke(&line(x + a.cos() * (r - 1.0), y + a.sin() * (r - 1.0), x + a.cos() * (r + 4.0), y + a.sin() * (r + 4.0)), col, 4.0);
    }
    c.fill(&circle(x, y, r), col);
    c.fill(&circle(x, y, r * 0.4), SCREEN_ON);
}

pub(super) fn bar(c: &mut Canvas, y: f32, k: f32, col: Rgba) {
    c.fill(&rrect(207.0, y, 62.0, 5.0, 2.5), WHITE.fade(0.12));
    c.fill(&rrect(207.0, y, 4.0 + 58.0 * k, 5.0, 2.5), col);
}

impl Scene {
    pub(super) fn label(&self, c: &mut Canvas, size: f32, x: f32, y: f32, s: &str, col: Rgba) {
        if let Some(font) = self.bold.as_ref() {
            let w = measure(font, size, s);
            c.text(font, size, x - w / 2.0, y, s, col);
        }
    }

    /// A label on a dark pill, so it reads over a busy screen.
    pub(super) fn label_on(&self, c: &mut Canvas, size: f32, x: f32, y: f32, s: &str, col: Rgba) {
        if let Some(font) = self.bold.as_ref() {
            let w = measure(font, size, s);
            c.fill(&rrect(x - w / 2.0 - 4.0, y - size - 1.0, w + 8.0, size + 5.0, (size + 5.0) / 2.0), SCREEN_ON.fade(0.9));
        }
        self.label(c, size, x, y, s, col);
    }

    pub(super) fn alt_screen(&self, c: &mut Canvas, t: f32, mood: Option<Mood>, look: usize, p: &Pose) {
        if look >= 5 && mood != Some(Mood::Testing) {
            return self.more_screen(c, t, mood, look, p);
        }
        let (cx, cy) = C;
        match mood {
            Some(Mood::Waiting) => self.waiting_alt(c, t, look),
            Some(Mood::Problem) => {
                let a = if (t * 2.0).fract() < 0.5 { 1.0 } else { 0.55 };
                match look {
                    1 => {
                        // a warning sign, blinking
                        c.fill(&poly(&[(cx, 132.0), (cx + 21.0, 169.0), (cx - 21.0, 169.0)]), SAND.fade(a));
                        c.fill(&rrect(cx - 2.2, 144.0, 4.4, 13.0, 2.0), SCREEN_BAD);
                        c.fill(&circle(cx, 163.0, 2.4), SCREEN_BAD);
                    }
                    2 => {
                        // a sad face over a progress bar that broke off
                        c.fill(&circle(226.0, 140.0, 3.2), RED.fade(a));
                        c.fill(&circle(250.0, 140.0, 3.2), RED.fade(a));
                        c.stroke(&quad(224.0, 160.0, cx, 146.0, 252.0, 160.0), RED.fade(a), 3.0);
                        c.fill(&rrect(207.0, 168.0, 34.0, 5.0, 2.5), RED.fade(0.8));
                        c.fill(&poly(&[(245.0, 168.5), (259.0, 171.0), (258.0, 176.0), (244.0, 173.5)]), RED.fade(0.45));
                    }
                    3 => {
                        // a bug scuttles to and fro under a "!"
                        let x = 214.0 + bounce(t * 16.0, 48.0);
                        let step = ((t * 10.0) as i64 % 2) as f32 * 1.5;
                        for k in [-1.0f32, 0.0, 1.0] {
                            c.stroke(&line(x + k * 4.0, 164.0, x + k * 5.5 - 2.0 + step, 172.0), RED.fade(0.8), 1.2);
                            c.stroke(&line(x + k * 4.0, 164.0, x + k * 5.5 + 2.0 - step, 156.0), RED.fade(0.8), 1.2);
                        }
                        c.fill(&oval(x - 8.0, 158.0, 16.0, 12.0), RED);
                        c.stroke(&line(x, 158.0, x, 170.0), SCREEN_BAD, 1.0);
                        c.fill(&rrect(x - 1.5, 133.0, 3.0, 11.0, 1.5), SAND.fade(a));
                        c.fill(&circle(x, 148.0, 1.7), SAND.fade(a));
                    }
                    _ => {
                        // a plug pulled out of its socket, a spark jumping the gap
                        c.stroke(&line(199.0, 153.0, 210.0, 153.0), CASE_SHADE, 3.0);
                        c.fill(&rrect(209.0, 145.0, 14.0, 16.0, 3.0), CASE);
                        c.fill(&rrect(223.0, 148.0, 7.0, 2.6, 1.0), SAND);
                        c.fill(&rrect(223.0, 155.4, 7.0, 2.6, 1.0), SAND);
                        c.fill(&rrect(250.0, 141.0, 22.0, 24.0, 5.0), CASE);
                        c.fill(&rrect(256.0, 147.5, 3.0, 4.0, 1.0), SCREEN_BAD);
                        c.fill(&rrect(256.0, 154.5, 3.0, 4.0, 1.0), SCREEN_BAD);
                        if (t * 3.0).fract() < 0.4 {
                            c.stroke(&polyline(&[(233.0, 146.0), (240.0, 151.0), (235.0, 155.0), (244.0, 161.0)]), SAND, 2.0);
                        }
                        c.fill(&rrect(207.0, 170.0, 62.0, 3.0, 1.5), RED.fade(0.5 * a));
                    }
                }
            }
            Some(Mood::Testing) => {
                let g = t - p.notes.map_or(0.0, |n| n * NOTES_SECS as f32);
                self.testing_alt(c, g, look);
                if p.notes.is_some() {
                    c.fill(&rrect(199.0, 121.0, 78.0, 60.0, 0.0), INK.fade(0.35));
                    c.fill(&rrect(231.0, 140.0, 5.0, 16.0, 1.5), WHITE);
                    c.fill(&rrect(240.0, 140.0, 5.0, 16.0, 1.5), WHITE);
                }
            }
            Some(Mood::Building) => self.building_alt(c, t, look),
            Some(Mood::Researching) => self.reading_alt(c, t, look),
            Some(Mood::Transfer(up)) => self.transfer_alt(c, t, look, up),
            Some(Mood::Thinking) => {
                let done = p.plan.map_or(0, |(k, _)| (k * 5.0).floor() as usize).min(4);
                self.planning_alt(c, t, look, done);
            }
            None => self.nap_alt(c, t, look),
            Some(Mood::Busy) => {}
        }
        let _ = cy;
    }

    fn waiting_alt(&self, c: &mut Canvas, t: f32, look: usize) {
        let (cx, cy) = C;
        match look {
            1 => {
                // a wall clock: the minute hand sweeps round, the hour hand creeps
                c.stroke(&circle(cx, cy, 20.0), CASE, 2.4);
                for k in 0..12 {
                    let a = k as f32 / 12.0 * 2.0 * PI;
                    let r0 = if k % 3 == 0 { 14.5 } else { 16.5 };
                    c.stroke(&line(cx + a.cos() * r0, cy + a.sin() * r0, cx + a.cos() * 18.0, cy + a.sin() * 18.0), CASE.fade(0.8), 1.4);
                }
                let m = t * 1.4 - PI / 2.0;
                let h = t * 1.4 / 12.0 + 0.6;
                c.stroke(&line(cx, cy, cx + h.cos() * 9.0, cy + h.sin() * 9.0), SAND, 3.0);
                c.stroke(&line(cx, cy, cx + m.cos() * 14.0, cy + m.sin() * 14.0), CODE[3], 2.0);
                c.fill(&circle(cx, cy, 2.2), CASE);
            }
            2 => {
                // three dots take turns hopping over a bar that keeps sweeping
                for i in 0..3 {
                    let ph = (t * 1.6 - i as f32 * 0.22).rem_euclid(1.0);
                    let hop = if ph < 0.4 { (ph / 0.4 * PI).sin() } else { 0.0 };
                    c.fill(&circle(226.0 + i as f32 * 12.0, 150.0 - 9.0 * hop, 4.0), CODE[i]);
                }
                c.fill(&rrect(207.0, 165.0, 62.0, 6.0, 3.0), WHITE.fade(0.12));
                c.fill(&rrect(207.0 + bounce(t * 30.0, 48.0), 165.0, 14.0, 6.0, 3.0), SAND.fade(0.75));
            }
            3 => {
                // a snail inches toward a flag
                let x = 208.0 + (t * 2.2) % 48.0;
                c.stroke(&line(204.0, 170.0, 272.0, 170.0), CASE.fade(0.4), 1.2);
                c.stroke(&line(208.0, 168.5, x - 4.0, 168.5), WHITE.fade(0.2), 1.6);
                c.stroke(&line(266.0, 170.0, 266.0, 146.0), CASE, 1.6);
                c.fill(&poly(&[(266.0, 146.0), (276.0, 150.0), (266.0, 154.0)]), RED);
                let wob = 0.8 * (t * 3.0).sin();
                c.fill(&rrect(x - 8.0, 164.0, 20.0, 6.0, 3.0), Rgba::rgb(170, 220, 160));
                c.stroke(&line(x + 9.0, 165.0, x + 11.0 + wob, 158.0), Rgba::rgb(170, 220, 160), 1.4);
                c.fill(&circle(x, 160.0, 7.0), SAND);
                c.stroke(&polyline(&arc_pts(x, 160.0, 3.5, 0.0, 5.0)), Rgba::rgb(222, 164, 40), 1.4);
            }
            _ => {
                // a spinner: a ring of dots with a bright head going round
                let head = (t * 10.0) as i64;
                for k in 0..10 {
                    let a = k as f32 / 10.0 * 2.0 * PI - PI / 2.0;
                    let age = (head - k).rem_euclid(10) as f32;
                    c.fill(&circle(cx + a.cos() * 16.0, cy + a.sin() * 16.0, 3.2 - age * 0.15), CODE[3].fade(1.0 - age * 0.09));
                }
            }
        }
    }

    fn testing_alt(&self, c: &mut Canvas, g: f32, look: usize) {
        let (cx, _) = C;
        match look {
            l if l >= 5 => self.more_testing(c, g, l),
            1 => {
                // a space shooter: the ship slides along the bottom, firing up at a row of bobbing saucers
                let ship = |g: f32| cx + 22.0 * (g * 1.3).sin();
                let drift = 6.0 * (g * 0.8).sin();
                for i in 0..5i64 {
                    let col = CODE[(i % 4) as usize];
                    let ix = 214.0 + i as f32 * 12.0 + drift;
                    let iy = 136.0 + 1.5 * (g * 3.0 + i as f32).sin();
                    c.fill(&oval(ix - 3.0, iy - 5.0, 6.0, 6.0), WHITE.fade(0.5));
                    c.fill(&oval(ix - 5.5, iy - 2.0, 11.0, 5.0), col);
                }
                for k in 0..3 {
                    let ph = (g * 1.5 + k as f32 / 3.0).fract();
                    let fx = ship(g - ph / 1.5);
                    c.fill(&rrect(fx - 0.8, 160.0 - 22.0 * ph, 1.6, 4.0, 0.8), SAND);
                }
                let sx = ship(g);
                c.fill(&poly(&[(sx, 162.0), (sx + 7.0, 172.0), (sx - 7.0, 172.0)]), GREEN);
            }
            2 => {
                // a racing game: the road rushes toward us and the car swerves round the cones
                c.fill(&poly(&[(230.0, 121.0), (246.0, 121.0), (274.0, 181.0), (202.0, 181.0)]), Rgba::rgb(70, 76, 90));
                for k in 0..4 {
                    let ph = (g * 1.2 + k as f32 / 4.0).fract();
                    c.fill(&rrect(cx - 0.6 - 0.8 * ph, 121.0 + 60.0 * ph * ph, 1.2 + 1.6 * ph, 2.0 + 6.0 * ph, 0.6), WHITE.fade(0.7));
                }
                let ph = (g * 0.6).fract();
                let side = if (g * 0.6) as i64 % 2 == 0 { -1.0 } else { 1.0 };
                let (x, y, s) = (cx + side * (3.0 + 18.0 * ph * ph), 124.0 + 52.0 * ph * ph, 1.0 + 4.0 * ph);
                c.fill(&poly(&[(x, y - s * 1.6), (x + s, y), (x - s, y)]), ORANGE);
                let dodge = if ph < 0.85 { (ph / 0.85 * PI / 2.0).sin() } else { 1.0 - (ph - 0.85) / 0.15 };
                let car = cx - side * 13.0 * dodge;
                c.fill(&rrect(car - 5.0, 164.0, 10.0, 12.0, 2.5), RED);
                c.fill(&rrect(car - 3.5, 166.0, 7.0, 3.0, 1.0), SCREEN_ON);
            }
            3 => {
                // pong: the ball bounces between two paddles that follow it
                let bx = 208.0 + bounce(g * 40.0, 60.0);
                let by = 126.0 + bounce(g * 27.0, 48.0);
                c.stroke(&line(cx, 124.0, cx, 178.0), WHITE.fade(0.15), 1.2);
                c.fill(&rrect(203.0, by - 7.0 + 3.0 * (g * 2.0).sin(), 3.0, 14.0, 1.5), CODE[3]);
                c.fill(&rrect(270.0, by - 7.0 - 3.0 * (g * 1.7).sin(), 3.0, 14.0, 1.5), CODE[1]);
                c.fill(&rrect(bx - 2.0, by - 2.0, 4.0, 4.0, 1.0), WHITE);
            }
            _ => {
                // a catch game: stars fall, the basket slides under each one in time
                const FALL: f32 = 1.4;
                let star_x = |n: i64| 210.0 + (hash(n * 13 + 5) % 56) as f32;
                let n = (g / FALL).floor() as i64;
                let k = (g / FALL).fract();
                let (sx, sy) = (star_x(n), 124.0 + 44.0 * k);
                let from = star_x(n - 1);
                let bx = from + (sx - from) * smooth((k / 0.7).min(1.0));
                let mut star = Vec::new();
                for i in 0..10 {
                    let a = i as f32 / 10.0 * 2.0 * PI - PI / 2.0 + g;
                    let r = if i % 2 == 0 { 4.5 } else { 2.0 };
                    star.push((sx + a.cos() * r, sy + a.sin() * r));
                }
                c.fill(&poly(&star), SAND);
                c.fill(&poly(&[(bx - 9.0, 168.0), (bx + 9.0, 168.0), (bx + 6.0, 176.0), (bx - 6.0, 176.0)]), ORANGE);
                c.stroke(&line(bx - 7.5, 171.0, bx + 7.5, 171.0), Rgba::rgb(200, 110, 50), 1.0);
                let caught = (n.rem_euclid(8)) as usize;
                for i in 0..caught {
                    c.fill(&circle(206.0 + i as f32 * 5.0, 128.0, 1.6), SAND);
                }
            }
        }
    }

    fn building_alt(&self, c: &mut Canvas, t: f32, look: usize) {
        let brick = Rgba::rgb(206, 96, 66);
        let wood = Rgba::rgb(206, 150, 90);
        match look {
            1 => {
                // a crane lowers crates onto a stack, which starts over once it is five high
                const DROP: f32 = 2.4;
                let n = (t / DROP) as usize % 5;
                let k = (t / DROP).fract();
                c.stroke(&line(266.0, 177.0, 266.0, 128.0), SAND, 2.4);
                c.stroke(&line(208.0, 129.0, 270.0, 129.0), SAND, 2.4);
                let bx = 222.0;
                let top = 177.0 - 8.0 * n as f32;
                let down = smooth((k / 0.6).min(1.0));
                let up = smooth(((k - 0.7) / 0.3).clamp(0.0, 1.0));
                let hy = 134.0 + (top - 8.0 - 134.0) * down * (1.0 - up);
                c.stroke(&line(bx + 5.0, 130.0, bx + 5.0, hy), CASE.fade(0.8), 1.0);
                let crate_at = |c: &mut Canvas, y: f32| {
                    c.fill(&rrect(bx, y, 10.0, 7.5, 1.0), wood);
                    c.stroke(&line(bx + 1.5, y + 1.5, bx + 8.5, y + 6.0), Rgba::rgb(160, 106, 60), 1.0);
                };
                for i in 0..n {
                    crate_at(c, 177.0 - 8.0 * (i + 1) as f32);
                }
                crate_at(c, if k < 0.7 { hy } else { top - 8.0 });
            }
            2 => {
                // two gears turn each other while a bar fills underneath
                gear(c, 226.0, 145.0, 11.0, t * 1.4, CODE[1]);
                gear(c, 252.0, 152.0, 8.0, -t * 1.4 * 11.0 / 8.0 + 0.2, CODE[3]);
                bar(c, 172.0, (t * 0.1).fract(), GREEN);
            }
            3 => {
                // blocks drop into a six-wide pit and pile up, then the pile clears
                let step = (t * 2.5) as usize % 19;
                let fall = 1.0 - (t * 2.5).fract();
                for i in 0..step.min(18) + 1 {
                    if i >= 18 {
                        break;
                    }
                    let col = (hash(i as i64 * 7 + 3) % 6) as usize;
                    let row = i / 6;
                    let x = 209.0 + ((col + i) % 6) as f32 * 10.0;
                    let mut y = 168.0 - row as f32 * 9.0;
                    if i == step {
                        y -= fall * 40.0;
                    }
                    c.fill(&rrect(x, y, 9.0, 8.0, 1.5), CODE[i % 4]);
                }
                c.stroke(&polyline(&[(207.0, 128.0), (207.0, 177.0), (270.0, 177.0), (270.0, 128.0)]), CASE.fade(0.5), 1.4);
            }
            _ => {
                // a little house goes up: the walls, the roof, then a door and a window
                let stage = (t / 1.4) as usize % 6;
                c.stroke(&line(204.0, 175.0, 272.0, 175.0), GREEN.fade(0.7), 1.6);
                if stage >= 1 {
                    let w = if stage == 1 { brick.fade(0.4) } else { brick };
                    c.fill(&rrect(218.0, 151.0, 40.0, 24.0, 1.0), w);
                }
                if stage >= 3 {
                    c.fill(&poly(&[(214.0, 152.0), (238.0, 133.0), (262.0, 152.0)]), Rgba::rgb(120, 86, 150));
                }
                if stage >= 4 {
                    c.fill(&rrect(234.0, 161.0, 8.0, 14.0, 1.0), wood);
                }
                if stage >= 5 {
                    c.fill(&rrect(222.0, 157.0, 8.0, 7.0, 1.0), SAND);
                    c.fill(&rrect(247.0, 157.0, 8.0, 7.0, 1.0), SAND);
                }
            }
        }
    }

    fn reading_alt(&self, c: &mut Canvas, t: f32, look: usize) {
        let (cx, cy) = C;
        match look {
            1 => {
                // a magnifying glass drifts along the lines, the word under it big
                for i in 0..6 {
                    let len = 40.0 + (hash(i + 20) % 22) as f32;
                    c.fill(&rrect(207.0, 129.0 + i as f32 * 8.5, len, 2.2, 1.1), TEXT);
                }
                let u = (t * 0.2).fract() * 6.0;
                let (lx, ly) = (212.0 + 40.0 * u.fract(), 130.0 + u.floor() * 8.5);
                c.fill(&circle(lx, ly, 8.0), WHITE.fade(0.18));
                c.fill(&rrect(lx - 6.0, ly - 2.0, 12.0, 4.0, 2.0), Rgba::rgb(214, 220, 232));
                c.stroke(&circle(lx, ly, 8.0), CASE, 2.0);
                c.stroke(&line(lx + 5.7, ly + 5.7, lx + 11.0, ly + 11.0), SAND, 3.0);
            }
            2 => {
                // an open book, its pages turning
                c.fill(&rrect(207.0, 131.0, 31.0, 40.0, 1.5), PAPER);
                c.fill(&rrect(238.0, 131.0, 31.0, 40.0, 1.5), PAPER);
                for (side, s) in [(210.0, 0), (241.0, 9)] {
                    for i in 0..5 {
                        let len = 18.0 + (hash(i + s) % 7) as f32;
                        c.fill(&rrect(side, 137.0 + i as f32 * 7.0, len, 2.0, 1.0), TEXT);
                    }
                }
                c.stroke(&line(238.0, 131.0, 238.0, 171.0), CASE_SHADE, 1.2);
                let k = smooth(((t * 0.4).fract() * 1.6).min(1.0));
                let w = 31.0 * (k * PI).cos();
                if w.abs() > 0.6 {
                    let (x, ww) = if w > 0.0 { (238.0, w) } else { (238.0 + w, -w) };
                    c.fill(&rrect(x, 131.0, ww, 40.0, 1.5), Rgba::rgb(250, 248, 240));
                    c.stroke(&rrect(x, 131.0, ww, 40.0, 1.5), CASE_SHADE, 0.8);
                }
            }
            3 => {
                // a search box with dots typed in, then the results drop in one by one
                c.fill(&rrect(207.0, 128.0, 62.0, 10.0, 5.0), WHITE.fade(0.9));
                c.stroke(&circle(262.0, 133.0, 2.6), TEXT, 1.2);
                let k = (t * 0.3).fract();
                let typed = ((k * 3.0).min(1.0) * 5.0) as usize;
                for i in 0..typed {
                    c.fill(&circle(213.0 + i as f32 * 5.0, 133.0, 1.4), SCREEN_ON);
                }
                let shown = (((k - 0.33) / 0.6).clamp(0.0, 1.0) * 4.0).ceil() as usize;
                for i in 0..shown.min(4) {
                    let y = 145.0 + i as f32 * 8.5;
                    c.fill(&rrect(207.0, y, 22.0 + (hash(i as i64 + 40) % 20) as f32, 2.6, 1.3), CODE[3]);
                    c.fill(&rrect(207.0, y + 3.6, 44.0, 1.8, 0.9), TEXT.fade(0.7));
                }
            }
            _ => {
                // a globe turning, its meridians sliding round
                c.fill(&circle(cx, cy, 19.0), Rgba::rgb(80, 140, 200));
                c.fill(&oval(cx - 12.0 + 6.0 * (t * 0.7).sin(), cy - 9.0, 13.0, 9.0), Rgba::rgb(110, 190, 120));
                c.fill(&oval(cx + 2.0 + 6.0 * (t * 0.7).sin(), cy + 3.0, 9.0, 11.0), Rgba::rgb(110, 190, 120));
                for k in 0..3 {
                    let ph = ((t * 0.15 + k as f32 / 3.0).fract() * PI).cos();
                    let w = (19.0 * ph).abs();
                    c.stroke(&oval(cx - w, cy - 19.0, 2.0 * w, 38.0), WHITE.fade(0.3), 1.0);
                }
                c.stroke(&line(cx - 19.0, cy, cx + 19.0, cy), WHITE.fade(0.3), 1.0);
                c.stroke(&circle(cx, cy, 19.0), CASE, 1.6);
            }
        }
    }

    fn transfer_alt(&self, c: &mut Canvas, t: f32, look: usize, up: bool) {
        let (cx, _) = C;
        let d = if up { -1.0 } else { 1.0 };
        match look {
            1 => {
                // packets hop from this little computer to the other one (or back, for a download)
                for x in [214.0, 262.0] {
                    c.fill(&rrect(x - 9.0, 139.0, 18.0, 13.0, 2.0), CASE);
                    c.fill(&rrect(x - 7.0, 141.0, 14.0, 9.0, 1.0), SCREEN_OFF);
                    c.fill(&rrect(x - 4.0, 153.0, 8.0, 3.0, 1.0), CASE_SHADE);
                }
                for i in 0..4 {
                    let ph = (t * 0.7 + i as f32 / 4.0).fract();
                    let u = if up { ph } else { 1.0 - ph };
                    c.fill(&circle(224.0 + 28.0 * u, 140.0 - 10.0 * (u * PI).sin(), 2.2), CODE[i].fade((ph * PI).sin()));
                }
                bar(c, 172.0, (t * 0.12).fract(), GREEN);
            }
            2 => {
                // an arrow bobbing in a ring that fills
                let k = (t * 0.12).fract();
                c.stroke(&circle(cx, 149.0, 17.0), WHITE.fade(0.12), 4.0);
                if k > 0.01 {
                    c.stroke(&polyline(&arc_pts(cx, 149.0, 17.0, -PI / 2.0, -PI / 2.0 + 2.0 * PI * k)), GREEN, 4.0);
                }
                let y = 149.0 + 2.0 * (t * 4.0).sin();
                c.stroke(&line(cx, y - 7.0 * d, cx, y + 7.0 * d), CODE[3], 3.0);
                c.stroke(&polyline(&[(cx - 5.0, y + 2.0 * d), (cx, y + 7.0 * d), (cx + 5.0, y + 2.0 * d)]), CODE[3], 3.0);
            }
            3 => {
                // parcels ride a belt into a folder (or out of it)
                c.fill(&rrect(250.0, 146.0, 22.0, 18.0, 2.0), SAND);
                c.fill(&rrect(250.0, 142.0, 10.0, 6.0, 2.0), SAND);
                c.fill(&rrect(203.0, 164.0, 52.0, 4.0, 2.0), CASE_SHADE);
                for i in 0..3 {
                    let ph = (t * 0.35 + i as f32 / 3.0).fract();
                    let u = if up { ph } else { 1.0 - ph };
                    let x = 200.0 + 52.0 * u;
                    let a = (ph * PI).sin().min(0.5) * 2.0;
                    c.fill(&rrect(x, 155.0, 10.0, 9.0, 1.5), Rgba::rgb(206, 150, 90).fade(a));
                    c.stroke(&line(x + 5.0, 155.0, x + 5.0, 164.0), Rgba::rgb(240, 210, 150).fade(a), 1.4);
                }
                c.fill(&rrect(250.0, 150.0, 22.0, 14.0, 2.0), Rgba::rgb(240, 196, 80));
            }
            _ => {
                // a dish sends rings out (or catches them coming in)
                let (dx, dy) = (222.0, 160.0);
                c.fill(&poly(&[(dx - 4.0, 176.0), (dx + 4.0, 176.0), (dx, 162.0)]), CASE_SHADE);
                let mut dish = arc_pts(dx, dy, 13.0, -PI * 0.95, -PI * 0.05);
                dish.reverse();
                c.fill(&poly(&dish), CASE);
                c.stroke(&line(dx, dy - 1.0, dx + 6.0, dy - 9.0), CASE, 1.4);
                for k in 0..3 {
                    let ph = (t * 0.8 + k as f32 / 3.0).fract();
                    let u = if up { ph } else { 1.0 - ph };
                    c.stroke(&polyline(&arc_pts(dx + 6.0, dy - 9.0, 6.0 + 34.0 * u, -PI * 0.42, -PI * 0.08)), GREEN.fade((ph * PI).sin()), 2.0);
                }
            }
        }
    }

    fn planning_alt(&self, c: &mut Canvas, t: f32, look: usize, done: usize) {
        let (cx, cy) = C;
        let notes = [SAND, PINK_DARK, CODE[3], CODE[0]];
        match look {
            1 => {
                // a mind map: an idea in the middle, a branch with every box the board gets
                let nodes = [(215.0, 133.0), (261.0, 133.0), (215.0, 169.0), (261.0, 169.0)];
                for &(x, y) in nodes.iter().take(done) {
                    c.stroke(&line(cx, cy, x, y), CASE.fade(0.6), 1.4);
                }
                c.fill(&rrect(cx - 11.0, cy - 5.0, 22.0, 10.0, 4.0), SAND);
                for (i, &(x, y)) in nodes.iter().enumerate().take(done) {
                    c.fill(&rrect(x - 9.0, y - 4.5, 18.0, 9.0, 3.5), CODE[i]);
                }
            }
            2 => {
                // sticky notes go up, one with every box
                for i in 0..done {
                    let (x, y) = (214.0 + (i % 2) as f32 * 27.0, 128.0 + (i / 2) as f32 * 25.0);
                    c.fill(&rrect(x, y, 21.0, 21.0, 1.5), notes[i]);
                    c.fill(&rrect(x, y, 21.0, 3.0, 1.0), WHITE.fade(0.3));
                    for l in 0..2 {
                        c.fill(&rrect(x + 4.0, y + 8.0 + l as f32 * 5.0, 13.0 - l as f32 * 4.0, 1.6, 0.8), INK.fade(0.35));
                    }
                }
            }
            3 => {
                // a light bulb that brightens with every box, rays once the plan is done
                let k = done as f32 / 4.0;
                if done == 4 {
                    for r in 0..8 {
                        let a = r as f32 / 8.0 * 2.0 * PI + (t * 0.8).sin() * 0.1;
                        c.stroke(&line(cx + a.cos() * 19.0, 147.0 + a.sin() * 19.0, cx + a.cos() * 25.0, 147.0 + a.sin() * 25.0), SAND, 2.0);
                    }
                }
                c.fill(&circle(cx, 147.0, 13.0), Rgba(255, 226, 120, (60.0 + 195.0 * k) as u8));
                c.stroke(&circle(cx, 147.0, 13.0), CASE, 1.6);
                c.fill(&rrect(cx - 6.0, 160.0, 12.0, 9.0, 2.0), CASE_SHADE);
                c.stroke(&line(cx - 6.0, 163.0, cx + 6.0, 163.0), SCREEN_ON.fade(0.5), 1.0);
                c.stroke(&line(cx - 6.0, 166.0, cx + 6.0, 166.0), SCREEN_ON.fade(0.5), 1.0);
            }
            _ => {
                // a timeline: a milestone fills in with every box, a marker walks to the newest
                c.stroke(&line(209.0, 151.0, 267.0, 151.0), CASE.fade(0.6), 2.0);
                for i in 0..4 {
                    let x = 212.0 + i as f32 * 17.0;
                    if i < done {
                        c.fill(&circle(x, 151.0, 4.5), notes[i]);
                    } else {
                        c.stroke(&circle(x, 151.0, 4.5), CASE.fade(0.6), 1.4);
                    }
                    c.fill(&rrect(x - 6.0, 160.0 + (i % 2) as f32 * 6.0, 12.0, 2.4, 1.2), TEXT);
                }
                let mx = 212.0 + done.max(1).saturating_sub(1) as f32 * 17.0;
                let bob = 1.5 * (t * 3.0).sin();
                c.fill(&poly(&[(mx - 4.0, 136.0 + bob), (mx + 4.0, 136.0 + bob), (mx, 143.0 + bob)]), RED);
            }
        }
    }

    fn nap_alt(&self, c: &mut Canvas, t: f32, look: usize) {
        match look {
            1 => {
                // a fish tank screensaver: fish drift across and bubbles rise
                c.fill(&rrect(199.0, 172.0, 78.0, 9.0, 0.0), Rgba::rgb(70, 64, 92));
                for i in 0..3 {
                    let f = i as f32;
                    let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
                    let u = (t * (0.05 + 0.02 * f) + f * 0.37).fract();
                    let x = if dir > 0.0 { 190.0 + 96.0 * u } else { 286.0 - 96.0 * u };
                    let y = 136.0 + f * 12.0 + 2.0 * (t * 1.3 + f).sin();
                    let col = [ORANGE, CODE[3], CODE[1]][i];
                    c.fill(&poly(&[(x - dir * 5.0, y), (x - dir * 10.0, y - 4.0), (x - dir * 10.0, y + 4.0)]), col.fade(0.8));
                    c.fill(&oval(x - 6.0, y - 3.5, 12.0, 7.0), col.fade(0.8));
                    c.fill(&circle(x + dir * 3.2, y - 1.0, 0.9), SCREEN_OFF);
                }
                for i in 0..4 {
                    let ph = (t * 0.3 + i as f32 * 0.25).fract();
                    let (bx, by) = (210.0 + i as f32 * 17.0 + 1.5 * (ph * 12.0).sin(), 172.0 - 50.0 * ph);
                    c.stroke(&circle(bx, by, 1.2 + ph), WHITE.fade(0.5 * (1.0 - ph)), 0.8);
                }
            }
            2 => {
                // a "zzz" badge drifts round the screen, changing color at every wall
                let (w, h) = (26.0, 12.0);
                let (sx, sy) = (74.0 - w, 56.0 - h);
                let (vx, vy) = (t * 12.0, t * 8.0);
                let (x, y) = (201.0 + bounce(vx, sx), 123.0 + bounce(vy, sy));
                let hits = (vx / sx).floor() as i64 + (vy / sy).floor() as i64;
                c.fill(&rrect(x, y, w, h, 5.0), CODE[hits.rem_euclid(4) as usize].fade(0.8));
                if let Some(font) = self.bold.as_ref() {
                    let tw = measure(font, 8.5, "zzz");
                    c.text(font, 8.5, x + (w - tw) / 2.0, y + 9.0, "zzz", SCREEN_OFF);
                }
            }
            3 => {
                // rain running down a dark window
                for i in 0..9 {
                    let ph = (t * (0.5 + (hash(i) % 5) as f32 * 0.08) + (hash(i + 9) % 100) as f32 / 100.0).fract();
                    let x = 204.0 + i as f32 * 8.3;
                    let y = 118.0 + 66.0 * ph;
                    c.stroke(&line(x, y, x - 0.8, y + 5.0), Rgba::rgb(170, 190, 230).fade(0.55), 1.1);
                }
                c.stroke(&line(238.0, 121.0, 238.0, 181.0), CASE.fade(0.25), 2.0);
                c.stroke(&line(199.0, 151.0, 277.0, 151.0), CASE.fade(0.25), 2.0);
            }
            _ => {
                // a lava lamp: soft blobs rise and sink
                c.fill(&poly(&[(228.0, 127.0), (248.0, 127.0), (254.0, 172.0), (222.0, 172.0)]), Rgba::rgb(76, 52, 90));
                for i in 0..3 {
                    let f = i as f32;
                    let y = 150.0 + 18.0 * (t * (0.25 + 0.07 * f) + f * 2.1).sin();
                    let r = 4.0 + f * 1.3;
                    c.fill(&circle(238.0 + (f - 1.0) * 4.0, y, r), Rgba::rgb(255, 120, 150).fade(0.85));
                }
                c.fill(&rrect(224.0, 122.0, 28.0, 6.0, 2.0), CASE_SHADE);
                c.fill(&rrect(220.0, 171.0, 36.0, 8.0, 2.0), CASE_SHADE);
            }
        }
    }
}
