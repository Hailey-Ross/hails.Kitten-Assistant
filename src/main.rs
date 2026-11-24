//! Kitten Assistant: a tiny desktop buddy for Windows.
//!
//! A kitten sits at a little computer on top of the taskbar. Point it at a status
//! file: while the file's first line has text, the kitten types and a speech bubble
//! shows that text and how long it has been going; when the file is empty, missing
//! or says "idle", the kitten naps.
#![windows_subsystem = "windows"]

mod brain;
mod canvas;
mod history;
mod listen;
mod menu;
mod scene;
mod sessions;
mod update;
mod watch;

use std::path::{Path, PathBuf};
use std::time::Instant;

use tiny_skia::Pixmap;

const USAGE: &str = "\
kitten-assistant                      run the kitten (watches the status file)
kitten-assistant say \"some text\"      kitten types, bubble shows the text (starts the kitten if none runs)
kitten-assistant idle                 kitten naps
kitten-assistant status               print the line the kitten shows, then every other busy program's line
kitten-assistant status --json        the same as JSON: every busy line with its session, mood, age, progress
kitten-assistant sessions             one busy line per row: session, mood, seconds, text (tab separated)
kitten-assistant sessions clear       every program's line goes away (the kitten naps)
kitten-assistant do ACTION [TEXT]     the running kitten acts now: sip, refill, stretch, yawn, look (while it works),
                                      wave [TEXT], cheer [TEXT] (any time; TEXT in the bubble)
