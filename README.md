# RustBird

Ambient bird sound mixer for macOS and Windows. Lives in your system tray — no dock icon, no window chrome, no distractions.

## Features

- 6 bundled bird soundscapes (Finland, France, Malaysia, Netherlands, Sweden, Black Forest)
- Import your own MP3 or WAV files
- Per-sound volume controls + master volume
- Smooth crossfade looping — no clicks at loop boundaries
- Pause all sounds at once
- Autostart on login
- Auto-pause when your screen locks
- Light and dark mode support

## Download & Install

Download the latest release for your platform from the [Releases](../../releases) page.

### macOS

1. Download `RustBird_<version>_aarch64.dmg` (Apple Silicon) or `RustBird_<version>_x64.dmg` (Intel) from the [Releases](../../releases) page.
2. Open the DMG and drag **RustBird** onto the **Applications** folder — the window background shows the same hint.
3. **Before first launch**, run this once in Terminal, otherwise macOS will refuse with _"RustBird.app is damaged"_:

   ```bash
   xattr -cr /Applications/RustBird.app
   ```

   RustBird is not yet Apple-notarized, so macOS quarantines every download. This command removes the quarantine flag; afterwards the app launches normally.

### Windows

1. Download `RustBird_<version>_x64_en-US.msi` from the [Releases](../../releases) page.
2. Run the installer and follow the prompts.

## Usage

Click the tray icon to open the mixer. Right-click the tray icon for a Quit option.

## Credits

Made by [soron2038](https://github.com/soron2038).
Bird recordings sourced from [xeno-canto.org](https://xeno-canto.org) — a collaborative database of bird sounds from around the world.
