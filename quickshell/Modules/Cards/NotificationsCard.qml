import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.System
import qs.Services.UI
import qs.Widgets

NBox {
  id: root

  property var screen
  property bool expanded: true
  property int compactItems: 5
  property bool showAll: false
  readonly property real compactViewportHeight: Math.round(180 * Style.uiScaleRatio)
  readonly property real expandedViewportHeight: Math.round(280 * Style.uiScaleRatio)
  readonly property real listViewportHeight: showAll ? expandedViewportHeight : compactViewportHeight
  // 0 = All, 1 = Today, 2 = Yesterday, 3 = Earlier
  property int currentRange: 1
  
  // Expose the target height (non-animated) for parent layout calculations
  readonly property real targetHeight: expanded ? (headerRow.implicitHeight + Style.margin2M + contentArea.implicitHeight + Style.margin2M) : (headerRow.implicitHeight + Style.margin2M)

  clip: true

  Behavior on implicitHeight {
    NumberAnimation {
      duration: Style.animationNormal
      easing.type: Easing.InOutQuad
    }
  }

  implicitHeight: expanded ? (headerRow.implicitHeight + Style.margin2M + contentArea.implicitHeight + Style.margin2M) : (headerRow.implicitHeight + Style.margin2M)

  readonly property int totalCount: NotificationService.historyList.count
  readonly property int unreadCount: {
    var since = NotificationService.lastSeenTs;
    var count = 0;
    var model = NotificationService.historyList;
    for (var i = 0; i < model.count; i++) {
      var item = model.get(i);
      var ts = item.timestamp instanceof Date ? item.timestamp.getTime() : item.timestamp;
      if (ts > since)
        count++;
    }
    return count;
  }

  readonly property var rangeCounts: {
    var counts = [0, 0, 0, 0];
    var model = NotificationService.historyList;
    counts[0] = model.count;
    for (var i = 0; i < model.count; i++) {
      var item = model.get(i);
      if (!item || item.timestamp === undefined)
        continue;
      var ts = item.timestamp instanceof Date ? item.timestamp.getTime() : Number(item.timestamp);
      if (!isFinite(ts))
        continue;
      var range = rangeForTimestamp(ts);
      counts[range + 1] = counts[range + 1] + 1;
    }
    return counts;
  }

  readonly property var filteredIndices: {
    var list = [];
    var model = NotificationService.historyList;
    for (var i = 0; i < model.count; i++) {
      var item = model.get(i);
      if (!item || item.timestamp === undefined)
        continue;
      var ts = item.timestamp instanceof Date ? item.timestamp.getTime() : Number(item.timestamp);
      if (!isFinite(ts))
        continue;
      if (currentRange === 0 || rangeForTimestamp(ts) === (currentRange - 1)) {
        list.push(i);
      }
    }
    return list;
  }

  readonly property bool canViewAll: filteredIndices.length > compactItems

  function dateOnly(dateObj) {
    return new Date(dateObj.getFullYear(), dateObj.getMonth(), dateObj.getDate());
  }

  function rangeForTimestamp(ts) {
    var dt = new Date(ts);
    var today = dateOnly(new Date());
    var thatDay = dateOnly(dt);
    var diffMs = today - thatDay;
    var diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0)
      return 0;
    if (diffDays === 1)
      return 1;
    return 2;
  }

  function formatTime(ts) {
    var date = new Date(ts);
    return Qt.locale("en").toString(date, "HH:mm");
  }

  function isUnread(ts) {
    return ts > NotificationService.lastSeenTs;
  }

  onUnreadCountChanged: {
    if (unreadCount > 0)
      expanded = true;
  }

  ColumnLayout {
    id: content
    anchors.fill: parent
    anchors.margins: Style.marginM
    spacing: Style.marginS

    RowLayout {
      id: headerRow
      Layout.fillWidth: true
      spacing: Style.marginS

      NIcon {
        icon: NotificationService.doNotDisturb ? "bell-off" : "bell"
        pointSize: Style.fontSizeXL
        color: Color.mPrimary
      }

      NText {
        text: "Notifications"
        pointSize: Style.fontSizeL
        font.weight: Style.fontWeightBold
      }

      Rectangle {
        visible: root.unreadCount > 0
        color: Color.mPrimary
        radius: Style.radiusL
        implicitWidth: unreadText.implicitWidth + Style.marginM
        implicitHeight: unreadText.implicitHeight + Style.marginXS

        NText {
          id: unreadText
          anchors.centerIn: parent
          text: root.unreadCount
          color: Color.mOnPrimary
          pointSize: Style.fontSizeXS
          font.weight: Style.fontWeightBold
        }
      }

      Item {
        Layout.fillWidth: true
      }

      NIconButton {
        icon: "check"
        tooltipText: "Mark all as read"
        baseSize: Style.baseWidgetSize * 0.8
        onClicked: NotificationService.updateLastSeenTs()
      }

      NIconButton {
        icon: NotificationService.doNotDisturb ? "bell-off" : "bell"
        tooltipText: NotificationService.doNotDisturb ? "Disable Do Not Disturb" : "Enable Do Not Disturb"
        baseSize: Style.baseWidgetSize * 0.8
        onClicked: NotificationService.doNotDisturb = !NotificationService.doNotDisturb
      }

      NIconButton {
        icon: "trash"
        tooltipText: "Clear notification history"
        baseSize: Style.baseWidgetSize * 0.8
        onClicked: NotificationService.clearHistory()
      }
    }

    // Collapsible content
    Item {
      id: contentArea
      visible: root.expanded
      Layout.fillWidth: true
      implicitHeight: contentColumn.implicitHeight

      ColumnLayout {
        id: contentColumn
        width: parent.width
        spacing: Style.marginS

        NDivider {
          Layout.fillWidth: true
        }

        NTabBar {
          id: rangeTabs
          visible: root.totalCount > 0
          Layout.fillWidth: true
          currentIndex: root.currentRange
          distributeEvenly: true
          tabHeight: Style.toOdd(Style.baseWidgetSize * 0.7)

          NTabButton {
            tabIndex: 0
            text: "All (" + root.rangeCounts[0] + ")"
            checked: rangeTabs.currentIndex === 0
            onClicked: root.currentRange = 0
            pointSize: Style.fontSizeXS
          }

          NTabButton {
            tabIndex: 1
            text: "Today (" + root.rangeCounts[1] + ")"
            checked: rangeTabs.currentIndex === 1
            onClicked: root.currentRange = 1
            pointSize: Style.fontSizeXS
          }

          NTabButton {
            tabIndex: 2
            text: "Yesterday (" + root.rangeCounts[2] + ")"
            checked: rangeTabs.currentIndex === 2
            onClicked: root.currentRange = 2
            pointSize: Style.fontSizeXS
          }

          NTabButton {
            tabIndex: 3
            text: "Earlier (" + root.rangeCounts[3] + ")"
            checked: rangeTabs.currentIndex === 3
            onClicked: root.currentRange = 3
            pointSize: Style.fontSizeXS
          }
        }

        // Empty state
        ColumnLayout {
          visible: root.totalCount === 0 || root.filteredIndices.length === 0
          Layout.fillWidth: true
          Layout.alignment: Qt.AlignHCenter
          spacing: Style.marginS

          Item { Layout.preferredHeight: Style.marginM }

          NIcon {
            icon: "bell-off"
            pointSize: Style.fontSizeXXL
            color: Color.mOnSurfaceVariant
            Layout.alignment: Qt.AlignHCenter
          }

          NText {
            text: root.totalCount === 0 ? "No notifications" : "No notifications in this range"
            pointSize: Style.fontSizeS
            color: Color.mOnSurfaceVariant
            horizontalAlignment: Text.AlignHCenter
            Layout.fillWidth: true
          }

          NText {
            visible: root.totalCount === 0
            text: "Notifications will appear here"
            pointSize: Style.fontSizeXS
            color: Color.mOnSurfaceVariant
            horizontalAlignment: Text.AlignHCenter
            Layout.fillWidth: true
          }

          Item { Layout.preferredHeight: Style.marginM }
        }

        // Notification list
        NScrollView {
          visible: root.filteredIndices.length > 0
          Layout.fillWidth: true
          Layout.preferredHeight: Math.min(notificationsList.implicitHeight, root.listViewportHeight)
          horizontalPolicy: ScrollBar.AlwaysOff
          verticalPolicy: ScrollBar.AsNeeded
          reserveScrollbarSpace: false

          ColumnLayout {
            id: notificationsList
            width: parent.width
            spacing: Style.marginXS

            Repeater {
              model: root.showAll ? root.filteredIndices : root.filteredIndices.slice(0, root.compactItems)

              delegate: NBox {
                id: notifItem
                required property var modelData

                readonly property int notifIndex: Number(modelData)
                readonly property var notif: NotificationService.historyList.get(notifIndex)
                readonly property real ts: notif && notif.timestamp ? (notif.timestamp instanceof Date ? notif.timestamp.getTime() : notif.timestamp) : 0
                readonly property bool isUnread: root.isUnread(ts)

                Layout.fillWidth: true
                implicitHeight: itemContent.implicitHeight + Style.marginM
                color: isUnread ? Qt.alpha(Color.mPrimary, 0.08) : Color.mSurface

                Behavior on color {
                  enabled: !Color.isTransitioning
                  ColorAnimation {
                    duration: Style.animationFast
                    easing.type: Easing.InOutQuad
                  }
                }

                RowLayout {
                  id: itemContent
                  anchors.fill: parent
                  anchors.margins: Style.marginS
                  spacing: Style.marginS

                  // Unread indicator
                  Rectangle {
                    visible: notifItem.isUnread
                    width: 6
                    height: 6
                    radius: width / 2
                    color: Color.mPrimary
                    Layout.alignment: Qt.AlignTop
                    Layout.topMargin: Style.marginXS
                  }

                  // App icon placeholder
                  Rectangle {
                    visible: !notifItem.isUnread
                    width: 6
                    height: 6
                    radius: width / 2
                    color: "transparent"
                    Layout.alignment: Qt.AlignTop
                    Layout.topMargin: Style.marginXS
                  }

                  ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2

                    // Summary/title
                    NText {
                      text: (notifItem.notif && notifItem.notif.summary) ? notifItem.notif.summary : "Notification"
                      font.weight: Style.fontWeightSemiBold
                      pointSize: Style.fontSizeS
                      color: Color.mOnSurface
                      elide: Text.ElideRight
                      maximumLineCount: 1
                      Layout.fillWidth: true
                    }

                    // Body preview
                    NText {
                      visible: notifItem.notif && notifItem.notif.body && notifItem.notif.body.length > 0
                      text: (notifItem.notif && notifItem.notif.body) ? notifItem.notif.body.replace(/\n/g, " ") : ""
                      pointSize: Style.fontSizeXS
                      color: Color.mOnSurfaceVariant
                      elide: Text.ElideRight
                      maximumLineCount: 1
                      Layout.fillWidth: true
                    }

                    // App name and time
                    NText {
                      text: (notifItem.notif && notifItem.notif.appName ? notifItem.notif.appName + " • " : "") + root.formatTime(notifItem.ts)
                      pointSize: Style.fontSizeXXS
                      color: Color.mOnSurfaceVariant
                      elide: Text.ElideRight
                      maximumLineCount: 1
                      Layout.fillWidth: true
                    }
                  }

                  // Dismiss button
                  NIconButton {
                    icon: "close"
                    baseSize: Style.baseWidgetSize * 0.6
                    tooltipText: "Dismiss"
                    visible: itemMouseArea.containsMouse || dismissMouseArea.containsMouse
                    onClicked: {
                      if (notifItem.notif) {
                        NotificationService.removeFromHistory(notifItem.notif.id);
                      }
                    }

                    MouseArea {
                      id: dismissMouseArea
                      anchors.fill: parent
                      hoverEnabled: true
                      onClicked: parent.clicked()
                    }
                  }
                }

                MouseArea {
                  id: itemMouseArea
                  anchors.fill: parent
                  hoverEnabled: true
                  cursorShape: Qt.PointingHandCursor
                  onClicked: {
                    // Open notification history panel for full view
                    PanelService.getPanel("notificationHistoryPanel", root.screen)?.toggle();
                  }
                }
              }
            }
          }
        }

        // View all / Show less button
        RowLayout {
          visible: root.filteredIndices.length > 0 && root.canViewAll
          Layout.fillWidth: true
          spacing: Style.marginS

          Item {
            Layout.fillWidth: true
          }

          NButton {
            icon: root.showAll ? "chevron-up" : "chevron-down"
            text: root.showAll ? "Show less" : "View all (" + root.filteredIndices.length + ")"
            outlined: true
            onClicked: root.showAll = !root.showAll
          }

          Item {
            Layout.fillWidth: true
          }
        }
      }
    }
  }
}