kitten-assistant wrap -- PROGRAM ARGS run PROGRAM with a line up (--line TEXT, default \"Running: PROGRAM ARGS\"),
                                      then idle on success or a Failed line; exits with PROGRAM's exit code
kitten-assistant wrap --pid N         the same for a process that is already running
kitten-assistant watch --log FILE     keep a line current from a job's log (progress from \"3/10\" or \"45%\")
kitten-assistant timer TIME [TEXT]    a countdown in the bubble and a cheer at the end (timer stop ends it)
kitten-assistant history              what ran today and for how long (history on|off; --days N, --json)
kitten-assistant quit                close a running kitten
kitten-assistant snapshot out.png     render one frame to a PNG
kitten-assistant update               check for a newer stable release now and install it
kitten-assistant version              show the version

options:
  --status FILE    status file (default: status.txt next to the exe, or %KITTEN_STATUS%)
  --session NAME   this program's own status line (or %KITTEN_SESSION%): several programs can report at once;
                   the kitten shows a problem first, then work, then waiting, and naps when all are idle
  --scale N        size multiplier (default: right-click > Size, else the screen DPI)
  --monitor N      start on display N and make it the default (right-click > Monitor does the same)
  --x N --y N      window position (default: bottom-right of the default display)
say options:
  --for TIME       the line clears itself after TIME (90, 30s, 5m, 1h30m)
  --progress P     a progress bar in the bubble: 3/10, 45% or 0.45 (say it again with the same text to move it)
  --steps A,B,C    a checklist on the screen; --step N marks step N (1-based; --step alone keeps the steps)
  --mood NAME      the mood whatever the first word says (typing, waiting, problem, question, testing, building,
                   researching, downloading, uploading, thinking)
  --open URL|PATH  a click on the bubble opens it (a web page, a folder, a log)
  --attention      the bubble hops until it is clicked
idle options:
  --cheer TEXT     cheer with TEXT as the line ends
watch options:
  --log FILE       the log to follow; --match TEXT only lines with TEXT; --tail shows the newest log line too
  --pid N          stop when process N ends (idle on exit code 0, else a Failed line)
  --background     keep watching in the background and return at once (idle --session NAME stops it)
run options:
  --listen ADDR    serve the local endpoint (e.g. 127.0.0.1:47613; token in listen-token.txt)
wrap options:
  --line TEXT      the line while PROGRAM runs (its first word picks the mood, as with say)
  --session NAME   default: \"wrap PROGRAM\"
snapshot options:
  --busy TEXT      render the busy pose with TEXT (default: napping)
  --time SECONDS   animation time to render (default 3.2)
  --minutes N      elapsed minutes shown in the bubble
  --bg RRGGBB      background color (default: transparent)
  --done           render the all-done moment
  --screen N       screen 0-4 of the mood (typing: code, progress, equalizer, gear, graph; default 0)
  --sip            render the middle of a coffee sip
  --coat NAME      the color (as in right-click > Color, e.g. \"Gray tabby\"; default: Orange tabby)
  --seed N         where the patches sit on patched coats (default 1)";

#[derive(Default)]
struct Opts {
    cmd: String,
    arg: Option<String>,
    status: Option<PathBuf>,
    session: Option<String>,
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
    egg_pick: Option<usize>,
    menu: Option<String>,
    hover: Option<usize>,
    refill: Option<f64>,
    event: Option<String>,
    uptodate: Option<f64>,
    coat: Option<String>,
    seed: Option<u32>,
    greet: Option<String>,
    frames: Option<u32>,
    fps: Option<f64>,
    /// `say --for`: seconds until the line clears itself
    for_secs: Option<f64>,
    progress: Option<sessions::Progress>,
    /// `wrap --line`
    line: Option<String>,
    json: bool,
    /// snapshot: the words of a `--event cheer@P` / `wave@P`
    text: Option<String>,
    /// everything after `--` (wrap's program and its arguments), untouched
    rest: Vec<String>,
    mood: Option<scene::Mood>,
    open: Option<String>,
    attention: bool,
    steps: Option<Vec<String>>,
    step: Option<usize>,
    /// `idle --cheer`
    cheer: Option<String>,
    log_file: Option<PathBuf>,
    matching: Option<String>,
    tail: bool,
    pid: Option<u32>,
    background: bool,
    listen: Option<String>,
    days: Option<u32>,
}

fn parse() -> Result<Opts, String> {
    let mut o = Opts { cmd: "run".into(), ..Default::default() };
    let mut it = std::env::args().skip(1);
    let mut positional = Vec::new();
    while let Some(a) = it.next() {
        let mut val = || it.next().ok_or_else(|| format!("{a} needs a value"));
        match a.as_str() {
            "--status" => o.status = Some(PathBuf::from(val()?)),
            "--session" => o.session = Some(val()?),
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
            "--eggpick" => { o.egg = true; o.egg_pick = Some(val()?.parse().map_err(|_| "bad --eggpick")?) }
            "--menu" => o.menu = Some(val()?),
            "--event" => o.event = Some(val()?),
        "--uptodate" => o.uptodate = Some(val()?.parse().map_err(|_| "bad --uptodate")?),
        "--refill" => o.refill = Some(val()?.parse().map_err(|_| "bad --refill")?),
            "--hover" => o.hover = Some(val()?.parse().map_err(|_| "bad --hover")?),
            "--coat" => o.coat = Some(val()?),
            "--seed" => o.seed = Some(val()?.parse().map_err(|_| "bad --seed")?),
            "--greet" => o.greet = Some(val()?),
            "--frames" => o.frames = Some(val()?.parse().map_err(|_| "bad --frames")?),
            "--fps" => o.fps = Some(val()?.parse().ok().filter(|f: &f64| *f > 0.0).ok_or("bad --fps")?),
            "--screen" => o.screen = Some(val()?.parse().map_err(|_| "bad --screen")?),
            "--for" => o.for_secs = Some(sessions::parse_duration(&val()?).ok_or("bad --for (try 90, 30s, 5m or 1h30m)")?),
            "--progress" => o.progress = Some(sessions::Progress::parse(&val()?).ok_or("bad --progress (try 3/10, 45% or 0.45)")?),
            "--line" => o.line = Some(val()?),
            "--json" => o.json = true,
            "--text" => o.text = Some(val()?),
            "--mood" => o.mood = Some(scene::Mood::named(&val()?).ok_or("bad --mood (try question, testing, building, waiting...)")?),
            "--open" => o.open = Some(val()?),
            "--attention" => o.attention = true,
            "--steps" => o.steps = Some(sessions::parse_steps(&val()?)).filter(|s| !s.is_empty()),
            "--step" => o.step = Some(val()?.parse().ok().filter(|s| *s >= 1).ok_or("bad --step (1 is the first)")?),
            "--cheer" => o.cheer = Some(val()?),
            "--log" => o.log_file = Some(PathBuf::from(val()?)),
            "--match" => o.matching = Some(val()?),
            "--tail" => o.tail = true,
            "--pid" => o.pid = Some(val()?.parse().map_err(|_| "bad --pid")?),
            "--background" => o.background = true,
            "--listen" => o.listen = Some(val()?),
            "--days" => o.days = Some(val()?.parse().map_err(|_| "bad --days")?),
            "--" => {
                o.rest = it.by_ref().collect();
                break;
            }
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

/// A status file's whole content (its text, then any `key=value` lines; see `sessions::split`); empty when missing.
fn read_status(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

/// The busy lines, the shown one first.
fn busy_lines(status: &Path) -> Vec<sessions::Line> {
    let lines = sessions::read_all(status, read_status, status_age, sessions::now());
    let Some(top) = sessions::pick(&lines).map(|(l, _)| l.clone()) else { return lines };
    std::iter::once(top.clone()).chain(lines.into_iter().filter(|l| *l != top)).collect()
}

/// What goes on the lines after a status line's text (see `sessions::Meta`), as the options gave it.
#[derive(Clone, Default)]
pub struct LineOpts {
    pub for_secs: Option<f64>,
    pub progress: Option<sessions::Progress>,
    pub mood: Option<scene::Mood>,
    pub open: Option<String>,
    pub attention: bool,
    pub steps: Option<Vec<String>>,
    pub step: Option<usize>,
    pub countdown: bool,
}

impl LineOpts {
    fn from(o: &Opts) -> LineOpts {
        LineOpts {
            for_secs: o.for_secs,
            progress: o.progress.clone(),
            mood: o.mood,
            open: o.open.clone(),
            attention: o.attention,
            steps: o.steps.clone(),
            step: o.step,
            countdown: false,
        }
    }
}

/// Writes a line (the session's own file, or the plain status file): the text and its options. `--step` without
/// `--steps` keeps the steps the line already has. `idle` or nothing is written as it is.
pub fn say_line(status: &Path, session: Option<&str>, text: &str, opts: &LineOpts) -> Result<(), String> {
    let file = match session {
        Some(name) => sessions::file(status, name),
        None => status.to_path_buf(),
    };
    let body = if text.trim().is_empty() || text.trim().eq_ignore_ascii_case("idle") {
        text.to_string()
    } else {
        let steps = match (&opts.steps, opts.step) {
            (Some(s), _) => s.clone(),
            (None, Some(_)) => sessions::split(&read_status(&file)).1.steps,
            (None, None) => Vec::new(),
        };
        sessions::Meta {
            until: opts.for_secs.map(|s| sessions::now() + s),
            progress: opts.progress.clone(),
            mood: opts.mood,
            open: opts.open.clone(),
            attention: opts.attention,
            steps,
            step: opts.step,
            countdown: opts.countdown,
        }
        .body(text)
    };
    match session {
        Some(name) => sessions::say(status, name, &body),
        None => std::fs::write(status, &body).map_err(|e| e.to_string()),
    }
}

/// `s` as a JSON string, quotes included.
fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// `status --json`: `{"idle":false,"lines":[{"session":..,"text":..,"mood":..,"age_secs":..,"progress":..,
/// "progress_label":..,"expires_in_secs":..,"shown":true},..]}`, the shown line first.
fn status_json(lines: &[sessions::Line], now: f64) -> String {
    let rows: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let (p, label) = match &l.meta.progress {
                Some(p) => (format!("{:.3}", p.frac), json_str(&p.label)),
                None => ("null".into(), "null".into()),
            };
            let left = l.meta.until.map_or("null".into(), |u| format!("{:.0}", (u - now).max(0.0)));
            format!(
                "{{\"session\":{},\"text\":{},\"mood\":\"{}\",\"age_secs\":{:.0},\"progress\":{p},\"progress_label\":{label},\"expires_in_secs\":{left},\"shown\":{}}}",
                if l.name.is_empty() { "null".into() } else { json_str(&l.name) },
                json_str(&l.text),
                l.mood().name(),
                l.age,
                i == 0
            )
        })
        .collect();
    format!("{{\"idle\":{},\"lines\":[{}]}}", lines.is_empty(), rows.join(","))
}

/// `do`: the actions a running kitten takes from another program, and their numbers in the message.
const ACTIONS: [&str; 7] = ["sip", "refill", "stretch", "yawn", "look", "wave", "cheer"];

/// `wrap`: the line it leaves when the program failed with `code` (None: stopped with Ctrl+C or killed).
fn wrap_failed(line: &str, code: Option<i32>) -> String {
    // the line's own mood word goes; "Failed" takes its place
    let what = match line.split_once(':') {
        Some((w, rest)) if !rest.trim().is_empty() && scene::Mood::of(w) != scene::Mood::Busy => rest.trim(),
        _ => line.trim(),
    };
    match code {
        Some(c) => format!("Failed: {what} (exit {c})"),
        None => format!("Failed: {what} (canceled)"),
    }
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
    // commands other programs hook into never show a message box (it would block a script until someone clicks):
    // they print the error and exit with 1
    let hook = std::env::args().skip(1).take_while(|a| a != "--").any(|a| matches!(a.as_str(), "do" | "wrap" | "sessions" | "watch" | "timer" | "history" | "--json"));
    let o = match parse() {
        Ok(o) => o,
        Err(e) if hook => {
            log(&e);
            win::print(&format!("kitten-assistant: {e}"));
            std::process::exit(1);
        }
        Err(e) => {
            win::message(&e);
            return;
        }
    };
    let status = status_path(&o);
    let session = o.session.clone().or_else(|| std::env::var("KITTEN_SESSION").ok()).as_deref().and_then(sessions::clean);
    let write = |text: &str| {
        let r = say_line(&status, session.as_deref(), text, &LineOpts::from(&o));
        if r.is_ok() && !text.trim().is_empty() && !text.trim().eq_ignore_ascii_case("idle") {
            win::wake(&status);
        }
        r
    };
    let result = match o.cmd.as_str() {
        "run" => win::run(&o, status.clone()),
        "say" => write(&o.arg.clone().unwrap_or_default()),
        "idle" => {
            let r = match &session {
                Some(name) => sessions::idle(&status, name),
                None => std::fs::write(&status, "idle").map_err(|e| e.to_string()),
            };
            // --cheer: the end of the job gets its own words (nothing to cheer when no kitten runs)
            if let (Ok(()), Some(text)) = (&r, o.cheer.as_deref()) {
                let _ = win::act(ACTIONS.len() - 1, text);
            }
            r
        }
        // the line the kitten shows, then every other busy program's line; `idle` when nothing is busy
        "status" => {
            let lines = busy_lines(&status);
            let text = if o.json {
                status_json(&lines, sessions::now())
            } else if lines.is_empty() {
                "idle".to_string()
            } else {
                lines
                    .iter()
                    .map(|l| if l.name.is_empty() { l.text.clone() } else { format!("[{}] {}", l.name, l.text) })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            win::print(&text);
            Ok(())
        }
        // one busy line per row for scripts: session, mood, seconds, text (tabs between); `sessions clear` ends them all
        "sessions" => match o.arg.as_deref() {
            None => {
                let rows: Vec<String> = busy_lines(&status)
                    .iter()
                    .map(|l| format!("{}\t{}\t{:.0}\t{}", l.who(), l.mood().name(), l.age, l.text))
                    .collect();
                if !rows.is_empty() {
                    win::print(&rows.join("\n"));
                }
                Ok(())
            }
            Some("clear") => sessions::clear_all(&status).map(|n| win::print(&format!("cleared {n} session line(s)"))),
            Some(other) => Err(format!("sessions: unknown `{other}` (try `sessions` or `sessions clear`)")),
        },
        "do" => {
            let arg = o.arg.clone().unwrap_or_default();
            let (action, text) = arg.split_once(' ').map_or((arg.as_str(), ""), |(a, t)| (a, t.trim()));
            let action = action.to_ascii_lowercase();
            match ACTIONS.iter().position(|a| *a == action) {
                Some(i) => win::act(i, text),
                None => Err(format!("do: unknown action `{action}` (one of {})", ACTIONS.join(", "))),
            }
        }
        "wrap" if o.pid.is_some() && o.rest.is_empty() => {
            let pid = o.pid.unwrap_or(0);
            let name = session.clone().unwrap_or_else(|| format!("wrap pid {pid}"));
            let line = o.line.clone().unwrap_or_else(|| format!("Running: process {pid}"));
            match win::watch(&status, &name, &line, None, &LineOpts::from(&o), Some(pid)) {
                Ok(code) => std::process::exit(code),
                Err(e) => Err(e),
            }
        }
        "watch" => {
            if o.log_file.is_none() && o.pid.is_none() {
                Err("watch: give --log FILE and/or --pid N".to_string())
            } else {
                let stem = o.log_file.as_deref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned());
                let name = session
                    .clone()
                    .or_else(|| sessions::clean(&format!("watch {}", stem.clone().unwrap_or_else(|| o.pid.unwrap_or(0).to_string()))))
                    .unwrap_or_else(|| "watch".into());
                let line = o.line.clone().unwrap_or_else(|| match (&stem, o.pid) {
                    (Some(s), _) => format!("Watching: {s}"),
                    (None, pid) => format!("Running: process {}", pid.unwrap_or(0)),
                });
                if o.background {
                    win::detach(&name)
                } else {
                    let log = o.log_file.clone().map(|p| (p, o.matching.clone(), o.tail));
                    match win::watch(&status, &name, &line, log, &LineOpts::from(&o), o.pid) {
                        Ok(code) => std::process::exit(code),
                        Err(e) => Err(e),
                    }
                }
            }
        }
        // `timer 25m Focus`: a countdown line, and a cheer when it runs out; `timer stop` ends it early
        "timer" => {
            let name = session.clone().unwrap_or_else(|| "timer".into());
            let arg = o.arg.clone().unwrap_or_default();
            let (first, text) = arg.split_once(' ').map_or((arg.as_str(), ""), |(a, b)| (a, b.trim()));
            if first.eq_ignore_ascii_case("stop") {
                sessions::idle(&status, &name)
            } else {
                match sessions::parse_duration(first) {
                    None => Err("timer: give a time first, e.g. kitten-assistant timer 25m Focus".to_string()),
                    Some(secs) => {
                        let mut opts = LineOpts::from(&o);
                        opts.for_secs = Some(secs);
                        opts.countdown = true;
                        opts.mood = opts.mood.or(Some(scene::Mood::Waiting));
                        let text = if text.is_empty() { "Timer" } else { text };
                        let r = say_line(&status, Some(&name), text, &opts);
                        if r.is_ok() {
                            win::wake(&status);
                        }
                        r
                    }
                }
            }
        }
        // what ran today (or the last --days N days), or turn the history on or off
        "history" => match o.arg.as_deref() {
            Some(v @ ("on" | "off")) => {
                win::save_setting("history", if v == "on" { "1" } else { "0" });
                win::print(&format!("history is {v} (the running kitten picks it up within 2 seconds)"));
                Ok(())
            }
            Some(other) => Err(format!("history: unknown `{other}` (try history, history on, history off)")),
            None => {
                let rows = history::read(&history::path(&status));
                let from = history::local_midnight(sessions::now(), o.days.unwrap_or(1).saturating_sub(1));
                let text = if o.json {
                    let items: Vec<String> = rows
                        .iter()
                        .filter(|r| r.end >= from)
                        .map(|r| {
                            format!(
                                "{{\"start\":{:.0},\"end\":{:.0},\"session\":{},\"mood\":{},\"text\":{}}}",
                                r.start,
                                r.end,
                                if r.session.is_empty() { "null".into() } else { json_str(&r.session) },
                                json_str(&r.mood),
                                json_str(&r.text)
                            )
                        })
                        .collect();
                    format!("[{}]", items.join(","))
                } else if win::setting("history").as_deref() != Some("1") && rows.is_empty() {
                    "the history is off: turn it on with `kitten-assistant history on` or right-click > Keep a history".into()
                } else {
                    history::summary(&rows, from)
                };
                win::print(&text);
                Ok(())
            }
        },
        "wrap" => {
            if o.rest.is_empty() {
                Err("wrap: give the program after --, e.g. kitten-assistant wrap -- cargo build".to_string())
            } else {
                let program = Path::new(&o.rest[0]).file_stem().map_or(o.rest[0].clone(), |s| s.to_string_lossy().into_owned());
                let name = session.clone().unwrap_or_else(|| sessions::clean(&format!("wrap {program}")).unwrap_or_else(|| "wrap".into()));
                let line = o.line.clone().unwrap_or_else(|| format!("Running: {}", o.rest.join(" ")));
                match win::wrap(&status, &name, &line, &o.rest, &LineOpts::from(&o)) {
                    Ok(code) => std::process::exit(code),
                    Err(e) => Err(e),
                }
            }
        }
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
            write(&text)
        }
    };
    if let Err(e) = result {
        log(&e);
        if hook {
            win::print(&format!("kitten-assistant: {e}"));
            std::process::exit(1);
        }
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
        sound: false,
        history: true,
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
    s.coat = scene::Coat::named(o.coat.as_deref().unwrap_or(""));
    s.patch_seed = o.seed.unwrap_or(1);
    // no surprise sips in snapshots; --sip puts one in its drinking moment
    // --sip draws a real sip (never an empty-mug refill) in its drinking moment
    s.brain.next_event = f64::MAX;
    if o.sip {
        s.brain.force(brain::Event::Sip, t_end - 1.25);
    }
    // --refill P: that moment (0..1) of an empty-mug refill (not in the usage text)
    if let Some(p) = o.refill {
        s.brain.force(brain::Event::Refill, t_end - p * 7.0);
    }
    // --uptodate P: that moment (0..1) of the "up to date" cheer (not in the usage text)
    if let Some(p) = o.uptodate {
        s.show_up_to_date(t_end - p * 3.0);
    }
    // --greet "TEXT|SECOND LINE": the hello wave, looping, with that text in the bubble (greeting cards and GIFs, see
    // tools\greeting-gif.ps1; not in the usage text)
    if let Some(g) = o.greet.as_deref() {
        let (text, sub) = match g.split_once('|') {
            Some((a, b)) => (a.trim(), Some(b.trim()).filter(|b| !b.is_empty())),
            None => (g.trim(), None),
        };
        s.greet_forever(0.0, text, sub);
    }
    // --progress: the bar in the bubble; --steps/--step: the checklist; --attention: the hop; --mood: the mood;
    // --for: a timer's countdown
    s.progress = o.progress.as_ref().map(|p| (p.frac, p.label.clone()));
    if let Some(steps) = o.steps.clone() {
        let at = o.step.unwrap_or(1).clamp(1, steps.len()) - 1;
        s.steps = Some((steps, at));
    }
    s.attention = o.attention;
    if o.mood.is_some() {
        s.set_status_mood(o.busy.as_deref().unwrap_or("idle"), 0.0, 0.0, o.mood);
    }
    if let Some(f) = o.for_secs {
        s.countdown_end = Some(t_end + f);
    }
    // --event cheer@P / wave@P (with --text): that moment of `do cheer` / `do wave` (not in the usage text)
    if let Some((name @ ("cheer" | "wave"), p)) = o.event.as_deref().and_then(|e| e.split_once('@')) {
        let p: f64 = p.parse().map_err(|_| "bad --event")?;
        if name == "cheer" {
            s.cheer_now(t_end - p * 3.0, o.text.as_deref());
        } else {
            s.wave(t_end - p * 3.5, o.text.as_deref());
        }
    }
    // --event NAME@P: that moment of a rare event (stretch, yawn, look; not in the usage text)
    if let Some((name, p)) = o.event.as_deref().and_then(|e| e.split_once('@')).filter(|(n, _)| !matches!(*n, "cheer" | "wave")) {
        let e = match name {
            "stretch" => brain::Event::Stretch,
            "yawn" => brain::Event::Yawn,
            "look" => brain::Event::LookAround,
            "sip" => brain::Event::Sip,
            _ => brain::Event::Refill,
        };
        let p: f64 = p.parse().map_err(|_| "bad --event")?;
        s.brain.force(e, t_end - p * e.secs());
    }
    if o.done {
        // the "all done!" moment, one second in
        s.set_status("job", 0.0);
        s.set_status("idle", (t_end - 1.0).max(0.0));
    }
    // the other moods and the nap show their screen N too (0 = their first one)
    s.looks = [o.screen.unwrap_or(0) % scene::LOOKS; scene::LOOK_SLOTS];
    s.egg = o.egg;
    // --eggpick N: which of a mood's extras (not in the usage text)
    s.egg_pick = o.egg_pick.unwrap_or(0);
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
    let bg = match &o.bg {
        Some(hex) => {
            let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).map_err(|_| "bad --bg")?;
            Some(tiny_skia::Color::from_rgba8((v >> 16) as u8, (v >> 8) as u8, v as u8, 255))
        }
        None => None,
    };
    let save = |pix: &mut Pixmap, t: f64, pose: &scene::Pose, path: &str, s: &scene::Scene| -> Result<(), String> {
        pix.fill(tiny_skia::Color::TRANSPARENT);
        s.render(pix, sc, t, pose);
        let img = match bg {
            Some(col) => {
                let mut b = Pixmap::new(w, h).ok_or("pixmap")?;
                b.fill(col);
                b.draw_pixmap(0, 0, pix.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
                b
            }
            None => pix.clone(),
        };
        img.save_png(path).map_err(|e| e.to_string())
    };
    // --frames N: N frames from --time on, --fps apart (default 25), as OUT-0000.png, OUT-0001.png... (not in the usage
    // text)
    let Some(n) = o.frames else {
        return save(&mut pix, t_end, &pose, &out, &s);
    };
    let step = 1.0 / o.fps.unwrap_or(25.0);
    let stem = out.strip_suffix(".png").unwrap_or(&out).to_string();
    let mut pose = pose;
    for i in 0..n {
        let t = t_end + i as f64 * step;
        if i > 0 {
            pose = s.update(t);
        }
        save(&mut pix, t, &pose, &format!("{stem}-{i:04}.png"), &s)?;
    }
    Ok(())
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

    /// One line to stdout. The exe is a windowed app: piped or captured output (scripts, `$x = ...`, Git Bash) has a
    /// stdout to write to; run straight in a console it has none, so it borrows the parent's console.
    pub fn print(text: &str) {
        use std::io::Write;
        use windows_sys::Win32::System::Console::{AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE};
        let has_out = || unsafe {
            let h = GetStdHandle(STD_OUTPUT_HANDLE);
            !h.is_null() && h != INVALID_HANDLE_VALUE
        };
        if has_out() || (unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } != 0 && has_out()) {
            let mut out = std::io::stdout();
            let _ = writeln!(out, "{text}");
            let _ = out.flush();
        } else if let Ok(mut con) = std::fs::OpenOptions::new().write(true).open("CONOUT$") {
            let _ = writeln!(con, "{text}");
        }
    }

    /// A busy line was written and no kitten is running: start one watching that status file.
    pub fn wake(status: &Path) {
        let c = wide(CLASS);
        if !unsafe { FindWindowW(c.as_ptr(), null()) }.is_null() {
            return;
        }
        let Ok(exe) = std::env::current_exe() else { return };
        keep_pipes();
        use std::process::Stdio;
        let spawn = std::process::Command::new(&exe)
            .arg("--status")
            .arg(std::path::absolute(status).unwrap_or_else(|_| status.to_path_buf()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Err(e) = spawn {
            log(&format!("wake: could not start the kitten: {e}"));
        }
    }

    /// `do`: hand action `i` (of ACTIONS) and its text to the running kitten. An error when none runs.
    pub fn act(i: usize, text: &str) -> Result<(), String> {
        use windows_sys::Win32::System::DataExchange::COPYDATASTRUCT;
        let c = wide(CLASS);
        let h = unsafe { FindWindowW(c.as_ptr(), null()) };
        if h.is_null() {
            return Err("no kitten is running (start it, or `say` something first)".into());
        }
        let cd = COPYDATASTRUCT { dwData: i, cbData: text.len() as u32, lpData: text.as_ptr() as *mut c_void };
        let mut answer = 0usize;
        let sent = unsafe {
            SendMessageTimeoutW(h, WM_COPYDATA, 0, &cd as *const _ as LPARAM, SMTO_ABORTIFHUNG, 2000, &mut answer)
        };
        match (sent, answer) {
            (0, _) => Err("the kitten did not answer in time".into()),
            (_, 0) => Err("this kitten is too old for `do`: update it (kitten-assistant update)".into()),
            _ => Ok(()),
        }
    }

    /// `wrap`: `line` up on session `name` while `cmd` runs on the caller's console, then idle (exit 0) or a Failed
    /// line. Returns the program's exit code.
    pub fn wrap(status: &Path, name: &str, line: &str, cmd: &[String], opts: &LineOpts) -> Result<i32, String> {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Console::{AttachConsole, GetStdHandle, SetConsoleCtrlHandler, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE};
        let has_out = || unsafe {
            let h = GetStdHandle(STD_OUTPUT_HANDLE);
            !h.is_null() && h != INVALID_HANDLE_VALUE
        };
        // the program prints to the caller's console; with no console at all it gets no window either
        let console = has_out() || unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } != 0;
        // Ctrl+C stops the program, not the wrapper, so the line is still set right afterwards
        unsafe { SetConsoleCtrlHandler(None, 1) };
        say_line(status, Some(name), line, opts)?;
        wake(status);
        let mut c = std::process::Command::new(&cmd[0]);
        c.args(&cmd[1..]);
        if !console {
            c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let code = match c.status() {
            Ok(st) => st.code(),
            Err(e) => {
                sessions::say(status, name, &format!("Failed: could not start {} ({e})", cmd[0]))?;
                return Err(format!("could not start {}: {e}", cmd[0]));
            }
        };
        // 0xC000013A: the program was stopped with Ctrl+C
        let code = code.filter(|c| *c as u32 != 0xC000_013A);
        match code {
            Some(0) => sessions::idle(status, name)?,
            other => sessions::say(status, name, &wrap_failed(line, other))?,
        }
        Ok(code.unwrap_or(1))
    }

    static STOP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    unsafe extern "system" fn on_ctrl(_: u32) -> windows_sys::core::BOOL {
        STOP.store(true, std::sync::atomic::Ordering::SeqCst);
        1
    }

    /// `watch` and `wrap --pid`: keep session `name`'s line current until the process ends (`pid`), Ctrl+C, or
    /// someone sends `idle` for the session. With a log, progress (and with `tail` the newest line) come from it.
    /// Every write lasts 60 s (`until`), so a watcher that dies cannot leave its line up. Returns the exit code to
    /// pass on (the process's own when it is known).
    pub fn watch(status: &Path, name: &str, line: &str, log: Option<(PathBuf, Option<String>, bool)>, base: &LineOpts, pid: Option<u32>) -> Result<i32, String> {
        use std::sync::atomic::Ordering;
        use windows_sys::Win32::System::Console::{AttachConsole, SetConsoleCtrlHandler, ATTACH_PARENT_PROCESS};
        use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, WaitForSingleObject};
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
            SetConsoleCtrlHandler(Some(on_ctrl), 1);
        }
        // SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION: enough to wait on it and read its exit code
        let proc = match pid {
            Some(p) => {
                let h = unsafe { OpenProcess(0x0010_0000 | 0x1000, 0, p) };
                if h.is_null() {
                    return Err(format!("no process {p} to watch (ended, or not ours to see)"));
                }
                Some(h)
            }
            None => None,
        };
        let (mut tail, matching, show) = match log {
            Some((p, m, show)) => (Some(watch::Tail::new(p, false)), m, show),
            None => (None, None, false),
        };
        let (mut last, mut progress) = (None::<String>, None::<sessions::Progress>);
        let (mut written, mut wrote_at) = (String::new(), 0.0);
        let file = sessions::file(status, name);
        let mut first = true;
        let result = loop {
            if let Some(t) = tail.as_mut() {
                for l in t.read() {
                    if matching.as_deref().is_none_or(|m| l.contains(m)) {
                        if let Some(p) = watch::progress_in(&l) {
                            progress = Some(p);
                        }
                        last = Some(l);
                    }
                }
            }
            // someone said idle for this session: the watching ends
            if !first && !file.exists() {
                break Ok(0);
            }
            let text = watch::line_text(line, show, last.as_deref());
            let mut opts = base.clone();
            opts.progress = progress.clone().or(base.progress.clone());
            opts.for_secs = Some(60.0);
            let now = sessions::now();
            let what = format!("{text}\n{:?}", opts.progress.as_ref().map(|p| &p.label));
            if what != written || now - wrote_at > 20.0 {
                say_line(status, Some(name), &text, &opts)?;
                if first {
                    wake(status);
                    first = false;
                }
                written = what;
                wrote_at = now;
            }
            if STOP.load(Ordering::SeqCst) {
                sessions::idle(status, name)?;
                break Ok(130);
            }
            match proc {
                Some(h) if unsafe { WaitForSingleObject(h, 2000) } == 0 => {
                    let mut code = 0u32;
                    let known = unsafe { GetExitCodeProcess(h, &mut code) } != 0;
                    let code = code as i32;
                    if !known || code == 0 {
                        sessions::idle(status, name)?;
                    } else {
                        let failed = if code as u32 == 0xC000_013A { wrap_failed(line, None) } else { wrap_failed(line, Some(code)) };
                        sessions::say(status, name, &failed)?;
                    }
                    break Ok(if known { code } else { 0 });
                }
                Some(_) => {}
                None => {
                    for _ in 0..20 {
                        if STOP.load(Ordering::SeqCst) {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        };
        if let Some(h) = proc {
            unsafe { CloseHandle(h) };
        }
        result
    }

    /// `watch --background`: the same command again, without `--background` and with the session named, as a
    /// process of its own with no window; this one returns at once.
    pub fn detach(name: &str) -> Result<(), String> {
        use std::os::windows::process::CommandExt;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        keep_pipes();
        let mut args: Vec<String> = std::env::args().skip(1).filter(|a| a != "--background").collect();
        if !args.iter().any(|a| a == "--session") {
            args.extend(["--session".to_string(), name.to_string()]);
        }
        let child = std::process::Command::new(exe)
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(0x0000_0008 | 0x0800_0000) // DETACHED_PROCESS | CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("could not start the watcher: {e}"))?;
        print(&format!("watching in the background (process {}); stop it with: kitten-assistant idle --session \"{name}\"", child.id()));
        Ok(())
    }

    /// Files a click may open with their own app; anything else is only shown in Explorer, never run.
    const OPENABLE: [&str; 13] = ["log", "txt", "md", "json", "csv", "xml", "yaml", "yml", "ini", "html", "htm", "png", "pdf"];

    /// A bubble's `--open`: a web page in the browser, a folder in Explorer, a log or text file in its app, any other
    /// file selected in Explorer.
    pub fn open_target(target: &str) {
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        let t = target.trim();
        let lower = t.to_ascii_lowercase();
        let p = Path::new(t);
        let (file, params) = if lower.starts_with("https://") || lower.starts_with("http://") {
            (t.to_string(), None)
        } else if p.is_dir() {
            (t.to_string(), None)
        } else if p.is_file() && p.extension().is_some_and(|e| OPENABLE.contains(&e.to_string_lossy().to_ascii_lowercase().as_str())) {
            (t.to_string(), None)
        } else if p.exists() {
            ("explorer.exe".to_string(), Some(format!("/select,\"{t}\"")))
        } else {
            log(&format!("open: nothing at {t}"));
            return;
        };
        let (verb, file) = (wide("open"), wide(&file));
        let params = params.map(|s| wide(&s));
        unsafe {
            ShellExecuteW(null_mut(), verb.as_ptr(), file.as_ptr(), params.as_ref().map_or(null(), |p| p.as_ptr()), null(), SW_SHOWNORMAL);
        }
    }

    /// The caller's pipes must not leak into a process that outlives this call (the kitten, a background watcher), or
    /// `... | Out-Null` waits until that process ends.
    fn keep_pipes() {
        use windows_sys::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
        for which in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            unsafe {
                let h = GetStdHandle(which);
                if !h.is_null() && h != INVALID_HANDLE_VALUE {
                    SetHandleInformation(h, HANDLE_FLAG_INHERIT, 0);
                }
            }
        }
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
        /// the shown line (session and text) and what a click on its bubble opens
        shown: Option<String>,
        open: Option<String>,
        /// lines whose `--attention` was clicked away
        acked: std::collections::HashSet<String>,
        /// the shown line's progress when first seen (line, time, fraction), for the time-left estimate
        eta_from: Option<(String, f64, f32)>,
        /// the timers' lines (name -> text, end time), to cheer when one runs out
        timers: std::collections::HashMap<String, (String, f64)>,
        history: history::Tracker,
        /// the line a sound was last played for
        sounded: Option<String>,
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
            self.first_poll = false;
            // every program's line: the one that matters most shows, its own age keeps the bubble's clock honest
            let lines = sessions::read_all(&self.status, read_status, status_age, sessions::now());
            let (status, age, others) = match sessions::pick(&lines) {
                Some((l, n)) => (l.text.clone(), l.age, n),
                None => (String::from("idle"), 0.0, 0),
            };
            self.scene.progress = sessions::pick(&lines).and_then(|(l, _)| l.meta.progress.as_ref()).map(|p| (p.frac, p.label.clone()));
            self.scene.others = others;
            // the other lines, newest first, for the bubble's "+N more" rotation
            let top = sessions::pick(&lines).map(|(l, _)| l.clone());
            let mut rest: Vec<&sessions::Line> = lines.iter().filter(|l| Some(*l) != top.as_ref()).collect();
            rest.sort_by(|a, b| a.age.total_cmp(&b.age));
            self.scene.other_lines = rest.into_iter().map(|l| (l.name.clone(), l.text.clone())).collect();
            self.scene.set_status_mood(&status, t, age, top.as_ref().and_then(|l| l.meta.mood));
            self.scene.auto_rotate(t);
            self.last_status = status;
            self.extras(&lines, top.as_ref(), t);
            sessions::sweep(&self.status, sessions::now());
            // a gentle frame rate whatever has the focus: smooth enough while typing, lighter while napping
            let ms = if self.scene.busy { 33 } else { 66 };
            if ms != self.frame_ms {
                self.frame_ms = ms;
                // a little slack lets Windows fold the tick into other wake-ups
                unsafe { SetCoalescableTimer(self.hwnd, FRAME, ms, None, ms / 8) };
            }
        }

        /// The shown line's checklist, hop, link, timer and time left; timers that ran out; the history; the sound.
        fn extras(&mut self, lines: &[sessions::Line], top: Option<&sessions::Line>, t: f64) {
            let now = sessions::now();
            let key = top.map(|l| format!("{}\n{}", l.name, l.text));
            self.scene.steps = top.filter(|l| !l.meta.steps.is_empty()).map(|l| {
                let n = l.meta.steps.len();
                (l.meta.steps.clone(), l.meta.step.unwrap_or(1).clamp(1, n) - 1)
            });
            self.scene.attention = top.is_some_and(|l| l.meta.attention) && key.as_ref().is_some_and(|k| !self.acked.contains(k));
            self.open = top.and_then(|l| l.meta.open.clone());
            self.scene.clickable = self.scene.attention || self.open.is_some();
            self.scene.countdown_end = top.filter(|l| l.meta.countdown).and_then(|l| l.meta.until).map(|u| t + (u - now));
            // time left: from how far the progress moved since the line was first seen
            let progress = top.and_then(|l| l.meta.progress.as_ref()).map(|p| p.frac);
            self.scene.eta = match (&key, progress) {
                (Some(k), Some(f)) => match &self.eta_from {
                    Some((k0, t0, f0)) if k0 == k && f >= *f0 => sessions::eta(*t0, *f0 as f64, t, f as f64),
                    _ => {
                        self.eta_from = Some((k.clone(), t, f));
                        None
                    }
                },
                _ => {
                    self.eta_from = None;
                    None
                }
            };
            // a timer that ran out (not one stopped early) gets its cheer
            let live: std::collections::HashMap<String, (String, f64)> = lines
                .iter()
                .filter(|l| l.meta.countdown)
                .filter_map(|l| l.meta.until.map(|u| (l.name.clone(), (l.text.clone(), u))))
                .collect();
            for (name, (text, until)) in std::mem::take(&mut self.timers) {
                if !live.contains_key(&name) && now >= until - 3.0 {
                    self.scene.cheer_now(t, Some(&format!("{text}: time's up!")));
                }
            }
            self.timers = live;
            if setting("history").as_deref() == Some("1") {
                let rows = self.history.update(lines, now);
                if let Err(e) = history::append(&history::path(&self.status), &rows) {
                    log(&format!("history: {e}"));
                }
            } else {
                self.history = Default::default();
            }
            // a sound when a problem or a question becomes the shown line (right-click > Sounds)
            if key != self.sounded {
                if let Some(l) = top.filter(|_| setting("sound").as_deref() == Some("1")) {
                    match l.mood() {
                        scene::Mood::Problem => unsafe { windows_sys::Win32::System::Diagnostics::Debug::MessageBeep(MB_ICONEXCLAMATION) },
                        scene::Mood::Question => unsafe { windows_sys::Win32::System::Diagnostics::Debug::MessageBeep(MB_ICONASTERISK) },
                        _ => 1,
                    };
                }
                self.sounded = key.clone();
            }
            self.shown = key;
        }

        /// The bubble was clicked: its hop stops for this line, and its link opens (handed back, to open outside the
        /// app's borrow).
        fn click_bubble(&mut self) -> Option<String> {
            if let Some(k) = &self.shown {
                self.acked.insert(k.clone());
            }
            self.scene.attention = false;
            self.scene.clickable = self.open.is_some();
            self.open.clone()
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

    pub(crate) fn setting(key: &str) -> Option<String> {
        let text = std::fs::read_to_string(settings_path()?).ok()?;
        text.lines().find_map(|l| l.trim().strip_prefix(key)?.trim_start().strip_prefix('=').map(|v| v.trim().to_string()))
    }

    pub(crate) fn save_setting(key: &str, value: &str) {
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
            sound: setting("sound").as_deref() == Some("1"),
            history: setting("history").as_deref() == Some("1"),
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

    /// Right-click > What ran today: today's history as a text file, opened in Notepad.
    fn show_today() {
        let status = APP.with(|a| a.try_borrow().ok().and_then(|g| g.as_ref().map(|app| app.status.clone())));
        let Some(status) = status else { return };
        let rows = history::read(&history::path(&status));
        let mut text = format!("Kitten Assistant: what ran today\r\n\r\n{}\r\n", history::summary(&rows, history::local_midnight(sessions::now(), 0)).replace('\n', "\r\n"));
        if setting("history").as_deref() != Some("1") {
            text += "\r\nThe history is off: right-click the kitten > Keep a history to turn it on.\r\n";
        }
        let p = std::env::temp_dir().join("kitten-assistant-today.txt");
        if let Err(e) = std::fs::write(&p, text) {
            log(&format!("today: {e}"));
            return;
        }
        let _ = std::process::Command::new("notepad.exe").arg(&p).spawn();
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
            menu::Action::Sound => {
                let on = setting("sound").as_deref() == Some("1");
                save_setting("sound", if on { "0" } else { "1" });
            }
            menu::Action::History => {
                let on = setting("history").as_deref() == Some("1");
                save_setting("history", if on { "0" } else { "1" });
            }
            menu::Action::Today => show_today(),
            menu::Action::CheckUpdate => check_from_menu(hwnd),
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
            // the whole kitten drags like a title bar (see-through pixels click through), except the bubble's
            // "+N more", which takes a click
            WM_NCHITTEST => {
                let r = DefWindowProcW(hwnd, msg, wp, lp);
                if r != HTCLIENT as LRESULT {
                    return r;
                }
                let mut p = POINT { x: (lp & 0xFFFF) as i16 as i32, y: ((lp >> 16) & 0xFFFF) as i16 as i32 };
                ScreenToClient(hwnd, &mut p);
                let more = APP
                    .with(|a| {
                        a.try_borrow().ok().and_then(|g| {
                            g.as_ref().map(|app| {
                                let (x, y) = (p.x as f32 / app.sc, p.y as f32 / app.sc);
                                app.scene.more_hit(x, y) || app.scene.bubble_hit(x, y)
                            })
                        })
                    })
                    .unwrap_or(false);
                if more { HTCLIENT as LRESULT } else { HTCAPTION as LRESULT }
            }
            WM_SETCURSOR if (lp & 0xFFFF) as u32 == HTCLIENT => {
                SetCursor(LoadCursorW(null_mut(), IDC_HAND));
                1
            }
            WM_LBUTTONUP => {
                let (x, y) = ((lp & 0xFFFF) as i16 as f32, ((lp >> 16) & 0xFFFF) as i16 as f32);
                let mut open = None;
                with_app(|a| {
                    let t = a.t();
                    let (sx, sy) = (x / a.sc, y / a.sc);
                    if a.scene.more_hit(sx, sy) {
                        a.scene.click_more(t);
                    } else if a.scene.bubble_hit(sx, sy) {
                        open = a.click_bubble();
                    }
                });
                if let Some(target) = open {
                    open_target(&target);
                }
                0
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
            WM_UP_TO_DATE => {
                with_app(|a| {
                    let t = a.t();
                    a.scene.show_up_to_date(t);
                });
                0
            }
            // `do ACTION [TEXT]` from another program: dwData is the action, the data its text (UTF-8)
            WM_COPYDATA => {
                use windows_sys::Win32::System::DataExchange::COPYDATASTRUCT;
                let cd = &*(lp as *const COPYDATASTRUCT);
                if cd.dwData < ACTIONS.len() && cd.cbData <= 4096 {
                    let bytes = if cd.lpData.is_null() || cd.cbData == 0 {
                        &[][..]
                    } else {
                        std::slice::from_raw_parts(cd.lpData as *const u8, cd.cbData as usize)
                    };
                    let text = String::from_utf8_lossy(bytes).trim().to_string();
                    let text = (!text.is_empty()).then_some(text.as_str());
                    let action = ACTIONS[cd.dwData];
                    with_app(|a| {
                        let t = a.t();
                        match action {
                            "sip" => a.scene.brain.force(brain::Event::Sip, t),
                            "refill" => a.scene.brain.force(brain::Event::Refill, t),
                            "stretch" => a.scene.brain.force(brain::Event::Stretch, t),
                            "yawn" => a.scene.brain.force(brain::Event::Yawn, t),
                            "look" => a.scene.brain.force(brain::Event::LookAround, t),
                            "wave" => a.scene.wave(t, text),
                            _ => a.scene.cheer_now(t, text),
                        }
                    });
                    1
                } else {
                    0
                }
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
    /// A manual check found nothing newer (posted by the update thread): the kitten shows "up to date".
    const WM_UP_TO_DATE: u32 = WM_APP + 3;
    static MUTEX: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    /// Builds with a build number only; off with `auto_update=0` in kitten-assistant.ini.
    fn auto_update() -> bool {
        update::updates_itself() && setting("auto_update").map_or(true, |v| v != "0")
    }

    const NO_BUILD_NUMBER: &str = "This is a dev build (build 0), so it does not update itself. Build it with tools\\build-local.ps1 to give it a build number.";

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

    /// Right-click > Check for Update: checks now on a background thread and says what it found; a newer release
    /// is downloaded, verified and swapped in like an automatic update.
    fn check_from_menu(hwnd: HWND) {
        if !update::updates_itself() {
            let _ = std::thread::Builder::new().name("update-msg".into()).spawn(|| message(NO_BUILD_NUMBER));
            return;
        }
        let at = hwnd as usize;
        let _ = std::thread::Builder::new().name("update".into()).spawn(move || match update::check_and_stage() {
            Ok(Some(tag)) => {
                log(&format!("update {tag} downloaded and verified (running {})", update::label()));
                unsafe { PostMessageW(at as HWND, WM_UPDATE_READY, 0, 0) };
            }
            Ok(None) => {
                log(&format!("update check: {} is up to date", update::label()));
                unsafe { PostMessageW(at as HWND, WM_UP_TO_DATE, 0, 0) };
            }
            Err(e) => {
                log(&format!("update check: {e}"));
                message(&format!("Could not check for updates: {e}"));
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
        if !update::updates_itself() {
            message(NO_BUILD_NUMBER);
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
            // a hello wave every time it starts (the status shows once it is done)
            scene.greet(0.0);
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
                shown: None,
                open: None,
                acked: Default::default(),
                eta_from: None,
                timers: Default::default(),
                history: Default::default(),
                sounded: None,
            };
            if setting("history").as_deref() == Some("1") {
                history::trim(&history::path(&app.status), sessions::now());
            }
            // the local endpoint, when asked for (off by default)
            if let Some(addr) = o.listen.clone().or_else(|| setting("listen")).filter(|a| !a.is_empty()) {
                if let Err(e) = listen::start(&addr, app.status.clone()) {
                    log(&e);
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_strings_escape_what_json_needs() {
        assert_eq!(json_str("a \"b\" \\ c\n\t\u{1}"), r#""a \"b\" \\ c\n\t\u0001""#);
        assert_eq!(json_str("caf\u{e9} \u{b7}"), "\"caf\u{e9} \u{b7}\"");
    }

    #[test]
    fn status_json_lists_the_shown_line_first() {
        let line = |name: &str, text: &str, meta| sessions::Line { name: name.into(), text: text.into(), age: 12.4, meta };
        let lines = [
            line("ci", "Failed: tests", sessions::Meta::default()),
            line("", "Copying: photos", sessions::Meta { until: Some(160.0), progress: sessions::Progress::parse("3/10"), ..Default::default() }),
        ];
        assert_eq!(
            status_json(&lines, 100.0),
            concat!(
                r#"{"idle":false,"lines":[{"session":"ci","text":"Failed: tests","mood":"problem","age_secs":12,"progress":null,"#,
                r#""progress_label":null,"expires_in_secs":null,"shown":true},{"session":null,"text":"Copying: photos","#,
                r#""mood":"downloading","age_secs":12,"progress":0.300,"progress_label":"3/10","expires_in_secs":60,"shown":false}]}"#
            )
        );
        assert_eq!(status_json(&[], 0.0), r#"{"idle":true,"lines":[]}"#);
    }

    #[test]
    fn a_failed_wrap_says_what_failed() {
        assert_eq!(wrap_failed("Building: the app", Some(2)), "Failed: the app (exit 2)");
        assert_eq!(wrap_failed("Running: cargo test", Some(101)), "Failed: Running: cargo test (exit 101)");
        assert_eq!(wrap_failed("Testing: suite", None), "Failed: suite (canceled)");
        assert_eq!(wrap_failed("Note: x", Some(1)), "Failed: Note: x (exit 1)");
    }
}
