import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NComboBox {
    label: "Position"
    description: "Choose where the launcher panel appears."
    Layout.fillWidth: true
    model: [
      {
        "key": "follow_bar",
        "name": "Follow bar"
      },
      {
        "key": "center",
        "name": "Center"
      },
      {
        "key": "top_center",
        "name": "Top center"
      },
      {
        "key": "top_left",
        "name": "Top left"
      },
      {
        "key": "top_right",
        "name": "Top right"
      },
      {
        "key": "bottom_left",
        "name": "Bottom left"
      },
      {
        "key": "bottom_right",
        "name": "Bottom right"
      },
      {
        "key": "bottom_center",
        "name": "Bottom center"
      }
    ]
    currentKey: Settings.data.appLauncher.position
    onSelected: function (key) {
      Settings.data.appLauncher.position = key;
    }

    defaultValue: Settings.getDefaultValue("appLauncher.position")
  }

  NToggle {
    label: "Show above fullscreen"
    description: "Display the launcher on the overlay layer, above fullscreen windows. When enabled, the launcher will not attach to the bar."
    checked: Settings.data.appLauncher.overviewLayer
    onToggled: checked => Settings.data.appLauncher.overviewLayer = checked
    defaultValue: Settings.getDefaultValue("appLauncher.overviewLayer")
  }

  NComboBox {
    label: "View mode"
    description: "Choose the layout for the launcher entries."
    Layout.fillWidth: true
    model: [
      {
        "key": "list",
        "name": "List"
      },
      {
        "key": "grid",
        "name": "Grid"
      }
    ]
    currentKey: Settings.data.appLauncher.viewMode
    onSelected: function (key) {
      Settings.data.appLauncher.viewMode = key;
    }
    defaultValue: Settings.getDefaultValue("appLauncher.viewMode")
  }

  NComboBox {
    label: "Density"
    description: "Adjust the size of application icons and the density of the launcher."
    Layout.fillWidth: true
    model: [
      {
        "key": "compact",
        "name": "Compact"
      },
      {
        "key": "default",
        "name": "Default"
      },
      {
        "key": "comfortable",
        "name": "Comfortable"
      }
    ]
    currentKey: Settings.data.appLauncher.density || "compact"
    onSelected: function (key) {
      Settings.data.appLauncher.density = key;
    }
    defaultValue: Settings.getDefaultValue("appLauncher.density")
  }

  NToggle {
    label: "Show categories"
    description: "Show category tabs for filtering applications."
    checked: Settings.data.appLauncher.showCategories
    onToggled: checked => Settings.data.appLauncher.showCategories = checked
    defaultValue: Settings.getDefaultValue("appLauncher.showCategories")
  }

  NToggle {
    label: "Use native icons"
    description: "Use native system icons instead of Tabler icons."
    checked: Settings.data.appLauncher.iconMode === "native"
    onToggled: checked => Settings.data.appLauncher.iconMode = checked ? "native" : "tabler"
    defaultValue: Settings.getDefaultValue("appLauncher.iconMode")
  }

  NToggle {
    label: "Show icon background"
    description: "Show a rounded rectangle background behind icons."
    checked: Settings.data.appLauncher.showIconBackground
    onToggled: checked => Settings.data.appLauncher.showIconBackground = checked
    defaultValue: Settings.getDefaultValue("appLauncher.showIconBackground")
  }

  NDivider {
    Layout.fillWidth: true
  }

  NToggle {
    label: "Sort by most used"
    description: "When enabled, frequently launched apps appear first in the list."
    checked: Settings.data.appLauncher.sortByMostUsed
    onToggled: checked => Settings.data.appLauncher.sortByMostUsed = checked
    defaultValue: Settings.getDefaultValue("appLauncher.sortByMostUsed")
  }

  NToggle {
    label: "Enable settings search"
    description: "Show settings results when searching in the launcher."
    checked: Settings.data.appLauncher.enableSettingsSearch
    onToggled: checked => Settings.data.appLauncher.enableSettingsSearch = checked
    defaultValue: Settings.getDefaultValue("appLauncher.enableSettingsSearch")
  }

  NToggle {
    label: "Enable windows search"
    description: "Search and focus active windows."
    checked: Settings.data.appLauncher.enableWindowsSearch
    onToggled: checked => Settings.data.appLauncher.enableWindowsSearch = checked
    defaultValue: Settings.getDefaultValue("appLauncher.enableWindowsSearch")
  }

  NToggle {
    label: "Enable session search"
    description: "Show session actions (lock, shutdown, reboot, etc.) when searching in the launcher."
    checked: Settings.data.appLauncher.enableSessionSearch
    onToggled: checked => Settings.data.appLauncher.enableSessionSearch = checked
    defaultValue: Settings.getDefaultValue("appLauncher.enableSessionSearch")
  }

  NToggle {
    label: "Ignore mouse input"
    description: "Disable mouse interaction and scrollwheel in the launcher."
    checked: Settings.data.appLauncher.ignoreMouseInput
    onToggled: checked => Settings.data.appLauncher.ignoreMouseInput = checked
    defaultValue: Settings.getDefaultValue("appLauncher.ignoreMouseInput")
  }
}
