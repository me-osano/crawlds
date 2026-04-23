# Theme System (v0.2)

CrawlDS uses a Rust-backed theme system that loads TOML theme files and serves them to Quickshell via IPC/SSE for real-time theme switching. It also supports dynamic theming using matugen to generate colors from wallpapers. Template rendering is handled entirely in QML with no Python dependencies.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Rust Daemon                                 │
├─────────────────────────────────────────────────────────────────────┤
│  crawlds-theme crate                                                │
│  ├── Static themes: TOML from assets/themes/                      │
│  ├── Dynamic themes: matugen CLI integration                       │
│  ├── ThemeManager - manages theme cache and current              │
│  └── HTTP endpoints for theme operations                           │
│         │                                                           │
│         ▼ broadcasts                                                │
│  CrawlEvent::Theme(ThemeEvent) ──SSE──> Quickshell                │
└─────────────────────────────────────────────────────────────────────┘
          │ SSE
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Quickshell QML                                │
├─────────────────────────────────────────────────────────────────────┤
│  ThemeService (singleton)                                          │
│  ├── themesList - available theme names                             │
│  ├── currentTheme - currently active theme                         │
│  ├── setTheme(name, variant)                                   │
│  ├── generate() - decides wallpaper vs static                    │
│  └── generateFromWallpaper() - dynamic theming                   │
│         │                                                           │
│         ▼ loads color map                                          │
│  TemplateService                                                │
│  ├── colorMapCache - {{primary}} substitution                 │
│  ├── terminalMapCache - terminal colors                       │
│  └── substitute(content) - renders templates                 │
│         │                                                           │
│         ▼ writes                                                 │
│  TemplateRegistry - app/terminal configs                       │
│  └── applies to: GTK, Qt, KDE, foot, ghostty, etc.             │
│         │                                                           │
│         ▼ binds to                                               │
│  Color.qml (singleton) ──theme colors as QML properties         │
└─────────────────────────────────────────────────────────────────────┘
```

### Dynamic Theming Flow

```
Wallpaper changed
    │
    ▼
ThemeService.generate()
    │ (checks useWallpaperColors setting)
    ├─ true: generateFromWallpaper()
    │                    │
    │                    ▼ (HTTP POST)
    │              daemon /theme/dynamic/generate
    │                    │
    │                    ▼ (calls matugen CLI)
    │              matugen image <wallpaper>
    │                    │
    │                    ▼
    │              ThemeData (validated)
    │                    │
    └────────────────────┼──────SSE──> ThemeService
                         │              │
                         ▼               ▼
                   ColorMap loaded  TemplateService.substitute()
                         │              │
                         └──────────────┼──> Rendered templates
                                        ▼
                                   UI updates (instant!)
