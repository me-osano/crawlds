import QtQuick
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Services.UI

Item {
  id: root

  // Provider metadata
  property string name: "Settings"
  property var launcher: null
  property bool handleSearch: Settings.data.appLauncher.enableSettingsSearch
  property string supportedLayouts: "list"
  property string iconMode: Settings.data.appLauncher.iconMode

  property var searchIndex: []

  FileView {
    id: searchIndexFile
    path: Quickshell.shellDir + "/Assets/settings-search-index.json"
    watchChanges: false
    printErrors: false

    onLoaded: {
      try {
        root.searchIndex = JSON.parse(text());
      } catch (e) {
        root.searchIndex = [];
      }
    }
  }

  function init() {
    Logger.d("SettingsProvider", "Initialized");
  }

  // Check if this provider handles the command
  function handleCommand(searchText) {
    return searchText.startsWith(">settings");
  }

  // Return available commands when user types ">"
  function commands() {
    return [
          {
            "name": ">settings",
            "description": "Search and navigate to settings",
            "icon": iconMode === "tabler" ? "settings" : "preferences-system",
            "isTablerIcon": true,
            "isImage": false,
            "onActivate": function () {
              launcher.setSearchText(">settings ");
            }
          }
        ];
  }

  function getResults(query) {
    if (!query || searchIndex.length === 0)
      return [];

    var trimmed = query.trim();

    // Handle command mode: ">settings" or ">settings <search>"
    var isCommandMode = trimmed.startsWith(">settings");
    if (isCommandMode) {
      // Extract search term after ">settings "
      var searchTerm = trimmed.substring(9).trim();
      // In command mode, show all settings if no search term
      if (searchTerm.length === 0) {
        return getAllSettings();
      }
      trimmed = searchTerm;
    } else {
      // Regular search mode - require at least 2 chars
      if (!trimmed || trimmed.length < 2)
        return [];
    }

    // Build searchable items
    let items = [];
    for (let j = 0; j < searchIndex.length; j++) {
      const entry = searchIndex[j];
      items.push({
                   "label": entry.label || "",
                   "description": entry.description || "",
                   "widget": entry.widget,
                   "tab": entry.tab,
                   "tabLabel": entry.tabLabel,
                   "subTab": entry.subTab,
                   "subTabLabel": entry.subTabLabel || null,
                   "subTabName": entry.subTabLabel || ""
                 });
    }

    const results = FuzzySort.go(trimmed, items, {
                                   "keys": ["label", "subTabName", "description"],
                                   "limit": 10,
                                   "scoreFn": function (r) {
                                     const labelScore = r[0].score;
                                     const subTabScore = r[1].score * 1.5;
                                     const descScore = r[2].score;
                                     return Math.max(labelScore, subTabScore, descScore);
                                   }
                                 });

    let launcherItems = [];
    for (let i = 0; i < results.length; i++) {
      const entry = results[i].obj;
      const score = results[i].score;
      const tabName = entry.tabLabel || "";
      const subTabName = entry.subTabName || "";
      const breadcrumb = subTabName ? (tabName + " › " + subTabName) : tabName;

      launcherItems.push({
                           "name": entry.label,
                           "description": breadcrumb,
                           "icon": iconMode === "tabler" ? "settings" : "preferences-system",
                           "isTablerIcon": true,
                           "isImage": false,
                           "_score": score - 2,
                           "provider": root,
                           "onActivate": createActivateHandler(entry)
                         });
    }

    return launcherItems;
  }

  function getAllSettings() {
    var launcherItems = [];

    for (var j = 0; j < searchIndex.length; j++) {
      var entry = searchIndex[j];
      var label = entry.label || "";
      var tabName = entry.tabLabel || "";
      var subTabName = entry.subTabLabel || "";
      var breadcrumb = subTabName ? (tabName + " › " + subTabName) : tabName;

      launcherItems.push({
                           "name": label,
                           "description": breadcrumb,
                           "icon": iconMode === "tabler" ? "settings" : "preferences-system",
                           "isTablerIcon": true,
                           "isImage": false,
                           "_score": 0,
                           "provider": root,
                           "onActivate": createActivateHandler({
                                                                 "label": entry.label,
                                                                 "description": entry.description,
                                                                 "widget": entry.widget,
                                                                 "tab": entry.tab,
                                                                 "tabLabel": entry.tabLabel,
                                                                 "subTab": entry.subTab,
                                                                 "subTabLabel": entry.subTabLabel || null
                                                               })
                         });
    }

    return launcherItems;
  }

  function createActivateHandler(entry) {
    return function () {
      if (launcher)
        launcher.close();

      Qt.callLater(() => {
                     SettingsPanelService.openToEntry(entry, launcher.screen);
                   });
    };
  }
}
