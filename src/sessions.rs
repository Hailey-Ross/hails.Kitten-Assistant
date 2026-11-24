//! Several programs at once: each one names itself (`--session NAME` or %KITTEN_SESSION%) and gets its own status
//! file in `sessions\` next to the status file. The kitten shows the line that matters most (a problem first, then
//! work, then waiting; the newest of those) and counts the others. Without a name, the old single status file.

use std::path::{Path, PathBuf};

use crate::scene::Mood;

/// A session line nobody has touched for this long is left over from a program that ended without `idle`.
pub const STALE_SECS: f64 = 3.0 * 3600.0;

pub fn dir(status: &Path) -> PathBuf {
    status.parent().map_or_else(|| PathBuf::from("sessions"), |d| d.join("sessions"))
}

/// Letters, digits, `.`, `-`, `_` and spaces become the file name; anything else is a `_`. None when empty.
pub fn clean(name: &str) -> Option<String> {
    let s: String = name
        .trim()
        .chars()
        .take(64)
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') { c } else { '_' })
        .collect();
    let s = s.trim().trim_matches('.').to_string();
    (!s.is_empty()).then_some(s)
}

pub fn file(status: &Path, name: &str) -> PathBuf {
    dir(status).join(format!("{name}.txt"))
}

/// `say` for a session: `body` is the text, then any `key=value` lines (see `Meta`).
pub fn say(status: &Path, name: &str, body: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir(status)).map_err(|e| e.to_string())?;
    std::fs::write(file(status, name), body).map_err(|e| e.to_string())
}

/// Every session line goes away, and the plain status file says idle (`sessions clear`). Returns how many session
/// lines there were.
pub fn clear_all(status: &Path) -> Result<usize, String> {
    let mut n = 0;
    if let Ok(entries) = std::fs::read_dir(dir(status)) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "txt") && std::fs::remove_file(&p).is_ok() {
                n += 1;
            }
        }
    }
    std::fs::write(status, "idle").map_err(|e| e.to_string())?;
    Ok(n)
}

/// Deletes the session files whose `until` has passed (the kitten already ignores them; this keeps the folder tidy).
pub fn sweep(status: &Path, now: f64) {
    let Ok(entries) = std::fs::read_dir(dir(status)) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "txt") {
            let (_, m) = split(&std::fs::read_to_string(&p).unwrap_or_default());
            if m.until.is_some_and(|u| now >= u + 5.0) {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
}

/// How far along a line is: 0..1 for the bar, and the label as given (`3/10`, `45%`).
#[derive(Clone, Debug, PartialEq)]
pub struct Progress {
    pub frac: f32,
    pub label: String,
}

impl Progress {
    /// `3/10`, `45%` or `0.45`; None when it is none of those.
    pub fn parse(s: &str) -> Option<Progress> {
        let s = s.trim();
        let frac = if let Some((a, b)) = s.split_once('/') {
            let (a, b): (f64, f64) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
            if b <= 0.0 || a < 0.0 {
                return None;
            }
            a / b
        } else if let Some(p) = s.strip_suffix('%') {
            p.trim().parse::<f64>().ok().filter(|p| *p >= 0.0)? / 100.0
        } else {
            s.parse::<f64>().ok().filter(|f| (0.0..=1.0).contains(f))?
        };
        if !frac.is_finite() {
            return None;
        }
        let label = if s.contains('/') || s.ends_with('%') { s.to_string() } else { format!("{:.0}%", frac * 100.0) };
        Some(Progress { frac: frac.clamp(0.0, 1.0) as f32, label })
    }
}

/// The lines after a status line's text: `until=<unix seconds>` (the line clears itself then) and
/// `progress=<3/10 or 45%>`. Older kittens read only the first line, so these never confuse them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meta {
    pub until: Option<f64>,
    pub progress: Option<Progress>,
    /// `mood=testing`: the mood whatever the first word says
    pub mood: Option<Mood>,
    /// `open=URL or path`: a click on the bubble opens it
    pub open: Option<String>,
    /// `attention=1`: the bubble hops until it is clicked
    pub attention: bool,
    /// `steps=fetch,build,test` and `step=2` (1-based): a checklist
    pub steps: Vec<String>,
    pub step: Option<usize>,
    /// `countdown=1`: the bubble counts down to `until` (a timer) instead of counting up
    pub countdown: bool,
}

