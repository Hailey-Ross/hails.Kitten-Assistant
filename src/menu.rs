//! The right-click menu, drawn like the kitten: a cream card with little ears that pops open, a highlight that
//! glides from row to row, pages that slide in (Monitor, Color, Size) instead of cascading submenus, and on the
//! Color page a portrait of the kitten in every coat. Platform free: the window code feeds it the time, the mouse
//! and the keys, shows its frames and applies the action it returns.

use std::f32::consts::PI;
use tiny_skia::{FillRule, Mask, Pixmap, PixmapPaint, Transform};

use crate::canvas::*;
use crate::scene;

/// What a click on the menu asks for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Corner,
    Topmost,
    CheckUpdate,
    Quit,
    SizeAuto,
    Size(usize),
    Coat(usize),
    Monitor(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Main,
    Monitor,
    Color,
    Size,
}

#[derive(Clone, Copy, PartialEq)]
enum Icon {
    Corner,
    Monitor,
    Color,
    Size,
    Pin,
    Update,
    Quit,
    None,
}

#[derive(Clone)]
struct Row {
    label: String,
    icon: Icon,
    check: Option<bool>,
    go: Option<Page>,
    act: Option<Action>,
    /// a thin divider above this row
    gap: bool,
}

/// What the menu shows: filled in by the window code when it opens.
pub struct Info {
    /// display names and which one is the kitten's home
    pub monitors: Vec<(String, bool)>,
    /// index into `scene::COATS` of the coat worn now, and its patch roll
    pub coat: usize,
    pub seed: u32,
    /// the size rows (label, picked) after the "Default" row (label, picked)
    pub default_size: (String, bool),
    pub sizes: Vec<(String, bool)>,
    pub topmost: bool,
}

/// Room around the card for its shadow, in design pixels.
pub const MARGIN: f32 = 22.0;
const ROW: f32 = 30.0;
const PAD: f32 = 8.0;
const HEADER: f32 = 36.0;
const COLS: usize = 8;
const TILE: f32 = 34.0;
const TILE_GAP: f32 = 3.0;
const FOOTER: f32 = 24.0;
const OPEN_SECS: f32 = 0.22;
const CLOSE_SECS: f32 = 0.14;
const SLIDE_SECS: f32 = 0.26;
const PRESS_SECS: f32 = 0.14;

const CARD: Rgba = Rgba::rgb(255, 249, 240);
const PILL: Rgba = Rgba::rgb(255, 221, 230);
const PILL_DOWN: Rgba = Rgba::rgb(250, 196, 212);
const ACCENT: Rgba = Rgba::rgb(247, 160, 90);
const PINK: Rgba = Rgba::rgb(240, 150, 170);
const SOFT: Rgba = Rgba::rgb(160, 112, 84);
const DIVIDER: Rgba = Rgba::rgb(236, 222, 206);

/// The window size the menu needs (design pixels): the largest card plus the margin.
pub fn window_size() -> (f32, f32) {
    let pages = [Page::Main, Page::Monitor, Page::Color, Page::Size];
    // the size page is the tallest list; monitors are few
    let w = pages.iter().map(|&p| card_width(p)).fold(0.0, f32::max);
    let h = (HEADER + PAD * 2.0 + 10.0 * ROW).max(HEADER + grid_height() + FOOTER + PAD * 2.0);
    (w + MARGIN * 2.0, h + MARGIN * 2.0)
}

fn card_width(p: Page) -> f32 {
    match p {
        Page::Main => 196.0,
        Page::Monitor => 236.0,
        Page::Size => 236.0,
        Page::Color => PAD * 2.0 + COLS as f32 * TILE + (COLS - 1) as f32 * TILE_GAP,
    }
}

fn grid_height() -> f32 {
    let rows = scene::COATS.len().div_ceil(COLS);
    rows as f32 * TILE + (rows - 1) as f32 * TILE_GAP
}

fn ease_out(k: f32) -> f32 {
    1.0 - (1.0 - k.clamp(0.0, 1.0)).powi(3)
}

