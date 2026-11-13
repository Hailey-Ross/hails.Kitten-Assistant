//! Self-update from the newest stable GitHub release: read its release.json (version + build), download the exe,
//! verify the SHA-256 the release publishes, then swap the files (a running exe can be renamed on Windows) and
//! restart.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The version (Cargo.toml); builds of one version are told apart by `BUILD`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The CI run that built this exe (0 for a local build).
pub fn build() -> u32 {
    option_env!("KITTEN_BUILD").and_then(|b| b.parse().ok()).unwrap_or(0)
}

/// Built by the release workflow for a published release. Local and per-push builds never replace themselves
/// with a release (that could swap new code for older).
pub const RELEASE_BUILD: bool = match option_env!("KITTEN_RELEASE") {
    Some(v) => v.len() == 1 && v.as_bytes()[0] == b'1',
    None => false,
};

/// "0.1.1 (build 42)", or "0.1.1 (local build)".
pub fn label() -> String {
    match build() {
        0 => format!("{VERSION} (local build)"),
        b => format!("{VERSION} (build {b})"),
    }
}

const LATEST: &str = "https://api.github.com/repos/Hailey-Ross/hails.Kitten-Assistant/releases/latest";
const EXE: &str = "kitten-assistant.exe";

/// `1.2.3` -> (1, 2, 3); a leading `v` and anything after the numbers are ignored, missing parts are 0.
fn parse(v: &str) -> (u32, u32, u32) {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let end = v.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(v.len());
    let mut n = v[..end].split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (n.next().unwrap_or(0), n.next().unwrap_or(0), n.next().unwrap_or(0))
}

/// Whether (`version`, `build`) is newer than (`current`, `current_build`): the version first, then the build.
pub fn newer(version: &str, build: u32, current: &str, current_build: u32) -> bool {
    (parse(version), build) > (parse(current), current_build)
}

struct Release {
    info: String,
    exe: String,
    sha: String,
}

/// The value of the first `"key":"value"` (or `"key": value`) after `from`, and where it ends.
fn json_value<'a>(text: &'a str, key: &str, from: usize) -> Option<(&'a str, usize)> {
    let pat = format!("\"{key}\"");
    let at = text[from..].find(&pat)? + from + pat.len();
    let colon = text[at..].find(':')? + at + 1;
    let rest = text[colon..].trim_start();
    let start = colon + (text[colon..].len() - rest.len());
    if let Some(quoted) = rest.strip_prefix('"') {
        let end = quoted.find('"')?;
        Some((&quoted[..end], start + 1 + end + 1))
    } else {
        let end = rest.find([',', '}', '\n', '\r']).unwrap_or(rest.len());
        Some((rest[..end].trim(), start + end))
    }
}

/// The latest release's asset links: release.json, the exe and its .sha256.
fn parse_release(json: &str) -> Option<Release> {
    let (mut info, mut exe, mut sha) = (None, None, None);
    let mut at = 0;
    while let Some((url, end)) = json_value(json, "browser_download_url", at) {
        if url.ends_with("/release.json") {
            info = Some(url.to_string());
        } else if url.ends_with(&format!("/{EXE}")) {
            exe = Some(url.to_string());
        } else if url.ends_with(&format!("/{EXE}.sha256")) {
            sha = Some(url.to_string());
        }
        at = end;
    }
    Some(Release { info: info?, exe: exe?, sha: sha? })
}

/// (version, build) from a release.json.
fn parse_info(json: &str) -> Option<(String, u32)> {
    let version = json_value(json, "version", 0)?.0.to_string();
    let build = json_value(json, "build", 0)?.0.parse().ok()?;
    Some((version, build))
}

/// Windows' own curl.exe, run without a console window.
fn curl(args: &[&str]) -> Result<Vec<u8>, String> {
    use std::os::windows::process::CommandExt;
    let root = std::env::var_os("SystemRoot").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
    let out = Command::new(root.join(r"System32\curl.exe"))
        .args(["-fsSL", "--max-time", "120", "-A", "kitten-assistant-updater"])
        .args(args)
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("curl {}: {}", args.last().unwrap_or(&""), String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(out.stdout)
}

fn sha256_hex(bytes: &[u8]) -> Result<String, String> {
    use windows_sys::Win32::Security::Cryptography::{BCryptHash, BCRYPT_SHA256_ALG_HANDLE};
    let mut out = [0u8; 32];
    // SAFETY: valid input and output buffers for the call's duration; the pseudo-handle needs no open / close.
    let status = unsafe { BCryptHash(BCRYPT_SHA256_ALG_HANDLE, std::ptr::null(), 0, bytes.as_ptr(), bytes.len() as u32, out.as_mut_ptr(), 32) };
    if status != 0 {
        return Err(format!("SHA-256 failed: 0x{status:08x}"));
    }
    Ok(out.iter().map(|b| format!("{b:02x}")).collect())
}

fn exe_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())
}

