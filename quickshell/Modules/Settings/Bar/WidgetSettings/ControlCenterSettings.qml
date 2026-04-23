import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.UI
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM

  // Properties to receive data from parent
  property var screen: null
  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  // Local state - Display mode
  property string valueDisplayMode: (widgetData && widgetData.displayMode !== undefined) ? widgetData.displayMode : ((widgetMetadata && widgetMetadata.displayMode) || "icon")
  property bool valueShowNetworkIcon: (widgetData && widgetData.showNetworkIcon !== undefined) ? widgetData.showNetworkIcon : ((widgetMetadata && widgetMetadata.showNetworkIcon !== undefined) ? widgetMetadata.showNetworkIcon : true)
  property bool valueShowBluetoothIcon: (widgetData && widgetData.showBluetoothIcon !== undefined) ? widgetData.showBluetoothIcon : ((widgetMetadata && widgetMetadata.showBluetoothIcon !== undefined) ? widgetMetadata.showBluetoothIcon : true)
  property bool valueShowNotificationIcon: (widgetData && widgetData.showNotificationIcon !== undefined) ? widgetData.showNotificationIcon : ((widgetMetadata && widgetMetadata.showNotificationIcon !== undefined) ? widgetMetadata.showNotificationIcon : true)

  // Local state - Icon mode
  property string valueIcon: (widgetData && widgetData.icon !== undefined) ? widgetData.icon : ((widgetMetadata && widgetMetadata.icon) || "crawlds")
  property bool valueUseDistroLogo: (widgetData && widgetData.useDistroLogo !== undefined) ? widgetData.useDistroLogo : ((widgetMetadata && widgetMetadata.useDistroLogo) || false)
  property string valueCustomIconPath: (widgetData && widgetData.customIconPath !== undefined) ? widgetData.customIconPath : ((widgetMetadata && widgetMetadata.customIconPath) || "")
  property bool valueEnableColorization: (widgetData && widgetData.enableColorization !== undefined) ? widgetData.enableColorization : ((widgetMetadata && widgetMetadata.enableColorization) || false)
  property string valueColorizeSystemIcon: (widgetData && widgetData.colorizeSystemIcon !== undefined) ? widgetData.colorizeSystemIcon : ((widgetMetadata && widgetMetadata.colorizeSystemIcon) || "none")

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.displayMode = valueDisplayMode;
    settings.showNetworkIcon = valueShowNetworkIcon;
    settings.showBluetoothIcon = valueShowBluetoothIcon;
    settings.showNotificationIcon = valueShowNotificationIcon;
    settings.icon = valueIcon;
    settings.useDistroLogo = valueUseDistroLogo;
    settings.customIconPath = valueCustomIconPath;
    settings.enableColorization = valueEnableColorization;
    settings.colorizeSystemIcon = valueColorizeSystemIcon;
    settingsChanged(settings);
  }

  // Display Mode Selection
  NHeader {
    label : "Display Mode"
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Widget style"
    description: "Choose how the control center button appears in the bar."
    model: [
      { "key": "icon", "name": "Single Icon" },
      { "key": "capsule", "name": "Capsule (Network, Bluetooth, Notifications)" }
    ]
    currentKey: valueDisplayMode
    onSelected: key => {
      valueDisplayMode = key;
      saveSettings();
    }
  }

  // Capsule mode settings
  NHeader {
    visible: valueDisplayMode === "capsule"
    label: "Capsule Icons"
  }

  NToggle {
    visible: valueDisplayMode === "capsule"
    label: "Show network icon"
    description: "Display network/Wi-Fi status icon in the capsule."
    checked: valueShowNetworkIcon
    onToggled: checked => {
      valueShowNetworkIcon = checked;
      saveSettings();
    }
  }

  NToggle {
    visible: valueDisplayMode === "capsule"
    label: "Show Bluetooth icon"
    description: "Display Bluetooth status icon in the capsule."
    checked: valueShowBluetoothIcon
    onToggled: checked => {
      valueShowBluetoothIcon = checked;
      saveSettings();
    }
  }

  NToggle {
    visible: valueDisplayMode === "capsule"
    label: "Show notification icon"
    description: "Display notification bell icon in the capsule."
    checked: valueShowNotificationIcon
    onToggled: checked => {
      valueShowNotificationIcon = checked;
      saveSettings();
    }
  }

  // Icon mode settings
  NHeader {
    visible: valueDisplayMode === "icon"
    label: "Icon Settings"
  }

  NToggle {
    visible: valueDisplayMode === "icon"
    label: "Use distro logo instead of icon"
    description: "Use your distribution's logo instead of a custom icon."
    checked: valueUseDistroLogo
    onToggled: checked => {
                 valueUseDistroLogo = checked;
                 saveSettings();
               }
  }

  NToggle {
    visible: valueDisplayMode === "icon"
    label: "Enable colorization"
    description: "Enable colorization for icon, applying theme colors."
    checked: valueEnableColorization
    onToggled: checked => {
                 valueEnableColorization = checked;
                 saveSettings();
               }
  }

  NColorChoice {
    visible: valueDisplayMode === "icon" && valueEnableColorization
    label: "Select icon color"
    description: "Apply theme colors to icons."
    currentKey: valueColorizeSystemIcon
    onSelected: function (key) {
      valueColorizeSystemIcon = key;
      saveSettings();
    }
  }

  RowLayout {
    visible: valueDisplayMode === "icon"
    spacing: Style.marginM

    NLabel {
      label: "Icon"
      description: "Select an icon from the library or a custom file."
    }

    NImageRounded {
      Layout.preferredWidth: Style.fontSizeXL * 2
      Layout.preferredHeight: Style.fontSizeXL * 2
      Layout.alignment: Qt.AlignVCenter
      radius: Math.min(Style.radiusL, Layout.preferredWidth / 2)
      imagePath: valueCustomIconPath
      visible: valueCustomIconPath !== "" && !valueUseDistroLogo
    }

    NIcon {
      Layout.alignment: Qt.AlignVCenter
      icon: valueIcon
      pointSize: Style.fontSizeXXL * 1.5
      visible: valueIcon !== "" && valueCustomIconPath === "" && !valueUseDistroLogo
    }
  }

  RowLayout {
    visible: valueDisplayMode === "icon"
    spacing: Style.marginM
    NButton {
      enabled: !valueUseDistroLogo
      text: "Browse Library"
      onClicked: iconPicker.open()
    }

    NButton {
      enabled: !valueUseDistroLogo
      text: "Browse File"
      onClicked: imagePicker.openFilePicker()
    }
  }

  NIconPicker {
    id: iconPicker
    initialIcon: valueIcon
    onIconSelected: iconName => {
                      valueIcon = iconName;
                      valueCustomIconPath = "";
                      saveSettings();
                    }
  }

  NFilePicker {
    id: imagePicker
    title: "Select a custom icon"
    selectionMode: "files"
    nameFilters: ImageCacheService.basicImageFilters.concat(["*.svg"])
    initialPath: Quickshell.env("HOME")
    onAccepted: paths => {
                  if (paths.length > 0) {
                    valueCustomIconPath = paths[0]; // Use first selected file
                    saveSettings();
                  }
                }
  }
}
