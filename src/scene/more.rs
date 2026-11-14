//! Screens 5..7 of each mood and of the nap (screens.rs has 1..4, scene.rs the first one).

use super::screens::{bar, bounce, C, ORANGE, PAPER, TEXT};
use super::*;

fn person(c: &mut Canvas, x: f32, y: f32, col: Rgba) {
    c.fill(&circle(x, y - 7.0, 3.0), col);
    c.fill(&rrect(x - 4.0, y - 3.5, 8.0, 9.0, 3.5), col);
}

impl Scene {
    /// Testing screens 5..7, on the game clock `g` (it holds still while notes are taken).
    pub(super) fn more_testing(&self, c: &mut Canvas, g: f32, look: usize) {
        let (cx, _) = C;
        match look {
            5 => {
                // a gem puzzle: two gems swap, a row of three flashes and clears
                let cols = [CODE[0], CODE[1], CODE[2], CODE[3], ORANGE];
                let step = (g / 1.5) as i64;
                let k = (g / 1.5).fract();
                for r in 0..4 {
                    for q in 0..6 {
                        let id = hash(step * 31 + r * 7 + q) % 5;
                        let (mut x, y) = (208.0 + q as f32 * 11.0, 130.0 + r as f32 * 12.0);
                        if r == 1 && (q == 2 || q == 3) {
                            let swap = smooth((k * 3.0).min(1.0)) * 11.0;
                            x += if q == 2 { swap } else { -swap };
                        }
                        let col = if r == 2 && q < 3 && k > 0.5 {
                            if (k * 10.0) as i64 % 2 == 0 { WHITE } else { cols[0] }
                        } else {
                            cols[id as usize]
                        };
                        c.fill(&poly(&[(x + 4.5, y), (x + 9.0, y + 4.5), (x + 4.5, y + 9.0), (x, y + 4.5)]), col);
                        c.fill(&poly(&[(x + 4.5, y + 1.5), (x + 7.0, y + 4.0), (x + 4.5, y + 4.0)]), WHITE.fade(0.4));
                    }
                }
            }
            6 => {
                // a golf game: the ball flies its arc and drops by the flag
                c.fill(&rrect(199.0, 168.0, 78.0, 13.0, 0.0), Rgba::rgb(80, 160, 90));
                c.fill(&oval(250.0, 166.0, 22.0, 5.0), Rgba::rgb(110, 190, 110));
                c.stroke(&line(262.0, 168.0, 262.0, 145.0), CASE, 1.2);
                c.fill(&poly(&[(262.0, 145.0), (271.0, 148.5), (262.0, 152.0)]), RED);
                c.fill(&oval(260.0, 167.0, 5.0, 2.0), INK);
                let k = (g * 0.45).fract();
                let fly = (k / 0.8).min(1.0);
                let x = 212.0 + 50.0 * fly;
                let y = 166.0 - 36.0 * (fly * PI).sin();
                if k < 0.8 {
                    for i in 1..6 {
                        let u = (fly - i as f32 * 0.05).max(0.0);
                        c.fill(&circle(212.0 + 50.0 * u, 166.0 - 36.0 * (u * PI).sin(), 0.7), WHITE.fade(0.3));
                    }
                    c.fill(&circle(x, y, 2.0), WHITE);
                }
                c.stroke(&line(207.0, 168.0, 210.0, 158.0), CASE_SHADE, 1.2);
            }
            _ => {
                // a fishing game: the float bobs, then a fish bites and the line pulls taut
                c.fill(&rrect(199.0, 150.0, 78.0, 31.0, 0.0), Rgba::rgb(50, 100, 150));
                for i in 0..4 {
                    let x = 205.0 + i as f32 * 20.0 + 3.0 * (g + i as f32).sin();
                    c.stroke(&line(x, 156.0 + i as f32 * 5.0, x + 8.0, 156.0 + i as f32 * 5.0), WHITE.fade(0.25), 1.0);
                }
                let k = (g * 0.3).fract();
                let bite = k > 0.7;
                let fy = 150.0 + if bite { 3.0 + 1.5 * (g * 20.0).sin() } else { 1.0 * (g * 3.0).sin() };
                c.stroke(&line(205.0, 125.0, cx + 8.0, fy - 2.0), WHITE.fade(0.6), 0.7);
                c.fill(&circle(cx + 8.0, fy, 2.6), RED);
                c.fill(&rrect(cx + 5.4, fy - 0.2, 5.2, 2.8, 1.4), WHITE);
                let fx = if bite { cx + 6.0 } else { 220.0 + 30.0 * ((g * 0.4).sin() * 0.5 + 0.5) };
                c.fill(&oval(fx - 6.0, 166.0, 12.0, 6.0), ORANGE.fade(0.8));
                c.fill(&poly(&[(fx + 5.0, 169.0), (fx + 10.0, 166.0), (fx + 10.0, 172.0)]), ORANGE.fade(0.8));
            }
        }
    }

