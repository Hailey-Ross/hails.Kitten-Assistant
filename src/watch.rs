//! `watch`: follow a job's log file and keep its line current: progress from the newest matching log line, and
//! optionally that log line itself.

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::sessions::Progress;

/// The first `a/b` or `n%` in a log line, as progress (`Compiling serde (52/310)`, `Downloading 45%`).
pub fn progress_in(line: &str) -> Option<Progress> {
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if !b[i].is_ascii_digit() || (i > 0 && (b[i - 1].is_ascii_digit() || b[i - 1] == b'.')) {
            i += 1;
            continue;
        }
        let num = |from: usize| {
            let mut j = from;
            while j < b.len() && (b[j].is_ascii_digit() || (b[j] == b'.' && j + 1 < b.len() && b[j + 1].is_ascii_digit())) {
                j += 1;
            }
            j
        };
        let end = num(i);
        // `a/b`, spaces allowed around the slash
        let mut k = end;
        while k < b.len() && b[k] == b' ' {
            k += 1;
        }
        if k < b.len() && b[k] == b'/' {
            let mut m = k + 1;
            while m < b.len() && b[m] == b' ' {
                m += 1;
            }
            if m < b.len() && b[m].is_ascii_digit() {
                let e2 = num(m);
                if let Some(p) = Progress::parse(&format!("{}/{}", &line[i..end], &line[m..e2])) {
                    return Some(p);
                }
            }
        }
        if end < b.len() && b[end] == b'%' {
            if let Some(p) = Progress::parse(&line[i..=end]) {
                return Some(p);
            }
        }
        i = end.max(i + 1);
    }
    None
}

/// Reads what was added to a file since the last call, in whole lines. UTF-16 files (a byte order mark FF FE, as
/// Windows PowerShell's `>` writes) are read as such; anything else as UTF-8. A file that shrank starts over.
pub struct Tail {
    pub path: PathBuf,
    pos: u64,
    utf16: bool,
    partial: String,
}

impl Tail {
    /// From the end of the file as it is now (`from_start` false), or from its beginning.
    pub fn new(path: PathBuf, from_start: bool) -> Tail {
        let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let mut t = Tail { path, pos: 0, utf16: false, partial: String::new() };
        t.sniff();
        if !from_start {
            t.pos = len;
        }
        t
    }

    fn sniff(&mut self) {
        let mut bom = [0u8; 2];
        self.utf16 = std::fs::File::open(&self.path).and_then(|mut f| f.read_exact(&mut bom)).is_ok() && bom == [0xFF, 0xFE];
    }

    /// The complete new lines (the last unfinished one waits for the next call).
    pub fn read(&mut self) -> Vec<String> {
        let Ok(mut f) = std::fs::File::open(&self.path) else { return Vec::new() };
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        if len < self.pos {
            // truncated or replaced: start over
            self.pos = 0;
            self.partial.clear();
            self.sniff();
        }
        if len == self.pos || f.seek(SeekFrom::Start(self.pos)).is_err() {
            return Vec::new();
        }
        // at most 1 MB a call; a huge burst is skimmed through on the next calls
        let mut buf = Vec::new();
        let _ = f.by_ref().take((len - self.pos).min(1 << 20)).read_to_end(&mut buf);
        let mut used = buf.len();
        let text = if self.utf16 {
            let start = if self.pos == 0 && buf.starts_with(&[0xFF, 0xFE]) { 2 } else { 0 };
            used -= (buf.len() - start) % 2;
            let units: Vec<u16> = buf[start..used].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            String::from_utf16_lossy(&units)
        } else {
            // keep an unfinished UTF-8 character for next time
            match std::str::from_utf8(&buf) {
                Ok(s) => s.to_string(),
                Err(e) if e.error_len().is_none() => {
                    used = e.valid_up_to();
                    String::from_utf8_lossy(&buf[..used]).into_owned()
                }
                Err(_) => String::from_utf8_lossy(&buf).into_owned(),
            }
        };
        self.pos += used as u64;
        self.partial.push_str(&text);
        let mut lines: Vec<String> = Vec::new();
        while let Some(at) = self.partial.find('\n') {
            let l: String = self.partial.drain(..=at).collect();
            let l = l.trim_end_matches(['\r', '\n']).trim_start_matches('\u{feff}');
            // a progress bar redrawn with \r: the last state counts
            let l = l.rsplit('\r').next().unwrap_or(l).trim();
            if !l.is_empty() {
                lines.push(l.to_string());
            }
        }
        lines
    }
}

/// The text a watched line shows: the line, and with `tail` the newest log line after it (shortened).
pub fn line_text(line: &str, tail: bool, last: Option<&str>) -> String {
    match last.filter(|_| tail) {
        Some(l) => {
            let l: String = if l.chars().count() > 80 { l.chars().take(79).chain(['\u{2026}']).collect() } else { l.to_string() };
            format!("{line}: {l}")
        }
        None => line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_found_in_log_lines() {
        let label = |s: &str| progress_in(s).map(|p| p.label);
        assert_eq!(label("   Compiling serde v1.0.200 (52/310)"), Some("52/310".into()));
        assert_eq!(label("Downloading 45% of 2 GB"), Some("45%".into()));
        assert_eq!(label("[ 3 / 12 ] tests"), Some("3/12".into()));
        assert_eq!(label("frame 120/0 and 7%"), Some("7%".into()));
        assert_eq!(label("version 1.2.3 built"), None);
        assert_eq!(label("nothing here"), None);
    }

    #[test]
    fn a_tail_reads_new_lines_only() {
        let dir = std::env::temp_dir().join(format!("kitten-watch-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("job.log");
        std::fs::write(&p, "old line\n").unwrap();
        let mut t = Tail::new(p.clone(), false);
        assert!(t.read().is_empty());
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().append(true).open(&p).unwrap();
        write!(f, "one\r\ntw").unwrap();
        assert_eq!(t.read(), ["one"]);
        write!(f, "o\n10%\r20%\r30%\n").unwrap();
        assert_eq!(t.read(), ["two", "30%"]);
        drop(f);
        // replaced by a shorter file: starts over
        std::fs::write(&p, "new\n").unwrap();
        assert_eq!(t.read(), ["new"]);
        // UTF-16 with a byte order mark
        let utf16: Vec<u8> = [0xFF, 0xFE].into_iter().chain("h\u{e9}llo 4/8\r\n".encode_utf16().flat_map(|u| u.to_le_bytes())).collect();
        std::fs::write(&p, utf16).unwrap();
        let mut t = Tail::new(p.clone(), true);
        assert_eq!(t.read(), ["h\u{e9}llo 4/8"]);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(line_text("Building", true, Some("Compiling x")), "Building: Compiling x");
        assert_eq!(line_text("Building", false, Some("Compiling x")), "Building");
    }
}