fn ease_out_back(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0) - 1.0;
    1.0 + 2.7 * k * k * k + 1.7 * k * k
}

pub struct Menu {
    info: Info,
    page: Page,
    /// the page sliding out and whether the move is back (to the left)
    from: Option<(Page, bool)>,
    slide_at: f32,
    opened_at: f32,
    closing_at: Option<f32>,
    /// hovered row (or tile on the Color page); `None` = nothing
    hover: Option<usize>,
    pill_y: f32,
    pill_a: f32,
    press: Option<(usize, f32)>,
    /// how far each color tile is lifted (0..1)
    lift: Vec<f32>,
    back_hover: bool,
    last_t: f32,
    sc: f32,
    portraits: Vec<Pixmap>,
    painter: scene::Scene,
    bold: Option<fontdue::Font>,
    regular: Option<fontdue::Font>,
    card: Pixmap,
    layer: Pixmap,
}

impl Menu {
    pub fn new(info: Info, sc: f32) -> Option<Menu> {
        let (w, h) = window_size();
        let (pw, ph) = ((w * sc).ceil() as u32, (h * sc).ceil() as u32);
        Some(Menu {
            info,
            page: Page::Main,
            from: None,
            slide_at: -1.0,
            opened_at: 0.0,
            closing_at: None,
            hover: None,
            pill_y: 0.0,
            pill_a: 0.0,
            press: None,
            lift: vec![0.0; scene::COATS.len()],
            back_hover: false,
            last_t: 0.0,
            sc,
            portraits: Vec::new(),
            painter: scene::Scene::new(),
            bold: scene::load_font(&["seguisb.ttf", "segoeuib.ttf", "arialbd.ttf"]),
            regular: scene::load_font(&["segoeui.ttf", "arial.ttf"]),
            card: Pixmap::new(pw, ph)?,
            layer: Pixmap::new(pw, ph)?,
        })
    }

    /// The menu finished closing: the window can go.
    pub fn finished(&self, t: f32) -> bool {
        self.closing_at.is_some_and(|c| t - c >= CLOSE_SECS)
    }

    pub fn close(&mut self, t: f32) {
        if self.closing_at.is_none() {
            self.closing_at = Some(t);
        }
    }

    fn rows(&self, page: Page) -> Vec<Row> {
        let row = |label: &str, icon: Icon| Row { label: label.into(), icon, check: None, go: None, act: None, gap: false };
        match page {
            Page::Main => vec![
                Row { act: Some(Action::Corner), ..row("Back to corner", Icon::Corner) },
                Row { go: Some(Page::Monitor), ..row("Monitor", Icon::Monitor) },
                Row { go: Some(Page::Color), ..row("Color", Icon::Color) },
                Row { go: Some(Page::Size), ..row("Size", Icon::Size) },
                Row { act: Some(Action::Topmost), check: Some(self.info.topmost), ..row("Always on top", Icon::Pin) },
                Row { act: Some(Action::CheckUpdate), gap: true, ..row("Check for Update", Icon::Update) },
                Row { act: Some(Action::Quit), ..row("Quit", Icon::Quit) },
            ],
            Page::Monitor => self.info.monitors.iter().enumerate()
                .map(|(i, (name, home))| Row { act: Some(Action::Monitor(i)), check: Some(*home), ..row(name, Icon::None) })
                .collect(),
            Page::Size => {
                let mut v = vec![Row { act: Some(Action::SizeAuto), check: Some(self.info.default_size.1), ..row(&self.info.default_size.0, Icon::None) }];
                v.extend(self.info.sizes.iter().enumerate().map(|(i, (label, on))| Row {
                    act: Some(Action::Size(i)),
                    check: Some(*on),
                    gap: i == 0,
                    ..row(label, Icon::None)
                }));
                v
            }
            Page::Color => Vec::new(),
        }
    }

