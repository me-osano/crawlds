import QtQuick
import Quickshell
import qs.Common

Item {
  property var launcher: null
  property string name: "Command"
  property string iconMode: Settings.data.appLauncher.iconMode

  function handleCommand(query) {
    return query.startsWith(">cmd");
  }

  function commands() {
    return [
          {
            "name": ">cmd",
            "description": "Run shell commands",
            "icon": iconMode === "tabler" ? "terminal" : "utilities-terminal",
            "isTablerIcon": true,
            "isImage": false,
            "onActivate": function () {
              launcher.setSearchText(">cmd ");
            }
          }
        ];
  }

  function getResults(query) {
    if (!query.startsWith(">cmd"))
      return [];

    let expression = query.substring(4).trim();
    return [
          {
            "name": "Command",
            "description": "Run shell commands",
            "icon": iconMode === "tabler" ? "terminal" : "utilities-terminal",
            "isTablerIcon": true,
            "isImage": false,
            "onActivate": function () {
              launcher.closeImmediately();
              Qt.callLater(() => {
                             Logger.d("CommandProvider", "Executing shell command: " + expression);
                             Quickshell.execDetached(["sh", "-c", expression]);
                           });
            }
          }
        ];
  }
}
