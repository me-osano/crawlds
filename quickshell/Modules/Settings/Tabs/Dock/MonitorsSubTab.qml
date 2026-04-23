import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.Compositor
import qs.Widgets

ColumnLayout {
  id: root
  enabled: Settings.data.dock.enabled
  spacing: Style.marginL
  Layout.fillWidth: true

  // Helper functions to update arrays immutably
  function addMonitor(list, name) {
    const arr = (list || []).slice();
    if (!arr.includes(name))
      arr.push(name);
    return arr;
  }
  function removeMonitor(list, name) {
    return (list || []).filter(function (n) {
      return n !== name;
    });
  }

  NText {
    text: "Show dock on specific monitors. Defaults to all if none are chosen."
    wrapMode: Text.WordWrap
    Layout.fillWidth: true
  }

  Repeater {
    model: Quickshell.screens || []
    delegate: NCheckbox {
      Layout.fillWidth: true
      label: modelData.name || "Unknown"
      description: {
        const compositorScale = CompositorService.getDisplayScale(modelData.name);
        "{model} ({width}x{height} @ {scale}x)";
      }
      checked: (Settings.data.dock.monitors || []).indexOf(modelData.name) !== -1
      onToggled: checked => {
                   if (checked) {
                     Settings.data.dock.monitors = root.addMonitor(Settings.data.dock.monitors, modelData.name);
                   } else {
                     Settings.data.dock.monitors = root.removeMonitor(Settings.data.dock.monitors, modelData.name);
                   }
                 }
    }
  }
}
