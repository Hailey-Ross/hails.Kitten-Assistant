//! The local endpoint (off unless `listen=ADDR` in the ini or `--listen ADDR`): a tiny HTTP/1.1 server inside the
//! kitten, so programs that cannot start the exe or share a file (a browser extension, a container, WSL, another
//! computer) can drive it. Every call needs `Authorization: Bearer <token>`, the token from `listen-token.txt` next
//! to the exe (made on first start). No CORS headers, so web pages cannot call it from a browser.
//!
//! GET /status            the `status --json` document
//! POST /say              text=..., and any of session, progress, for, mood, open, attention, steps, step
//! POST /idle             session=..., cheer=...
//! POST /do               action=..., text=...
//! Fields go in the query string or a form body (application/x-www-form-urlencoded).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::sessions;

const MAX_REQUEST: usize = 8192;
const MAX_OPEN: usize = 8;
static OPEN: AtomicUsize = AtomicUsize::new(0);

pub fn token_path() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(|d| d.join("listen-token.txt"))
}

/// The token, made (32 random bytes, hex) and saved the first time.
pub fn token() -> Result<String, String> {
    let p = token_path().ok_or("no exe folder")?;
    if let Ok(t) = std::fs::read_to_string(&p) {
        let t = t.trim().to_string();
        if t.len() >= 32 {
            return Ok(t);
        }
    }
    let mut b = [0u8; 32];
    use windows_sys::Win32::Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG};
    if unsafe { BCryptGenRandom(std::ptr::null_mut(), b.as_mut_ptr(), b.len() as u32, BCRYPT_USE_SYSTEM_PREFERRED_RNG) } != 0 {
        return Err("could not make a token".into());
    }
    let t: String = b.iter().map(|x| format!("{x:02x}")).collect();
    std::fs::write(&p, &t).map_err(|e| format!("could not save {}: {e}", p.display()))?;
    Ok(t)
}

/// Starts the server on its own thread. Errors (address in use, no token) are returned and logged by the caller.
pub fn start(addr: &str, status: PathBuf) -> Result<(), String> {
    let token = token()?;
    let listener = TcpListener::bind(addr).map_err(|e| format!("listen {addr}: {e}"))?;
    crate::log(&format!("listening on {addr}"));
    std::thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            if OPEN.load(Ordering::SeqCst) >= MAX_OPEN {
                continue; // dropped: too many at once
            }
            OPEN.fetch_add(1, Ordering::SeqCst);
            let (token, status) = (token.clone(), status.clone());
            std::thread::spawn(move || {
                let _ = serve(conn, &token, &status);
                OPEN.fetch_sub(1, Ordering::SeqCst);
            });
        }
    });
    Ok(())
}

pub struct Request {
    pub method: String,
    pub path: String,
    pub auth: Option<String>,
    pub fields: HashMap<String, String>,
}

fn hex(b: u8) -> Option<u8> {
    (b as char).to_digit(16).map(|d| d as u8)
}

/// `a+b%20c` -> `a b c` (bad escapes are kept as they are).
pub fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => match (hex(b[i + 1]), hex(b[i + 2])) {
                (Some(h), Some(l)) => {
                    out.push(h * 16 + l);
                    i += 2;
                }
                _ => out.push(b'%'),
            },
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn fields_into(map: &mut HashMap<String, String>, s: &str) {
    for pair in s.split('&').filter(|p| !p.is_empty()) {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        map.insert(decode(k), decode(v));
    }
}

/// Parses a whole request (head and body). None when it is not one we take.
pub fn parse(raw: &[u8]) -> Option<Request> {
    let head_end = raw.windows(4).position(|w| w == b"\r\n\r\n")?;
    let head = std::str::from_utf8(&raw[..head_end]).ok()?;
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, target) = (first.next()?.to_string(), first.next()?);
    let mut auth = None;
    let mut form = true;
    for l in lines {
        let (k, v) = l.split_once(':')?;
        match k.trim().to_ascii_lowercase().as_str() {
            "authorization" => auth = v.trim().strip_prefix("Bearer ").map(|t| t.trim().to_string()),
            "content-type" => form = v.trim().starts_with("application/x-www-form-urlencoded"),
            _ => {}
        }
    }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut fields = HashMap::new();
    fields_into(&mut fields, query);
    let body = &raw[head_end + 4..];
    if form && !body.is_empty() {
        fields_into(&mut fields, std::str::from_utf8(body).ok()?);
    }
    Some(Request { method, path: path.to_string(), auth, fields })
}

/// Same length and same bytes, without stopping at the first difference.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Reads until the head and its Content-Length body are in, at most MAX_REQUEST bytes.
fn read_request(conn: &mut TcpStream) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 2048];
    loop {
        let n = conn.read(&mut chunk).ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.len() > MAX_REQUEST {
            return None;
        }
        if let Some(h) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..h]).to_ascii_lowercase();
            let len = head
                .lines()
                .find_map(|l| l.strip_prefix("content-length:").and_then(|v| v.trim().parse::<usize>().ok()))
                .unwrap_or(0);
            if h + 4 + len > MAX_REQUEST {
                return None;
            }
            if buf.len() >= h + 4 + len {
                buf.truncate(h + 4 + len);
                return Some(buf);
            }
        }
    }
}

