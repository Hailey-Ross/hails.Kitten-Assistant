//! Small drawing helpers on top of tiny-skia: shapes in a 300 x 250 design space,
//! scaled to the screen, plus simple text drawing with fontdue.

use tiny_skia::{
    FillRule, LineCap, LineJoin, Mask, Paint, Path, PathBuilder, Pixmap, Rect, Stroke, Transform,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Rgba(r, g, b, 255)
    }
    /// Same color with its alpha multiplied by `f` (0..1).
    pub fn fade(self, f: f32) -> Self {
        let f = f.clamp(0.0, 1.0);
        Rgba(self.0, self.1, self.2, (self.3 as f32 * f).round() as u8)
    }
}

/// Outline color and width used for the sticker look.
pub const INK: Rgba = Rgba::rgb(78, 50, 42);
pub const OUTLINE: f32 = 2.0;

pub struct Canvas<'a> {
    pub pix: &'a mut Pixmap,
    pub sc: f32,
    ts: Transform,
    clip: Option<Mask>,
    /// Pixel offset of the design origin (shapes and text alike).
    ox: f32,
    oy: f32,
}

impl<'a> Canvas<'a> {
    pub fn new(pix: &'a mut Pixmap, sc: f32) -> Self {
        Canvas { pix, sc, ts: Transform::from_scale(sc, sc), clip: None, ox: 0.0, oy: 0.0 }
    }

    /// Like `new`, with the design origin at pixel (`ox`, `oy`).
    pub fn new_at(pix: &'a mut Pixmap, sc: f32, ox: f32, oy: f32) -> Self {
        Canvas { pix, sc, ts: Transform::from_row(sc, 0.0, 0.0, sc, ox, oy), clip: None, ox, oy }
    }

    fn paint(c: Rgba) -> Paint<'static> {
        let mut p = Paint::default();
        p.set_color_rgba8(c.0, c.1, c.2, c.3);
        p.anti_alias = true;
        p
    }

    pub fn fill(&mut self, path: &Path, c: Rgba) {
        if c.3 == 0 {
            return;
        }
        self.pix.fill_path(path, &Self::paint(c), FillRule::Winding, self.ts, self.clip.as_ref());
    }

    pub fn stroke(&mut self, path: &Path, c: Rgba, width: f32) {
        if c.3 == 0 {
            return;
        }
        let s = Stroke { width, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Stroke::default() };
        self.pix.stroke_path(path, &Self::paint(c), &s, self.ts, self.clip.as_ref());
    }

    /// Fill with a soft dark outline (the sticker look every shape shares).
    pub fn blob(&mut self, path: &Path, c: Rgba) {
        self.fill(path, c);
        self.stroke(path, INK, OUTLINE);
    }

    /// Restrict drawing to `path` (None removes the clip). The mask's memory is kept for the next clip of the same
    /// size instead of a new full-size mask every time.
    pub fn clip(&mut self, path: Option<&Path>) {
        let old = self.clip.take();
        let Some(p) = path else {
            if let Some(m) = old {
                SPARE_MASK.with(|s| *s.borrow_mut() = Some(m));
            }
            return;
        };
        let (w, h) = (self.pix.width(), self.pix.height());
        let reuse = old.or_else(|| SPARE_MASK.with(|s| s.borrow_mut().take())).filter(|m| m.width() == w && m.height() == h);
        let mut m = match reuse {
            Some(mut m) => {
                m.data_mut().fill(0);
                m
            }
            None => Mask::new(w, h).expect("mask"),
        };
        m.fill_path(p, FillRule::Winding, true, self.ts);
        self.clip = Some(m);
    }

    /// Text at design coordinates (x, baseline), size in design pixels.
    pub fn text(&mut self, font: &fontdue::Font, size: f32, x: f32, baseline: f32, s: &str, c: Rgba) {
        let px = size * self.sc;
        let mut pen = x * self.sc + self.ox;
        let base = baseline * self.sc + self.oy;
        let (w, h) = (self.pix.width() as i32, self.pix.height() as i32);
        let data = self.pix.data_mut();
        for ch in s.chars() {
            let (m, cov) = glyph(font, ch, px);
            let gx = (pen + m.xmin as f32).round() as i32;
            let gy = (base - m.height as f32 - m.ymin as f32).round() as i32;
            for yy in 0..m.height {
                let py = gy + yy as i32;
                if py < 0 || py >= h {
                    continue;
                }
                for xx in 0..m.width {
                    let pxl = gx + xx as i32;
                    if pxl < 0 || pxl >= w {
                        continue;
                    }
                    let a = cov[yy * m.width + xx] as u32 * c.3 as u32 / 255;
                    if a == 0 {
                        continue;
                    }
                    let i = ((py * w + pxl) * 4) as usize;
                    let inv = 255 - a;
                    data[i] = ((c.0 as u32 * a + data[i] as u32 * inv) / 255) as u8;
                    data[i + 1] = ((c.1 as u32 * a + data[i + 1] as u32 * inv) / 255) as u8;
                    data[i + 2] = ((c.2 as u32 * a + data[i + 2] as u32 * inv) / 255) as u8;
                    data[i + 3] = (a + data[i + 3] as u32 * inv / 255) as u8;
                }
            }
            pen += m.advance_width;
        }
    }
}

