//! Kitten Assistant: a tiny desktop buddy for Windows.
//!
//! A kitten sits at a little computer on top of the taskbar. Point it at a status
//! file: while the file's first line has text, the kitten types and a speech bubble
//! shows that text and how long it has been going; when the file is empty, missing
//! or says "idle", the kitten naps.
#![windows_subsystem = "windows"]

mod canvas;
mod menu;
mod scene;
mod update;

use std::path::{Path, PathBuf};
use std::time::Instant;

use tiny_skia::Pixmap;

const USAGE: &str = "\
kitten-assistant                      run the kitten (watches the status file)
kitten-assistant say \"some text\"      kitten types, bubble shows the text
kitten-assistant idle                 kitten naps
kitten-assistant quit                 close a running kitten
kitten-assistant snapshot out.png     render one frame to a PNG
kitten-assistant update               check for a newer stable release now and install it
kitten-assistant version              show the version

options:
  --status FILE    status file (default: status.txt next to the exe, or %KITTEN_STATUS%)
  --scale N        size multiplier (default: right-click > Size, else the screen DPI)
  --monitor N      start on display N and make it the default (right-click > Monitor does the same)
  --x N --y N      window position (default: bottom-right of the default display)
snapshot options:
  --busy TEXT      render the busy pose with TEXT (default: napping)
  --time SECONDS   animation time to render (default 3.2)
  --minutes N      elapsed minutes shown in the bubble
  --bg RRGGBB      background color (default: transparent)
  --done           render the all-done moment
  --screen N       typing screen 0-4 (code, progress, equalizer, gear, graph; default 0)
  --sip            render the middle of a coffee sip
  --coat NAME      the color (as in right-click > Color, e.g. \"Gray tabby\"; default: Orange tabby)
  --seed N         where the patches sit on patched coats (default 1)";

#[derive(Default)]
struct Opts {
    cmd: String,
    arg: Option<String>,
    status: Option<PathBuf>,
    scale: Option<f32>,
    x: Option<i32>,
    y: Option<i32>,
    monitor: Option<u32>,
    busy: Option<String>,
    time: Option<f64>,
    minutes: Option<f64>,
    bg: Option<String>,
    done: bool,
    screen: Option<usize>,
    sip: bool,
    egg: bool,
    menu: Option<String>,
    hover: Option<usize>,
    refill: Option<f64>,
    coat: Option<String>,
    seed: Option<u32>,
}

fn parse() -> Result<Opts, String> {
    let mut o = Opts { cmd: "run".into(), ..Default::default() };
    let mut it = std::env::args().skip(1);
    let mut positional = Vec::new();
    while let Some(a) = it.next() {
        let mut val = || it.next().ok_or_else(|| format!("{a} needs a value"));
        match a.as_str() {
            "--status" => o.status = Some(PathBuf::from(val()?)),
            "--scale" => o.scale = Some(val()?.parse().map_err(|_| "bad --scale")?),
            "--x" => o.x = Some(val()?.parse().map_err(|_| "bad --x")?),
            "--y" => o.y = Some(val()?.parse().map_err(|_| "bad --y")?),
            "--monitor" => o.monitor = Some(val()?.parse().map_err(|_| "bad --monitor")?),
            "--busy" => o.busy = Some(val()?),
            "--time" => o.time = Some(val()?.parse().map_err(|_| "bad --time")?),
            "--minutes" => o.minutes = Some(val()?.parse().map_err(|_| "bad --minutes")?),
            "--bg" => o.bg = Some(val()?),
            "--done" => o.done = true,
            "--sip" => o.sip = true,
            "--egg" => o.egg = true,
            "--menu" => o.menu = Some(val()?),
            "--refill" => o.refill = Some(val()?.parse().map_err(|_| "bad --refill")?),
            "--hover" => o.hover = Some(val()?.parse().map_err(|_| "bad --hover")?),
            "--coat" => o.coat = Some(val()?),
            "--seed" => o.seed = Some(val()?.parse().map_err(|_| "bad --seed")?),
            "--screen" => o.screen = Some(val()?.parse().map_err(|_| "bad --screen")?),
            "-h" | "--help" | "help" => return Err(USAGE.into()),
            _ if a.starts_with("--") => return Err(format!("unknown option {a}\n\n{USAGE}")),
            _ => positional.push(a),
        }
    }
    let mut p = positional.into_iter();
    if let Some(c) = p.next() {
        o.cmd = c;
    }
    o.arg = p.collect::<Vec<_>>().join(" ").into();
    if o.arg.as_deref() == Some("") {
        o.arg = None;
    }
    Ok(o)
}

fn status_path(o: &Opts) -> PathBuf {
    if let Some(p) = &o.status {
        return p.clone();
    }
    if let Ok(p) = std::env::var("KITTEN_STATUS") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join("status.txt")))
        .unwrap_or_else(|| PathBuf::from("status.txt"))
}

fn read_status(p: &Path) -> String {
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| s.trim_start_matches('\u{feff}').lines().next().map(|l| l.trim().to_string()))
        .unwrap_or_default()
}

/// Seconds since the status file was last written; 0 if unknown.
fn status_age(p: &Path) -> f64 {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|m| m.elapsed().ok())
        .map_or(0.0, |d| d.as_secs_f64())
}