fn reply(conn: &mut TcpStream, code: u16, body: &str) -> std::io::Result<()> {
    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    write!(conn, "HTTP/1.1 {code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
}

fn serve(mut conn: TcpStream, token: &str, status: &Path) -> std::io::Result<()> {
    conn.set_read_timeout(Some(Duration::from_secs(5)))?;
    conn.set_write_timeout(Some(Duration::from_secs(5)))?;
    let Some(req) = read_request(&mut conn).as_deref().and_then(parse) else {
        return reply(&mut conn, 400, r#"{"ok":false,"error":"bad request"}"#);
    };
    if !req.auth.as_deref().is_some_and(|a| same(a, token)) {
        return reply(&mut conn, 401, r#"{"ok":false,"error":"missing or wrong token"}"#);
    }
    let (code, body) = handle(&req, status);
    reply(&mut conn, code, &body)
}

fn err(code: u16, e: &str) -> (u16, String) {
    (code, format!(r#"{{"ok":false,"error":{}}}"#, crate::json_str(e)))
}

/// What a request does: (HTTP status, JSON body).
pub fn handle(req: &Request, status: &Path) -> (u16, String) {
    let f = |k: &str| req.fields.get(k).map(String::as_str).filter(|v| !v.is_empty());
    let session = f("session").and_then(sessions::clean);
    let ok = || (200, r#"{"ok":true}"#.to_string());
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/status") => (200, crate::status_json(&crate::busy_lines(status), sessions::now())),
        ("POST", "/say") => {
            let Some(text) = f("text") else { return err(400, "text is missing") };
            let mut opts = crate::LineOpts::default();
            let parsed = (|| -> Result<(), String> {
                opts.for_secs = f("for").map(|v| sessions::parse_duration(v).ok_or("bad for")).transpose()?;
                opts.progress = f("progress").map(|v| sessions::Progress::parse(v).ok_or("bad progress")).transpose()?;
                opts.mood = f("mood").map(|v| crate::scene::Mood::named(v).ok_or("bad mood")).transpose()?;
                opts.open = f("open").map(str::to_string);
                opts.attention = f("attention").is_some_and(|v| matches!(v, "1" | "true" | "yes"));
                opts.steps = f("steps").map(sessions::parse_steps);
                opts.step = f("step").map(|v| v.parse().ok().filter(|s| *s >= 1).ok_or("bad step")).transpose()?;
                Ok(())
            })();
            if let Err(e) = parsed {
                return err(400, &e);
            }
            match crate::say_line(status, session.as_deref(), text, &opts) {
                Ok(()) => ok(),
                Err(e) => err(500, &e),
            }
        }
        ("POST", "/idle") => {
            let r = match &session {
                Some(name) => sessions::idle(status, name),
                None => std::fs::write(status, "idle").map_err(|e| e.to_string()),
            };
            if let Err(e) = r {
                return err(500, &e);
            }
            if let Some(c) = f("cheer") {
                let _ = crate::win::act(crate::ACTIONS.len() - 1, c);
            }
            ok()
        }
        ("POST", "/do") => {
            let action = f("action").unwrap_or("").to_ascii_lowercase();
            let Some(i) = crate::ACTIONS.iter().position(|a| *a == action) else { return err(400, "unknown action") };
            match crate::win::act(i, f("text").unwrap_or("")) {
                Ok(()) => ok(),
                Err(e) => err(500, &e),
            }
        }
        (_, "/status" | "/say" | "/idle" | "/do") => err(405, "wrong method"),
        _ => err(404, "no such path"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_parse() {
        let raw = b"POST /say?session=ci&text=Building%3A+app HTTP/1.1\r\nHost: x\r\nAuthorization: Bearer abc\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 14\r\n\r\nprogress=3%2F10";
        let r = parse(raw).unwrap();
        assert_eq!((r.method.as_str(), r.path.as_str(), r.auth.as_deref()), ("POST", "/say", Some("abc")));
        assert_eq!(r.fields.get("text").map(String::as_str), Some("Building: app"));
        assert_eq!(r.fields.get("progress").map(String::as_str), Some("3/10"));
        assert!(parse(b"GET / HTTP/1.1\r\n").is_none(), "no end of head");
        assert_eq!(decode("100%25+done%"), "100% done%");
        assert!(same("abc", "abc") && !same("abc", "abd") && !same("abc", "ab"));
    }

    #[test]
    fn say_and_idle_write_the_session_file() {
        let dir = std::env::temp_dir().join(format!("kitten-listen-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let status = dir.join("status.txt");
        let req = |m: &str, p: &str, q: &str| {
            let mut fields = HashMap::new();
            fields_into(&mut fields, q);
            Request { method: m.into(), path: p.into(), auth: None, fields }
        };
        assert_eq!(handle(&req("POST", "/say", "session=web&text=Copying&progress=2/4&mood=uploading"), &status).0, 200);
        let body = std::fs::read_to_string(sessions::file(&status, "web")).unwrap();
        assert_eq!(body, "Copying\nprogress=2/4\nmood=uploading");
        assert_eq!(handle(&req("POST", "/say", "session=web&text=x&progress=nope"), &status).0, 400);
        assert_eq!(handle(&req("POST", "/say", "session=web"), &status).0, 400);
        let (code, json) = handle(&req("GET", "/status", ""), &status);
        assert!(code == 200 && json.contains(r#""session":"web""#), "{json}");
        assert_eq!(handle(&req("POST", "/idle", "session=web"), &status).0, 200);
        assert!(!sessions::file(&status, "web").exists());
        assert_eq!(handle(&req("GET", "/say", ""), &status).0, 405);
        assert_eq!(handle(&req("GET", "/nope", ""), &status).0, 404);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
