import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.Compositor
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NComboBox {
    Layout.fillWidth: true
    label: "Bar position"
    description: "Choose where to place the bar on the screen."
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
    currentKey: Settings.data.bar.position
    defaultValue: Settings.getDefaultValue("bar.position")
    onSelected: key => Settings.data.bar.position = key
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Bar density"
    description: "Adjust the bar's padding for a compact or spacious look."
    model: [
      {
        "key": "mini",
        "name": "Mini"
      },
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
      },
      {
        "key": "spacious",
        "name": "Spacious"
      }
    ]
    currentKey: Settings.data.bar.density
    defaultValue: Settings.getDefaultValue("bar.density")
    onSelected: key => Settings.data.bar.density = key
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Bar type"
    description: "Choose the style of the bar: Simple, Floating or Framed."
    model: [
      {
        "key": "simple",
        "name": "Simple"
      },
      {
        "key": "floating",
        "name": "Floating"
      },
      {
        "key": "framed",
        "name": "Framed"
      }
    ]
    currentKey: Settings.data.bar.barType
    defaultValue: Settings.getDefaultValue("bar.barType")
    onSelected: key => {
                  Settings.data.bar.barType = key;
                  Settings.data.bar.floating = (key === "floating");
                }
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Display mode"
    description: "Choose when the bar is visible."
    model: [
      {
        "key": "always_visible",
        "name": "Always visible"
      },
      {
        "key": "non_exclusive",
        "name": "Non-exclusive"
      },
      {
        "key": "auto_hide",
        "name": "Auto-hide"
      }
    ]
    currentKey: Settings.data.bar.displayMode
    defaultValue: Settings.getDefaultValue("bar.displayMode")
    onSelected: key => Settings.data.bar.displayMode = key
  }

  NToggle {
    label: "Use separate bar opacity"
    description: "Enable to use a separate opacity value for the bar background."
    checked: Settings.data.bar.useSeparateOpacity
    defaultValue: Settings.getDefaultValue("bar.useSeparateOpacity")
    onToggled: checked => Settings.data.bar.useSeparateOpacity = checked
  }

  NValueSlider {
    Layout.fillWidth: true
    visible: Settings.data.bar.useSeparateOpacity
    label: "Bar background opacity"
    description: "Set the background opacity specifically for the bar."
    from: 0
    to: 1
    stepSize: 0.01
    showReset: true
    value: Settings.data.bar.backgroundOpacity
    defaultValue: Settings.getDefaultValue("bar.backgroundOpacity")
    onMoved: value => Settings.data.bar.backgroundOpacity = value
    text: Math.floor(Settings.data.bar.backgroundOpacity * 100) + "%"
  }

  NToggle {
    Layout.fillWidth: true
    visible: Settings.data.bar.useSeparateOpacity
    label: "Force bar content above background"
    description: "Keeps bar capsules and widgets above the bar background layer. Disable only if your compositor has overlay-layer issues."
    checked: Settings.data.bar.forceContentOverlay !== false
    defaultValue: Settings.getDefaultValue("bar.forceContentOverlay")
    onToggled: checked => Settings.data.bar.forceContentOverlay = checked
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Font scale"
    description: "Adjust the font size scale for text displayed in the bar."
    from: 0.5
    to: 2.0
    stepSize: 0.01
    showReset: true
    value: Settings.data.bar.fontScale
    defaultValue: Settings.getDefaultValue("bar.fontScale")
    onMoved: value => Settings.data.bar.fontScale = value
    text: Math.floor(Settings.data.bar.fontScale * 100) + "%"
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Widget spacing"
    description: "Adjust the spacing between each widget in the bar."
    from: 0
    to: 30
    stepSize: 1
    showReset: true
    value: Settings.data.bar.widgetSpacing
    defaultValue: Settings.getDefaultValue("bar.widgetSpacing")
    onMoved: value => Settings.data.bar.widgetSpacing = value
    text: Settings.data.bar.widgetSpacing + "px"
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Content padding"
    description: "Adjust the padding between bar edges and widgets."
    from: 0
    to: 30
    stepSize: 1
    showReset: true
    value: Settings.data.bar.contentPadding
    defaultValue: Settings.getDefaultValue("bar.contentPadding")
    onMoved: value => Settings.data.bar.contentPadding = value
    text: Settings.data.bar.contentPadding + "px"
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show widget outlines"
    description: "Displays a visible border around every widget."
    checked: Settings.data.bar.showOutline
    defaultValue: Settings.getDefaultValue("bar.showOutline")
    onToggled: checked => Settings.data.bar.showOutline = checked
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show capsule"
    description: "Show widget backgrounds."
    checked: Settings.data.bar.showCapsule
    defaultValue: Settings.getDefaultValue("bar.showCapsule")
    onToggled: checked => Settings.data.bar.showCapsule = checked
  }

  NColorChoice {
    Layout.fillWidth: true
    visible: Settings.data.bar.showCapsule
    label: "Capsule color"
    description: "Choose a color for bar capsules, or use none for the default surface color."
    noneColor: Color.mSurfaceVariant
    noneOnColor: Color.mOnSurfaceVariant
    currentKey: Settings.data.bar.capsuleColorKey
    onSelected: key => Settings.data.bar.capsuleColorKey = key
  }

  NValueSlider {
    Layout.fillWidth: true
    visible: Settings.data.bar.showCapsule
    label: "Capsule opacity"
    description: "Set the opacity level for widget backgrounds when capsule is shown."
    from: 0
    to: 1
    stepSize: 0.01
    showReset: true
    value: Settings.data.bar.capsuleOpacity
    defaultValue: Settings.getDefaultValue("bar.capsuleOpacity")
    onMoved: value => Settings.data.bar.capsuleOpacity = value
    text: Math.floor(Settings.data.bar.capsuleOpacity * 100) + "%"
  }

  NToggle {
    Layout.fillWidth: true
    visible: CompositorService.isNiri
    label: "Hide bar on overview"
    description: "Hide the bar and close panels when the compositor overview is active."
    checked: Settings.data.bar.hideOnOverview
    defaultValue: Settings.getDefaultValue("bar.hideOnOverview")
    onToggled: checked => Settings.data.bar.hideOnOverview = checked
  }

  NToggle {
    Layout.fillWidth: true
    label: "Outer corners"
    description: "Display outwardly curved corners on the bar."
    checked: Settings.data.bar.outerCorners
    visible: Settings.data.bar.barType === "simple"
    defaultValue: Settings.getDefaultValue("bar.outerCorners")
    onToggled: checked => Settings.data.bar.outerCorners = checked
  }

  ColumnLayout {
    visible: Settings.data.bar.barType === "framed"
    spacing: Style.marginS
    Layout.fillWidth: true

    NLabel {
      label: "Frame Settings"
      description: "Adjust frame thickness and inner corner radius"
    }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginL

      NValueSlider {
        Layout.fillWidth: true
        label: "Thickness"
        from: 4
        to: 24
        stepSize: 1
        showReset: true
        value: Settings.data.bar.frameThickness
        defaultValue: Settings.getDefaultValue("bar.frameThickness")
        onMoved: value => Settings.data.bar.frameThickness = value
        text: Settings.data.bar.frameThickness + "px"
      }

      NValueSlider {
        Layout.fillWidth: true
        label: "Inner Radius"
        from: 4
        to: 24
        stepSize: 1
        showReset: true
        value: Settings.data.bar.frameRadius
        defaultValue: Settings.getDefaultValue("bar.frameRadius")
        onMoved: value => Settings.data.bar.frameRadius = value
        text: Settings.data.bar.frameRadius + "px"
      }
    }
  }

  ColumnLayout {
    visible: Settings.data.bar.barType === "floating"
    spacing: Style.marginS
    Layout.fillWidth: true

    NLabel {
      label: "Margins"
      description: "Adjust the margins around the floating bar."
    }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginL

      NValueSlider {
        Layout.fillWidth: true
        label: "Vertical"
        from: 0
        to: 18
        stepSize: 1
        showReset: true
        value: Settings.data.bar.marginVertical
        defaultValue: Settings.getDefaultValue("bar.marginVertical")
        onMoved: value => Settings.data.bar.marginVertical = value
        text: Settings.data.bar.marginVertical + "px"
      }

      NValueSlider {
        Layout.fillWidth: true
        label: "Horizontal"
        from: 0
        to: 18
        stepSize: 1
        showReset: true
        value: Settings.data.bar.marginHorizontal
        defaultValue: Settings.getDefaultValue("bar.marginHorizontal")
        onMoved: value => Settings.data.bar.marginHorizontal = value
        text: Settings.data.bar.marginHorizontal + "px"
      }
    }
  }

  NDivider {
    Layout.fillWidth: true
    Layout.topMargin: Style.marginS
    visible: Settings.data.bar.displayMode === "auto_hide"
  }

  ColumnLayout {
    visible: Settings.data.bar.displayMode === "auto_hide"
    spacing: Style.marginS
    Layout.fillWidth: true

    NValueSlider {
      Layout.fillWidth: true
      label: "Hide Delay"
      description: "Time before bar hides after mouse leaves"
      from: 100
      to: 2000
      stepSize: 100
      showReset: true
      value: Settings.data.bar.autoHideDelay
      defaultValue: Settings.getDefaultValue("bar.autoHideDelay")
      onMoved: value => Settings.data.bar.autoHideDelay = value
      text: Settings.data.bar.autoHideDelay + "ms"
    }

    NValueSlider {
      Layout.fillWidth: true
      label: "Show Delay"
      description: "Time before bar shows when mouse enters edge"
      from: 0
      to: 500
      stepSize: 50
      showReset: true
      value: Settings.data.bar.autoShowDelay
      defaultValue: Settings.getDefaultValue("bar.autoShowDelay")
      onMoved: value => Settings.data.bar.autoShowDelay = value
      text: Settings.data.bar.autoShowDelay + "ms"
    }

    NToggle {
      Layout.fillWidth: true
      label: "Show bar on workspace switch"
      description: "Automatically show the bar briefly when the workspace changes."
      checked: Settings.data.bar.showOnWorkspaceSwitch
      defaultValue: Settings.getDefaultValue("bar.showOnWorkspaceSwitch")
      onToggled: checked => Settings.data.bar.showOnWorkspaceSwitch = checked
    }
  }
}
