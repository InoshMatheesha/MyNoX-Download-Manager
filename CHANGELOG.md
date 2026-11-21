# Changelog

Each release's notes on GitHub come from its section here (`## [X.Y.Z]`, see
`.github/workflows/release.yml`).

## [Unreleased]

## [1.0.5] - 2026-11-21

### New

- **MyNoX Island, a second theme of MyNoX's own** (View → Theme, grouped with MyNoX Choice).
  Technical glass: an instrument panel under frosted glass, light and dark, in graphite with
  one amber signal colour. Its own layout:
  - **The Dynamic Island**, a dark capsule at the top. While downloads run it shows the first
    one's progress ring, name and speed, and a count of the others; the pointer on it opens it
    further (how many run, their speed together, the time left, a bar). A download that just
    finished shows there for a few seconds. A click opens that download's window (or the
    finished file); idle, it opens Add URL.
  - **A header set in type**: the view's name large, the scope and count above it, and the
    download speed in large monospaced figures over a minute and a half of it as a sparkline.
  - **A filter bar**: All Downloads, Unfinished and Finished as a sliding switch, then the
    categories and queues as chips (a queue's dot shows whether it runs; right-click a chip for
    its menu; the strip scrolls sideways when they don't all fit), and Find.
  - **A floating dock of round buttons** for the toolbar's commands, Add URL the large amber
    one, Add Torrent and the queues with their drop-downs, Always dark at its end; each names
    itself in a small dark label above it. What shows and the Speed Limiter's switch sit at its
    sides.
- **A new download window in MyNoX Island**: the state in a small island, a progress ring with
  the share done, the file's name and speed set large, four readings without boxes, and two
  rows of chips that act at once: a **speed limit** (Off, 256 KB/sec, 1 MB/sec, 4 MB/sec) and
  **When done** (show the complete dialog, exit MyNoX, or turn the computer off). **Details**
  opens the connections as an amber rail with their table (a torrent's own tabs), the full
  Speed Limiter and Options on completion.
- MyNoX Island carries its own type: Space Grotesk for headings and labels and JetBrains Mono
  for every number (both under the SIL Open Font License, see ATTRIBUTION.md).

### Changed

- **A new release signing key.** The previous one's private half was never available to sign a
  release with, so no release before this one ever installed through Help → Check for updates;
  this version's own `MyNoX-1.0.5-*.exe` still needs installing by hand, but from the next
  release on, Check for updates works normally.

## [1.0.4] - 2026-11-17

### Fixed

- **No more game overlay in MyNoX's windows.** RivaTuner Statistics Server (MSI Afterburner's
  on-screen display) painted its CPU/GPU/FPS readout over the title bar of the download window
  and the others. MyNoX now tells RivaTuner to leave it alone (the exe exports
  `RTSSHooksCompatibility`); anyone who wants the overlay back can set `ForceHooking=1` in the
  `[Hooking]` section of RivaTuner's profile for mynox.exe.
- **Round buttons without the underline.** The bottom edge of Pause, Cancel, Hide details and
  every other button was a straight line that ran past MyNoX Choice's rounded corners. It now
  follows the button's own curve, solid along the bottom and fading as it climbs the corners.

## [1.0.3] - 2026-11-16

### Changed

- **MyNoX Choice's dark mode** has depth instead of pure black: the sidebar (#0F1115), the canvas
  (#13161B) and the cards and list (#191D24) are three steps apart, with crisp 1px borders
  (#262B35) round the buttons, the search box and the list.
- **MyNoX Choice's title bar** shows only the logo and "MyNoX", centred (the window keeps its
  full title in the taskbar).
- **Help → Home Page** opens the website, https://mynox-download-manager.vercel.app, which now
  has a floating island navigation bar, the total downloads and 3D motion.

## [1.0.2] - 2026-11-16

### New

- **MyNoX Choice is the default** for a new install (an existing one keeps its theme), now in
  graphite and white: a black-and-grey dark mode and a real light mode, the sidebar included,
  with blue only as the accent.
- **A new download window** in MyNoX Choice: a segmented tab switch, the file with its type and
  share done, tiles for downloaded, speed and time left, the state with resume capability and
  the server, a slim progress rail, and Pause as the primary button.
- **View → Tray icon** works: Minimize to the notification area, Keep running when closing the
  window, and Hide to the notification area now.
- **The website** (`web-ui/`): MyNoX's download page, with the latest release's files.

### Changed

- **Help** no longer has Tell a friend or Support MyNoX, and About no longer links to Sponsors.

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
