# Theme System (v0.3)

CrawlDS uses a Rust-backed theme system that generates Material Design 3 themes using HCT color space. Both static themes (TOML) and dynamic themes (wallpaper-based) are supported via the crawlds-theme crate. Template rendering is handled in Rust with support for dual-mode (dark/light) templates.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Rust Daemon                                 │
├─────────────────────────────────────────────────────────────────────┤
│  crawlds-theme crate                                                │
│  ├── Static themes: TOML from assets/themes/                      │
│  ├── Dynamic themes: HCT color space (native Rust)                 │
│  │   ├── Color quantization (Wu + WSMeans)                        │
│  │   ├── Scheme generation (8 types)                           │
│  │   └── Template rendering                                      │
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
│  ├── themesList - available theme names                           │
│  ├── currentTheme - currently active theme                    │
│  ├── setTheme(name, variant)                                    │
│  ├── generate() - decides wallpaper vs static                 │
│  └── generateFromWallpaper() - dynamic theming                │
│         │                                                       │
│         ▼ loads                                                 │
│  TemplateService                                               │
│  └── applies to: GTK, Qt, KDE, foot, ghostty, etc.            │
│         │                                                       │
│         ▼ binds to                                               │
│  Color.qml (singleton) ──theme colors as QML properties       │
└─────────────────────────────────────────────────────────────────────┘
```

### HCT-Based Dynamic Theming

```
Wallpaper changed
    │
    ▼
ThemeService.generate()
    │ (checks useWallpaperColors setting)
    ├─ true: generateFromWallpaper()
    │                    │
    │                    ▼ extract colors
    │              Color Quantization (Wu/WSMeans)
    │                    │
    │                    ▼ generate M3 scheme
    │              HCT Color Space (8 schemes)
    │                    │
    │                    ▼
    │              ThemeData (dark + light)
    │                    │
    └────────────────────┼──────SSE──> ThemeService
                         │              │
                         ▼               ▼
                   ColorMap loaded  TemplateService.substitute()
                         │              │
                         └──────────────┼──> Rendered templates
                                        ▼
                                   UI updates
