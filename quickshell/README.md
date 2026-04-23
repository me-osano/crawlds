# CrawlDS Shell

A beautiful, minimal Wayland desktop shell built on [Quickshell](https://quickshell.outfoxxed.me/) with a warm lavender aesthetic.

## Features

- Native support for Niri, Hyprland, Sway, Scroll, Labwc and MangoWC
- Extensive theming with predefined color schemes and automatic color generation from wallpaper
- Wallpaper management with Wallhaven integration
- Notification system with history and Do Not Disturb
- Multi-monitor support
- Lock screen
- Desktop widgets (clock, media player and more)
- On-screen display for volume and brightness
- Built on Quickshell for performance

## Requirements

- Wayland compositor (Niri, Hyprland, Sway, Scroll, MangoWC or Labwc recommended)
- Quickshell
- crawlds-daemon (for system services)

## Quick Start

```bash
# Install crawlDS daemon first
curl -fsSL https://raw.githubusercontent.com/me-osano/crawlds/master/install.sh | sh

# Run the shell
qs -p ~/.config/quickshell/crawldesktopshell

# Or use the crawlds CLI
crawlds run
```

Add to your compositor's autostart to launch at login.

## Configuration

Settings are stored in `$XDG_CONFIG_HOME/crawlds/settings.json` (typically `~/.config/crawlds/settings.json`).

Key settings in `~/.config/crawlds/core.toml`:

```toml
[daemon]
log_level = "info"

[wallpaper]
enabled = true
directory = "~/Pictures/Wallpapers"

[notifications]
enabled = true
```

## Project Structure

```
quickshell/
├── shell.qml           # Entry point
├── Common/             # Shared components (Settings, Helpers, Types)
├── Services/          # Backend services (IPC, Theming, Compositor)
├── Modules/           # UI modules (Bar, Panels, Cards, Dock, LockScreen)
└── Widgets/           # Reusable QML widgets
```

## License

MIT — see [LICENSE](./LICENSE).
