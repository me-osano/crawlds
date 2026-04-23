# Webservice Module

CrawlDS Webservice provides RSS feed aggregation and Wallhaven wallpaper API integration through the unified daemon architecture.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           BACKEND (Rust)                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-webservice                                                    │   │
│  │   ├── WebserviceState    (shared feeds, API keys)                   │   │
│  │   ├── RssWorker          (feed-rs, polls configured feeds)           │   │
│  │   └── WallhavenWorker    (reqwest, wallpaper search/fetch)         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-daemon AppState                                              │   │
│  │   └── webservice_store: Arc<WebserviceState>                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ HTTP Router (axum)                                                  │   │
│  │   GET  /rss/feeds       → webservice_store.feeds                    │   │
│  │   POST /rss/add          → webservice_store.add_feed()              │   │
│  │   POST /rss/remove       → webservice_store.remove_feed()          │   │
│  │   POST /rss/refresh      → triggers RSS worker refresh              │   │
│  │   GET  /wallhaven/search → WallhavenWorker.search()                │   │
│  │   GET  /wallhaven/random → WallhavenWorker.random()                │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                         (SSE + HTTP Response)                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND (QML)                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CrawlDSService                                                       │   │
│  │   ├── rssFeedUpdated(data) signal    (via SSE)                      │   │
│  │   ├── wallhavenResults(data) signal  (via SSE)                      │   │
│  │   ├── rssAddFeed(url)              (HTTP → add to WebserviceState) │   │
│  │   ├── rssRemoveFeed(url)            (HTTP → remove from store)      │   │
│  │   ├── rssRefresh()                  (HTTP → trigger worker refresh)  │   │
│  │   ├── wallhavenSearch(q, tags, page) (HTTP → WallhavenWorker)       │   │
│  │   └── wallhavenRandom(count)         (HTTP → WallhavenWorker)       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ RSSService.qml                                                       │   │
│  │   ├── Manages feeds[] and items[] from SSE events                   │   │
│  │   ├── Syncs feeds to Settings.data.rss (settings.json)              │   │
│  │   └── Provides addFeed(), removeFeed(), search(), openItem()        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ WallhavenService.qml                                                 │   │
│  │   ├── Uses CrawlDSService for all wallpaper operations              │   │
│  │   ├── Listens for wallhavenResults signal                           │   │
│  │   ├── Stores favorites in Settings.data.wallhavenFavorites           │   │
│  │   └── Provides downloadWallpaper(), setAsWallpaper()                │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Configuration

In `core.toml`:

```toml
[webservice]

[webservice.rss]
enabled = true
feeds = [
    "https://example.com/feed.xml",
    "https://blog.example.com/rss",
]
poll_interval_secs = 300  # 5 minutes
user_agent = "crawlds/0.1"
timeout_secs = 10

[webservice.wallhaven]
api_key = ""  # Optional: get from wallhaven.cc
default_purity = ["sfw"]
default_categories = "111"  # anime, people, general
```

### RSS Categories
- `100` = general only
- `010` = anime only
- `001` = people only
- `111` = all (default)

### Wallhaven Purity
- `100` = SFW (safe for work)
- `010` = Sketchy
- `001` = NSFW
- Combinations like `110`, `101`, `011`, `111`

## HTTP API

### RSS Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/rss/feeds` | GET | List configured feeds |
| `/rss/add` | POST | Add a feed URL |
| `/rss/remove` | POST | Remove a feed URL |
| `/rss/refresh` | POST | Force refresh all feeds |

#### Request/Response Examples

**Add Feed**
```bash
curl -X POST \
  --unix-socket /run/crawlds.sock \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/feed.xml"}' \
  http://localhost/rss/add
```
```json
{"ok": true, "message": "Feed added", "url": "https://example.com/feed.xml"}
```

**Remove Feed**
```bash
curl -X POST \
  --unix-socket /run/crawlds.sock \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/feed.xml"}' \
  http://localhost/rss/remove
```

**List Feeds**
```bash
curl --unix-socket /run/crawlds.sock http://localhost/rss/feeds
```
```json
{
  "feeds": [
    "https://example.com/feed.xml",
    "https://blog.example.com/rss"
  ],
  "status": "ok"
}
```

