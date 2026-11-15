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

/// `say` for a session.
pub fn say(status: &Path, name: &str, text: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir(status)).map_err(|e| e.to_string())?;
    std::fs::write(file(status, name), text).map_err(|e| e.to_string())
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
}

fn busy(text: &str) -> bool {
    !text.is_empty() && !text.eq_ignore_ascii_case("idle")
}

fn rank(text: &str) -> u8 {
    match Mood::of(text) {
        Mood::Problem => 0,
        Mood::Waiting => 2,
        _ => 1,
    }
}

/// Every busy line: the plain status file and each fresh session file.
pub fn read_all(status: &Path, read: impl Fn(&Path) -> String, age: impl Fn(&Path) -> f64) -> Vec<Line> {
    let mut lines = Vec::new();
    let text = read(status);
    if busy(&text) {
        lines.push(Line { name: String::new(), text, age: age(status) });
    }
    if let Ok(entries) = std::fs::read_dir(dir(status)) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().is_none_or(|x| x != "txt") {
                continue;
            }
            let (text, a) = (read(&p), age(&p));
            if busy(&text) && a < STALE_SECS {
                let name = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                lines.push(Line { name, text, age: a });
            }
        }
    }
    lines
}

/// The line to show and how many other busy lines there are.
pub fn pick(lines: &[Line]) -> Option<(&Line, usize)> {
    let best = lines.iter().min_by(|a, b| {
        rank(&a.text).cmp(&rank(&b.text)).then(a.age.total_cmp(&b.age)).then(a.name.cmp(&b.name))
    })?;
    Some((best, lines.len() - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(name: &str, text: &str, age: f64) -> Line {
        Line { name: name.into(), text: text.into(), age }
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
        let read = |p: &Path| std::fs::read_to_string(p).unwrap_or_default().trim().to_string();
        let all = read_all(&status, read, |_| 0.0);
        assert_eq!(all.len(), 2);
        assert_eq!(pick(&all).map(|(p, n)| (p.text.as_str(), n)), Some(("Building the app", 1)));
        idle(&status, "one").unwrap();
        idle(&status, "one").unwrap();
        let all = read_all(&status, read, |_| 0.0);
        assert_eq!(all.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(), ["two"]);
        assert!(read_all(&status, read, |_| STALE_SECS + 1.0).is_empty(), "stale lines are dropped");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
