//! A history of the lines the kitten showed (off unless `history=1` in the ini): every line that ends becomes a row
//! of `history.tsv` next to the status file: start and end (Unix seconds), session, mood, text, tab separated.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::sessions::Line;

/// Rows older than this are dropped when the kitten starts.
pub const KEEP_DAYS: f64 = 30.0;

pub fn path(status: &Path) -> PathBuf {
    status.parent().map_or_else(|| PathBuf::from("history.tsv"), |d| d.join("history.tsv"))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub start: f64,
    pub end: f64,
    /// empty for the plain status file
    pub session: String,
    pub mood: String,
    pub text: String,
}

fn clean(s: &str) -> String {
    s.replace(['\t', '\r', '\n'], " ")
}

impl Row {
    pub fn to_tsv(&self) -> String {
        format!("{:.0}\t{:.0}\t{}\t{}\t{}", self.start, self.end, clean(&self.session), clean(&self.mood), clean(&self.text))
    }

    pub fn parse(l: &str) -> Option<Row> {
        let mut f = l.splitn(5, '\t');
        let start = f.next()?.parse().ok()?;
        let end = f.next()?.parse().ok()?;
        Some(Row { start, end, session: f.next()?.into(), mood: f.next()?.into(), text: f.next()?.trim_end().into() })
    }
}

/// Follows the busy lines poll by poll and hands back each one that ended (gone, or its text changed).
#[derive(Default)]
pub struct Tracker {
    open: HashMap<String, Row>,
}

impl Tracker {
    /// `now` in Unix seconds. A line seen for the first time counts from its file's age.
    pub fn update(&mut self, lines: &[Line], now: f64) -> Vec<Row> {
        let mut seen: HashMap<String, &Line> = HashMap::new();
        for l in lines {
            seen.insert(l.name.clone(), l);
        }
        let mut ended = Vec::new();
        self.open.retain(|name, row| match seen.get(name) {
            Some(l) if l.text == row.text => true,
            _ => {
                ended.push(Row { end: now, ..row.clone() });
                false
            }
        });
        for (name, l) in seen {
            self.open.entry(name.clone()).or_insert_with(|| Row {
                start: now - l.age.max(0.0),
                end: now,
                session: name,
                mood: l.mood().name().into(),
                text: l.text.clone(),
            });
        }
        ended.sort_by(|a, b| a.start.total_cmp(&b.start));
        ended
    }
}