fn log_path() -> PathBuf {
    std::env::temp_dir().join("kitten-assistant.log")
}

fn log(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log_path()) {
        let _ = writeln!(f, "{msg}");
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| log(&format!("panic: {info}"))));
    let o = match parse() {
        Ok(o) => o,
        Err(e) => {
            win::message(&e);
            return;
        }
    };
    let status = status_path(&o);
    let result = match o.cmd.as_str() {
        "run" => win::run(&o, status),
        "say" => std::fs::write(&status, o.arg.clone().unwrap_or_default()).map_err(|e| e.to_string()),
        "idle" => std::fs::write(&status, "idle").map_err(|e| e.to_string()),
        "quit" => {
            win::quit();
            Ok(())
        }
        "snapshot" => snapshot(&o),
        "update" => win::update_now(),
        "version" => {
            win::message(&format!("Kitten Assistant {}", update::label()));
            Ok(())
        }
        // A status line passed without `say` (`kitten-assistant "Building"`) is taken as one: no message box, which
        // would block a script waiting for the call to return.
        other => {
            let text = match &o.arg {
                Some(a) => format!("{other} {a}"),
                None => other.to_string(),
            };
            log(&format!("no command `{other}`: taken as say \"{text}\""));
            std::fs::write(&status, text).map_err(|e| e.to_string())
        }
    };
    if let Err(e) = result {
        log(&e);
        win::message(&e);
    }
}