impl Meta {
    pub fn parse(rest: &str) -> Meta {
        let mut m = Meta::default();
        for l in rest.lines() {
            match l.split_once('=').map(|(k, v)| (k.trim(), v.trim())) {
                Some(("until", v)) => m.until = v.parse().ok().filter(|f: &f64| f.is_finite()),
                Some(("progress", v)) => m.progress = Progress::parse(v),
                Some(("mood", v)) => m.mood = Mood::named(v),
                Some(("open", v)) if !v.is_empty() => m.open = Some(v.to_string()),
                Some(("attention", v)) => m.attention = matches!(v, "1" | "true" | "yes"),
                Some(("steps", v)) => m.steps = parse_steps(v),
                Some(("step", v)) => m.step = v.parse().ok().filter(|s| *s >= 1),
                Some(("countdown", v)) => m.countdown = matches!(v, "1" | "true" | "yes"),
                _ => {}
            }
        }
        m
    }

    /// `text` with these lines after it, as a status file holds it.
    pub fn body(&self, text: &str) -> String {
        let mut b = text.trim().to_string();
        let one_line = |s: &str| s.replace(['\r', '\n'], " ");
        if let Some(u) = self.until {
            b += &format!("\nuntil={u:.0}");
        }
        if let Some(p) = &self.progress {
            b += &format!("\nprogress={}", p.label);
        }
        if let Some(m) = self.mood {
            b += &format!("\nmood={}", m.name());
        }
        if let Some(o) = &self.open {
            b += &format!("\nopen={}", one_line(o));
        }
        if self.attention {
            b += "\nattention=1";
        }
        if !self.steps.is_empty() {
            b += &format!("\nsteps={}", one_line(&self.steps.join(",")));
        }
        if let Some(s) = self.step {
            b += &format!("\nstep={s}");
        }
        if self.countdown {
            b += "\ncountdown=1";
        }
        b
    }
}

/// About how many seconds are left, from progress `f0` at time `t0` to `f` at `t`: None until it has moved for 10 s,
/// when it has not moved, is done, or would take longer than 100 hours.
pub fn eta(t0: f64, f0: f64, t: f64, f: f64) -> Option<f64> {
    if t - t0 < 10.0 || f <= f0 || f >= 1.0 {
        return None;
    }
    let left = (1.0 - f) / ((f - f0) / (t - t0));
    (left.is_finite() && left < 360_000.0).then_some(left)
}

/// `fetch, build,test` -> the step names, empty ones left out.
pub fn parse_steps(s: &str) -> Vec<String> {
    s.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).take(32).collect()
}

/// `90`, `30s`, `5m`, `1h`, `1h30m`, `2m30s`: seconds; None when it is not one of those.
pub fn parse_duration(s: &str) -> Option<f64> {
    let s = s.trim().to_ascii_lowercase();
    if let Ok(n) = s.parse::<f64>() {
        return (n.is_finite() && n > 0.0).then_some(n);
    }
    let (mut total, mut num) = (0.0, String::new());
    for c in s.chars() {
        if c.is_ascii_digit() || c == '.' {
            num.push(c);
            continue;
        }
        let unit = match c {
            'h' => 3600.0,
            'm' => 60.0,
            's' => 1.0,
            _ => return None,
        };
        total += num.parse::<f64>().ok()? * unit;
        num.clear();
    }
    (num.is_empty() && total > 0.0).then_some(total)
}

/// Seconds since 1970 (the `until=` clock).
pub fn now() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64())
}

