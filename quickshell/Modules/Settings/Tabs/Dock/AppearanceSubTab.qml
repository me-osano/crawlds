import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NToggle {
    Layout.fillWidth: true
    label: "Enable dock"
    description: "Show or hide the dock entirely."
    checked: Settings.data.dock.enabled
    defaultValue: Settings.getDefaultValue("dock.enabled")
    onToggled: checked => Settings.data.dock.enabled = checked
  }

  ColumnLayout {
    spacing: Style.marginL
    enabled: Settings.data.dock.enabled

    NComboBox {
      Layout.fillWidth: true
      label: "Position"
      description: "Choose where the dock appears on screen."
      model: [
        {
          "key": "top",
          "name": "Top"
        },
        {
          "key": "bottom",
          "name": "Bottom"
        },
        {
          "key": "left",
          "name": "Left"
        },
        {
          "key": "right",
          "name": "Right"
        }
      ]
      currentKey: Settings.data.dock.position
      defaultValue: Settings.getDefaultValue("dock.position")
      onSelected: key => Settings.data.dock.position = key
    }

    NComboBox {
      Layout.fillWidth: true
      label: "Dock style"
      description: "Choose between a floating pill or a static bar attached to the edge."
      model: [
        {
          "key": "floating",
          "name": "Floating"
        },
        {
          "key": "static",
          "name": "Static"
        }
      ]
      currentKey: Settings.data.dock.dockType
      defaultValue: Settings.getDefaultValue("dock.dockType")
      onSelected: key => Settings.data.dock.dockType = key
    }

    NComboBox {
      visible: Settings.data.dock.dockType === "floating"
      Layout.fillWidth: true
      label: "Display"
      description: "Choose how the dock behaves."
      model: [
        {
          "key": "always_visible",
          "name": "Always visible"
        },
        {
          "key": "auto_hide",
          "name": "Auto hide"
        },
        {
          "key": "exclusive",
          "name": "Exclusive"
        }
      ]
      currentKey: Settings.data.dock.displayMode
      defaultValue: Settings.getDefaultValue("dock.displayMode")
      onSelected: key => {
                    Settings.data.dock.displayMode = key;
                  }
    }

    NToggle {
      Layout.fillWidth: true
      visible: Settings.data.dock.dockType === "static" && Settings.data.bar.barType === "framed"
      label: "Dock sits on frame"
      description: "Align the dock inside the frame border instead of sitting on top."
      checked: Settings.data.dock.sitOnFrame
      defaultValue: Settings.getDefaultValue("dock.sitOnFrame")
      onToggled: checked => Settings.data.dock.sitOnFrame = checked
    }

    NToggle {
      Layout.fillWidth: true
      visible: Settings.data.dock.dockType === "static" && Settings.data.bar.barType === "framed"
      label: "Frame dock indicator"
      description: "Show a small indicator on the frame when the dock is hidden."
      checked: Settings.data.dock.showFrameIndicator
      defaultValue: Settings.getDefaultValue("dock.showFrameIndicator")
      onToggled: checked => Settings.data.dock.showFrameIndicator = checked
    }

    NValueSlider {
      Layout.fillWidth: true
      label: "Background opacity"
      description: "Adjust the dock's background opacity."
      from: 0
      to: 1
      stepSize: 0.01
      showReset: true
      value: Settings.data.dock.backgroundOpacity
      defaultValue: Settings.getDefaultValue("dock.backgroundOpacity")
      onMoved: value => Settings.data.dock.backgroundOpacity = value
      text: Math.floor(Settings.data.dock.backgroundOpacity * 100) + "%"
    }

    NValueSlider {
      Layout.fillWidth: true
      label: "Dead opacity"
      description: "Adjust the opacity of app icons that are not running."
      from: 0
      to: 1
      stepSize: 0.01
      showReset: true
      value: Settings.data.dock.deadOpacity
      defaultValue: Settings.getDefaultValue("dock.deadOpacity")
      onMoved: value => Settings.data.dock.deadOpacity = value
      text: Math.floor(Settings.data.dock.deadOpacity * 100) + "%"
    }

    NValueSlider {
      Layout.fillWidth: true
      visible: Settings.data.dock.dockType === "floating"
      label: "Dock floating distance"
      description: "Set the distance between the dock and the edge of the screen."
      from: 0
      to: 4
      stepSize: 0.01
      showReset: true
      value: Settings.data.dock.floatingRatio
      defaultValue: Settings.getDefaultValue("dock.floatingRatio")
      onMoved: value => Settings.data.dock.floatingRatio = value
      text: Math.floor(Settings.data.dock.floatingRatio * 100) + "%"
    }

    NValueSlider {
      Layout.fillWidth: true
      label: "Dock size"
      description: "Adjust the overall size of the dock."
      from: 0
      to: 2
      stepSize: 0.01
      showReset: true
      value: Settings.data.dock.size
      defaultValue: Settings.getDefaultValue("dock.size")
      onMoved: value => Settings.data.dock.size = value
      text: Math.floor(Settings.data.dock.size * 100) + "%"
    }

    NValueSlider {
      visible: Settings.data.dock.dockType === "floating" && Settings.data.dock.displayMode === "auto_hide"
      Layout.fillWidth: true
      label: "Hide/show speed"
      description: "Adjust the speed of the dock hide/show animation."
      from: 0.1
      to: 2.0
      stepSize: 0.01
      showReset: true
      value: Settings.data.dock.animationSpeed
      defaultValue: Settings.getDefaultValue("dock.animationSpeed")
      onMoved: value => Settings.data.dock.animationSpeed = value
      text: (Settings.data.dock.animationSpeed * 100).toFixed(0) + "%"
    }

    NToggle {
      label: "Running indicators"
      description: "Display indicator pills for all apps, not just the currently active one."
      checked: Settings.data.dock.inactiveIndicators
      defaultValue: Settings.getDefaultValue("dock.inactiveIndicators")
      onToggled: checked => Settings.data.dock.inactiveIndicators = checked
    }

    NToggle {
      label: "Static pinned apps"
      description: "Always push pinned app icons to the left in static order."
      checked: Settings.data.dock.pinnedStatic
      defaultValue: Settings.getDefaultValue("dock.pinnedStatic")
      onToggled: checked => Settings.data.dock.pinnedStatic = checked
    }

    NToggle {
      label: "Group same apps"
      description: "Group multiple windows from the same app into one dock entry."
      checked: Settings.data.dock.groupApps
      defaultValue: Settings.getDefaultValue("dock.groupApps")
      onToggled: checked => Settings.data.dock.groupApps = checked
    }

    NComboBox {
      Layout.fillWidth: true
      visible: Settings.data.dock.groupApps
      label: "Grouped app click action"
      description: "Choose what left-click does for grouped apps."
      model: [
        {
          "key": "cycle",
          "name": "Cycle windows"
        },
        {
          "key": "list",
          "name": "Open window list"
        }
      ]
      currentKey: Settings.data.dock.groupClickAction
      defaultValue: Settings.getDefaultValue("dock.groupClickAction")
      onSelected: key => Settings.data.dock.groupClickAction = key
    }

    NComboBox {
      Layout.fillWidth: true
      visible: Settings.data.dock.groupApps
      label: "Grouped app menu mode"
      description: "Choose how the context menu behaves for grouped apps."
      model: [
        {
          "key": "list",
          "name": "Window list"
        },
        {
          "key": "extended",
          "name": "Extended"
        }
      ]
      currentKey: Settings.data.dock.groupContextMenuMode
      defaultValue: Settings.getDefaultValue("dock.groupContextMenuMode")
      onSelected: key => Settings.data.dock.groupContextMenuMode = key
    }

    NComboBox {
      Layout.fillWidth: true
      visible: Settings.data.dock.groupApps
      label: "Grouped indicator style"
      description: "Choose how grouped running indicators display focused window state."
      model: [
        {
          "key": "number",
          "name": "Number"
        },
        {
          "key": "dots",
          "name": "Dots"
        }
      ]
      currentKey: Settings.data.dock.groupIndicatorStyle
      defaultValue: Settings.getDefaultValue("dock.groupIndicatorStyle")
      onSelected: key => Settings.data.dock.groupIndicatorStyle = key
    }

    NToggle {
      label: "Only apps from same monitor"
      description: "Show only apps from the monitor where the dock is located."
      checked: Settings.data.dock.onlySameOutput
      defaultValue: Settings.getDefaultValue("dock.onlySameOutput")
      onToggled: checked => Settings.data.dock.onlySameOutput = checked
    }

    NToggle {
      Layout.fillWidth: true
      label: "Colorize icons"
      description: "Apply theme colors to dock app icons (non-focused apps only)."
      checked: Settings.data.dock.colorizeIcons
      defaultValue: Settings.getDefaultValue("dock.colorizeIcons")
      onToggled: checked => Settings.data.dock.colorizeIcons = checked
    }

    NToggle {
      Layout.fillWidth: true
      label: "Show app launcher"
      description: "Show the application launcher icon in the dock."
      checked: Settings.data.dock.showLauncherIcon
      defaultValue: Settings.getDefaultValue("dock.showLauncherIcon")
      onToggled: checked => Settings.data.dock.showLauncherIcon = checked
    }

    NComboBox {
      Layout.fillWidth: true
      visible: Settings.data.dock.showLauncherIcon
      label: "Launcher position"
      description: "Choose where the launcher icon appears in the dock."
      model: [
        {
          "key": "start",
          "name": "Start"
        },
        {
          "key": "end",
          "name": "End"
        }
      ]
      currentKey: Settings.data.dock.launcherPosition
      defaultValue: Settings.getDefaultValue("dock.launcherPosition")
      onSelected: key => Settings.data.dock.launcherPosition = key
    }

    NColorChoice {
      Layout.fillWidth: true
      visible: Settings.data.dock.showLauncherIcon
      label: "Select icon color"
      currentKey: Settings.data.dock.launcherIconColor
      defaultValue: Settings.getDefaultValue("dock.launcherIconColor")
      onSelected: key => Settings.data.dock.launcherIconColor = key
    }
  }
}