pub fn append(path: &Path, rows: &[Row]) -> Result<(), String> {
    use std::io::Write;
    if rows.is_empty() {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(|e| e.to_string())?;
    for r in rows {
        writeln!(f, "{}", r.to_tsv()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn read(path: &Path) -> Vec<Row> {
    std::fs::read_to_string(path).unwrap_or_default().lines().filter_map(Row::parse).collect()
}

/// Drops rows that ended more than `KEEP_DAYS` ago.
pub fn trim(path: &Path, now: f64) {
    let rows = read(path);
    let keep: Vec<&Row> = rows.iter().filter(|r| now - r.end < KEEP_DAYS * 86400.0).collect();
    if keep.len() < rows.len() {
        let body: String = keep.iter().map(|r| r.to_tsv() + "\n").collect();
        let _ = std::fs::write(path, body);
    }
}

/// Seconds since 1970 of the local midnight that starts today, `days_back` days ago (0 = today).
pub fn local_midnight(now: f64, days_back: u32) -> f64 {
    let offset = local_offset();
    let local = now + offset;
    (local - local.rem_euclid(86400.0)) - offset - days_back as f64 * 86400.0
}

/// The local time zone's offset from UTC in seconds (daylight saving included), from Windows.
fn local_offset() -> f64 {
    use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
    use windows_sys::Win32::System::Time::{FileTimeToSystemTime, SystemTimeToFileTime, SystemTimeToTzSpecificLocalTime};
    let as_u64 = |f: &FILETIME| ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64;
    unsafe {
        // now in UTC, turned into local time and back into a FILETIME: the difference is the offset
        let t = (crate::sessions::now() as u64 + 11_644_473_600) * 10_000_000;
        let utc_ft = FILETIME { dwLowDateTime: t as u32, dwHighDateTime: (t >> 32) as u32 };
        let (mut utc, mut local): (SYSTEMTIME, SYSTEMTIME) = (std::mem::zeroed(), std::mem::zeroed());
        let mut local_ft: FILETIME = std::mem::zeroed();
        if FileTimeToSystemTime(&utc_ft, &mut utc) == 0
            || SystemTimeToTzSpecificLocalTime(std::ptr::null(), &utc, &mut local) == 0
            || SystemTimeToFileTime(&local, &mut local_ft) == 0
        {
            return 0.0;
        }
        (as_u64(&local_ft) as f64 - as_u64(&utc_ft) as f64) / 10_000_000.0
    }
}

/// `09:05` local time for Unix seconds.
pub fn clock(t: f64) -> String {
    let local = t + local_offset();
    let m = (local.rem_euclid(86400.0) / 60.0) as i64;
    format!("{:02}:{:02}", m / 60, m % 60)
}

fn length(secs: f64) -> String {
    let m = (secs / 60.0).round() as i64;
    match m {
        0 => "<1 min".into(),
        1..=59 => format!("{m} min"),
        _ => format!("{} h {} min", m / 60, m % 60),
    }
}

/// The rows since `from` as readable lines, oldest first: `09:05-09:40  35 min  [session] text`.
pub fn summary(rows: &[Row], from: f64) -> String {
    let mut out: Vec<String> = rows
        .iter()
        .filter(|r| r.end >= from)
        .map(|r| {
            let who = if r.session.is_empty() { String::new() } else { format!("[{}] ", r.session) };
            format!("{}-{}  {:>9}  {who}{}", clock(r.start), clock(r.end), length(r.end - r.start), r.text)
        })
        .collect();
    if out.is_empty() {
        out.push("nothing yet".into());
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::Meta;

    fn l(name: &str, text: &str, age: f64) -> Line {
        Line { name: name.into(), text: text.into(), age, meta: Meta::default() }
    }

    #[test]
    fn lines_become_rows_when_they_end_or_change() {
        let mut t = Tracker::default();
        assert!(t.update(&[l("a", "Building: app", 5.0), l("b", "Waiting: CI", 0.0)], 100.0).is_empty());
        assert!(t.update(&[l("a", "Building: app", 15.0), l("b", "Waiting: CI", 10.0)], 110.0).is_empty());
        let ended = t.update(&[l("a", "Testing: app", 0.0)], 120.0);
        assert_eq!(
            ended,
            vec![
                Row { start: 95.0, end: 120.0, session: "a".into(), mood: "building".into(), text: "Building: app".into() },
                Row { start: 100.0, end: 120.0, session: "b".into(), mood: "waiting".into(), text: "Waiting: CI".into() },
            ]
        );
        let ended = t.update(&[], 130.0);
        assert_eq!(ended.iter().map(|r| (r.text.as_str(), r.start, r.end)).collect::<Vec<_>>(), [("Testing: app", 120.0, 130.0)]);
    }

    #[test]
    fn rows_round_trip_and_old_ones_go() {
        let r = Row { start: 1.0, end: 2.0, session: "x\ty".into(), mood: "typing".into(), text: "a\tb\nc".into() };
        assert_eq!(Row::parse(&r.to_tsv()), Some(Row { session: "x y".into(), text: "a b c".into(), ..r.clone() }));
        assert_eq!(Row::parse("junk"), None);
        let dir = std::env::temp_dir().join(format!("kitten-history-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("history.tsv");
        let now = 100.0 * 86400.0;
        let old = Row { start: 0.0, end: now - 31.0 * 86400.0, ..r.clone() };
        let new = Row { start: now - 60.0, end: now, ..r.clone() };
        append(&p, &[old, new.clone()]).unwrap();
        trim(&p, now);
        assert_eq!(read(&p).len(), 1);
        assert!(summary(&read(&p), now - 3600.0).contains("1 min"));
        assert_eq!(summary(&[], 0.0), "nothing yet");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
