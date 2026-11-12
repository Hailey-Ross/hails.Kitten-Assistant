//! Kitten Assistant: a tiny desktop buddy for Windows.
//!
//! A kitten sits at a little computer on top of the taskbar. Point it at a status
//! file: while the file's first line has text, the kitten types and a speech bubble
//! shows that text and how long it has been going; when the file is empty, missing
//! or says "idle", the kitten naps.
#![windows_subsystem = "windows"]

mod canvas;
mod scene;

use std::path::{Path, PathBuf};
use std::time::Instant;

use tiny_skia::Pixmap;

const USAGE: &str = "\
kitten-assistant                      run the kitten (watches the status file)
kitten-assistant say \"some text\"      kitten types, bubble shows the text
kitten-assistant idle                 kitten naps
kitten-assistant quit                 close a running kitten
kitten-assistant snapshot out.png     render one frame to a PNG

options:
  --status FILE    status file (default: status.txt next to the exe, or %KITTEN_STATUS%)
  --scale N        size multiplier (default: follows screen DPI)
  --x N --y N      window position (default: bottom-right, on the taskbar)
snapshot options:
  --busy TEXT      render the busy pose with TEXT (default: napping)
  --time SECONDS   animation time to render (default 3.2)
  --minutes N      elapsed minutes shown in the bubble
  --bg RRGGBB      background colour (default: transparent)";

#[derive(Default)]
struct Opts {
    cmd: String,
    arg: Option<String>,
    status: Option<PathBuf>,
    scale: Option<f32>,
    x: Option<i32>,
    y: Option<i32>,
    busy: Option<String>,
    time: Option<f64>,
    minutes: Option<f64>,
    bg: Option<String>,
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
            "--busy" => o.busy = Some(val()?),
            "--time" => o.time = Some(val()?.parse().map_err(|_| "bad --time")?),
            "--minutes" => o.minutes = Some(val()?.parse().map_err(|_| "bad --minutes")?),
            "--bg" => o.bg = Some(val()?),
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
        other => Err(format!("unknown command {other}\n\n{USAGE}")),
    };
    if let Err(e) = result {
        log(&e);
        win::message(&e);
    }
}

fn snapshot(o: &Opts) -> Result<(), String> {
    let out = o.arg.clone().unwrap_or_else(|| "kitten.png".into());
    let sc = o.scale.unwrap_or(2.0);
    let t_end = o.time.unwrap_or(3.2);
    let mut s = scene::Scene::new();
    s.set_status(o.busy.as_deref().unwrap_or("idle"), 0.0);
    s.since = -o.minutes.unwrap_or(0.0) * 60.0;
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
    use windows_sys::Win32::System::Threading::CreateMutexW;
    use windows_sys::Win32::UI::HiDpi::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const CLASS: &str = "KittenAssistantWindow";
    const FRAME: usize = 1;
    const POLL: usize = 2;
    const MENU_CORNER: usize = 1;
    const MENU_QUIT: usize = 2;

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
        start: Instant,
        status: PathBuf,
        mem_dc: HDC,
        dib: HBITMAP,
        old: HGDIOBJ,
        bits: *mut u8,
        frame_ms: u32,
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

        fn poll(&mut self) {
            let t = self.t();
            self.scene.set_status(&read_status(&self.status), t);
            // smooth while typing, lighter while napping
            let ms = if self.scene.busy { 33 } else { 66 };
            if ms != self.frame_ms {
                self.frame_ms = ms;
                unsafe { SetTimer(self.hwnd, FRAME, ms, None) };
            }
        }

        /// Copy the frame (premultiplied RGBA) into the DIB (premultiplied BGRA) and
        /// hand it to Windows as a per-pixel-alpha layered window.
        fn push(&mut self) {
            let (w, h) = (self.pix.width() as i32, self.pix.height() as i32);
            let src = self.pix.data();
            let dst = unsafe { std::slice::from_raw_parts_mut(self.bits, src.len()) };
            for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
                d[0] = s[2];
                d[1] = s[1];
                d[2] = s[0];
                d[3] = s[3];
            }
            unsafe {
                let screen = GetDC(null_mut());
                let size = SIZE { cx: w, cy: h };
                let origin = POINT { x: 0, y: 0 };
                let blend = BLENDFUNCTION {
                    BlendOp: AC_SRC_OVER as u8,
                    BlendFlags: 0,
                    SourceConstantAlpha: 255,
                    AlphaFormat: AC_SRC_ALPHA as u8,
                };
                UpdateLayeredWindow(self.hwnd, screen, null(), &size, self.mem_dc, &origin, 0, &blend, ULW_ALPHA);
                ReleaseDC(null_mut(), screen);
            }
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

    fn corner(w: i32, h: i32) -> (i32, i32) {
        let mut wa = RECT { left: 0, top: 0, right: 1280, bottom: 720 };
        unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut wa as *mut RECT as *mut c_void, 0) };
        (wa.right - w - 20, wa.bottom - h)
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

    unsafe fn show_menu(hwnd: HWND) {
        let m = CreatePopupMenu();
        let (a, b) = (wide("Back to corner"), wide("Quit"));
        AppendMenuW(m, MF_STRING, MENU_CORNER, a.as_ptr());
        AppendMenuW(m, MF_STRING, MENU_QUIT, b.as_ptr());
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(m, TPM_RETURNCMD | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, null()) as usize;
        DestroyMenu(m);
        match cmd {
            MENU_CORNER => {
                let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                GetWindowRect(hwnd, &mut r);
                let (x, y) = corner(r.right - r.left, r.bottom - r.top);
                SetWindowPos(hwnd, null_mut(), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
            }
            MENU_QUIT => {
                DestroyWindow(hwnd);
            }
            _ => {}
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
            WM_TIMER => {
                with_app(|a| if wp == FRAME { a.frame() } else { a.poll() });
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }

    pub fn run(o: &Opts, status: PathBuf) -> Result<(), String> {
        unsafe {
            let name = wide("Local\\KittenAssistant");
            let mutex = CreateMutexW(null(), 0, name.as_ptr());
            if GetLastError() == ERROR_ALREADY_EXISTS {
                return Ok(()); // one kitten is enough
            }
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            let sc = o.scale.unwrap_or_else(|| GetDpiForSystem() as f32 / 96.0).clamp(0.25, 8.0);
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
            let (cx, cy) = corner(w, h);
            let (x, y) = (o.x.unwrap_or(cx), o.y.unwrap_or(cy));
            let title = wide("Kitten Assistant");
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
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

            // 32-bit top-down DIB the frame is copied into each tick
            let screen = GetDC(null_mut());
            let mem_dc = CreateCompatibleDC(screen);
            ReleaseDC(null_mut(), screen);
            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = w;
            bmi.bmiHeader.biHeight = -h;
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB;
            let mut bits: *mut c_void = null_mut();
            let dib = CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
            if dib.is_null() || bits.is_null() {
                return Err("could not create the drawing surface".into());
            }
            let old = SelectObject(mem_dc, dib as HGDIOBJ);

            let mut app = App {
                hwnd,
                scene: scene::Scene::new(),
                pix: Pixmap::new(w as u32, h as u32).ok_or("pixmap")?,
                sc,
                start: Instant::now(),
                status,
                mem_dc,
                dib,
                old,
                bits: bits as *mut u8,
                frame_ms: 0,
            };
            app.poll();
            app.frame();
            APP.with(|a| *a.borrow_mut() = Some(app));
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            SetTimer(hwnd, POLL, 2000, None);

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            APP.with(|a| *a.borrow_mut() = None);
            CloseHandle(mutex);
        }
        Ok(())
    }
}