/// `idle` for a session: its line goes away.
pub fn idle(status: &Path, name: &str) -> Result<(), String> {
    match std::fs::remove_file(file(status, name)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// the session's name; empty for the plain status file
    pub name: String,
    pub text: String,
    /// seconds since it was written
    pub age: f64,
    pub meta: Meta,
}

impl Line {
    /// The session's name, or `(plain)` for the plain status file.
    pub fn who(&self) -> &str {
        if self.name.is_empty() { "(plain)" } else { &self.name }
    }

    /// Its mood: `mood=` when given, else its first word.
    pub fn mood(&self) -> Mood {
        self.meta.mood.unwrap_or_else(|| Mood::of(&self.text))
    }
}

fn busy(text: &str) -> bool {
    !text.is_empty() && !text.eq_ignore_ascii_case("idle")
}

/// A problem or a question first (they need the user), then work, then waiting.
fn rank(l: &Line) -> u8 {
    match l.mood() {
        Mood::Problem | Mood::Question => 0,
        Mood::Waiting => 2,
        _ => 1,
    }
}

/// A status file's text (its first line) and the `key=value` lines after it.
pub fn split(content: &str) -> (String, Meta) {
    let content = content.trim_start_matches('\u{feff}');
    let (first, rest) = content.split_once('\n').unwrap_or((content, ""));
    (first.trim().to_string(), Meta::parse(rest))
}

/// Every busy line: the plain status file and each fresh session file. `read` gives a file's whole content (empty
/// when it is missing), `now` the `until=` clock: lines past their end time are left out.
pub fn read_all(status: &Path, read: impl Fn(&Path) -> String, age: impl Fn(&Path) -> f64, now: f64) -> Vec<Line> {
    let mut lines = Vec::new();
    let live = |text: &str, m: &Meta| busy(text) && m.until.is_none_or(|u| now < u);
    let (text, meta) = split(&read(status));
    if live(&text, &meta) {
        lines.push(Line { name: String::new(), text, age: age(status), meta });
    }
    if let Ok(entries) = std::fs::read_dir(dir(status)) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().is_none_or(|x| x != "txt") {
                continue;
            }
            let ((text, meta), a) = (split(&read(&p)), age(&p));
            if live(&text, &meta) && a < STALE_SECS {
                let name = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                lines.push(Line { name, text, age: a, meta });
            }
        }
    }
    lines
}

