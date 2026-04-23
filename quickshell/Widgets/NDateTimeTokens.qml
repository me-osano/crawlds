import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common

Rectangle {
  id: root

  property date sampleDate: new Date() // Dec 25, 2023, 2:30:45.123 PM

  signal tokenClicked(string token)

  color: Color.mSurface
  border.color: Color.mOutline
  border.width: Style.borderS
  radius: Style.iRadiusM

  ColumnLayout {
    id: column
    anchors.fill: parent
    anchors.margins: Style.marginS
    spacing: Style.marginS

    Flickable {
      Layout.fillWidth: true
      Layout.fillHeight: true
      contentHeight: tokensColumn.implicitHeight
      clip: true

      Column {
        id: tokensColumn
        width: parent.width

        Repeater {
          model: [
            // Common format combinations
            {
              "category": "Common",
              "token": "h:mm AP",
              "description": "12-hour time with minutes",
              "example": "2:30 PM"
            },
            {
              "category": "Common",
              "token": "HH:mm",
              "description": "24-hour time with minutes",
              "example": "14:30"
            },
            {
              "category": "Common",
              "token": "HH:mm:ss",
              "description": "24-hour time with seconds",
              "example": "14:30:45"
            },
            {
              "category": "Common",
              "token": "ddd MMM d",
              "description": "Weekday, month and day",
              "example": "Mon Dec 25"
            },
            {
              "category": "Common",
              "token": "yyyy-MM-dd",
              "description": "ISO date format",
              "example": "2023-12-25"
            },
            {
              "category": "Common",
              "token": "MM/dd/yyyy",
              "description": "US date format",
              "example": "12/25/2023"
            },
            {
              "category": "Common",
              "token": "dd.MM.yyyy",
              "description": "European date format",
              "example": "25.12.2023"
            },
            {
              "category": "Common",
              "token": "ddd, MMM dd",
              "description": "Weekday with date",
              "example": "Fri, Dec 12"
            } // Hour tokens
            ,
            {
              "category": "Hour",
              "token": "H",
              "description": "Hour without leading zero (0-23) — 24-hour format",
              "example": "14"
            },
            {
              "category": "Hour",
              "token": "HH",
              "description": "Hour with leading zero (00-23) — 24-hour format",
              "example": "14"
            } // Minute tokens
            ,
            {
              "category": "Minute",
              "token": "m",
              "description": "Minute without leading zero (0-59)",
              "example": "30"
            },
            {
              "category": "Minute",
              "token": "mm",
              "description": "Minute with leading zero (00-59)",
              "example": "30"
            } // Second tokens
            ,
            {
              "category": "Second",
              "token": "s",
              "description": "Second without leading zero (0-59)",
              "example": "45"
            },
            {
              "category": "Second",
              "token": "ss",
              "description": "Second with leading zero (00-59)",
              "example": "45"
            } // AM/PM tokens
            ,
            {
              "category": "AM/PM",
              "token": "AP",
              "description": "AM/PM in uppercase",
              "example": "PM"
            },
            {
              "category": "AM/PM",
              "token": "ap",
              "description": "am/pm in lowercase",
              "example": "pm"
            } // Timezone tokens
            ,
            {
              "category": "Timezone",
              "token": "t",
              "description": "Timezone abbreviation",
              "example": "UTC"
            } // Year tokens
            ,
            {
              "category": "Year",
              "token": "yy",
              "description": "Year as two-digit number (00-99)",
              "example": "23"
            },
            {
              "category": "Year",
              "token": "yyyy",
              "description": "Year as four-digit number",
              "example": "2023"
            } // Month tokens
            ,
            {
              "category": "Month",
              "token": "M",
              "description": "Month as number without leading zero (1-12)",
              "example": "12"
            },
            {
              "category": "Month",
              "token": "MM",
              "description": "Month as number with leading zero (01-12)",
              "example": "12"
            },
            {
              "category": "Month",
              "token": "MMM",
              "description": "Abbreviated month name",
              "example": "Dec"
            },
            {
              "category": "Month",
              "token": "MMMM",
              "description": "Full month name",
              "example": "December"
            } // Day tokens
            ,
            {
              "category": "Day",
              "token": "d",
              "description": "Day without leading zero (1-31)",
              "example": "25"
            },
            {
              "category": "Day",
              "token": "dd",
              "description": "Day with leading zero (01-31)",
              "example": "25"
            },
            {
              "category": "Day",
              "token": "ddd",
              "description": "Abbreviated day name",
              "example": "Mon"
            },
            {
              "category": "Day",
              "token": "dddd",
              "description": "Full day name",
              "example": "Monday"
            }
          ]

          delegate: Rectangle {
            id: tokenDelegate
            width: tokensColumn.width
            height: layout.implicitHeight + Style.marginS
            radius: Style.iRadiusS
            color: {
              if (tokenMouseArea.containsMouse) {
                return Qt.alpha(Color.mPrimary, 0.1);
              }
              return index % 2 === 0 ? Color.mSurfaceVariant : Qt.alpha(Color.mSurfaceVariant, 0.6);
            }

            // Mouse area for the entire delegate
            MouseArea {
              id: tokenMouseArea
              anchors.fill: parent
              hoverEnabled: true
              cursorShape: Qt.PointingHandCursor

              onClicked: {
                root.tokenClicked(modelData.token);
                clickAnimation.start();
              }
            }

            // Click animation
            SequentialAnimation {
              id: clickAnimation
              PropertyAnimation {
                target: tokenDelegate
                property: "color"
                to: Qt.alpha(Color.mPrimary, 0.3)
                duration: 100
              }
              PropertyAnimation {
                target: tokenDelegate
                property: "color"
                to: tokenMouseArea.containsMouse ? Qt.alpha(Color.mPrimary, 0.1) : (index % 2 === 0 ? Color.mSurface : Color.mSurfaceVariant)
                duration: 200
              }
            }

            RowLayout {
              id: layout
              anchors.fill: parent
              anchors.margins: Style.marginXS
              spacing: Style.marginM

              // Category badge
              Rectangle {
                Layout.alignment: Qt.AlignVCenter
                width: 70
                height: 22
                color: getCategoryColor(modelData.category)[0]
                radius: Style.iRadiusS
                opacity: tokenMouseArea.containsMouse ? 0.9 : 1.0

                Behavior on opacity {
                  NumberAnimation {
                    duration: Style.animationFast
                  }
                }

                NText {
                  anchors.centerIn: parent
                  text: modelData.category
                  color: getCategoryColor(modelData.category)[1]
                  pointSize: Style.fontSizeXS
                }
              }

              // Token - Made more prominent and clickable
              Rectangle {
                id: tokenButton
                Layout.alignment: Qt.AlignVCenter // Added this line
                width: 100
                height: 22
                color: tokenMouseArea.containsMouse ? Color.mPrimary : Color.mOnSurface
                radius: Style.iRadiusS

                Behavior on color {
                  ColorAnimation {
                    duration: Style.animationFast
                  }
                }

                NText {
                  anchors.centerIn: parent
                  text: modelData.token
                  color: tokenMouseArea.containsMouse ? Color.mOnPrimary : Color.mSurface
                  pointSize: Style.fontSizeS
                  font.weight: Style.fontWeightBold

                  Behavior on color {
                    ColorAnimation {
                      duration: Style.animationFast
                    }
                  }
                }
              }

              // Description
              NText {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter // Added this line
                text: modelData.description
                color: tokenMouseArea.containsMouse ? Color.mOnSurface : Color.mOnSurfaceVariant
                pointSize: Style.fontSizeS
                wrapMode: Text.WordWrap

                Behavior on color {
                  ColorAnimation {
                    duration: Style.animationFast
                  }
                }
              }

              // Live example
              Rectangle {
                Layout.alignment: Qt.AlignVCenter // Added this line
                width: 90
                height: 22
                color: tokenMouseArea.containsMouse ? Color.mPrimary : Color.mOnSurfaceVariant
                radius: Style.iRadiusS
                border.color: tokenMouseArea.containsMouse ? Color.mPrimary : Color.mOutline
                border.width: Style.borderS

                Behavior on color {
                  ColorAnimation {
                    duration: Style.animationFast
                  }
                }

                Behavior on border.color {
                  ColorAnimation {
                    duration: Style.animationFast
                  }
                }

                NText {
                  anchors.centerIn: parent
                  text: Qt.locale("en").toString(root.sampleDate, modelData.token)
                  color: tokenMouseArea.containsMouse ? Color.mOnPrimary : Color.mSurfaceVariant
                  pointSize: Style.fontSizeS

                  Behavior on color {
                    ColorAnimation {
                      duration: Style.animationFast
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  function getCategoryColor(category) {
    switch (category) {
    case "Year":
      return [Color.mPrimary, Color.mOnPrimary];
    case "Month":
      return [Color.mSecondary, Color.mOnSecondary];
    case "Day":
      return [Color.mTertiary, Color.mOnTertiary];
    case "Hour":
      return [Color.mPrimary, Color.mOnPrimary];
    case "Minute":
      return [Color.mSecondary, Color.mOnSecondary];
    case "Second":
      return [Color.mTertiary, Color.mOnTertiary];
    case "AM/PM":
      return [Color.mError, Color.mOnError];
    case "Timezone":
      return [Color.mOnSurface, Color.mSurface];
    case "Common":
      return [Color.mError, Color.mOnError];
    default:
      return [Color.mOnSurfaceVariant, Color.mSurfaceVariant];
    }
  }
}
