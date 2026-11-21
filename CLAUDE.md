# MyNoX Download Manager (MyNoX) - agent guide

MyNoX Download Manager is a download manager for Windows in Rust (egui), made by MoOdY69. Start
with [README.md](README.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Rules

1. **Clean room.** MyNoX contains no code, binaries, artwork, icons, sounds or text from Internet
   Download Manager or any other download manager. Never disassemble, decompile, debug-attach or
   patch any IDM file, and never bypass its licensing or trial.
2. Every user-visible text goes through `crate::i18n`; run `python tools/i18n/extract.py` after adding one.
3. Windows-only calls live in `crates/app/src/win32.rs`; the rest of the workspace is safe Rust
   (`unsafe_code = "deny"`).
4. Fix root causes, keep data contracts aligned between producers and consumers, and verify the
   whole system end to end after each change.
5. Every change updates [CHANGELOG.md](CHANGELOG.md) and [ROADMAP.md](ROADMAP.md), and bumps the
   version (Cargo.toml, tools/installer/mynox.iss; the extension's manifests when it changes).

## Commands

- `cargo build` / `cargo test` / `cargo clippy --all-targets` / `cargo fmt --all` - workspace.
- `cargo run -p mynox-labserver -- --faults "rate=2M"` - test server (GET / lists the faults).
- `cargo run -p mynox-app -- --cli get <url> -o <dir>` - headless download with progress.
- `tools/extension/pack.ps1` - the extension for Chromium and Firefox into `target/extension/`;
  `node tools/extension/test.mjs` - its logic against fake browser APIs.
- `tools/installer/build.ps1` - the x64 setup and the portable .exe (`mynox-portable.iss`) into
  `target/dist/`; a release is those two plus the signed `SHA256SUMS.txt`.
- `web-ui/` is the website (static, for Vercel with Root Directory `web-ui`); it reads the latest
  release from the GitHub API. Preview: `python -m http.server 8000 --directory web-ui`.
- MyNoX Island (`mynox-island`) is the default theme of a new list (settings.rs); docs-shots.ps1
  pins `classic` so the help pages' pictures stay the classic layout.
- The MyNoX Choice theme's own layout is `crates/app/src/choice.rs` (a child module of `app.rs`);
  a theme picks it with `layout = "choice"`.
- `python -I tools/icon/build_icon.py` - every icon (app, extension, Download video button) from
  `art/MyNoX Download Manager.png`.
- A release: bump the version, push the commit, then `powershell -File tools\installer\release.ps1`
  builds, signs SHA256SUMS.txt with the key in `.private\gnupg` (the updater checks it against
  `crates/tools/keys/mynox-release.asc`) and publishes `vX.Y.Z` on GitHub. `release.yml` does the
  same on GitHub for a pushed tag when the `MYNOX_RELEASE_KEY` secret is set.
- This PC has no MSVC: build with the GNU toolchain and `C:\msys64\mingw64\bin` on PATH, e.g.
  `$env:Path = "$env:USERPROFILE\.cargo\bin;C:\msys64\mingw64\bin;$env:Path"`.