/// The line to show and how many other busy lines there are.
pub fn pick(lines: &[Line]) -> Option<(&Line, usize)> {
    let best = lines.iter().min_by(|a, b| {
        rank(a).cmp(&rank(b)).then(a.age.total_cmp(&b.age)).then(a.name.cmp(&b.name))
    })?;
    Some((best, lines.len() - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(name: &str, text: &str, age: f64) -> Line {
        Line { name: name.into(), text: text.into(), age, meta: Meta::default() }
    }

    #[test]
    fn a_problem_beats_work_and_work_beats_waiting() {
        let lines = [l("a", "Waiting: CI", 1.0), l("b", "Building the app", 50.0), l("c", "Failed: tests", 90.0)];
        assert_eq!(pick(&lines).map(|(p, n)| (p.name.as_str(), n)), Some(("c", 2)));
        assert_eq!(pick(&lines[..2]).map(|(p, n)| (p.name.as_str(), n)), Some(("b", 1)));
    }

    #[test]
    fn the_newest_of_the_same_kind_shows() {
        let lines = [l("a", "Building one", 30.0), l("b", "Testing two", 2.0)];
        assert_eq!(pick(&lines).map(|(p, _)| p.name.as_str()), Some("b"));
        assert!(pick(&[]).is_none());
    }

    #[test]
    fn names_become_safe_file_names() {
        assert_eq!(clean("skate 3 / wt-challenges").as_deref(), Some("skate 3 _ wt-challenges"));
        assert_eq!(clean("..\\..\\x").as_deref(), Some("_.._x"));
        assert_eq!(clean("  "), None);
    }

    #[test]
    fn sessions_write_their_own_files_and_idle_removes_them() {
        let root = std::env::temp_dir().join(format!("kitten-sessions-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let status = root.join("status.txt");
        std::fs::write(&status, "idle").unwrap();
        say(&status, "one", "Building the app").unwrap();
        say(&status, "two", "Waiting: CI").unwrap();
        let read = |p: &Path| std::fs::read_to_string(p).unwrap_or_default();
        let all = read_all(&status, read, |_| 0.0, 0.0);
        assert_eq!(all.len(), 2);
        assert_eq!(pick(&all).map(|(p, n)| (p.text.as_str(), n)), Some(("Building the app", 1)));
        idle(&status, "one").unwrap();
        idle(&status, "one").unwrap();
        let all = read_all(&status, read, |_| 0.0, 0.0);
        assert_eq!(all.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(), ["two"]);
        assert!(read_all(&status, read, |_| STALE_SECS + 1.0, 0.0).is_empty(), "stale lines are dropped");
        // a line with an end time shows until then, with its progress
        let m = Meta { until: Some(100.0), progress: Progress::parse("3/10"), ..Meta::default() };
        say(&status, "three", &m.body("Copying: photos")).unwrap();
        let at = |now| read_all(&status, read, |_| 0.0, now).into_iter().find(|l| l.name == "three");
        let l = at(99.0).expect("on before its end time");
        assert_eq!((l.text.as_str(), l.meta.progress.map(|p| p.label)), ("Copying: photos", Some("3/10".into())));
        assert!(at(100.0).is_none(), "gone at its end time");
        sweep(&status, 102.0);
        assert!(file(&status, "three").exists(), "a few seconds' grace");
        sweep(&status, 106.0);
        assert!(!file(&status, "three").exists(), "an ended line's file is tidied away");
        assert_eq!(clear_all(&status).unwrap(), 1);
        assert!(read_all(&status, read, |_| 0.0, 0.0).is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn durations_and_progress_parse() {
        assert_eq!(parse_duration("90"), Some(90.0));
        assert_eq!(parse_duration("30s"), Some(30.0));
        assert_eq!(parse_duration("5m"), Some(300.0));
        assert_eq!(parse_duration("1h30m"), Some(5400.0));
        assert_eq!(parse_duration("2M30S"), Some(150.0));
        for bad in ["", "0", "-5", "5x", "m", "1h30", "abc"] {
            assert_eq!(parse_duration(bad), None, "{bad}");
        }
        assert_eq!(Progress::parse("3/10"), Some(Progress { frac: 0.3, label: "3/10".into() }));
        assert_eq!(Progress::parse("45%"), Some(Progress { frac: 0.45, label: "45%".into() }));
        assert_eq!(Progress::parse("0.5"), Some(Progress { frac: 0.5, label: "50%".into() }));
        assert_eq!(Progress::parse("12/10").map(|p| p.frac), Some(1.0));
        for bad in ["3/0", "x/10", "abc", "7", "", "-1/4", "-5%"] {
            assert_eq!(Progress::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn time_left_comes_from_how_fast_progress_moves() {
        assert_eq!(eta(0.0, 0.1, 20.0, 0.3), Some(70.0));
        assert_eq!(eta(0.0, 0.1, 5.0, 0.3), None, "too soon");
        assert_eq!(eta(0.0, 0.3, 20.0, 0.3), None, "no movement");
        assert_eq!(eta(0.0, 0.3, 20.0, 1.0), None, "done");
        assert_eq!(eta(0.0, 0.0, 10.0, 0.000_001), None, "far too slow");
    }

    #[test]
    fn new_fields_round_trip() {
        let m = Meta {
            until: None,
            progress: None,
            mood: Some(Mood::Question),
            open: Some("https://example.com/run/1".into()),
            attention: true,
            steps: parse_steps(" fetch, build ,,test "),
            step: Some(2),
            countdown: true,
        };
        let body = m.body("Deploy");
        assert_eq!(body, "Deploy\nmood=question\nopen=https://example.com/run/1\nattention=1\nsteps=fetch,build,test\nstep=2\ncountdown=1");
        assert_eq!(split(&body), ("Deploy".to_string(), m));
        let l = Line { name: "x".into(), text: "Unit tests: 3 of 9".into(), age: 0.0, meta: Meta { mood: Mood::named("testing"), ..Meta::default() } };
        assert_eq!(l.mood(), Mood::Testing);
        assert_eq!(Mood::named("bogus"), None);
        let q = [l.clone(), Line { name: "q".into(), text: "Question: which branch?".into(), age: 50.0, meta: Meta::default() }];
        assert_eq!(pick(&q).map(|(p, _)| p.name.as_str()), Some("q"), "a question outranks work");
    }

    #[test]
    fn the_text_is_the_first_line_and_old_files_still_read() {
        assert_eq!(split("\u{feff}Building: app\r\n"), ("Building: app".into(), Meta::default()));
        let (t, m) = split("Testing: suite\nuntil=1700000000\nprogress=45%\nsomething=else");
        assert_eq!((t.as_str(), m.until, m.progress.map(|p| p.label)), ("Testing: suite", Some(1.7e9), Some("45%".into())));
        let m = Meta { until: Some(12.4), progress: None, ..Meta::default() };
        assert_eq!(m.body("  Copying  "), "Copying\nuntil=12");
    }
}