/// The right-click menu on page (main, monitor, color, size) with row or tile --hover lit, to a PNG (for checking
/// its look; not in the usage text).
fn menu_snapshot(o: &Opts, page: &str, out: &str, sc: f32) -> Result<(), String> {
    let info = menu::Info {
        monitors: vec![("Display 1 (primary)".into(), true), ("Display 2".into(), false)],
        coat: scene::COATS.iter().position(|c| Some(c.name) == o.coat.as_deref()).unwrap_or(0),
        seed: o.seed.unwrap_or(1),
        default_size: ("Default (125%)".into(), true),
        sizes: [50, 75, 100, 125, 150, 175, 200, 250, 300].iter().map(|p| (format!("{p}%"), false)).collect(),
        topmost: true,
    };
    let mut m = menu::Menu::new(info, sc).ok_or("menu")?;
    let opens = match page {
        "monitor" => Some(1),
        "color" => Some(2),
        "size" => Some(3),
        _ => None,
    };
    if let Some(row) = opens {
        for _ in 0..=row {
            m.key(menu::Key::Down, 0.3);
        }
        m.key(menu::Key::Enter, 0.3);
    }
    let grid = page == "color";
    for _ in 0..=o.hover.unwrap_or(usize::MAX - 1).min(64) {
        if o.hover.is_none() {
            break;
        }
        m.key(if grid { menu::Key::Right } else { menu::Key::Down }, 0.6);
    }
    let (dw, dh) = menu::window_size();
    let mut pix = Pixmap::new((dw * sc).ceil() as u32, (dh * sc).ceil() as u32).ok_or("pixmap")?;
    let end = o.time.unwrap_or(1.6) as f32;
    let mut t = 0.0;
    while t < end {
        m.render(&mut pix, t);
        t += 1.0 / 60.0;
    }
    m.render(&mut pix, end);
    let mut bg = Pixmap::new(pix.width(), pix.height()).ok_or("pixmap")?;
    bg.fill(tiny_skia::Color::from_rgba8(106, 140, 175, 255));
    bg.draw_pixmap(0, 0, pix.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
    bg.save_png(out).map_err(|e| e.to_string())
}

fn snapshot(o: &Opts) -> Result<(), String> {
    let out = o.arg.clone().unwrap_or_else(|| "kitten.png".into());
    let sc = o.scale.unwrap_or(2.0);
    let t_end = o.time.unwrap_or(3.2);
    if let Some(page) = o.menu.as_deref() {
        return menu_snapshot(o, page, &out, sc);
    }
    let mut s = scene::Scene::new();
    s.set_status(o.busy.as_deref().unwrap_or("idle"), 0.0);
    s.since = -o.minutes.unwrap_or(0.0) * 60.0;
    s.screen = o.screen.unwrap_or(0) % 5;
    s.egg = o.egg;
    s.coat = scene::Coat::named(o.coat.as_deref().unwrap_or(""));
    s.patch_seed = o.seed.unwrap_or(1);
    // no surprise sips in snapshots; --sip puts one in its drinking moment
    s.next_sip = if o.sip { t_end - 1.25 } else { f64::MAX };
    // --refill P: that moment (0..1) of an empty-mug refill (not in the usage text)
    if let Some(p) = o.refill {
        s.refill_start = t_end - p * 7.0;
    }
    if o.done {
        // the "all done!" moment, one second in
        s.set_status("job", 0.0);
        s.set_status("idle", (t_end - 1.0).max(0.0));
    }
    let (w, h) = ((scene::W * sc).ceil() as u32, (scene::H * sc).ceil() as u32);
    let mut pix = Pixmap::new(w, h).ok_or("pixmap")?;
    let mut t = 0.0;
    let pose = loop {
        let p = s.update(t);
        if t >= t_end {
            break p;
        }
        t = (t + 1.0 / 30.0).min(t_end);
    };
    s.render(&mut pix, sc, t_end, &pose);
    let img = match &o.bg {
        Some(hex) => {
            let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).map_err(|_| "bad --bg")?;
            let mut bg = Pixmap::new(w, h).ok_or("pixmap")?;
            bg.fill(tiny_skia::Color::from_rgba8((v >> 16) as u8, (v >> 8) as u8, v as u8, 255));
            bg.draw_pixmap(0, 0, pix.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
            bg
        }
        None => pix,
    };
    img.save_png(&out).map_err(|e| e.to_string())
}

mod win {
    use super::*;
    use std::cell::RefCell;
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::{
        CreateMutexW, GetCurrentProcess, ProcessPowerThrottling, SetPriorityClass, SetProcessInformation,
        IDLE_PRIORITY_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        PROCESS_POWER_THROTTLING_STATE,
    };
    use windows_sys::Win32::UI::HiDpi::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const CLASS: &str = "KittenAssistantWindow";
    const FRAME: usize = 1;
    const POLL: usize = 2;
    const SIZES: [f32; 9] = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0];

    fn dpi_scale() -> f32 {
        unsafe { GetDpiForSystem() as f32 / 96.0 }
    }

    /// `scale` in the ini, from right-click > Size.
    fn saved_scale() -> Option<f32> {
        setting("scale")?.parse::<f32>().ok().filter(|s| s.is_finite())
    }

    /// A 32-bit top-down DIB of w x h selected into `dc`; returns it, its pixels and what it replaced.
    unsafe fn new_dib(dc: HDC, w: i32, h: i32) -> Option<(HBITMAP, *mut u8, HGDIOBJ)> {
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = w;
        bmi.bmiHeader.biHeight = -h;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;
        let mut bits: *mut c_void = null_mut();
        let dib = CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if dib.is_null() || bits.is_null() {
            return None;
        }
        Some((dib, bits as *mut u8, SelectObject(dc, dib as HGDIOBJ)))
    }

    /// Stay above other windows (`always_on_top` in the ini; on unless it says 0).
    fn topmost() -> bool {
        setting("always_on_top").map_or(true, |v| v != "0")
    }

    unsafe fn apply_topmost(hwnd: HWND, on: bool) {
        let after = if on { HWND_TOPMOST } else { HWND_NOTOPMOST };
        SetWindowPos(hwnd, after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn message(text: &str) {
        let (t, c) = (wide(text), wide("Kitten Assistant"));
        unsafe { MessageBoxW(null_mut(), t.as_ptr(), c.as_ptr(), MB_OK) };
    }

    pub fn quit() {
        let c = wide(CLASS);
        unsafe {
            let h = FindWindowW(c.as_ptr(), null());
            if !h.is_null() {
                PostMessageW(h, WM_CLOSE, 0, 0);
            }
        }
    }

    struct App {
        hwnd: HWND,
        scene: scene::Scene,
        pix: Pixmap,
        sc: f32,
        /// The size follows the screen DPI (no --scale and nothing saved).
        auto_size: bool,
        start: Instant,
        status: PathBuf,
        mem_dc: HDC,
        dib: HBITMAP,
        old: HGDIOBJ,
        bits: *mut u8,
        frame_ms: u32,
        first_poll: bool,
        last_frame: Vec<u8>,
        last_status: String,
    }

    thread_local! {
        static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    }

    impl App {
        fn t(&self) -> f64 {
            self.start.elapsed().as_secs_f64()
        }

        fn frame(&mut self) {
            let t = self.t();
            let pose = self.scene.update(t);
            self.scene.render(&mut self.pix, self.sc, t, &pose);
            self.push();
        }

        /// New size: a new surface, and the window grows or shrinks around its bottom-right corner.
        fn resize(&mut self, sc: f32) {
            let sc = sc.clamp(0.25, 8.0);
            let (w, h) = ((scene::W * sc).ceil() as i32, (scene::H * sc).ceil() as i32);
            let Some(pix) = Pixmap::new(w as u32, h as u32) else { return };
            unsafe {
                let Some((dib, bits, prev)) = new_dib(self.mem_dc, w, h) else {
                    log("could not create the drawing surface for the new size");
                    return;
                };
                DeleteObject(prev);
                self.dib = dib;
                self.bits = bits;
                self.pix = pix;
                self.sc = sc;
                self.last_frame.clear();
                let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                GetWindowRect(self.hwnd, &mut r);
                let (x, y) = if setting("position").is_some() {
                    let (x, y) = (r.right - w, r.bottom - h);
                    save_setting("position", &format!("{x},{y}"));
                    (x, y)
                } else {
                    corner(w, h)
                };
                SetWindowPos(self.hwnd, null_mut(), x, y, w, h, SWP_NOZORDER | SWP_NOACTIVATE);
            }
            self.frame();
        }

        fn poll(&mut self) {
            let t = self.t();
            let age = if std::mem::take(&mut self.first_poll) { status_age(&self.status) } else { 0.0 };
            let status = read_status(&self.status);
            self.scene.set_status_aged(&status, t, age);
            self.last_status = status;
            // a gentle frame rate whatever has the focus: smooth enough while typing, lighter while napping
            let ms = if self.scene.busy { 33 } else { 66 };
            if ms != self.frame_ms {
                self.frame_ms = ms;
                // a little slack lets Windows fold the tick into other wake-ups
                unsafe { SetCoalescableTimer(self.hwnd, FRAME, ms, None, ms / 8) };
            }
        }

        /// Hand the frame to Windows as a per-pixel-alpha layered window, as cheaply as possible:
        /// only the rectangle that changed since the last frame is copied (premultiplied RGBA
        /// into the DIB's premultiplied BGRA) and marked dirty, and nothing is sent when the
        /// picture did not change at all.
        fn push(&mut self) {
            let (w, h) = (self.pix.width() as usize, self.pix.height() as usize);
            let src = self.pix.data();
            let full = self.last_frame.len() != src.len();
            let (mut x0, mut y0, mut x1, mut y1) = if full { (0, 0, w, h) } else { (w, 0, 0, 0) };
            if !full {
                let mut any = false;
                for y in 0..h {
                    let row = y * w * 4..(y + 1) * w * 4;
                    let (a, b) = (&src[row.clone()], &self.last_frame[row]);
                    if a == b {
                        continue;
                    }
                    let differs = |i: usize| a[i * 4..i * 4 + 4] != b[i * 4..i * 4 + 4];
                    let first = (0..w).find(|&i| differs(i)).unwrap_or(0);
                    let last = (0..w).rev().find(|&i| differs(i)).unwrap_or(w - 1);
                    if !any {
                        y0 = y;
                        any = true;
                    }
                    x0 = x0.min(first);
                    x1 = x1.max(last + 1);
                    y1 = y + 1;
                }
                if !any {
                    return;
                }
            } else {
                self.last_frame = vec![0; src.len()];
            }
            let dst = unsafe { std::slice::from_raw_parts_mut(self.bits, src.len()) };
            for y in y0..y1 {
                let span = (y * w + x0) * 4..(y * w + x1) * 4;
                self.last_frame[span.clone()].copy_from_slice(&src[span.clone()]);
                for (d, s) in dst[span.clone()].chunks_exact_mut(4).zip(src[span].chunks_exact(4)) {
                    d[0] = s[2];
                    d[1] = s[1];
                    d[2] = s[0];
                    d[3] = s[3];
                }
            }
            let size = SIZE { cx: w as i32, cy: h as i32 };
            let origin = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let dirty = RECT { left: x0 as i32, top: y0 as i32, right: x1 as i32, bottom: y1 as i32 };
            let info = UPDATELAYEREDWINDOWINFO {
                cbSize: std::mem::size_of::<UPDATELAYEREDWINDOWINFO>() as u32,
                hdcDst: null_mut(),
                pptDst: null(),
                psize: &size,
                hdcSrc: self.mem_dc,
                pptSrc: &origin,
                crKey: 0,
                pblend: &blend,
                dwFlags: ULW_ALPHA,
                prcDirty: &dirty,
            };
            unsafe { UpdateLayeredWindowIndirect(self.hwnd, &info) };
        }
    }

    impl Drop for App {
        fn drop(&mut self) {
            unsafe {
                SelectObject(self.mem_dc, self.old);
                DeleteObject(self.dib as HGDIOBJ);
                DeleteDC(self.mem_dc);
            }
        }
    }

    struct Monitor {
        device: String,
        label: String,
        work: RECT,
        primary: bool,
    }

    fn from_wide(s: &[u16]) -> String {
        String::from_utf16_lossy(&s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())])
    }

    /// `\\.\DISPLAY2` -> 2, the number Windows shows in Display settings.
    fn display_number(device: &str) -> u32 {
        device.trim_start_matches(r"\\.\DISPLAY").parse().unwrap_or(u32::MAX)
    }

    fn monitors() -> Vec<Monitor> {
        unsafe extern "system" fn each(hm: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> windows_sys::core::BOOL {
            let list = &mut *(lp as *mut Vec<Monitor>);
            let mut mi: MONITORINFOEXW = std::mem::zeroed();
            mi.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
            if GetMonitorInfoW(hm, &mut mi as *mut MONITORINFOEXW as *mut MONITORINFO) != 0 {
                let device = from_wide(&mi.szDevice);
                let mut dd: DISPLAY_DEVICEW = std::mem::zeroed();
                dd.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
                let dev = wide(&device);
                let model = if EnumDisplayDevicesW(dev.as_ptr(), 0, &mut dd, 0) != 0 { from_wide(&dd.DeviceString) } else { String::new() };
                let r = mi.monitorInfo.rcMonitor;
                let primary = mi.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0;
                let mut label = format!("Display {}", display_number(&device));
                if !model.is_empty() && !model.starts_with("Generic") {
                    label += &format!(": {model}");
                }
                label += &format!(" ({} x {}{})", r.right - r.left, r.bottom - r.top, if primary { ", main" } else { "" });
                list.push(Monitor { device, label, work: mi.monitorInfo.rcWork, primary });
            }
            1
        }
        let mut list: Vec<Monitor> = Vec::new();
        unsafe { EnumDisplayMonitors(null_mut(), null(), Some(each), &mut list as *mut Vec<Monitor> as LPARAM) };
        list.sort_by_key(|m| display_number(&m.device));
        list
    }

    /// `kitten-assistant.ini` next to the exe: `key=value` lines.
    fn settings_path() -> Option<PathBuf> {
        std::env::current_exe().ok()?.parent().map(|d| d.join("kitten-assistant.ini"))
    }

    fn setting(key: &str) -> Option<String> {
        let text = std::fs::read_to_string(settings_path()?).ok()?;
        text.lines().find_map(|l| l.trim().strip_prefix(key)?.trim_start().strip_prefix('=').map(|v| v.trim().to_string()))
    }

    fn save_setting(key: &str, value: &str) {
        write_setting(key, Some(value));
    }

    fn clear_setting(key: &str) {
        write_setting(key, None);
    }

    fn write_setting(key: &str, value: Option<&str>) {
        let Some(p) = settings_path() else { return };
        let old = std::fs::read_to_string(&p).unwrap_or_default();
        let mut lines: Vec<String> = old.lines().filter(|l| l.split('=').next().map(str::trim) != Some(key)).map(String::from).collect();
        if let Some(value) = value {
            lines.push(format!("{key}={value}"));
        }
        if let Err(e) = std::fs::write(&p, lines.join("\n") + "\n") {
            log(&format!("could not save {}: {e}", p.display()));
        }
    }

    thread_local! {
        /// The display the kitten calls home: `\\.\DISPLAYn`, empty = the main display.
        static HOME: RefCell<String> = const { RefCell::new(String::new()) };
    }

    /// Bottom-right of the home display's work area (above its taskbar), falling back to the
    /// main display when the saved one is gone.
    fn corner(w: i32, h: i32) -> (i32, i32) {
        let home = HOME.with(|h| h.borrow().clone());
        let list = monitors();
        let wa = match list.iter().find(|m| m.device == home).or_else(|| list.iter().find(|m| m.primary)) {
            Some(m) => m.work,
            None => {
                let mut wa = RECT { left: 0, top: 0, right: 1280, bottom: 720 };
                unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut wa as *mut RECT as *mut c_void, 0) };
                wa
            }
        };
        (wa.right - w - 20, wa.bottom - h)
    }

    /// Back to the default spot; a dragged-to position is forgotten.
    unsafe fn to_corner(hwnd: HWND) {
        let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetWindowRect(hwnd, &mut r);
        let (x, y) = corner(r.right - r.left, r.bottom - r.top);
        SetWindowPos(hwnd, null_mut(), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        clear_setting("position");
    }

    /// Where the user last dragged the kitten (`position=x,y`), while that spot is still on a display.
    fn saved_position(w: i32, h: i32) -> Option<(i32, i32)> {
        let v = setting("position")?;
        let (x, y) = v.split_once(',')?;
        let (x, y) = (x.trim().parse::<i32>().ok()?, y.trim().parse::<i32>().ok()?);
        let (cx, cy) = (x + w / 2, y + h / 2);
        monitors().iter().any(|m| cx >= m.work.left && cx < m.work.right && cy >= m.work.top && cy < m.work.bottom).then_some((x, y))
    }

    fn with_app(f: impl FnOnce(&mut App)) {
        APP.with(|a| {
            // try_borrow: the menu runs its own message loop, so timers can re-enter
            if let Ok(mut g) = a.try_borrow_mut() {
                if let Some(app) = g.as_mut() {
                    f(app);
                }
            }
        });
    }

    /// The open right-click menu (`menu.rs`): its own see-through window, drawing surface and state.
    struct MenuWin {
        hwnd: HWND,
        owner: HWND,
        menu: menu::Menu,
        pix: Pixmap,
        mem_dc: HDC,
        dib: HBITMAP,
        old: HGDIOBJ,
        bits: *mut u8,
        start: Instant,
        sc: f32,
    }

    impl Drop for MenuWin {
        fn drop(&mut self) {
            unsafe {
                SelectObject(self.mem_dc, self.old);
                DeleteObject(self.dib as HGDIOBJ);
                DeleteDC(self.mem_dc);
            }
        }
    }

    thread_local! {
        static MENU: RefCell<Option<MenuWin>> = const { RefCell::new(None) };
    }

    const MENU_CLASS: &str = "KittenAssistantMenu";
    const MENU_TICK: usize = 3;
    /// WM_MOUSELEAVE (TrackMouseEvent's notice), not in the modules imported here.
    const MOUSE_LEFT: u32 = 0x02A3;
    static MENU_CLASS_READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    unsafe fn show_menu(hwnd: HWND) {
        // a second right-click while one is open starts a fresh one
        close_menu_now();
        let list = monitors();
        let home = HOME.with(|h| h.borrow().clone());
        let current = list.iter().position(|d| d.device == home).or_else(|| list.iter().position(|d| d.primary));
        let (worn, seed, sc_now, auto) = APP
            .with(|a| a.try_borrow().ok().and_then(|g| g.as_ref().map(|app| (app.scene.coat.name, app.scene.patch_seed, app.sc, app.auto_size))))
            .unwrap_or((scene::COATS[0].name, 1, 1.0, true));
        let info = menu::Info {
            monitors: list.iter().enumerate().map(|(i, d)| (d.label.clone(), Some(i) == current)).collect(),
            coat: scene::COATS.iter().position(|c| c.name == worn).unwrap_or(0),
            seed,
            default_size: (format!("Default ({}%)", (dpi_scale() * 100.0).round()), auto),
            sizes: SIZES.iter().map(|s| (format!("{}%", (s * 100.0).round()), !auto && (s - sc_now).abs() < 0.005)).collect(),
            topmost: topmost(),
        };
        let sc = dpi_scale();
        let Some(m) = menu::Menu::new(info, sc) else { return };
        let (dw, dh) = menu::window_size();
        let (w, h) = ((dw * sc).ceil() as i32, (dh * sc).ceil() as i32);
        // the card's corner at the pointer, kept on the pointer's display
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST), &mut mi);
        let wa = mi.rcWork;
        let margin = (menu::MARGIN * sc) as i32;
        let x = (pt.x - margin).min(wa.right - w).max(wa.left);
        let y = (pt.y - margin).min(wa.bottom - h).max(wa.top);

        let hinst = GetModuleHandleW(null());
        let class = wide(MENU_CLASS);
        if !MENU_CLASS_READY.swap(true, std::sync::atomic::Ordering::Relaxed) {
            let wc = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(menu_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinst,
                hIcon: null_mut(),
                hCursor: LoadCursorW(null_mut(), IDC_HAND),
                hbrBackground: null_mut(),
                lpszMenuName: null(),
                lpszClassName: class.as_ptr(),
            };
            RegisterClassW(&wc);
        }
        let mh = CreateWindowExW(WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST, class.as_ptr(), null(), WS_POPUP, x, y, w, h, null_mut(), null_mut(), hinst, null());
        if mh.is_null() {
            log("could not open the menu window");
            return;
        }
        let screen = GetDC(null_mut());
        let mem_dc = CreateCompatibleDC(screen);
        ReleaseDC(null_mut(), screen);
        let (Some((dib, bits, old)), Some(pix)) = (new_dib(mem_dc, w, h), Pixmap::new(w as u32, h as u32)) else {
            DeleteDC(mem_dc);
            DestroyWindow(mh);
            return;
        };
        MENU.with(|s| *s.borrow_mut() = Some(MenuWin { hwnd: mh, owner: hwnd, menu: m, pix, mem_dc, dib, old, bits, start: Instant::now(), sc }));
        menu_tick();
        ShowWindow(mh, SW_SHOW);
        SetForegroundWindow(mh);
        windows_sys::Win32::UI::Input::KeyboardAndMouse::SetFocus(mh);
        SetTimer(mh, MENU_TICK, 15, None);
    }

    /// Close an open menu at once (no animation).
    unsafe fn close_menu_now() {
        let open = MENU.with(|s| s.borrow().as_ref().map(|m| m.hwnd));
        if let Some(h) = open {
            DestroyWindow(h);
        }
    }

    /// One menu frame; the window goes once the close animation ends.
    unsafe fn menu_tick() {
        let done = MENU.with(|s| {
            let mut g = s.borrow_mut();
            let w = g.as_mut()?;
            let t = w.start.elapsed().as_secs_f32();
            w.menu.render(&mut w.pix, t);
            push_layered(w.hwnd, w.mem_dc, w.bits, &w.pix);
            w.menu.finished(t).then_some(w.hwnd)
        });
        if let Some(h) = done {
            DestroyWindow(h);
        }
    }

    /// The whole pixmap to a per-pixel-alpha layered window (premultiplied RGBA into the DIB's BGRA).
    unsafe fn push_layered(hwnd: HWND, mem_dc: HDC, bits: *mut u8, pix: &Pixmap) {
        let src = pix.data();
        let dst = std::slice::from_raw_parts_mut(bits, src.len());
        for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            d[0] = s[2];
            d[1] = s[1];
            d[2] = s[0];
            d[3] = s[3];
        }
        let size = SIZE { cx: pix.width() as i32, cy: pix.height() as i32 };
        let origin = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
        UpdateLayeredWindow(hwnd, null_mut(), null(), &size, mem_dc, &origin, 0, &blend, ULW_ALPHA);
    }

    /// Run `f` on the open menu with the time since it opened; its answer comes back after the borrow ends.
    fn with_menu<R>(f: impl FnOnce(&mut menu::Menu, f32, f32) -> R) -> Option<(R, HWND)> {
        MENU.with(|s| {
            let mut g = s.try_borrow_mut().ok()?;
            let w = g.as_mut()?;
            let t = w.start.elapsed().as_secs_f32();
            Some((f(&mut w.menu, t, w.sc), w.owner))
        })
    }

    unsafe extern "system" fn menu_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
        let pos = || ((lp & 0xFFFF) as i16 as f32, ((lp >> 16) & 0xFFFF) as i16 as f32);
        match msg {
            WM_TIMER => {
                menu_tick();
                0
            }
            WM_MOUSEMOVE => {
                let (x, y) = pos();
                with_menu(|m, _, sc| m.mouse_move(x / sc, y / sc));
                let mut tme = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: hwnd, dwHoverTime: 0 };
                TrackMouseEvent(&mut tme);
                0
            }
            MOUSE_LEFT => {
                with_menu(|m, _, _| m.mouse_leave());
                0
            }
            WM_LBUTTONUP | WM_RBUTTONUP => {
                let (x, y) = pos();
                if let Some((Some(act), owner)) = with_menu(|m, t, sc| m.click(x / sc, y / sc, t)) {
                    apply_menu(owner, act);
                }
                0
            }
            WM_KEYDOWN => {
                let key = match wp as u16 {
                    VK_UP => Some(menu::Key::Up),
                    VK_DOWN => Some(menu::Key::Down),
                    VK_LEFT => Some(menu::Key::Left),
                    VK_RIGHT => Some(menu::Key::Right),
                    VK_RETURN | VK_SPACE => Some(menu::Key::Enter),
                    VK_BACK => Some(menu::Key::Back),
                    VK_ESCAPE => Some(menu::Key::Escape),
                    _ => None,
                };
                if let Some(key) = key {
                    if let Some((Some(act), owner)) = with_menu(|m, t, _| m.key(key, t)) {
                        apply_menu(owner, act);
                    }
                }
                0
            }
            // clicked somewhere else: fade away
            WM_ACTIVATE => {
                if (wp & 0xFFFF) as u32 == WA_INACTIVE {
                    with_menu(|m, t, _| m.close(t));
                }
                0
            }
            WM_DESTROY => {
                KillTimer(hwnd, MENU_TICK);
                let gone = MENU.with(|s| s.try_borrow_mut().ok().and_then(|mut g| g.take()));
                drop(gone);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }

    /// What a menu pick does.
    unsafe fn apply_menu(hwnd: HWND, act: menu::Action) {
        match act {
            menu::Action::Corner => to_corner(hwnd),
            menu::Action::Topmost => {
                let on = topmost();
                save_setting("always_on_top", if on { "0" } else { "1" });
                apply_topmost(hwnd, !on);
            }
            menu::Action::Quit => {
                close_menu_now();
                DestroyWindow(hwnd);
            }
            menu::Action::SizeAuto => {
                clear_setting("scale");
                with_app(|a| {
                    a.auto_size = true;
                    a.resize(dpi_scale());
                });
            }
            menu::Action::Size(i) => {
                let Some(&s) = SIZES.get(i) else { return };
                save_setting("scale", &s.to_string());
                with_app(|a| {
                    a.auto_size = false;
                    a.resize(s);
                });
            }
            menu::Action::Coat(i) => {
                let Some(&coat) = scene::COATS.get(i) else { return };
                // a fresh roll of where the patches sit, kept until the next pick
                let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.subsec_nanos() ^ d.as_secs() as u32) | 1;
                save_setting("coat", coat.name);
                save_setting("coat_seed", &seed.to_string());
                with_app(|a| {
                    a.scene.coat = coat;
                    a.scene.patch_seed = seed;
                    a.frame();
                });
            }
            menu::Action::Monitor(i) => {
                let Some(device) = monitors().get(i).map(|d| d.device.clone()) else { return };
                save_setting("monitor", &device);
                HOME.with(|h| *h.borrow_mut() = device);
                to_corner(hwnd);
            }
        }
    }

    unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        match msg {
            // the whole kitten drags like a title bar (see-through pixels click through)
            WM_NCHITTEST => {
                let r = DefWindowProcW(hwnd, msg, wp, lp);
                if r == HTCLIENT as LRESULT { HTCAPTION as LRESULT } else { r }
            }
            WM_NCLBUTTONDBLCLK => 0,
            WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
            WM_NCRBUTTONUP => {
                show_menu(hwnd);
                0
            }
            // a drag ended: open here next time
            WM_EXITSIZEMOVE => {
                let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                GetWindowRect(hwnd, &mut r);
                save_setting("position", &format!("{},{}", r.left, r.top));
                0
            }
            WM_TIMER => {
                with_app(|a| if wp == FRAME { a.frame() } else { a.poll() });
                0
            }
            WM_UPDATE_READY => {
                apply_update(hwnd);
                0
            }
            WM_UPDATE_CHECK => {
                start_update_checks(hwnd, false);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }

    /// A staged update is ready (posted by the update thread): swap the exe, let go of the
    /// one-kitten mutex and start the new exe with the same arguments, then close.
    const WM_UPDATE_READY: u32 = WM_APP + 1;
    /// `kitten-assistant update` asks the running kitten to check now.
    const WM_UPDATE_CHECK: u32 = WM_APP + 2;
    static MUTEX: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    /// Release builds only; off with `auto_update=0` in kitten-assistant.ini.
    fn auto_update() -> bool {
        update::RELEASE_BUILD && setting("auto_update").map_or(true, |v| v != "0")
    }

    /// Checks on a background thread: once now (`repeat` false), or 30 s after start and then
    /// every 6 hours; posts WM_UPDATE_READY once a newer release is downloaded and verified.
    fn start_update_checks(hwnd: HWND, repeat: bool) {
        let at = hwnd as usize;
        let _ = std::thread::Builder::new().name("update".into()).spawn(move || {
            if repeat {
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
            loop {
                match update::check_and_stage() {
                    Ok(Some(tag)) => {
                        log(&format!("update {tag} downloaded and verified (running {})", update::label()));
                        unsafe { PostMessageW(at as HWND, WM_UPDATE_READY, 0, 0) };
                        return;
                    }
                    Ok(None) => {}
                    Err(e) => log(&format!("update check: {e}")),
                }
                if !repeat {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_secs(6 * 3600));
            }
        });
    }

    fn apply_update(hwnd: HWND) {
        if let Err(e) = update::swap_in() {
            log(&format!("update: {e}"));
            return;
        }
        let Ok(exe) = std::env::current_exe() else { return };
        unsafe { CloseHandle(MUTEX.swap(0, std::sync::atomic::Ordering::SeqCst) as HANDLE) };
        match std::process::Command::new(&exe).args(std::env::args_os().skip(1)).spawn() {
            Ok(_) => unsafe {
                DestroyWindow(hwnd);
            },
            Err(e) => log(&format!("update: could not start the new exe: {e}")),
        }
    }

    /// `kitten-assistant update`: the running kitten checks now; with none running, update the exe here.
    pub fn update_now() -> Result<(), String> {
        if !update::RELEASE_BUILD {
            message(&format!("This is a development build ({}): only release builds update themselves.", update::label()));
            return Ok(());
        }
        let c = wide(CLASS);
        let h = unsafe { FindWindowW(c.as_ptr(), null()) };
        if !h.is_null() {
            unsafe { PostMessageW(h, WM_UPDATE_CHECK, 0, 0) };
            return Ok(());
        }
        match update::check_and_stage()? {
            Some(tag) => {
                update::swap_in()?;
                message(&format!("Updated to {tag}."));
            }
            None => message(&format!("Kitten Assistant {} is up to date.", update::label())),
        }
        Ok(())
    }

    pub fn run(o: &Opts, status: PathBuf) -> Result<(), String> {
        unsafe {
            let name = wide("Local\\KittenAssistant");
            let mutex = CreateMutexW(null(), 0, name.as_ptr());
            if GetLastError() == ERROR_ALREADY_EXISTS {
                return Ok(()); // one kitten is enough
            }
            MUTEX.store(mutex as usize, std::sync::atomic::Ordering::SeqCst);
            update::clean_up();
            // Never compete with real work: the lowest scheduling priority, and Windows'
            // efficiency mode (EcoQoS: efficient cores, low clocks) for its few milliseconds a frame.
            SetPriorityClass(GetCurrentProcess(), IDLE_PRIORITY_CLASS);
            let eco = PROCESS_POWER_THROTTLING_STATE {
                Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
                ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
                StateMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            };
            SetProcessInformation(GetCurrentProcess(), ProcessPowerThrottling, &eco as *const _ as *const c_void, std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32);
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            // --scale wins over the saved size, which wins over the screen DPI
            let picked = o.scale.or_else(saved_scale);
            let sc = picked.unwrap_or_else(dpi_scale).clamp(0.25, 8.0);
            let (w, h) = ((scene::W * sc).ceil() as i32, (scene::H * sc).ceil() as i32);

            let hinst = GetModuleHandleW(null());
            let class = wide(CLASS);
            let wc = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinst,
                hIcon: null_mut(),
                hCursor: LoadCursorW(null_mut(), IDC_HAND),
                hbrBackground: null_mut(),
                lpszMenuName: null(),
                lpszClassName: class.as_ptr(),
            };
            if RegisterClassW(&wc) == 0 {
                return Err("could not register the window class".into());
            }
            let home = match o.monitor {
                Some(n) => {
                    let device = format!(r"\\.\DISPLAY{n}");
                    if !monitors().iter().any(|m| m.device == device) {
                        return Err(format!("there is no display {n}; see right-click > Monitor for the list"));
                    }
                    save_setting("monitor", &device);
                    device
                }
                None => setting("monitor").unwrap_or_default(),
            };
            HOME.with(|h| *h.borrow_mut() = home);
            if o.monitor.is_some() {
                clear_setting("position");
            }
            let (cx, cy) = saved_position(w, h).unwrap_or_else(|| corner(w, h));
            let (x, y) = (o.x.unwrap_or(cx), o.y.unwrap_or(cy));
            let title = wide("Kitten Assistant");
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | if topmost() { WS_EX_TOPMOST } else { 0 },
                class.as_ptr(),
                title.as_ptr(),
                WS_POPUP,
                x,
                y,
                w,
                h,
                null_mut(),
                null_mut(),
                hinst,
                null(),
            );
            if hwnd.is_null() {
                return Err("could not create the window".into());
            }

            // the DIB the frame is copied into each tick
            let screen = GetDC(null_mut());
            let mem_dc = CreateCompatibleDC(screen);
            ReleaseDC(null_mut(), screen);
            let (dib, bits, old) = new_dib(mem_dc, w, h).ok_or("could not create the drawing surface")?;

            let mut scene = scene::Scene::new();
            scene.coat = scene::Coat::named(&setting("coat").unwrap_or_default());
            scene.patch_seed = setting("coat_seed").and_then(|v| v.parse().ok()).unwrap_or(1);
            let mut app = App {
                hwnd,
                scene,
                pix: Pixmap::new(w as u32, h as u32).ok_or("pixmap")?,
                sc,
                auto_size: picked.is_none(),
                start: Instant::now(),
                status,
                mem_dc,
                dib,
                old,
                bits,
                frame_ms: 0,
                first_poll: true,
                last_frame: Vec::new(),
                last_status: String::new(),
            };
            app.poll();
            app.frame();
            APP.with(|a| *a.borrow_mut() = Some(app));
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            SetTimer(hwnd, POLL, 2000, None);
            if auto_update() {
                start_update_checks(hwnd, true);
            }

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            APP.with(|a| *a.borrow_mut() = None);
            let m = MUTEX.swap(0, std::sync::atomic::Ordering::SeqCst);
            if m != 0 {
                CloseHandle(m as HANDLE);
            }
        }
        Ok(())
    }
}