    /// Screens 5..7 of the moods other than testing, and of the nap.
    pub(super) fn more_screen(&self, c: &mut Canvas, t: f32, mood: Option<Mood>, look: usize, p: &Pose) {
        let (cx, cy) = C;
        match (mood, look) {
            (Some(Mood::Waiting), 5) => {
                // stuck at 99 percent, the last bit blinking
                bar(c, 152.0, 0.97, SAND);
                if (t * 2.0).fract() < 0.5 {
                    c.fill(&rrect(263.0, 152.0, 6.0, 5.0, 2.5), SAND.fade(0.5));
                }
                self.label(c, 10.0, cx, 144.0, "99%", WHITE.fade(0.85));
            }
            (Some(Mood::Waiting), 6) => {
                // a queue shuffling forward to a counter, one place at a time
                c.fill(&rrect(262.0, 146.0, 12.0, 26.0, 1.5), CASE_SHADE);
                c.stroke(&line(199.0, 172.0, 277.0, 172.0), CASE.fade(0.4), 1.2);
                let k = (t * 0.4).fract();
                let step = smooth(((k - 0.7) / 0.3).clamp(0.0, 1.0)) * 13.0;
                for i in 0..4 {
                    let x = 250.0 - i as f32 * 13.0 + step;
                    if x < 258.0 {
                        person(c, x, 164.0, if i == 3 { SAND } else { CODE[3].fade(0.7) });
                    }
                }
                self.label(c, 6.5, 222.0, 134.0, "you are #4", WHITE.fade(0.7));
            }
            (Some(Mood::Waiting), _) => {
                // the day goes by: the sun crosses the sky, then the moon
                let k = (t * 0.12).fract();
                let day = k < 0.5;
                let u = (k % 0.5) * 2.0;
                let (x, y) = (204.0 + 68.0 * u, 172.0 - 40.0 * (u * PI).sin());
                if day {
                    c.fill(&circle(x, y, 6.0), SAND);
                } else {
                    c.fill(&circle(x, y, 5.0), Rgba::rgb(230, 226, 255));
                    c.fill(&circle(x + 2.5, y - 1.5, 4.5), SCREEN_ON);
                }
                c.fill(&poly(&[(199.0, 181.0), (199.0, 170.0), (230.0, 164.0), (260.0, 168.0), (277.0, 166.0), (277.0, 181.0)]), Rgba::rgb(50, 90, 70));
            }
            (Some(Mood::Problem), 5) => {
                // a chain snapped in the middle
                let a = if (t * 2.0).fract() < 0.5 { 1.0 } else { 0.6 };
                for (x, off) in [(206.0, 0.0), (218.0, 0.0), (250.0, 2.0), (262.0, 2.0)] {
                    c.stroke(&rrect(x, 147.0 + off, 12.0, 7.0, 3.5), CASE, 2.2);
                }
                c.stroke(&quad(229.0, 150.0, 232.0, 143.0, 235.0, 148.0), RED.fade(a), 2.2);
                c.stroke(&quad(245.0, 152.0, 242.0, 159.0, 240.0, 154.0), RED.fade(a), 2.2);
            }
            (Some(Mood::Problem), 6) => {
                // a crack runs over the glass
                c.fill(&circle(244.0, 146.0, 3.0), WHITE.fade(0.5));
                for (i, pts) in [
                    vec![(244.0, 146.0), (236.0, 140.0), (226.0, 138.0), (214.0, 131.0)],
                    vec![(244.0, 146.0), (252.0, 155.0), (255.0, 166.0), (264.0, 178.0)],
                    vec![(244.0, 146.0), (232.0, 154.0), (224.0, 168.0)],
                    vec![(244.0, 146.0), (258.0, 140.0), (272.0, 136.0)],
                ].iter().enumerate() {
                    let n = (((t * 0.8).fract() * 6.0) as usize + 2).min(pts.len());
                    c.stroke(&polyline(&pts[..n.max(2)]), WHITE.fade(0.7 - i as f32 * 0.08), 1.1);
                }
            }
            (Some(Mood::Problem), _) => {
                // a thermometer running hot
                let k = 0.6 + 0.4 * (t * 1.5).sin().abs();
                c.fill(&rrect(234.0, 127.0, 8.0, 38.0, 4.0), WHITE.fade(0.85));
                c.fill(&circle(238.0, 167.0, 7.0), WHITE.fade(0.85));
                c.fill(&circle(238.0, 167.0, 5.0), RED);
                c.fill(&rrect(236.0, 165.0 - 34.0 * k, 4.0, 34.0 * k, 2.0), RED);
                for i in 0..5 {
                    c.stroke(&line(244.0, 132.0 + i as f32 * 7.0, 248.0, 132.0 + i as f32 * 7.0), WHITE.fade(0.5), 1.0);
                }
                for s in [-1.0f32, 1.0] {
                    let w = 2.0 * (t * 6.0 + s).sin();
                    c.stroke(&quad(238.0 + s * 18.0, 150.0, 238.0 + s * 20.0 + w, 140.0, 238.0 + s * 18.0, 130.0), ORANGE.fade(0.6), 1.4);
                }
            }
            (Some(Mood::Building), 5) => {
                // a bridge goes across the gap, one span at a time
                c.fill(&rrect(199.0, 158.0, 14.0, 23.0, 0.0), Rgba::rgb(110, 90, 70));
                c.fill(&rrect(263.0, 158.0, 14.0, 23.0, 0.0), Rgba::rgb(110, 90, 70));
                c.fill(&rrect(213.0, 172.0, 50.0, 9.0, 0.0), Rgba::rgb(60, 100, 150));
                let n = (t * 1.2) as usize % 7;
                for i in 0..n.min(5) {
                    let x = 213.0 + i as f32 * 10.0;
                    c.fill(&rrect(x, 155.0, 10.0, 3.0, 0.5), SAND);
                    c.stroke(&line(x, 158.0, x + 5.0, 165.0), SAND.fade(0.7), 1.0);
                    c.stroke(&line(x + 10.0, 158.0, x + 5.0, 165.0), SAND.fade(0.7), 1.0);
                }
            }
            (Some(Mood::Building), 6) => {
                // a robot arm welds a seam, sparks spraying
                c.fill(&rrect(222.0, 168.0, 44.0, 6.0, 1.0), CASE_SHADE);
                let k = (t * 0.3).fract();
                let tip = (226.0 + 36.0 * bounce(k * 2.0, 1.0), 166.0);
                c.fill(&rrect(205.0, 166.0, 12.0, 8.0, 2.0), CASE);
                let elbow = (216.0 + (tip.0 - 216.0) * 0.4, 140.0);
                c.stroke(&line(211.0, 166.0, elbow.0, elbow.1), SAND, 3.0);
                c.stroke(&line(elbow.0, elbow.1, tip.0, tip.1 - 3.0), SAND, 2.6);
                c.fill(&circle(elbow.0, elbow.1, 2.4), CASE);
                c.fill(&rrect(226.0, 167.0, tip.0 - 226.0, 1.6, 0.8), ORANGE.fade(0.7));
                for i in 0..6 {
                    let ph = (t * 4.0 + i as f32 / 6.0).fract();
                    let a = -PI / 2.0 + (i as f32 - 2.5) * 0.4;
                    c.fill(&circle(tip.0 + a.cos() * 10.0 * ph, tip.1 - 2.0 + a.sin() * 8.0 * ph + 10.0 * ph * ph, 0.9), SAND.fade(1.0 - ph));
                }
            }
            (Some(Mood::Building), _) => {
                // scaffolding climbs floor by floor
                let n = (t * 0.8) as usize % 7;
                c.stroke(&line(199.0, 177.0, 277.0, 177.0), CASE.fade(0.5), 1.4);
                for f in 0..n.min(5) {
                    let y = 177.0 - (f + 1) as f32 * 10.0;
                    c.fill(&rrect(220.0, y, 36.0, 10.0, 0.0), Rgba::rgb(206, 96, 66).fade(0.6));
                    c.stroke(&line(218.0, y, 258.0, y), SAND, 1.4);
                    c.stroke(&line(218.0, y, 218.0, y + 10.0), SAND, 1.2);
                    c.stroke(&line(258.0, y, 258.0, y + 10.0), SAND, 1.2);
                    c.stroke(&line(218.0, y + 10.0, 258.0, y), SAND.fade(0.5), 0.8);
                }
            }
            (Some(Mood::Researching), 5) => {
                // a microscope view: cells drift and wobble
                c.fill(&circle(cx, cy, 27.0), Rgba::rgb(220, 236, 210));
                for i in 0..5 {
                    let f = i as f32;
                    let x = cx + 14.0 * (t * 0.3 + f * 1.7).sin();
                    let y = cy + 10.0 * (t * 0.25 + f * 2.3).cos();
                    let r = 4.0 + (hash(i) % 3) as f32 + 0.5 * (t * 2.0 + f).sin();
                    c.fill(&circle(x, y, r), Rgba::rgb(150, 200, 140).fade(0.8));
                    c.fill(&circle(x + 1.0, y - 0.5, r * 0.35), Rgba::rgb(100, 140, 120));
                }
                c.stroke(&circle(cx, cy, 27.0), INK.fade(0.8), 4.0);
            }
            (Some(Mood::Researching), 6) => {
                // a bar chart, a pointer reading each bar in turn
                let bars = [18.0, 30.0, 24.0, 38.0, 14.0];
                c.stroke(&line(206.0, 176.0, 270.0, 176.0), CASE.fade(0.6), 1.2);
                let at = (t * 0.8) as usize % 5;
                for (i, h) in bars.iter().enumerate() {
                    let x = 210.0 + i as f32 * 12.0;
                    c.fill(&rrect(x, 176.0 - h, 8.0, *h, 1.0), if i == at { SAND } else { CODE[3].fade(0.7) });
                }
                let x = 214.0 + at as f32 * 12.0;
                c.fill(&poly(&[(x, 172.0 - bars[at]), (x - 3.5, 166.0 - bars[at]), (x + 3.5, 166.0 - bars[at])]), WHITE);
            }
            (Some(Mood::Researching), _) => {
                // a map: a dotted route crawls from the pin to the cross
                c.fill(&rrect(203.0, 125.0, 70.0, 52.0, 2.0), PAPER);
                c.fill(&oval(240.0, 145.0, 26.0, 18.0), Rgba::rgb(150, 200, 230));
                c.fill(&oval(208.0, 160.0, 18.0, 12.0), Rgba::rgb(170, 210, 150));
                let pts: Vec<(f32, f32)> = (0..=24).map(|i| {
                    let u = i as f32 / 24.0;
                    (214.0 + 48.0 * u, 140.0 + 22.0 * u + 8.0 * (u * 7.0).sin())
                }).collect();
                let shown = (((t * 0.2).fract() * 30.0) as usize).min(pts.len());
                for (i, &(x, y)) in pts.iter().take(shown).enumerate() {
                    if i % 2 == 0 {
                        c.fill(&circle(x, y, 0.9), RED.fade(0.8));
                    }
                }
                c.fill(&circle(214.0, 136.0, 3.0), RED);
                c.fill(&poly(&[(211.5, 137.5), (216.5, 137.5), (214.0, 142.0)]), RED);
                let (ex, ey) = (262.0, 162.0);
                c.stroke(&line(ex - 3.0, ey - 3.0, ex + 3.0, ey + 3.0), INK.fade(0.8), 1.6);
                c.stroke(&line(ex + 3.0, ey - 3.0, ex - 3.0, ey + 3.0), INK.fade(0.8), 1.6);
                c.stroke(&rrect(203.0, 125.0, 70.0, 52.0, 2.0), TEXT, 0.8);
            }
            (Some(Mood::Transfer(up)), 5) => {
                // a paper plane glides across, leaving a dashed trail
                let u = (t * 0.25).fract();
                let x = if up { 196.0 + 86.0 * u } else { 282.0 - 86.0 * u };
                let d = if up { 1.0 } else { -1.0 };
                let y = 155.0 - 12.0 * u + 4.0 * (t * 2.0).sin();
                for i in 1..6 {
                    let tx = x - d * i as f32 * 7.0;
                    c.fill(&rrect(tx - 2.0, y + i as f32 * 1.2, 3.0, 1.0, 0.5), WHITE.fade(0.4 - i as f32 * 0.06));
                }
                c.fill(&poly(&[(x + d * 9.0, y), (x - d * 7.0, y - 5.0), (x - d * 3.0, y), (x - d * 7.0, y + 3.0)]), WHITE);
                c.fill(&poly(&[(x + d * 9.0, y), (x - d * 3.0, y), (x - d * 5.0, y + 2.0)]), CASE_SHADE);
                bar(c, 174.0, (t * 0.12).fract(), GREEN);
            }
            (Some(Mood::Transfer(up)), 6) => {
                // bubbles of data stream through a pipe
                c.fill(&rrect(199.0, 143.0, 78.0, 16.0, 0.0), Rgba::rgb(150, 200, 220).fade(0.25));
                c.stroke(&line(199.0, 143.0, 277.0, 143.0), CASE.fade(0.6), 1.4);
                c.stroke(&line(199.0, 159.0, 277.0, 159.0), CASE.fade(0.6), 1.4);
                for i in 0..7 {
                    let ph = (t * 0.5 + i as f32 / 7.0).fract();
                    let u = if up { ph } else { 1.0 - ph };
                    let x = 196.0 + 84.0 * u;
                    let y = 151.0 + 3.5 * (t * 3.0 + i as f32 * 1.9).sin();
                    c.fill(&circle(x, y, 2.0 + (hash(i) % 2) as f32), CODE[(i % 4) as usize].fade(0.85));
                }
                bar(c, 172.0, (t * 0.12).fract(), GREEN);
            }
            (Some(Mood::Transfer(up)), _) => {
                // a moving truck drives across with its boxes
                c.fill(&rrect(199.0, 170.0, 78.0, 11.0, 0.0), Rgba::rgb(70, 76, 90));
                for i in 0..4 {
                    let x = 199.0 + ((i as f32 * 22.0 - t * 30.0).rem_euclid(88.0));
                    c.fill(&rrect(x, 175.0, 10.0, 1.5, 0.7), WHITE.fade(0.5));
                }
                let x = cx + 3.0 * (t * 2.0).sin();
                let bob = if (t * 8.0).fract() < 0.5 { 0.0 } else { 0.5 };
                let cab = x + if up { 8.0 } else { -30.0 };
                c.fill(&rrect(x - 18.0, 150.0 + bob, 26.0, 18.0, 1.5), CASE);
                c.fill(&rrect(cab, 157.0 + bob, 12.0, 11.0, 2.0), RED);
                c.fill(&rrect(cab + if up { 5.0 } else { 1.0 }, 159.0 + bob, 6.0, 4.0, 1.0), Rgba::rgb(150, 200, 230));
                for wx in [x - 12.0, x + 2.0, cab + 6.0] {
                    c.fill(&circle(wx, 169.0, 3.0), INK);
                    c.fill(&circle(wx, 169.0, 1.1), CASE_SHADE);
                }
                c.fill(&rrect(x - 14.0, 145.0 + bob, 9.0, 6.0, 0.8), Rgba::rgb(206, 150, 90));
                c.fill(&rrect(x - 4.0, 146.0 + bob, 8.0, 5.0, 0.8), Rgba::rgb(206, 150, 90));
            }
            (Some(Mood::Thinking), _) => {
                let done = p.plan.map_or(0, |(k, _)| (k * 5.0).floor() as usize).min(4);
                match look {
                    5 => {
                        // a jigsaw fills in, a piece with every box
                        for i in 0..4 {
                            let (x, y) = (220.0 + (i % 2) as f32 * 18.0, 133.0 + (i / 2) as f32 * 18.0);
                            if i < done {
                                c.fill(&rrect(x, y, 18.0, 18.0, 1.5), CODE[i]);
                                c.fill(&circle(x + 18.0, y + 9.0, 3.0), CODE[i]);
                            } else {
                                c.stroke(&rrect(x + 0.5, y + 0.5, 17.0, 17.0, 1.5), CASE.fade(0.4), 1.0);
                            }
                        }
                    }
                    6 => {
                        // a blueprint: the plan's lines draw in as it grows
                        c.fill(&rrect(203.0, 125.0, 70.0, 52.0, 1.0), Rgba::rgb(40, 90, 160));
                        for i in 0..7 {
                            c.stroke(&line(203.0 + i as f32 * 10.0, 125.0, 203.0 + i as f32 * 10.0, 177.0), WHITE.fade(0.08), 0.6);
                        }
                        let parts: [&[(f32, f32)]; 4] = [
                            &[(214.0, 170.0), (262.0, 170.0)],
                            &[(218.0, 170.0), (218.0, 150.0), (258.0, 150.0), (258.0, 170.0)],
                            &[(214.0, 150.0), (238.0, 133.0), (262.0, 150.0)],
                            &[(234.0, 170.0), (234.0, 158.0), (242.0, 158.0), (242.0, 170.0)],
                        ];
                        for part in parts.iter().take(done) {
                            c.stroke(&polyline(part), WHITE.fade(0.85), 1.2);
                        }
                    }
                    _ => {
                        // index cards line up in order, one with every box
                        for i in 0..4 {
                            let y = 130.0 + i as f32 * 11.0;
                            let x = if i < done { 210.0 } else { 222.0 + (hash(i as i64) % 20) as f32 };
                            c.fill(&rrect(x, y, 34.0, 9.0, 1.5), PAPER.fade(if i < done { 1.0 } else { 0.5 }));
                            c.fill(&rrect(x + 3.0, y + 3.5, 18.0, 2.0, 1.0), TEXT);
                            self.label(c, 6.0, x + 29.0, y + 7.0, &(i + 1).to_string(), CODE[i]);
                        }
                    }
                }
            }
            (None, 5) => {
                // a candle flickering in the dark
                c.fill(&circle(cx, 146.0, 16.0), SAND.fade(0.08 + 0.04 * (t * 7.0).sin()));
                c.fill(&rrect(cx - 4.0, 150.0, 8.0, 24.0, 1.5), Rgba::rgb(240, 230, 210));
                c.stroke(&line(cx, 150.0, cx, 147.0), INK, 0.8);
                let w = 1.0 * (t * 9.0).sin();
                c.fill(&poly(&[(cx - 2.6, 147.0), (cx + w, 136.0), (cx + 2.6, 147.0)]), ORANGE);
                c.fill(&oval(cx - 2.6, 142.0, 5.2, 6.0), ORANGE);
                c.fill(&oval(cx - 1.2, 143.5, 2.4, 3.5), SAND);
            }
            (None, 6) => {
                // northern lights rippling over a dark hill
                for k in 0..3 {
                    let pts: Vec<(f32, f32)> = (0..=26).map(|i| {
                        let x = 199.0 + i as f32 * 3.0;
                        (x, 140.0 + k as f32 * 5.0 + 5.0 * (i as f32 * 0.3 + t * 0.8 + k as f32 * 1.3).sin())
                    }).collect();
                    c.stroke(&polyline(&pts), [GREEN, CODE[3], CODE[1]][k].fade(0.35), 6.0);
                }
                c.fill(&poly(&[(199.0, 181.0), (199.0, 168.0), (236.0, 162.0), (277.0, 170.0), (277.0, 181.0)]), Rgba::rgb(24, 30, 44));
            }
            (None, _) => {
                // a snow globe, the snow slowly settling
                c.fill(&circle(cx, 147.0, 20.0), Rgba::rgb(120, 160, 210).fade(0.35));
                c.fill(&poly(&[(cx - 8.0, 158.0), (cx, 141.0), (cx + 8.0, 158.0)]), Rgba::rgb(80, 150, 100));
                c.fill(&rrect(cx - 1.5, 158.0, 3.0, 4.0, 0.5), Rgba::rgb(130, 90, 60));
                for i in 0..14 {
                    let ph = (t * 0.12 + (hash(i) % 100) as f32 / 100.0).fract();
                    let x = cx - 16.0 + (hash(i + 30) % 32) as f32 + 2.0 * (t + i as f32).sin();
                    let y = 130.0 + 32.0 * ph;
                    c.fill(&circle(x, y, 0.9), WHITE.fade(0.9));
                }
                c.stroke(&circle(cx, 147.0, 20.0), WHITE.fade(0.5), 1.2);
                c.fill(&rrect(cx - 18.0, 164.0, 36.0, 9.0, 2.0), Rgba::rgb(130, 90, 60));
            }
            _ => {}
        }
    }
}