### Wallhaven Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/wallhaven/search` | GET | Search wallpapers |
| `/wallhaven/random` | GET | Get random wallpapers |

#### Query Parameters for /wallhaven/search

| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Search query |
| `tags` | string[] | Filter by tags |
| `categories` | string | Category filter (e.g., "111") |
| `purity` | string[] | Purity filter (e.g., ["sfw"]) |
| `page` | u32 | Page number (default: 1) |

#### Request/Response Examples

**Search Wallpapers**
```bash
curl "http://localhost/wallhaven/search?q=nature&purity=sfw&page=1" \
  --unix-socket /run/crawlds.sock
```
```json
{
  "walls": [
    {
      "id": "abc123",
      "url": "https://wallhaven.cc/w/abc123",
      "thumb_url": "https://th.wallhaven.cc/lg/xy/abc123.jpg",
      "resolution": "1920x1080",
      "purity": "sfw",
      "tags": ["nature", "landscape"]
    }
  ]
}
```

**Random Wallpapers**
```bash
curl "http://localhost/wallhaven/random?count=10" \
  --unix-socket /run/crawlds.sock
```

## Backend Crate

**Location**: `core/crates/crawlds-webservice/`

### Modules

- `lib.rs` - Main entry, `WebserviceState`, `run()` functions
- `config.rs` - Configuration types (`Config`, `RssConfig`, `WallhavenConfig`)
- `rss.rs` - RSS feed fetching and parsing
- `wallhaven.rs` - Wallhaven API client
- `http_client.rs` - Shared HTTP client with connection pooling and caching

### WebserviceState

The `WebserviceState` is the central shared state that connects the HTTP router to the workers:

```rust
#[derive(Clone)]
pub struct WebserviceState {
    pub feeds: Arc<Mutex<Vec<String>>>,        // RSS feed URLs
    pub wallhaven_api_key: Option<String>,     // Wallhaven API key
    pub wallhaven_config: WallhavenConfig,     // Wallhaven settings
    pub tx: broadcast::Sender<CrawlEvent>,      // Event broadcaster
}
```

**Key Methods:**
- `add_feed(url)` - Add a feed URL
- `remove_feed(url)` - Remove a feed URL
- `wallhaven_search(...)` - Spawn async wallpaper search
- `wallhaven_random(count)` - Spawn async random wallpaper fetch

### HTTP Client

Shared `HttpClient` providing:
- Connection pooling (TCP keep-alive, max idle per host)
- HTTP caching for RSS (ETag, Last-Modified headers)
- Per-request timeouts

```rust
pub struct HttpClient {
    client: reqwest::Client,     // Shared connection pool
    cache: Arc<RwLock<HttpCache>>, // ETag/Last-Modified storage
}
```

**Optimization Features:**
- TCP keep-alive: 30 seconds
- Pool max idle per host: 5 connections
- Automatic 304 Not Modified handling

### RSS Worker

Uses `feed-rs` crate for parsing:
- RSS 0.9/1.0/2.0
- Atom feeds
- JSON Feed

**Features:**
- HTTP caching (ETag, If-Modified-Since)
- Configurable poll interval
- Deduplication
- Thumbnail extraction from media extensions
- Error handling with retry

**Performance:**
- On 304 Not Modified: skips parsing entirely
- Reduces network ~80% when feeds haven't changed

### Wallhaven Worker

**Features:**
- Search with query, tags, categories, purity filters
- Random wallpaper fetching
- Optional API key support
- Pagination

**Performance:**
- Uses shared HTTP client with connection pooling
- On-demand (triggered by UI), not background polling

## QML Services

### RSSService

```qml
// Initialize (auto-loads from Settings.data.rss)
RSSService.init()

// Add a feed (syncs to backend WebserviceState and settings.json)
RSSService.addFeed("https://example.com/feed.xml")

// Remove a feed
RSSService.removeFeed("https://example.com/feed.xml")

// Refresh all feeds
RSSService.refresh()

// Search items
const results = RSSService.search("keyword")

// Open item in browser
RSSService.openItem(item)

// Get items from specific feed
const feedItems = RSSService.getItemsByFeed("https://example.com/feed.xml")

// Enable/disable RSS
RSSService.setEnabled(true)

// Properties
RSSService.items    // Array of RSS items
RSSService.feeds    // Array of feed URLs
RSSService.enabled  // Boolean

// Signals
RSSService.itemsUpdated()
RSSService.feedsUpdated()
```