```

## Scheme Types

The crawlds-theme crate supports 8 Material Design 3 scheme types:

| Scheme | Description |
|--------|-------------|
| `tonal-spot` | Default Android 12+ scheme |
| `rainbow` | Rainbow hue rotation |
| `content` | Content-based from source image |
| `monochrome` | Monochrome tones |
| `fruit-salad` | -50° hue rotation |
| `vibrant` | Maximum chroma |
| `faithful` | Area-weighted tones |
| `muted` | Low saturation |

## Theme File Format (TOML)
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
    │                    ▼ extract colors
    │              Color Quantization (Wu/WSMeans)
    │                    │
    │                    ▼ generate M3 scheme
    │              HCT Color Space (8 schemes)
    │                    │
    │                    ▼
    │              ThemeData (dark + light)
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

Generate a dynamic theme from a wallpaper image.

**Request:**
```json
{
  "wallpaper_path": "/path/to/wallpaper.jpg",
  "variant": "dark",           // "dark" or "light"
  "color_index": 0,             // which extracted color to use (0-3)
  "theme_name": "Dynamic",       // optional name for the theme
  "scheme": "tonal-spot"        // optional: scheme type (default: tonal-spot)
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

#### POST /theme/dynamic/from_color

Generate a dynamic theme from a hex color.

**Request:**
```json
{
  "color": "#6200ee",
  "variant": "dark",
  "scheme": "vibrant",
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
- `dynamicTheme` - Currently active dynamic theme (from wallpaper)
- `hasDynamic` - Whether a dynamic theme is active

**Note:** Dynamic theming uses native HCT color space (no external dependencies).

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

## HCT Color Space

The theme system uses native HCT (Hue-Chroma-Tone) color space implementation:

### Components

1. **HCT** - Color space conversion (RGB <-> HCT <-> LAB)
2. **TonalPalette** - Generates tone-based palettes
3. **Color Quantization** - Extracts dominant colors from images
   - Wu's quantization algorithm
   - Weighted Spatial K-Means (WSMeans)
4. **Scheme Generation** - Creates M3 color schemes
   - 8 scheme types (tonal-spot, rainbow, content, monochrome, fruit-salad, vibrant, faithful, muted)

### Crate API

```rust
use crawlds_theme::{ThemeGenerator, GeneratorConfig, SchemeType};

// Generate from color
let generator = ThemeGenerator::new(GeneratorConfig {
    scheme_type: SchemeType::TonalSpot,
    color_index: 0,
});
let theme = generator.generate_from_color("#4285f4");
let theme_data = theme.to_theme_data("My Theme", "wallpaper.png");

// Generate from image
let theme = generator.generate_from_image_pixels(&pixels);
```

### Template Rendering

```rust
use crawlds_theme::{render_template, render_template_dual_mode, SchemeTonalSpot};

let scheme = SchemeTonalSpot::new(220.0);
let colors = scheme.get_light();

// Single mode
let output = render_template(template_content, &colors);

// Dual mode (dark + light)
let dark = scheme.get_dark();
let light = scheme.get_light();
let output = render_template_dual_mode(template_content, &dark, &light);
```

### Scheme Types

The `scheme` parameter in dynamic theme requests:

| Scheme | Usage |
|--------|-------|
| `tonal-spot` | Default Android 12+ look |
| `rainbow` | Full hue spectrum |
| `content` | Based on source image colors |
| `monochrome` | Grayscale |
| `fruit-salad` | Warm, saturated |
| `vibrant` | High contrast |
| `faithful` | Natural tones |
| `muted` | Subtle, desaturated |

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
5. **Dynamic theming** via native HCT (no external dependencies)



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
├── lib.rs           # Main exports, conversion functions
├── config.rs         # ThemeConfig, TemplateConfig
├── manager.rs        # ThemeManager
├── template/
│   └── mod.rs      # TemplateRenderer, DualModeTemplateRenderer
├── generic/
│   └── loader.rs   # TOML loading
└── dynamic/
    ├── mod.rs
    ├── generator/  # ThemeGenerator, SchemeType
    ├── hct/       # HCT color space
    ├── quantizer/  # Wu + WSMeans quantization
    ├── scheme/     # 8 M3 scheme implementations
    └── terminal/   # Terminal config generators
```

## File Structure

```
crawlds/
├── core/
│   ├──crates/
│   │   ├── crawlds-theme/       # Theme generation crate
│   │   │   └── src/
│   │   │       ├── lib.rs         # Main exports
│   │   │       ├── template/     # Template rendering
│   │   │       ├── generic/     # Static theme handling
│   │   │       └── dynamic/     # Dynamic theme (HCT)
│   │   ├── crawlds-ipc/        # Shared types
│   │   │   └── src/
│   │   │       ├── types.rs    # ThemeData, ThemeColors
│   │   │       └── events.rs   # ThemeEvent
│   │   └── crawlds-daemon/     # HTTP endpoints
│   │       └── src/router.rs   # /theme/* handlers
│   └── assets/
│       ├── themes/            # TOML themes
│       │   ├── Nord.toml
│       │   ├── Dracula.toml
│       │   └── ...
│       └── templates/          # Theme templates
│           ├── kitty.conf
│           ├── gtk3.css
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

### Completed (v0.3)

- [x] Rust-backed theme system with TOML themes
- [x] ThemeService with cache for fast startup
- [x] HCT color space (native Rust implementation)
- [x] Color quantization (Wu + WSMeans algorithms)
- [x] 8 scheme types (tonal-spot, rainbow, content, monochrome, fruit-salad, vibrant, faithful, muted)
- [x] Template rendering in Rust (DualModeTemplateRenderer)
- [x] TemplateService - pure QML template rendering (no Python)
- [x] Eliminated template-processor.py
- [x] Eliminated AppThemeService (merged into ThemeService)
- [x] Terminal theme generation (foot, ghostty, kitty, alacritty, wezterm, etc.)
- [x] Application templates (GTK, Qt, KDE, VSCode, Zed, helix, yazi, walker, etc.)
- [x] Theme list via ThemeService (migrated from ColorSchemeService)
- [x] Consistent JSON output (ThemeData) for both generic and dynamic theming
- [x] Dual-mode template support (dark/light in single template file)

### In Progress

- [ ] Full color palette support (expand from 16 to 48 colors)
- [ ] User template customization UI in settings
- [ ] Theme preview in settings
- [ ] Template file loading from core/assets/templates

### Planned

- [ ] Theme adapter abstraction (trait-based, swap Matugen/native/other engines)
- [ ] Explicit `PaletteGenerator` trait for custom palette generation
- [ ] ANSI mapper separation (extract terminal mapping to `mappers/` module)
- [ ] Contrast correction helper (`ensure_contrast()` for accessibility)
- [ ] Formalized IPC events (ThemeEvent enum beyond ThemeData)
- [ ] Theme editor (create custom themes)
- [ ] Import/export themes
- [ ] Theme gallery with online themes
- [ ] Animated color transitions
- [ ] Per-application color overrides
- [ ] Color blindness accessibility modes
- [ ] Theme presets for specific apps (gaming, coding, media)

# Architecture (Target)

```
crawlds-theme/
├── core/          # Pure logic (HCT, color math, tone selection)
├── adapters/     # Theme sources (Matugen, native, manual)
├── mappers/      # Material → terminal/UI role mapping
├── templates/    # Output generators (kitty, gtk, quickshell, etc.)
├── apply/        # Write configs + reload apps
└── ipc/         # Integration with crawlds-daemon
```

### Planned Crate Breakdown

| Module | Purpose |
|--------|---------|
| `core/` | Theme struct, Color types, HCT, tonal palettes |
| `adapters/` | `ThemeAdapter` trait, implementations |
| `mappers/` | ANSI mapping, contrast correction, Quickshell roles |
| `templates/` | Template generators using handlebars/tera |
| `apply/` | File writing, app reload signals |
| `ipc/` | Unix socket protocol, events |

### Target Data Flow

```
Wallpaper → Adapter → core → Mapper → Templates → Apply
                          ↓
                      Quickshell (via IPC)
```

### Deprecated

- `template-processor.py` - replaced by TemplateService (QML)
- `AppThemeService.qml` - merged into ThemeService
- `ColorSchemeService` - legacy, superseded by ThemeService
- `matugen` - optional, native HCT preferred
```