```

## Theme File Format (TOML)

Themes are stored in `asset/themes/*.toml`:

```toml
[metadata]
name = "Nord"
author = " arcticicestudio"
isDark = false

[dark]
primary = "#8fbcbb"
onPrimary = "#2e3440"
secondary = "#88c0d0"
onSecondary = "#2e3440"
tertiary = "#5e81ac"
onTertiary = "#2e3440"
error = "#bf616a"
onError = "#2e3440"
surface = "#2e3440"
onSurface = "#eceff4"
surfaceVariant = "#3b4252"
onSurfaceVariant = "#d8dee9"
outline = "#505a70"
shadow = "#2e3440"
hover = "#5e81ac"
onHover = "#2e3440"

[dark.terminal.normal]
black = "#3b4252"
red = "#bf616a"
green = "#a3be8c"
yellow = "#ebcb8b"
blue = "#81a1c1"
magenta = "#b48ead"
cyan = "#88c0d0"
white = "#e5e9f0"

[dark.terminal.bright]
black = "#596377"
red = "#bf616a"
green = "#a3be8c"
yellow = "#ebcb8b"
blue = "#81a1c1"
magenta = "#b48ead"
cyan = "#8fbcbb"
white = "#eceff4"

[dark.terminal]
foreground = "#d8dee9"
background = "#2e3440"
selectionFg = "#4c566a"
selectionBg = "#eceff4"
cursorText = "#282828"
cursor = "#eceff4"

[light]
# ... same structure for light variant
```

### Schema

| Section | Field | Type | Description |
|---------|-------|------|-------------|
| `metadata` | `name` | string | Theme display name |
| `metadata` | `author` | string | Theme author |
| `metadata` | `isDark` | bool | Default dark mode preference |
| `dark`/`light` | `primary` | hex color | Primary accent color |
| `dark`/`light` | `onPrimary` | hex color | Text on primary |
| `dark`/`light` | `secondary` | hex color | Secondary accent |
| `dark`/`light` | `onSecondary` | hex color | Text on secondary |
| `dark`/`light` | `tertiary` | hex color | Tertiary accent |
| `dark`/`light` | `onTertiary` | hex color | Text on tertiary |
| `dark`/`light` | `error` | hex color | Error color |
| `dark`/`light` | `onError` | hex color | Text on error |
| `dark`/`light` | `surface` | hex color | Main background |
| `dark`/`light` | `onSurface` | hex color | Text on surface |
| `dark`/`light` | `surfaceVariant` | hex color | Elevated surface |
| `dark`/`light` | `onSurfaceVariant` | hex color | Text on surface variant |
| `dark`/`light` | `outline` | hex color | Border/divider |
| `dark`/`light` | `shadow` | hex color | Shadow color |
| `dark`/`light` | `hover` | hex color | Hover state |
| `dark`/`light` | `onHover` | hex color | Text on hover |
| `*.terminal.normal` | `black`..`white` | hex color | Standard terminal colors |
| `*.terminal.bright` | `black`..`white` | hex color | Bright terminal colors |
| `*.terminal` | `foreground` | hex color | Terminal text |
| `*.terminal` | `background` | hex color | Terminal background |
| `*.terminal` | `selectionFg` | hex color | Selection text |
| `*.terminal` | `selectionBg` | hex color | Selection background |
| `*.terminal` | `cursorText` | hex color | Cursor text color |
| `*.terminal` | `cursor` | hex color | Cursor color |

## HTTP API

All endpoints are relative to the daemon socket (e.g., `http://unix:/run/user/1000/crawlds.sock` or TCP if configured).

### Static Theme Endpoints

#### GET /theme/list

List all available themes.

**Response:**
```json
{
  "ok": true,
  "themes": ["Nord", "Dracula", "Catppuccin", ...]
}
```

#### GET /theme/current

Get the currently active theme.

**Response:**
```json
{
  "ok": true,
  "name": "Nord",
  "theme": { ... }
}
```

#### GET /theme/get?name={name}

Get a specific theme by name.

**Response:**
```json
{
  "ok": true,
  "theme": { ... }
}
```

#### POST /theme/set

Set the active theme.

**Request:**
```json
{
  "name": "Nord",
  "variant": "dark"  // optional: "dark" or "light"
}
```

**Response:**
```json
{
  "ok": true,
  "name": "Nord",
  "variant": "dark",
  "theme": { ... }
}
```

**Side effect:** Broadcasts `ThemeEvent::Changed` via SSE.

### Dynamic Theme Endpoints

#### POST /theme/dynamic/generate

Generate a dynamic theme from a wallpaper image using matugen.

**Request:**
```json
{
  "wallpaper_path": "/path/to/wallpaper.jpg",
  "variant": "dark",           // "dark" or "light"
  "color_index": 0,             // which extracted color to use (0-3)
  "theme_name": "Dynamic"       // optional name for the theme
}
```

**Response:**
```json
{
  "ok": true,
  "name": "Dynamic",
  "variant": "dark",
  "theme": { ... }
}
```

**Requirements:** matugen CLI must be installed and available in PATH.

#### POST /theme/dynamic/from_color

Generate a dynamic theme from a hex color using matugen.

**Request:**
```json
{
  "color": "#6200ee",
  "variant": "dark",
  "name": "Dynamic-Custom"
}
```

**Response:**
```json
{
  "ok": true,
  "name": "Dynamic-Custom",
  "variant": "dark",
  "theme": { ... }
}
```

## SSE Events

Theme events are broadcast over the `/events` SSE endpoint:

```json
{
  "domain": "theme",
  "data": {
    "event": "changed",
    "theme": { ... },
    "variant": "dark"
  }
}
```

### Event Types

| Event | Data | Description |
|-------|------|-------------|
| `loaded` | `{theme}` | Initial theme loaded |
| `changed` | `{theme, variant}` | Theme changed |
| `list_updated` | `{themes}` | Theme list updated |
| `error` | `{message}` | Error occurred |
| `dynamic_generated` | `{theme, variant}` | Dynamic theme generated from wallpaper |
| `dynamic_error` | `{message}` | Error generating dynamic theme |

## QML Services

### ThemeService

**Location:** `quickshell/Services/Theme/ThemeService.qml`

**Properties:**
- `currentTheme` - Current theme data object
- `themesList` - Array of available theme names
- `themeCache` - Map of name -> theme data (preloaded)
- `currentThemeName` - Name of current theme
- `currentVariant` - "dark" or "light"
- `isDark` - Boolean dark mode state
- `loaded` - Whether initial load completed
- `cacheReady` - Whether all themes are preloaded
- `dynamicTheme` - Currently active dynamic theme (from wallpaper)
- `hasDynamic` - Whether a dynamic theme is active
- `dynamicAvailable` - Whether matugen is available

**Methods:**
- `init()` - Initialize and load themes
- `create()` - Load current theme from daemon
- `setTheme(name, variant)` - Change to a static theme
- `getTheme(name)` - Get theme from cache
- `generate()` - Generate theme based on settings (wallpaper vs predefined)
- `generateFromWallpaper()` - Generate dynamic theme from wallpaper
- `generateFromWallpaper(path, variant, colorIndex, name)` - Generate dynamic theme from wallpaper path
- `generateFromColor(color, variant, name)` - Generate dynamic theme from hex color
- `clearDynamicTheme()` - Clear the dynamic theme

**Signals:**
- `cacheUpdated()` - Emitted when all themes are preloaded
- `dynamicThemeGenerated()` - Emitted when dynamic theme is ready
- `dynamicThemeError(message)` - Emitted when dynamic theme generation fails

### Color

**Location:** `quickshell/Common/Theme.qml`

Singleton providing direct color properties bound to the current theme:

```qml
import qs.Services.Theme

Rectangle {
    color: Theme.primary
    text.color: Theme.onPrimary
}
```

**Properties:**
- `primary`, `onPrimary`
- `secondary`, `onSecondary`
- `tertiary`, `onTertiary`
- `error`, `onError`
- `surface`, `onSurface`
- `surfaceVariant`, `onSurfaceVariant`
- `outline`, `shadow`, `hover`, `onHover`
- Terminal colors: `terminalBlack`, `terminalRed`, etc.
- `terminalForeground`, `terminalBackground`, etc.
- `isDark`, `variant`

### Style

**Location:** `quickshell/Services/Theme/Style.qml`

Singleton providing derived style objects:

```qml
import qs.Services.Theme

Rectangle {
    color: Style.buttonPrimary.background
    text.color: Style.buttonPrimary.text
}
```

**Style Groups:**
- `buttonPrimary`, `buttonSecondary`, `buttonTertiary`, `buttonOutline`, `buttonGhost`
- `surfaceDefault`, `surfaceElevated`, `surfaceCard`
- `inputField`
- `scrollbar`
- `divider`
- `badgePrimary`, `badgeSecondary`, `badgeError`
- `progress`, `slider`
- `tooltip`, `popup`
- `tab`, `navigation`

### TemplateService

**Location:** `quickshell/Services/Theme/TemplateService.qml`

Handles template rendering (applying theme to applications):

- Reads colors from `ThemeService.currentTheme`
- Substitutes `{{primary}}`, `{{onPrimary}}`, etc. in template files
- Writes rendered templates to user config directories
- Executes post-apply hooks (e.g., `template-apply.sh foot`)

**Properties:**
- `colorMapCache` - Map of color names to hex values (dark variant)
- `terminalMapCache` - Map of terminal color names to hex values

**Methods:**
- `init()` - Initialize and load color maps
- `loadColorMap()` - Load colors from current theme
- `substitute(content)` - Replace template variables in content
- `applyAllTemplates()` - Apply theme to all enabled applications

**Signals:**
- `templatesApplied()` - Emitted when templates are rendered

### TemplateRegistry

**Location:** `quickshell/Services/Theme/TemplateRegistry.qml`

Defines available template configurations:

- `terminals` - Array of terminal configs (foot, ghostty, kitty, alacritty, wezterm)
- `applications` - Array of app configs (GTK, Qt, KDE, Discord, VSCode, Zed, etc.)

## Matugen Integration

### Installation

Install matugen CLI:

```bash
# Via cargo
cargo install matugen

# Or via package manager (if available)
# Arch: pacman -S matugen
```

### How It Works

1. matugen extracts dominant colors from the wallpaper using `material-color-utilities` library
2. Applies Material Design 3 (Material You) color scheme generation
3. Outputs JSON with full color palette for dark/light/amoled variants
4. crawlds-theme parses this and converts to `ThemeData` format
5. Terminal colors are auto-generated from the primary color

### Color Index

The `--source-color-index` flag (default: 0) controls which extracted color to use:
- 0 - Primary dominant color
- 1 - Secondary color
- 2 - Tertiary color
- 3 - Quaternary color

## IPC Integration

Themes can be switched via IPC:

```
crawlds-cli ipc theme set Nord dark
crawlds-cli ipc theme list
crawlds-cli ipc theme get
```

Or via QML IPC handler:

```javascript
await IPC.invoke("theme", "set", "Nord", "dark")
```

## Migration from ColorSchemeService

The old `ColorSchemeService` used JSON files in `ColorScheme/` directory and wrote to `colors.json`. The new system:

1. **Uses TOML** instead of JSON (cleaner, validated)
2. **Rust-backed** for validation and consistency
3. **SSE-based** for real-time updates
4. **Preloaded cache** for instant theme switching
5. **Dynamic theming** via matugen for wallpaper-based colors



## Configuration

The daemon needs `assets_dir` configured to point to the `quickshell/assets` directory:

```toml
# ~/.config/crawlds/core.toml
assets_dir = "/path/to/quickshell/assets"
```

Default: `$HOME/Projects/CrawlDS/quickshell/assets`

### Enabling Dynamic Theming

In settings, enable wallpaper-based colors:

```json
{
  "colorSchemes": {
    "useWallpaperColors": true,
    "darkMode": true,
    "monitorForColors": "DP-1"
  }
}
```

## Crate Structure

```
crawlds-theme/src/
├── lib.rs           # Main exports
├── error.rs         # ThemeError, ThemeResult
├── types.rs         # Re-exports from crawlds-ipc
├── static/
│   ├── mod.rs
│   ├── loader.rs    # TOML loading + validation
│   └── manager.rs   # ThemeManager
└── dynamic/
    ├── mod.rs
    ├── matugen.rs   # matugen CLI wrapper
    └── generator.rs # ThemeData generation
```

## File Structure

```
crawlds/
├── core/
│   ├──crates/
│   │   ├── crawlds-theme/       # Theme loading crate
│   │   └── src/
│   │   │       ├── lib.rs       # Main exports
│   │   │       ├── static/      # Static theme handling
│   │   │       └── dynamic/     # Dynamic theme (matugen)
│   │   ├── crawlds-ipc/        # Shared types
│   │   │   └── src/
│   │   │       ├── types.rs    # ThemeData, ThemeColors
│   │   │       └── events.rs   # ThemeEvent
│   │   └── crawlds-daemon/     # HTTP endpoints
│   │       └── src/router.rs   # /theme/* handlers
│   └── assets/
│       └── themes/
│           ├── Nord.toml
│           ├── Dracula.toml
│           └── ...
├── quickshell/
│   └── Services/
│       └── Theme/
│           ├── ThemeService.qml    # Main service
│           ├── TemplateService.qml # Template rendering
│           ├── TemplateRegistry.qml # Template configs
│           ├── Color.qml           # Theme colors
│           └── Style.qml           # Derived styles
└── docs/
    └── THEME.md
```

## Roadmap

### Completed (v0.2)

- [x] Rust-backed theme system with TOML themes
- [x] ThemeService with cache for fast startup
- [x] Dynamic theming via matugen
- [x] TemplateService - pure QML template rendering (no Python)
- [x] Eliminated template-processor.py
- [x] Eliminated AppThemeService (merged into ThemeService)
- [x] Terminal theme generation (foot, ghostty, kitty, alacritty, wezterm)
- [x] Application templates (GTK, Qt, KDE, Discord, VSCode, Zed, etc.)
- [x] Theme list via ThemeService (migrated from ColorSchemeService)

### In Progress

- [ ] Full color palette support (expand from 16 to 48 colors)
- [ ] User template customization UI in settings
- [ ] Theme preview in settings

### Planned

- [ ] Theme editor (create custom themes)
- [ ] Import/export themes
- [ ] Theme gallery with online themes
- [ ] Animated color transitions
- [ ] Per-application color overrides
- [ ] Color blindness accessibility modes
- [ ] Theme presets for specific apps (gaming, coding, media)
- [ ] Wallpaper color extraction (native Rust, remove Python dependency)

### Deprecated

- `template-processor.py` - replaced by TemplateService (QML)
- `AppThemeService.qml` - merged into ThemeService
- `ColorSchemeService` - legacy, superseded by ThemeService
```