### WallhavenService

```qml
// Search wallpapers
WallhavenService.search("nature", 1)

// Search with filters
WallhavenService.searchWithFilters("landscape", 1, {
    categories: "111",
    purity: ["sfw"],
    sorting: "random"
})

// Get random wallpapers
WallhavenService.random(30)

// Listen for results
Connections {
    target: CrawlDSService
    function onWallhavenResults(data) {
        console.log("Got wallpapers:", data.walls.length)
    }
}

// Download wallpaper
WallhavenService.downloadWallpaper(wallpaper, callback)

// Set as desktop wallpaper
WallhavenService.setAsWallpaper(wallpaper)

// Manage favorites
WallhavenService.addToFavorites(wallpaper)
WallhavenService.removeFromFavorites(wallpaper.id)
WallhavenService.isFavorite(wallpaper.id)

// Properties
WallhavenService.currentResults  // Array of wallpaper objects
WallhavenService.favorites       // Array of favorited wallpapers
WallhavenService.effectiveApiKey // Resolved API key
WallhavenService.apiKeyManagedByEnv  // Boolean

// Signals
WallhavenService.searchCompleted(results, meta)
WallhavenService.searchFailed(error)
WallhavenService.wallpaperDownloaded(id, localPath)
```

## Events (SSE)

### RSS Events

```json
{
    "domain": "webservice",
    "data": {
        "event": "rss_feed_updated",
        "feed_url": "https://example.com/feed.xml",
        "items": [
            {
                "feed_title": "Example Blog",
                "title": "Post Title",
                "link": "https://example.com/post",
                "published": "2024-01-15T10:30:00Z",
                "summary": "Post summary...",
                "thumbnail": "https://example.com/thumb.jpg"
            }
        ]
    }
}
```

### Wallhaven Events

```json
{
    "domain": "webservice",
    "data": {
        "event": "wallhaven_results",
        "walls": [
            {
                "id": "abc123",
                "url": "https://wallhaven.cc/w/abc123",
                "thumb_url": "https://th.wallhaven.cc/lg/xy/abc123.jpg",
                "resolution": "1920x1080",
                "purity": "sfw",
                "tags": ["nature", "landscape"],
                "uploaded_at": "2024-01-10",
                "file_size": 2456789
            }
        ]
    }
}
```

## Dependencies

### Backend

- `feed-rs` - RSS/Atom/JSON Feed parsing
- `reqwest` - HTTP client with TLS support
- `tokio` - Async runtime
- `sysinfo` - System information (future use)

### Frontend

- `CrawlDSService` - SSE connection and HTTP API
- `NotificationService` - For download notifications

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CRAWLDS_WALLHAVEN_API_KEY` | Wallhaven API key (overrides config) |

## Settings Storage

RSS feeds and Wallhaven settings are persisted in two places:

1. **settings.json** (QML/Frontend)
   - `Settings.data.rss` - RSS configuration including feed URLs
   - `Settings.data.wallhavenFavorites` - Saved wallpaper favorites

2. **core.toml** (Rust/Backend)
   - `[webservice.rss]` - Default feeds, poll intervals
   - `[webservice.wallhaven]` - API key, default filters

The QML services sync feeds to both locations:
- On startup: Load feeds from `Settings.data.rss`
- On add/remove: Update both backend `WebserviceState` and `Settings.data`

## Troubleshooting

### RSS feeds not updating

1. Check feed URLs are valid:
   ```bash
   curl -I https://example.com/feed.xml
   ```

2. Check config:
   ```toml
   [webservice.rss]
   enabled = true
   ```

3. Check daemon logs for RSS errors

### Wallhaven search fails

1. Check API key is valid
2. Check rate limiting (45 requests/minute for unauthenticated)
3. Try without API key for basic search

### No events received

1. Check SSE connection:
   ```bash
   curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/events
   ```

2. Verify daemon is running:
   ```bash
   curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/health
   ```

### Feed added but not appearing

Feeds added via QML are stored in `Settings.data.rss` and synced to `WebserviceState`. They are not automatically written to `core.toml`. To persist feeds across daemon restarts, they should be added to the config file.
