//! Sign in through the person's own Chrome (or Edge), like other apps do: a real browser
//! window opens on the sign-in page with a profile of its own and a local debugging port,
//! and once the session cookie shows up the app reads the cookies through that port and
//! closes the browser. Needs no extension and no password handling by the app.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, anyhow};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

/// What to wait for.
pub struct Wanted<'a> {
    /// The sign-in page to open.
    pub start: &'a str,
    /// The host cookies must be for (music.youtube.com).
    pub host: &'a str,
    /// The cookie only a signed-in session has.
    pub session_cookie: &'a str,
    /// Turn off device-bound sessions (see `login::run`).
    pub unbound: bool,
}

/// Closes the browser on the way out, whichever way that is.
struct Browser(Child);

impl Drop for Browser {
    fn drop(&mut self) {
        let end = Instant::now() + Duration::from_secs(3);
        while Instant::now() < end {
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn find_browser() -> Option<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for var in ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"] {
        if let Some(base) = std::env::var_os(var) {
            found.push(Path::new(&base).join("Google").join("Chrome").join("Application").join("chrome.exe"));
        }
    }
    for var in ["PROGRAMFILES(X86)", "PROGRAMFILES"] {
        if let Some(base) = std::env::var_os(var) {
            found.push(Path::new(&base).join("Microsoft").join("Edge").join("Application").join("msedge.exe"));
        }
    }
    found.extend(
        [
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/usr/bin/microsoft-edge",
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ]
        .map(PathBuf::from),
    );
    found.into_iter().find(|p| p.exists())
}

/// Whether a cookie for `domain` (".youtube.com") goes to `host`.
fn domain_matches(host: &str, domain: &str) -> bool {
    let domain = domain.trim_start_matches('.').to_ascii_lowercase();
    let host = host.to_ascii_lowercase();
    !domain.is_empty() && (host == domain || host.ends_with(&format!(".{domain}")))
}

/// The cookies (name, value) for the wanted host out of a `Storage.getCookies` answer.
fn matching(answer: &serde_json::Value, host: &str) -> Vec<(String, String)> {
    answer["result"]["cookies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c["domain"].as_str().is_some_and(|d| domain_matches(host, d)))
        .filter_map(|c| Some((c["name"].as_str()?.to_owned(), c["value"].as_str()?.to_owned())))
        .collect()
}

/// Open the browser and wait for the sign-in. `Ok(None)`: the browser was closed first.
/// `Err`: it couldn't be used at all (the caller can fall back to its own window).
pub fn run(wanted: &Wanted, profile_dir: &Path) -> anyhow::Result<Option<Vec<(String, String)>>> {
    let exe = find_browser().ok_or_else(|| anyhow!("no Chrome or Edge found"))?;
    let dir = profile_dir.join("browser");
    std::fs::create_dir_all(&dir)?;
    let port_file = dir.join("DevToolsActivePort");
    let _ = std::fs::remove_file(&port_file);

    let mut features = String::from("--disable-features=Translate");
    if wanted.unbound {
        features.push_str(",DeviceBoundSessions,EnableBoundSessionCredentials");
    }
    let child = Command::new(exe)
        .arg(format!("--user-data-dir={}", dir.display()))
        .arg("--remote-debugging-port=0")
        .args(["--no-first-run", "--no-default-browser-check", "--new-window", "--window-size=520,760"])
        .arg(features)
        .arg(wanted.start)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("couldn't start the browser")?;
    let mut browser = Browser(child);

    // Chrome writes the port it picked, then the socket's path, once it's listening.
    let start = Instant::now();
    let socket = loop {
        if let Ok(text) = std::fs::read_to_string(&port_file) {
            let mut lines = text.lines();
            if let (Some(port), Some(path)) = (lines.next(), lines.next())
                && !path.is_empty()
            {
                break format!("ws://127.0.0.1:{}{}", port.trim(), path.trim());
            }
        }
        if browser.0.try_wait()?.is_some() {
            return Ok(None);
        }
        if start.elapsed() > Duration::from_secs(20) {
            return Err(anyhow!("the browser didn't open its debugging port"));
        }
        std::thread::sleep(Duration::from_millis(200));
    };

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let got = rt.block_on(async {
        let (mut ws, _) = tokio_tungstenite::connect_async(socket.as_str()).await?;
        let give_up = Instant::now() + Duration::from_secs(15 * 60);
        let mut id = 0u64;
        while Instant::now() < give_up {
            tokio::time::sleep(Duration::from_millis(1500)).await;
            id += 1;
            let ask = serde_json::json!({ "id": id, "method": "Storage.getCookies" });
            if ws.send(Message::Text(ask.to_string().into())).await.is_err() {
                return anyhow::Ok(None);
            }
            let answer = loop {
                match ws.next().await {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text)
                            && v["id"] == id
                        {
                            break v;
                        }
                    }
                    Some(Ok(_)) => {}
                    // The person closed the browser.
                    _ => return Ok(None),
                }
            };
            let cookies = matching(&answer, wanted.host);
            if cookies.iter().any(|(name, _)| name == wanted.session_cookie) {
                let _ = ws.send(Message::Text(serde_json::json!({ "id": id + 1, "method": "Browser.close" }).to_string().into())).await;
                return Ok(Some(cookies));
            }
        }
        Ok(None)
    })?;
    drop(browser);
    Ok(got)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_cookies_for_the_host() {
        let answer = serde_json::json!({ "id": 1, "result": { "cookies": [
            { "name": "SAPISID", "value": "a", "domain": ".youtube.com" },
            { "name": "NID", "value": "b", "domain": ".google.com" },
            { "name": "VISITOR", "value": "c", "domain": "music.youtube.com" },
        ] } });
        let got = matching(&answer, "music.youtube.com");
        assert_eq!(got, vec![("SAPISID".to_owned(), "a".to_owned()), ("VISITOR".to_owned(), "c".to_owned())]);
    }
}
