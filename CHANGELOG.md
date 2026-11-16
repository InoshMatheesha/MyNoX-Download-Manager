# Changelog

Each release's notes on GitHub come from its section here (`## [X.Y.Z]`, see
`.github/workflows/release.yml`).

## [Unreleased]

## [1.0.1] - 2026-11-16

### New

- **MyNoX Choice**, a theme of MyNoX's own with its own layout (**View → Theme → MyNoX Choice**):
  a midnight sidebar with the downloads library and a big **Add URL** button, a header with the
  view's name and a search box, an action bar, the list on a card with a progress bar under each
  download's status, a live download speed card and a **Speed Limiter** switch. Fold the sidebar
  to icons with its button. Every menu, shortcut and command is still there, in light and dark.
- **Add the browser extension in one click:** **Add it now** in the browser extension's window
  (**Tasks → Browser extension...**, or **Add to browser** in MyNoX Choice's sidebar) opens the
  browser's extensions page, copies the extension's folder and shows the three clicks left to do
  there, then says when the extension is connected.
- **Portable .exe:** the portable copy is now one `MyNoX-<version>-portable.exe` that unpacks
  MyNoX into any folder or USB stick, and **Update now** updates such a copy with it.

### Changed

- A release has the setup (x64), the portable .exe and the signed `SHA256SUMS.txt`; the zips are
  gone, and the extension comes inside MyNoX (setup and portable).
- The README thanks the original project MyNoX is built on.

## [0.5.4] - 2026-11-15

The first release of **MyNoX Download Manager** for Windows 10 and 11.

### Highlights

- **Fast downloads:** each file is split over several connections (8 by default, up to 32), and a
  download that stops (the internet, the power, the server) picks up where it was.
- **Browser integration:** the extension for Chrome, Edge, Brave, Opera, Vivaldi and Firefox hands
  every download link to MyNoX and puts a **Download video** button on video pages. It comes with
  MyNoX and with every release, ready to add (see the README).
- **Videos and playlists** from YouTube and many other sites, **torrents and magnet links**, the
  **link grabber**, **queues and the scheduler**, the **speed limiter**, **portable mode** and a
  **remote control** from your phone.
- **86 languages** and several themes, light and dark.
- **Help → Check for updates** finds a new release on GitHub and installs it after checking its
  signed `SHA256SUMS.txt`.
- The new MyNoX logo across the app, the installer and the browser extension.