    fn title(page: Page) -> &'static str {
        match page {
            Page::Main => "",
            Page::Monitor => "Monitor",
            Page::Color => "Color",
            Page::Size => "Size",
        }
    }

    fn top(page: Page) -> f32 {
        if page == Page::Main { PAD } else { HEADER }
    }

    /// Row tops and heights (card coordinates), with the dividers' extra room.
    fn row_spans(rows: &[Row], page: Page) -> Vec<(f32, f32)> {
        let mut y = Self::top(page);
        rows.iter().map(|r| {
            if r.gap {
                y += 7.0;
            }
            let s = (y, ROW);
            y += ROW;
            s
        }).collect()
    }

    fn card_size(&self, page: Page) -> (f32, f32) {
        let h = match page {
            Page::Color => HEADER + grid_height() + FOOTER + PAD,
            p => {
                let rows = self.rows(p);
                Self::row_spans(&rows, p).last().map_or(Self::top(p), |&(y, h)| y + h) + PAD
            }
        };
        (card_width(page), h)
    }

    fn tile_at(i: usize) -> (f32, f32) {
        let (r, c) = (i / COLS, i % COLS);
        (PAD + c as f32 * (TILE + TILE_GAP), HEADER + r as f32 * (TILE + TILE_GAP))
    }

    fn count(&self) -> usize {
        if self.page == Page::Color { scene::COATS.len() } else { self.rows(self.page).len() }
    }

    /// What is under the point (design pixels from the window's top-left).
    fn hit(&self, x: f32, y: f32) -> (Option<usize>, bool) {
        let (x, y) = (x - MARGIN, y - MARGIN);
        let (w, h) = self.card_size(self.page);
        if x < 0.0 || y < 0.0 || x > w || y > h {
            return (None, false);
        }
        if self.page != Page::Main && y < HEADER - 4.0 && x < 40.0 {
            return (None, true);
        }
        if self.page == Page::Color {
            let hit = (0..scene::COATS.len()).find(|&i| {
                let (tx, ty) = Self::tile_at(i);
                x >= tx && x < tx + TILE && y >= ty && y < ty + TILE
            });
            return (hit, false);
        }
        let rows = self.rows(self.page);
        (Self::row_spans(&rows, self.page).iter().position(|&(ry, rh)| y >= ry && y < ry + rh), false)
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        if self.from.is_some() {
            return;
        }
        let (hit, back) = self.hit(x, y);
        self.hover = hit;
        self.back_hover = back;
    }

    pub fn mouse_leave(&mut self) {
        self.hover = None;
        self.back_hover = false;
    }

    /// A click; returns the action to apply now (the menu then closes itself).
    pub fn click(&mut self, x: f32, y: f32, t: f32) -> Option<Action> {
        if self.closing_at.is_some() {
            return None;
        }
        let (hit, back) = self.hit(x, y);
        if back {
            self.go(Page::Main, true, t);
            return None;
        }
        self.activate(hit?, t)
    }

    fn activate(&mut self, i: usize, t: f32) -> Option<Action> {
        if self.page == Page::Color {
            self.press = Some((i, t));
            self.closing_at = Some(t + PRESS_SECS);
            return Some(Action::Coat(i));
        }
        let row = self.rows(self.page).get(i)?.clone();
        if let Some(p) = row.go {
            self.press = Some((i, t));
            self.go(p, false, t);
            return None;
        }
        let act = row.act?;
        self.press = Some((i, t));
        self.closing_at = Some(t + PRESS_SECS);
        Some(act)
    }

    fn go(&mut self, page: Page, back: bool, t: f32) {
        if page == self.page {
            return;
        }
        if page == Page::Color && self.portraits.is_empty() {
            self.paint_portraits();
        }
        self.from = Some((self.page, back));
        self.page = page;
        self.press = None;
        self.slide_at = t;
        self.hover = None;
        self.pill_a = 0.0;
    }

    /// Keyboard: arrows move, Enter / Space / Right pick or open, Left / Backspace go back, Escape goes back or closes.
    pub fn key(&mut self, key: Key, t: f32) -> Option<Action> {
        let n = self.count();
        let grid = self.page == Page::Color;
        let cur = self.hover;
        let step = |d: isize| -> Option<usize> {
            let start = cur.map_or(if d > 0 { -1 } else { n as isize }, |c| c as isize);
            let next = start + d;
            (0..n as isize).contains(&next).then_some(next as usize).or(cur)
        };
        match key {
            Key::Down => self.hover = step(if grid { COLS as isize } else { 1 }).or(Some(0)),
            Key::Up => self.hover = step(if grid { -(COLS as isize) } else { -1 }).or(Some(0)),
            Key::Right if grid => self.hover = step(1).or(Some(0)),
            Key::Left if grid => self.hover = step(-1).or(Some(0)),
            Key::Enter | Key::Right => return self.hover.and_then(|i| self.activate(i, t)),
            Key::Left | Key::Back => self.go(Page::Main, true, t),
            Key::Escape => {
                if self.page == Page::Main {
                    self.close(t);
                } else {
                    self.go(Page::Main, true, t);
                }
            }
        }
        None
    }

    fn paint_portraits(&mut self) {
        let px = (TILE * 1.2 * self.sc).ceil() as u32;
        for (i, coat) in scene::COATS.iter().enumerate() {
            let Some(mut pix) = Pixmap::new(px, px) else { continue };
            let seed = if i == self.info.coat { self.info.seed } else { 1 };
            // the head is about 110 design pixels across with its ears and whiskers
            let s = px as f32 / 118.0;
            self.painter.portrait(*coat, seed, &mut pix, s, px as f32 / 2.0, px as f32 / 2.0 + 4.0 * s);
            self.portraits.push(pix);
        }
    }

    /// One frame at time `t` (seconds since the menu opened) into `out` (the window's pixmap).
    pub fn render(&mut self, out: &mut Pixmap, t: f32) {
        let dt = (t - self.last_t).clamp(0.0, 0.1);
        self.last_t = t;
        let sc = self.sc;
        // the slide and the card's size between the two pages
        let slide = if self.from.is_some() { ease_out((t - self.slide_at) / SLIDE_SECS) } else { 1.0 };
        if slide >= 1.0 {
            self.from = None;
        }
        let (nw, nh) = self.card_size(self.page);
        let (cw, ch) = match self.from {
            Some((p, _)) => {
                let (ow, oh) = self.card_size(p);
                (ow + (nw - ow) * slide, oh + (nh - oh) * slide)
            }
            None => (nw, nh),
        };

        // the hover pill glides to its row
        let rows = self.rows(self.page);
        let spans = Self::row_spans(&rows, self.page);
        if let Some(&(y, _)) = self.hover.and_then(|i| spans.get(i)) {
            if self.pill_a <= 0.01 {
                self.pill_y = y;
            }
            self.pill_y += (y - self.pill_y) * (1.0 - (-dt * 22.0).exp());
            self.pill_a = (self.pill_a + dt * 8.0).min(1.0);
        } else {
            self.pill_a = (self.pill_a - dt * 8.0).max(0.0);
        }
        for (i, l) in self.lift.iter_mut().enumerate() {
            let goal = if self.page == Page::Color && self.hover == Some(i) { 1.0 } else { 0.0 };
            *l += (goal - *l) * (1.0 - (-dt * 18.0).exp());
        }

        // the card: shadow, ears, body
        self.card.fill(tiny_skia::Color::TRANSPARENT);
        {
            let mut c = Canvas::new_at(&mut self.card, sc, MARGIN * sc, MARGIN * sc);
            for k in 1..=5 {
                let g = k as f32 * 1.6;
                c.fill(&rrect(-g + 1.0, -g + 4.0, cw + g * 2.0 - 2.0, ch + g * 2.0 - 2.0, 14.0 + g), INK.fade(0.035));
            }
            for x in [16.0, 54.0] {
                c.blob(&poly(&[(x, 7.0), (x + 9.0, -14.0), (x + 20.0, 5.0)]), CARD);
                c.fill(&poly(&[(x + 5.0, 4.0), (x + 9.5, -7.5), (x + 15.0, 3.0)]), PINK.fade(0.75));
            }
            c.blob(&rrect(0.0, 0.0, cw, ch, 14.0), CARD);
        }

        // the pages, clipped to the card: the old one slides out, the new one in
        let mut mask = Mask::new(self.card.width(), self.card.height()).expect("mask");
        mask.fill_path(&rrect(1.0, 1.0, cw - 2.0, ch - 2.0, 13.0), FillRule::Winding, true, Transform::from_row(sc, 0.0, 0.0, sc, MARGIN * sc, MARGIN * sc));
        if let Some((old, back)) = self.from {
            let dir = if back { 1.0 } else { -1.0 };
            self.draw_page(old, t, false);
            let off = dir * slide * cw * 0.5 * sc;
            let paint = PixmapPaint { opacity: 1.0 - slide, ..PixmapPaint::default() };
            self.card.draw_pixmap(0, 0, self.layer.as_ref(), &paint, Transform::from_translate(off, 0.0), Some(&mask));
            self.draw_page(self.page, t, true);
            let off = -dir * (1.0 - slide) * cw * 0.5 * sc;
            let paint = PixmapPaint { opacity: slide, ..PixmapPaint::default() };
            self.card.draw_pixmap(0, 0, self.layer.as_ref(), &paint, Transform::from_translate(off, 0.0), Some(&mask));
        } else {
            self.draw_page(self.page, t, true);
            self.card.draw_pixmap(0, 0, self.layer.as_ref(), &PixmapPaint::default(), Transform::identity(), Some(&mask));
        }

        // pop open (a little overshoot), fade out on close
        let open = (t - self.opened_at) / OPEN_SECS;
        let close = self.closing_at.map_or(0.0, |c| ((t - c) / CLOSE_SECS).clamp(0.0, 1.0));
        let scale = (0.86 + 0.14 * ease_out_back(open)) * (1.0 - 0.06 * close);
        let alpha = ease_out(open * 1.4) * (1.0 - close);
        out.fill(tiny_skia::Color::TRANSPARENT);
        let (ox, oy) = (MARGIN * sc, MARGIN * sc);
        let ts = Transform::from_row(scale, 0.0, 0.0, scale, ox - ox * scale, oy - oy * scale);
        let paint = PixmapPaint { opacity: alpha, quality: tiny_skia::FilterQuality::Bilinear, ..PixmapPaint::default() };
        out.draw_pixmap(0, 0, self.card.as_ref(), &paint, ts, None);
    }

    /// A page's contents (card coordinates) into the layer.
    fn draw_page(&mut self, page: Page, t: f32, live: bool) {
        self.layer.fill(tiny_skia::Color::TRANSPARENT);
        let sc = self.sc;
        let rows = self.rows(page);
        let spans = Self::row_spans(&rows, page);
        let (cw, ch) = self.card_size(page);
        let press = |i: usize| self.press.filter(|_| live).filter(|&(p, _)| p == i).map(|(_, at)| ((t - at) / 0.2).clamp(0.0, 1.0));
        let mut c = Canvas::new_at(&mut self.layer, sc, MARGIN * sc, MARGIN * sc);

        if page != Page::Main {
            // header: a round back button, the page's title, a divider
            let hov = live && self.back_hover;
            c.fill(&circle(20.0, 18.0, 11.0), if hov { PILL } else { DIVIDER.fade(0.6) });
            c.stroke(&polyline(&[(22.5, 12.5), (17.0, 18.0), (22.5, 23.5)]), INK, 2.2);
            if let Some(font) = self.bold.as_ref() {
                c.text(font, 13.0, 40.0, 23.0, Self::title(page), INK);
            }
            for i in 0..3 {
                let x = cw - 34.0 + i as f32 * 8.0;
                c.fill(&circle(x, 18.0 - if i == 1 { 2.0 } else { 0.0 }, 1.8), PINK.fade(0.6));
            }
            c.stroke(&line(12.0, HEADER - 3.0, cw - 12.0, HEADER - 3.0), DIVIDER, 1.4);
        }

        if page == Page::Color {
            // portraits; the hovered one lifts, the worn one has a ring
            let order: Vec<usize> = {
                let mut v: Vec<usize> = (0..scene::COATS.len()).collect();
                v.sort_by(|a, b| self.lift[*a].partial_cmp(&self.lift[*b]).unwrap_or(std::cmp::Ordering::Equal));
                v
            };
            for i in order {
                let (tx, ty) = Self::tile_at(i);
                let lift = if live { self.lift[i] } else { 0.0 };
                let pressed = press(i).map_or(0.0, |k| (k * PI).sin());
                let grow = 1.0 + 0.16 * lift - 0.08 * pressed;
                let (cx, cy) = (tx + TILE / 2.0, ty + TILE / 2.0 - 2.5 * lift);
                let half = TILE / 2.0 * grow;
                if lift > 0.01 {
                    c.fill(&rrect(cx - half + 1.0, cy - half + 3.0, half * 2.0, half * 2.0, 9.0), INK.fade(0.12 * lift));
                }
                let worn = i == self.info.coat;
                c.fill(&rrect(cx - half, cy - half, half * 2.0, half * 2.0, 9.0), if lift > 0.01 { PILL } else { Rgba::rgb(252, 240, 228) });
                if worn {
                    c.stroke(&rrect(cx - half, cy - half, half * 2.0, half * 2.0, 9.0), ACCENT, 2.4);
                }
                drop(c);
                if let Some(p) = self.portraits.get(i) {
                    let s = grow * TILE / (p.width() as f32 / sc) * 1.05;
                    let (pw, ph) = (p.width() as f32 * s, p.height() as f32 * s);
                    let ts = Transform::from_row(s, 0.0, 0.0, s, (MARGIN + cx) * sc - pw / 2.0, (MARGIN + cy) * sc - ph / 2.0);
                    let paint = PixmapPaint { quality: tiny_skia::FilterQuality::Bilinear, ..PixmapPaint::default() };
                    self.layer.draw_pixmap(0, 0, p.as_ref(), &paint, ts, None);
                }
                c = Canvas::new_at(&mut self.layer, sc, MARGIN * sc, MARGIN * sc);
            }
            // the hovered coat's name (or the worn one's) under the grid
            let shown = if live { self.hover } else { None }.unwrap_or(self.info.coat);
            if let Some(font) = self.bold.as_ref() {
                let name = scene::COATS[shown].name;
                let w = measure(font, 12.0, name);
                c.text(font, 12.0, (cw - w) / 2.0, ch - PAD - 7.0, name, if Some(shown) == self.hover { INK } else { SOFT });
            }
            return;
        }

        if live && self.pill_a > 0.0 {
            let down = self.hover.and_then(&press).is_some();
            c.fill(&rrect(6.0, self.pill_y + 2.0, cw - 12.0, ROW - 4.0, 11.0), (if down { PILL_DOWN } else { PILL }).fade(self.pill_a));
        }
        for (i, (row, &(y, h))) in rows.iter().zip(&spans).enumerate() {
            if row.gap {
                c.stroke(&line(14.0, y - 4.0, cw - 14.0, y - 4.0), DIVIDER, 1.4);
            }
            let squish = press(i).map_or(0.0, |k| (k * PI).sin());
            let mid = y + h / 2.0;
            let x0 = if row.icon == Icon::None { 16.0 } else { 38.0 } + 2.0 * squish;
            icon(&mut c, row.icon, 22.0, mid, live && self.hover == Some(i), t);
            if let Some(font) = self.regular.as_ref() {
                c.text(font, 12.5, x0, mid + 4.5, &row.label, if row.act == Some(Action::Quit) { Rgba::rgb(196, 80, 80) } else { INK });
            }
            if row.go.is_some() {
                let nudge = if live && self.hover == Some(i) { 2.0 } else { 0.0 };
                let x = cw - 20.0 + nudge;
                c.stroke(&polyline(&[(x - 3.0, mid - 4.5), (x + 1.5, mid), (x - 3.0, mid + 4.5)]), SOFT, 2.0);
            }
            if let Some(on) = row.check {
                let x = cw - 22.0;
                if on {
                    c.fill(&circle(x, mid, 7.5), ACCENT);
                    c.stroke(&polyline(&[(x - 3.5, mid + 0.2), (x - 1.0, mid + 2.8), (x + 3.8, mid - 2.8)]), Rgba::rgb(255, 255, 255), 2.0);
                } else {
                    c.stroke(&circle(x, mid, 7.0), DIVIDER, 1.8);
                }
            }
        }
    }
}

