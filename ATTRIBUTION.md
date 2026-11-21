# Third-party assets

| Asset | Author | Licence | Where |
|---|---|---|---|
| Lucide icons v1.63.0 (toolbar, dialog glyphs) | Lucide Icons and Contributors | ISC — `crates/app/assets/icons/lucide/LICENSE` | `crates/app/assets/icons/lucide/` |
| Tango Icon Theme 0.8.90, 25 of its icons (the icon before a file's or folder's name where the system has none of its own, and until the system's is ready: every window that names a file; this computer and the local network where a download's server is) | The Tango Desktop Project (Ulisse Perusin, Steven Garrity, Lapo Calamandrei, Ryan Collier, Rodney Dawes, Andreas Nilsson, Tuomas Kuosmanen, Garrett LeSage, Jakub Steiner), <http://tango.freedesktop.org> | Public domain — crates/app/assets/icons/tango/README.txt | crates/app/assets/icons/tango/ |
| flag-icons 7.6.0 (language flags: picker, View → Language, README) | Panayiotis Lipiridis and contributors, <https://github.com/lipis/flag-icons> | MIT — `crates/app/assets/flags/LICENSE` | `crates/app/assets/flags/` |
| flag-icons 7.6.0, all 250 country flags in one picture (where a download's server is: Download File Info, the progress window, Properties), drawn by `crates/geo/examples/flag_atlas.rs` | Panayiotis Lipiridis and contributors, <https://github.com/lipis/flag-icons> | MIT — `crates/geo/data/LICENSE-flag-icons` | `crates/geo/data/flags.png` |
| DB-IP IP to Country Lite, a monthly edition (which country a download's server is in, looked up on the computer, never sent anywhere), packed by `tools/geo/build.py` | DB-IP, <https://db-ip.com>: IP Geolocation by DB-IP | Creative Commons Attribution 4.0 International, <https://creativecommons.org/licenses/by/4.0/>; the edition used is in `crates/geo/data/SOURCE.txt` | `crates/geo/data/ipcountry.bin.xz` |
| Patrick Hand (the README's hand-drawn pictures, as outlines; not in the app) | Patrick Wagesreiter | SIL Open Font License 1.1 — `tools/readme/node_modules/@fontsource/patrick-hand/LICENSE` | `docs/images/races-*.svg` |
| *Big Buck Bunny* (the film in the README's pictures of the download button over a video; not in the app) | Blender Foundation, <https://peach.blender.org> | Creative Commons Attribution 3.0 | `docs/images/video-button-*.png` |
| Space Grotesk 2.0, Medium and Bold (MyNoX Island's headings and labels) | The Space Grotesk Project Authors (Florian Karsten), <https://github.com/floriankarsten/space-grotesk> | SIL Open Font License 1.1 — `crates/app/assets/fonts/OFL-SpaceGrotesk.txt` | `crates/app/assets/fonts/` |
| JetBrains Mono 2.304, Medium (MyNoX Island's numbers) | The JetBrains Mono Project Authors, <https://github.com/JetBrains/JetBrainsMono> | SIL Open Font License 1.1 — `crates/app/assets/fonts/OFL-JetBrainsMono.txt` | `crates/app/assets/fonts/` |
| Silk icon set 1.3 (Categories tree) | Mark James, <http://www.famfamfam.com/lab/icons/silk/> | Creative Commons Attribution 2.5 — `crates/app/assets/icons/silk/readme.txt` | `crates/app/assets/icons/silk/` |
| Fluent UI System Icons 1.1.345, 49 of its icons (the Metro theme's toolbar, categories, file types and title-bar buttons), recoloured white to be tinted | Microsoft Corporation, <https://github.com/microsoft/fluentui-system-icons> (commit 9eb184e) | MIT — `crates/app/assets/themes/metro/icons/LICENSE` | `crates/app/assets/themes/metro/icons/` (its magnet, Torrents, torrent-file and Hide to the notification area icons are MyNoX's own) |
| adwaita-icon-theme 51.0, 50 of its icons (the Adwaita theme's toolbar, categories, file types and title-bar buttons), unchanged; its one-colour icons are drawn in the theme's text colour when read | The GNOME Project, <https://gitlab.gnome.org/GNOME/adwaita-icon-theme> (commit 70ecd52) | LGPL-3.0 (of LGPL-3.0 or CC BY-SA 3.0 US) — `crates/app/assets/themes/adwaita/icons/LICENSE`, `COPYING` | `crates/app/assets/themes/adwaita/icons/` (its PDF, torrent-file and Hide to the notification area icons are MyNoX's own) |
| breeze-icons 6.31, 51 of its icons (the Breeze theme's toolbar, categories, file types and title-bar buttons), unchanged; its one-colour icons are drawn in the theme's text colour when read | Uri Herrera and the KDE Visual Design Group, <https://invent.kde.org/frameworks/breeze-icons> (commit 60ca517) | LGPL-3.0-or-later — `crates/app/assets/themes/breeze/icons/LICENSE` | `crates/app/assets/themes/breeze/icons/` (its PDF and Hide to the notification area icons are MyNoX's own) |

The app icon (`art/MyNoX Download Manager.png`, the MyNoX "M" with a download arrow) is
MyNoX's own logo by MoOdY69; every icon size is built from it by `tools/icon/build_icon.py`.

## Licence notice

MyNoX Download Manager is free software under the GNU GPL, version 3 or later ([LICENSE](LICENSE)).
It includes code first released under the same licence, copyright (C) 2026 Ahmed Nefzaoui and
contributors, modified by MoOdY69.

The Luna, Aero and Aqua themes' icons (`crates/app/assets/themes/luna/icons/`, `aero/icons/`,
`aqua/icons/`) were drawn for MyNoX Download Manager, as were the Metro, Adwaita and Breeze
icons named MyNoX's own above; each set's `SOURCES.txt` lists them.

## Archive extraction (mynox-extract)

The libraries Link grabber's archive extraction is built on (all compatible with MyNoX's GPL-3.0):

| Library | Licence | For |
|---|---|---|
| [zip](https://crates.io/crates/zip) 9.1 | MIT | .zip, with [deflate64](https://crates.io/crates/deflate64) (MIT) and WinZip AES ([RustCrypto](https://github.com/RustCrypto) aes, hmac, pbkdf2: MIT OR Apache-2.0) |
| [lzma-rust2](https://crates.io/crates/lzma-rust2) 0.23 | Apache-2.0 | .xz |
| [tar](https://crates.io/crates/tar) 0.4 | MIT OR Apache-2.0 | .tar |

7z and RAR archives are extracted by [7-Zip](https://www.7-zip.org/) (GNU LGPL, with the
unRAR restriction for its RAR code), run as a separate program when the user has it installed.
It is neither bundled nor linked: UnRAR's licence doesn't allow its code in a GPL program, and a
built-in 7z decoder made MyNoX about 1 MB larger.

System resources used at run time and **not** bundled: Windows fonts (Segoe UI, Tahoma,
Arial) and each system's own file-type icons (the Windows shell's, macOS Finder's, the Linux
desktop's icon theme).

MyNoX contains no artwork, text or code from Internet Download Manager (see the README).
