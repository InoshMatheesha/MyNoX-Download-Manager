# Portable mode

[← Back to docs index](README.md)

The portable copy of MyNoX runs from any folder, including a USB stick, without installing anything.

## How it works

`MyNoX-<version>-portable.exe` unpacks the same programs as the setup, plus a folder named `data` next to `mynox.exe`. When that folder is there and MyNoX can write to it, MyNoX is in portable mode:

- The download list, settings, helpers for video and galleries (yt-dlp, ffmpeg, gallery-dl), torrent data, your [theme files](themes.md) (`data\themes`) and the logs are kept in `data`, not in your Windows profile.
- Passwords (Sites Logins, the proxy's, debrid accounts) stay in `data` too, instead of in Windows Credential Manager. They are sealed for your Windows account: on another computer, or another account, type them again (and sign in to a debrid account again).
- Nothing is written to the registry.

Delete the `data` folder (or make it read-only) and the same copy keeps its data in `%APPDATA%\mynox`, like an installed one.

## Set it up

1. Download `MyNoX-<version>-portable.exe` from [Releases](https://github.com/InoshMatheesha/MyNoX-Download-Manager/releases).
2. Run it and choose where MyNoX should live, for example `E:\MyNoX` on a USB stick. Nothing is installed: it only unpacks there.
3. Run `mynox.exe` from that folder (the last page can start it for you).

**Help → Check for updates** updates a portable copy in its own folder: it downloads the new portable .exe, checks it, and unpacks it over the programs, keeping `data`.

On the first start MyNoX asks for the language; see [Languages](languages.md).

## What is different

- **Downloads still go to the computer's Downloads folder** unless you change it. To keep them with MyNoX, set each category's folder in **Options → Save to**. [Carry MyNoX on a USB stick](guide-usb-stick.md) shows how.
- **Launch MyNoX Download Manager on startup** in **Options → General** is greyed: starting with Windows needs the registry.
- **Browser integration needs the installer.** The browser finds MyNoX's helper through the registry, so a portable copy can't take downloads from the browser. Use **Add URL**, the clipboard or the drop target instead (see [Adding downloads](adding-downloads.md)).
- **Tasks → Torrent settings → Open magnet links and .torrent files with MyNoX Download Manager** does write to the registry, for your Windows account only. Leave it off to keep the portable copy registry-free.

## Moving between computers

The list keeps each download's full folder path. If Windows gives the USB stick a different drive letter on another computer, check **Options → Save to**, and expect downloads saved under the old letter to be missing until the letter matches again. When MyNoX is about to save into a folder that doesn't exist, it asks whether to create it.

Quit MyNoX with **Tasks → Exit** before you unplug the stick, so it saves the list.

## Next steps

- [Tutorial: carry MyNoX on a USB stick →](guide-usb-stick.md)
- [Getting started →](getting-started.md)
- [Options → Save to →](options.md#save-to)