/// Keys the menu understands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    Escape,
}

/// The small drawn icon of a main-page row; it wiggles a little while hovered.
fn icon(c: &mut Canvas, icon: Icon, x: f32, y: f32, hot: bool, t: f32) {
    let wig = if hot { 1.5 * (t * 8.0).sin() } else { 0.0 };
    let col = if hot { ACCENT } else { SOFT };
    match icon {
        Icon::None => {}
        Icon::Corner => {
            c.stroke(&rrect(x - 7.0, y - 6.0, 14.0, 12.0, 3.0), col, 1.6);
            c.fill(&rrect(x + 1.5 + wig * 0.5, y + 0.5, 4.0, 3.5, 1.0), col);
        }
        Icon::Monitor => {
            c.stroke(&rrect(x - 7.5, y - 6.5, 15.0, 10.5, 2.5), col, 1.6);
            c.stroke(&line(x, y + 4.0, x, y + 6.5), col, 1.6);
            c.stroke(&line(x - 3.5, y + 7.0, x + 3.5, y + 7.0), col, 1.6);
        }
        Icon::Color => {
            // a paw print
            c.fill(&oval(x - 4.0, y - 0.5 + wig * 0.3, 8.0, 6.5), col);
            for (dx, dy) in [(-5.0f32, -3.0f32), (-1.8, -5.6), (1.8, -5.6), (5.0, -3.0)] {
                c.fill(&circle(x + dx, y + dy + wig * 0.3, 1.6), col);
            }
        }
        Icon::Size => {
            c.stroke(&rrect(x - 7.0, y - 2.0, 8.0, 8.0, 2.0), col, 1.6);
            let g = 1.0 + 0.08 * wig;
            c.stroke(&rrect(x - 3.0, y - 7.0 * g, 10.0 * g, 10.0 * g, 2.5), col, 1.6);
        }
        Icon::Pin => {
            let a = wig * 0.08;
            let (s, k) = a.sin_cos();
            let p = |px: f32, py: f32| (x + px * k - py * s, y + px * s + py * k);
            c.fill(&circle(p(0.0, -3.0).0, p(0.0, -3.0).1, 3.8), col);
            let (a0, a1) = (p(0.0, 0.0), p(0.0, 7.0));
            c.stroke(&line(a0.0, a0.1, a1.0, a1.1), col, 1.6);
        }
        Icon::Update => {
            // a circular arrow, turning a little while hovered
            let a0 = wig * 0.25;
            let pts: Vec<(f32, f32)> = (0..=20).map(|i| {
                let a = a0 + 0.5 + i as f32 / 20.0 * 4.9;
                (x + 5.5 * a.cos(), y + 5.5 * a.sin())
            }).collect();
            c.stroke(&polyline(&pts), col, 1.6);
            let a = a0 + 5.4;
            let (tx, ty) = (x + 5.5 * a.cos(), y + 5.5 * a.sin());
            let (dx, dy) = (-a.sin(), a.cos());
            let (nx, ny) = (a.cos(), a.sin());
            c.fill(&poly(&[(tx + 3.2 * dx, ty + 3.2 * dy), (tx + 2.6 * nx, ty + 2.6 * ny), (tx - 2.6 * nx, ty - 2.6 * ny)]), col);
        }
        Icon::Quit => {
            c.stroke(&line(x - 4.5 - wig * 0.2, y - 4.5, x + 4.5 + wig * 0.2, y + 4.5), Rgba::rgb(196, 80, 80), 1.9);
            c.stroke(&line(x + 4.5 + wig * 0.2, y - 4.5, x - 4.5 - wig * 0.2, y + 4.5), Rgba::rgb(196, 80, 80), 1.9);
        }
    }
}
