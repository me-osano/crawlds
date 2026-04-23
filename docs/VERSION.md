Analysis
1. Daemon Version Endpoint
- The daemon already has a JSON-RPC structure in json_server.rs
- Commands are defined as an enum Command 
- There's already a Hello command that returns version info (line 324-330)
- The Command::Hello already returns env!("CARGO_PKG_VERSION") - but we need git info too
- The daemon needs a build.rs similar to CLI to inject git version
2. Update Checker
- Need to add GitHub API query to either daemon or CLI
- Can use reqwest or the existing HTTP client
- Repository is https://github.com/me-osano/crawlds
- Need to handle: fetch latest release, compare versions, detect if update available
3. Quickshell Integration
- CrawlDSService.qml already connects to the daemon
- It has connection state properties like crawldsAvailable
- Could add version info to CrawlDSService
- Settings → About UI could display this
Let me create the plan now.
Now I have enough context. Here's the comprehensive plan:
---
Plan: Version + Update Checker Implementation
1. Daemon Version Endpoint
Files to Modify:
- core/crates/crawlds-daemon/build.rs (new) — inject git version like CLI
- core/crates/crawlds-daemon/src/json_server.rs — add Version command
Changes:
// In Command enum:
Version,
// In execute():
Command::Version => self.version().await,
// New handler:
async fn version(&self) -> JsonRpcResponse {
    JsonRpcResponse::success(None, serde_json::json!({
        "version": env!("CRAWLDS_VERSION"),
        "git_sha": env!("CRAWLDS_GIT_SHA"),
        "git_tag": option_env!("CRAWLDS_GIT_TAG"),
    }))
}
---
2. Update Checker (CLI + Daemon)
Option: Add to daemon as VersionLatest command (daemon queries GitHub API)
Files:
- core/crates/crawlds-daemon/src/json_server.rs — add VersionLatest command
- core/crates/crawlds-cli/src/cmd/version.rs — add --check flag
GitHub API:
GET https://api.github.com/repos/me-osano/crawlds/releases/latest
Response: { "tag_name": "v0.1.0", ... }
Dependencies: Add reqwest to daemon for HTTP calls
Response:
{
  "current": "0.1.0",
  "latest": "v0.2.0", 
  "has_update": true,
  "release_url": "https://github.com/me-osano/crawlds/releases/tag/v0.2.0"
}
---
3. Quickshell Integration
Files:
- quickshell/Services/Core/CrawlDSService.qml — add version properties
- quickshell/Modules/Settings/ — add version display (About section)
Changes:
// CrawlDSService.qml - add properties:
property string daemonVersion: ""
property string daemonGitSha: ""
property bool hasUpdate: false
property string latestVersion: ""
// Fetch on connection:
function fetchVersion() {
    ipc.request("Version", {}, (resp) => {
        daemonVersion = resp.result.version;
        daemonGitSha = resp.result.git_sha;
    });
}
---
Summary Table
Feature	Location
Daemon /Version endpoint	json_server.rs
Daemon build.rs	New file
/VersionLatest (GitHub API)	json_server.rs
CLI version --check	cmd/version.rs
Quickshell version display	CrawlDSService.qml
Settings → About UI	New QML component
---
Questions
1. GitHub API: Should update checking be in daemon or CLI?
   - Daemon: persistent, can show notification on new release
   - CLI: only when user runs crawlds version --check
2. Update notification: Should daemon notify Quickshell when new version available? (adds complexity)
3. Quickshell version display: Where exactly in Settings? A new "About" tab or add to existing?


opencode
                                   ▄
  █▀▀█ █▀▀█ █▀▀█ █▀▀▄ █▀▀▀ █▀▀█ █▀▀█ █▀▀█
  █  █ █  █ █▀▀▀ █  █ █    █  █ █  █ █▀▀▀
  ▀▀▀▀ █▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀

  Session   Plan mode: system reminder dialogue and read-only…
  Continue  opencode -s ses_251308953ffeudlRYgh78eSyb3