thread_local! {
    /// The last clip mask's memory, for the next clip.
    static SPARE_MASK: std::cell::RefCell<Option<Mask>> = const { std::cell::RefCell::new(None) };
    /// Rasterized glyphs by font, character and pixel size: the same text is drawn every frame.
    static GLYPHS: std::cell::RefCell<std::collections::HashMap<(usize, char, u32), (fontdue::Metrics, std::rc::Rc<Vec<u8>>)>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// `font.rasterize`, remembered (a font is named by its file's hash: copies of one font file share glyphs).
fn glyph(font: &fontdue::Font, ch: char, px: f32) -> (fontdue::Metrics, std::rc::Rc<Vec<u8>>) {
    let key = (font.file_hash(), ch, px.to_bits());
    GLYPHS.with(|g| {
        let mut g = g.borrow_mut();
        if g.len() > 4096 {
            g.clear();
        }
        g.entry(key)
            .or_insert_with(|| {
                let (m, cov) = font.rasterize(ch, px);
                (m, std::rc::Rc::new(cov))
            })
            .clone()
    })
}

/// Width of `s` in design pixels.
pub fn measure(font: &fontdue::Font, size: f32, s: &str) -> f32 {
    s.chars().map(|ch| font.metrics(ch, size).advance_width).sum()
}

/// Greedy word wrap into at most `max_lines`; the last line gets an ellipsis if cut.
pub fn wrap(font: &fontdue::Font, size: f32, text: &str, max_w: f32, max_lines: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cut = false;
    for word in text.split_whitespace() {
        let trial = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if measure(font, size, &trial) <= max_w {
            cur = trial;
            continue;
        }
        if !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
            if lines.len() == max_lines {
                cut = true;
                break;
            }
        }
        // a single word wider than the bubble is broken by characters
        for ch in word.chars() {
            cur.push(ch);
            if measure(font, size, &cur) > max_w {
                cur.pop();
                lines.push(std::mem::take(&mut cur));
                if lines.len() == max_lines {
                    cut = true;
                    break;
                }
                cur.push(ch);
            }
        }
        if cut {
            break;
        }
    }
    if !cut && !cur.is_empty() {
        if lines.len() < max_lines {
            lines.push(cur);
        } else {
            cut = true;
        }
    }
    if cut {
        if let Some(last) = lines.last_mut() {
            while !last.is_empty() && measure(font, size, &format!("{last}\u{2026}")) > max_w {
                last.pop();
            }
            let trimmed = last.trim_end().to_string();
            *last = format!("{trimmed}\u{2026}");
        }
    }
    lines
}

// ---- path builders (design coordinates) ----

pub fn oval(x: f32, y: f32, w: f32, h: f32) -> Path {
    PathBuilder::from_oval(Rect::from_xywh(x, y, w.max(0.1), h.max(0.1)).unwrap()).unwrap()
}

pub fn circle(x: f32, y: f32, r: f32) -> Path {
    PathBuilder::from_circle(x, y, r.max(0.1)).unwrap()
}

pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.min(w / 2.0).min(h / 2.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish().unwrap()
}

pub fn poly(pts: &[(f32, f32)]) -> Path {
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        pb.line_to(p.0, p.1);
    }
    pb.close();
    pb.finish().unwrap()
}

pub fn line(x1: f32, y1: f32, x2: f32, y2: f32) -> Path {
    let mut pb = PathBuilder::new();
    pb.move_to(x1, y1);
    pb.line_to(x2, y2);
    pb.finish().unwrap()
}

pub fn quad(x1: f32, y1: f32, cx: f32, cy: f32, x2: f32, y2: f32) -> Path {
    let mut pb = PathBuilder::new();
    pb.move_to(x1, y1);
    pb.quad_to(cx, cy, x2, y2);
    pb.finish().unwrap()
}

pub fn polyline(pts: &[(f32, f32)]) -> Path {
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        pb.line_to(p.0, p.1);
    }
    pb.finish().unwrap()
}

/// Point on a cubic bezier.
pub fn bezier(p: [(f32, f32); 4], u: f32) -> (f32, f32) {
    let v = 1.0 - u;
    let a = v * v * v;
    let b = 3.0 * v * v * u;
    let c = 3.0 * v * u * u;
    let d = u * u * u;
    (
        a * p[0].0 + b * p[1].0 + c * p[2].0 + d * p[3].0,
        a * p[0].1 + b * p[1].1 + c * p[2].1 + d * p[3].1,
    )
}
