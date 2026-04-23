import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Modules.Cards
import qs.Modules.Screen
import qs.Services.Media
import qs.Services.UI
import qs.Widgets

SmartPanel {
  id: root

  // Positioning
  readonly property string controlCenterPosition: Settings.data.controlCenter.position

  // Check if there's a bar on this screen
  readonly property bool hasBarOnScreen: {
    var monitors = Settings.data.bar.monitors || [];
    return monitors.length === 0 || monitors.includes(screen?.name);
  }

  // When position is "close_to_bar_button" but there's no bar, fall back to center
  readonly property bool shouldCenter: controlCenterPosition === "close_to_bar_button" && !hasBarOnScreen

  panelAnchorHorizontalCenter: shouldCenter || (controlCenterPosition !== "close_to_bar_button" && (controlCenterPosition.endsWith("_center") || controlCenterPosition === "center"))
  panelAnchorVerticalCenter: shouldCenter || controlCenterPosition === "center"
  panelAnchorLeft: !shouldCenter && controlCenterPosition !== "close_to_bar_button" && controlCenterPosition.endsWith("_left")
  panelAnchorRight: !shouldCenter && controlCenterPosition !== "close_to_bar_button" && controlCenterPosition.endsWith("_right")
  panelAnchorBottom: !shouldCenter && controlCenterPosition !== "close_to_bar_button" && controlCenterPosition.startsWith("bottom_")
  panelAnchorTop: !shouldCenter && controlCenterPosition !== "close_to_bar_button" && controlCenterPosition.startsWith("top_")

  preferredWidth: Math.round(440 * Style.uiScaleRatio)

  // Overlay card states - which floating card is currently shown
  property string activeOverlay: "" // "", "network", "bluetooth"
  
  // Convenience properties for widgets to check
  property bool networkCardExpanded: activeOverlay === "network"
  property bool bluetoothCardExpanded: activeOverlay === "bluetooth"
  
  // Toggle overlay - ensures only one is open at a time
  function toggleOverlay(name) {
    if (activeOverlay === name) {
      activeOverlay = "";
    } else {
      activeOverlay = name;
    }
  }

  // Notifications card expanded state
  property bool notificationsCardExpanded: true

  readonly property var cardsForRender: {
    const sourceCards = Settings.data.controlCenter.cards || [];
    let cards = [];
    for (var i = 0; i < sourceCards.length; i++) {
      cards.push(sourceCards[i]);
    }

    var notificationsIndex = -1;
    for (var j = 0; j < cards.length; j++) {
      if (cards[j].id === "notifications-card")
        notificationsIndex = j;
    }

    if (notificationsIndex === -1) {
      const notificationCard = {
        "enabled": true,
        "id": "notifications-card"
      };
      cards.push(notificationCard);
    }

    return cards;
  }

  preferredHeight: {
    var height = 0;
    var count = 0;
    for (var i = 0; i < cardsForRender.length; i++) {
      const card = cardsForRender[i];
      if (!card.enabled)
        continue;
      const contributes = true;
      if (!contributes)
        continue;
      count++;
      switch (card.id) {
      case "profile-card":
        height += profileHeight;
        break;
      case "shortcuts-card":
        height += shortcutsHeight;
        break;
      case "audio-card":
        height += audioHeight;
        break;
      case "brightness-card":
        height += brightnessHeight;
        break;
      case "notifications-card":
        height += notificationsHeight;
        break;
      case "network-card":
        // Network card is now inline in shortcuts card, skip it
        count--; // Don't count it as a separate card
        break;
      case "media-sysmon-card":
        height += mediaSysMonHeight;
        break;
      default:
        break;
      }
    }
    return height + (count + 1) * Style.marginL;
  }

  readonly property int profileHeight: Math.round(64 * Style.uiScaleRatio)
  readonly property int shortcutsHeight: Math.round(52 * Style.uiScaleRatio)
  readonly property int audioHeight: Math.round(60 * Style.uiScaleRatio)
  readonly property int brightnessHeight: Math.round(60 * Style.uiScaleRatio)
  property int notificationsHeight: Math.round(110 * Style.uiScaleRatio)
  readonly property int mediaSysMonHeight: Math.round(260 * Style.uiScaleRatio)
  readonly property int networkHeight: Math.round(100 * Style.uiScaleRatio)

  // When preferredHeight changes, recalculate the panel position/size
  onPreferredHeightChanged: {
    if (isPanelOpen && isPanelVisible) {
      setPosition();
    }
  }

  onOpened: {
    MediaService.autoSwitchingPaused = true;
  }

  onClosed: {
    MediaService.autoSwitchingPaused = false;
    // Reset overlay state when panel closes
    activeOverlay = "";
  }

  panelContent: Item {
    id: panelContent

    ColumnLayout {
      id: layout
      x: Style.marginL
      y: Style.marginL
      width: parent.width - Style.margin2L
      spacing: Style.marginL

      Repeater {
        model: root.cardsForRender
        Loader {
          // Skip network-card as it's now inline in shortcuts card
          active: modelData.enabled && modelData.id !== "network-card"
          visible: active
          Layout.fillWidth: true
          Layout.preferredHeight: {
            switch (modelData.id) {
            case "profile-card":
              return profileHeight;
            case "shortcuts-card":
              return shortcutsHeight;
            case "audio-card":
              return audioHeight;
            case "brightness-card":
              return brightnessHeight;
            case "notifications-card":
              // Dynamic height based on expanded state
              return notificationsHeight;
            case "media-sysmon-card":
              return mediaSysMonHeight;
            default:
              return 0;
            }
          }
          sourceComponent: {
            switch (modelData.id) {
            case "profile-card":
              return profileCard;
            case "shortcuts-card":
              return shortcutsCard;
            case "audio-card":
              return audioCard;
            case "brightness-card":
              return brightnessCard;
            case "notifications-card":
              return notificationsCard;
            case "media-sysmon-card":
              return mediaSysMonCard;
            default:
              return null;
            }
          }
        }
      }
    }

    Component {
      id: profileCard
      ProfileCard {}
    }

    Component {
      id: shortcutsCard
      ShortcutsCard {}
    }

    Component {
      id: audioCard
      AudioCard {}
    }

    Component {
      id: brightnessCard
      BrightnessCard {}
    }

    Component {
      id: notificationsCard
      NotificationsCard {
        screen: root.screen
        expanded: root.notificationsCardExpanded
        onTargetHeightChanged: root.notificationsHeight = targetHeight
        Component.onCompleted: root.notificationsHeight = targetHeight
      }
    }

    Component {
      id: mediaSysMonCard
      RowLayout {
        spacing: Style.marginL

        // Media card
        MediaCard {
          Layout.fillWidth: true
          Layout.fillHeight: true
        }

        // System monitors combined in one card
        SystemMonitorCard {
          Layout.preferredWidth: Math.round(Style.baseWidgetSize * 2.625)
          Layout.fillHeight: true
        }
      }
    }

    // Floating overlay cards - appear below shortcuts card
    // Calculate the Y offset dynamically based on card order
    readonly property real overlayTopOffset: {
      var offset = Style.marginL; // Initial top margin
      for (var i = 0; i < root.cardsForRender.length; i++) {
        const card = root.cardsForRender[i];
        if (!card.enabled || card.id === "network-card")
          continue;

        // Add the card's height
        switch (card.id) {
        case "profile-card":
          offset += root.profileHeight + Style.marginL;
          break;
        case "shortcuts-card":
          // Stop after shortcuts - we want to position just below shortcuts
          return offset;
        case "audio-card":
          offset += root.audioHeight + Style.marginL;
          break;
        case "brightness-card":
          offset += root.brightnessHeight + Style.marginL;
          break;
        case "notifications-card":
          offset += root.notificationsHeight + Style.marginL;
          break;
        case "media-sysmon-card":
          offset += root.mediaSysMonHeight + Style.marginL;
          break;
        }
      }
      // Fallback if shortcuts not found
      return offset;
    }

    // Transparent backdrop when overlay is open (for click-to-dismiss)
    Item {
      id: overlayBackdrop
      anchors.fill: parent
      visible: root.activeOverlay !== ""
      z: 99
      
      MouseArea {
        anchors.fill: parent
        onClicked: root.activeOverlay = ""
      }
    }

    // Network card overlay
    NetworkCard {
      id: networkOverlay
      screen: root.screen
      expanded: root.networkCardExpanded
      
      x: Style.marginL
      y: panelContent.overlayTopOffset
      width: parent.width - Style.margin2L
      
      z: 100
      visible: root.activeOverlay === "network"
      
      // Override the visibility binding from expanded
      Binding {
        target: networkOverlay
        property: "visible"
        value: root.activeOverlay === "network"
      }
    }

    // Bluetooth card overlay
    BluetoothCard {
      id: bluetoothOverlay
      screen: root.screen
      expanded: root.bluetoothCardExpanded
      
      x: Style.marginL
      y: panelContent.overlayTopOffset
      width: parent.width - Style.margin2L
      
      z: 100
      visible: root.activeOverlay === "bluetooth"
      
      // Override the visibility binding from expanded
      Binding {
        target: bluetoothOverlay
        property: "visible"
        value: root.activeOverlay === "bluetooth"
      }
    }
  }
}
