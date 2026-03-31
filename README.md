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

1. Download `RustBird_1.0.0_aarch64.dmg` (Apple Silicon) or `RustBird_1.0.0_x64.dmg` (Intel)
2. Open the DMG and drag RustBird to your Applications folder
3. **First launch:** macOS will block the app because it is not notarized by Apple. To open it:
   - Right-click the app icon → **Open** → confirm in the dialog, **or**
   - Run in Terminal: `xattr -rd com.apple.quarantine /Applications/RustBird.app`

### Windows

1. Download `RustBird_1.0.0_x64-setup.exe`
2. Run the installer and follow the prompts

## Usage

Click the tray icon to open the mixer. Right-click the tray icon for a Quit option.

## Credits

Made by [soron2038](https://github.com/soron2038).
Bird recordings sourced from [xeno-canto.org](https://xeno-canto.org) — a collaborative database of bird sounds from around the world.
