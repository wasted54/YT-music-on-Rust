# Wasted

A fast, lightweight, native YouTube Music desktop app written in Rust. No Electron, no embedded browser player.

## Run (Windows, portable)

Download `Wasted-portable.zip` from the Releases page, unzip anywhere and run `wasted.exe`. Settings are kept in a `data` folder beside it while `portable.txt` is there.

## Build

Install [Rust](https://rustup.rs/) and the Visual Studio C++ Build Tools, then run `build-portable.bat`. The result is `dist\Wasted-portable.zip`.

## Sign in

**Settings › Account › Sign in** opens your Chrome (or Edge) in a window with a profile of its own. Once you are signed in, Wasted reads the session cookies through a local debugging port and closes the browser. Without Chrome or Edge it falls back to a built-in window (set `WASTED_LOGIN_WEBVIEW=1` to force that).

## License

GPL-3.0. See [LICENSE](LICENSE).