fn staged(exe: &Path) -> PathBuf {
    exe.with_extension("exe.new")
}

/// Leftovers of an earlier update (the old exe).
pub fn clean_up() {
    if let Ok(exe) = exe_path() {
        let _ = std::fs::remove_file(exe.with_extension("exe.old"));
    }
}

/// Download and verify a newer stable release next to the exe; `Some("0.1.1 build 42")` once it is staged.
pub fn check_and_stage() -> Result<Option<String>, String> {
    let json = String::from_utf8_lossy(&curl(&["-H", "Accept: application/vnd.github+json", LATEST])?).into_owned();
    let release = parse_release(&json).ok_or("the latest release lacks release.json, kitten-assistant.exe or its .sha256")?;
    let info = String::from_utf8_lossy(&curl(&[&release.info])?).into_owned();
    let (version, build_no) = parse_info(&info).ok_or("release.json has no version / build")?;
    if !newer(&version, build_no, VERSION, build()) {
        return Ok(None);
    }
    let want = String::from_utf8_lossy(&curl(&[&release.sha])?).split_whitespace().next().unwrap_or("").to_ascii_lowercase();
    let bytes = curl(&[&release.exe])?;
    let got = sha256_hex(&bytes)?;
    if want.len() != 64 || got != want {
        return Err(format!("{version} build {build_no} failed its SHA-256 check (got {got}, release says {want})"));
    }
    let exe = exe_path()?;
    std::fs::write(staged(&exe), &bytes).map_err(|e| format!("could not save the update: {e}"))?;
    Ok(Some(format!("{version} build {build_no}")))
}

/// Put the staged exe in place of the running one (kept as .old until the next start).
pub fn swap_in() -> Result<(), String> {
    let exe = exe_path()?;
    let old = exe.with_extension("exe.old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&exe, &old).map_err(|e| format!("could not move the running exe aside: {e}"))?;
    if let Err(e) = std::fs::rename(staged(&exe), &exe) {
        let _ = std::fs::rename(&old, &exe);
        return Err(format!("could not put the update in place: {e}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_first_then_build() {
        assert!(newer("0.1.1", 43, "0.1.1", 42));
        assert!(!newer("0.1.1", 42, "0.1.1", 42));
        assert!(!newer("0.1.1", 41, "0.1.1", 42));
        assert!(newer("0.1.2", 1, "0.1.1", 99));
        assert!(!newer("0.1.0", 500, "0.1.1", 1));
        assert!(newer("v1.0.0", 1, "0.9.9", 900));
    }

    #[test]
    fn the_release_gives_its_three_downloads_and_release_json_its_version() {
        let json = r#"{"tag_name":"v0.1.1","assets":[{"browser_download_url":"https://x/v0.1.1/release.json"},{"name":"kitten-assistant.exe","browser_download_url":"https://x/v0.1.1/kitten-assistant.exe"},{"browser_download_url":"https://x/v0.1.1/kitten-assistant.exe.sha256"}]}"#;
        let r = parse_release(json).unwrap();
        assert_eq!((r.info.as_str(), r.exe.as_str(), r.sha.as_str()), ("https://x/v0.1.1/release.json", "https://x/v0.1.1/kitten-assistant.exe", "https://x/v0.1.1/kitten-assistant.exe.sha256"));
        assert!(parse_release(r#"{"tag_name":"v0.1.1","assets":[]}"#).is_none());
        // PowerShell's ConvertTo-Json layout
        let info = "{\r\n    \"version\":  \"0.1.1\",\r\n    \"build\":  42,\r\n    \"sha256\":  \"ab\"\r\n}";
        assert_eq!(parse_info(info), Some(("0.1.1".to_string(), 42)));
    }

    #[test]
    fn sha256_matches_a_known_digest() {
        assert_eq!(sha256_hex(b"abc").unwrap(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
