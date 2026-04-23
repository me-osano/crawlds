import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.System
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM
  width: 700

  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  property bool valueShowBackground: widgetData.showBackground !== undefined ? widgetData.showBackground : widgetMetadata.showBackground
  property bool valueRoundedCorners: widgetData.roundedCorners !== undefined ? widgetData.roundedCorners : true
  property string valueClockStyle: widgetData.clockStyle !== undefined ? widgetData.clockStyle : widgetMetadata.clockStyle
  property string valueClockColor: widgetData.clockColor !== undefined ? widgetData.clockColor : widgetMetadata.clockColor
  property bool valueUseCustomFont: widgetData.useCustomFont !== undefined ? widgetData.useCustomFont : widgetMetadata.useCustomFont
  property string valueCustomFont: widgetData.customFont !== undefined ? widgetData.customFont : ""
  property string valueFormat: widgetData.format !== undefined ? widgetData.format : widgetMetadata.format

  // Track the currently focused input field
  property var focusedInput: null

  readonly property bool isMinimalMode: valueClockStyle === "minimal"
  readonly property var now: Time.now

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.showBackground = valueShowBackground;
    settings.roundedCorners = valueRoundedCorners;
    settings.clockStyle = valueClockStyle;
    settings.clockColor = valueClockColor;
    settings.useCustomFont = valueUseCustomFont;
    settings.customFont = valueCustomFont;
    settings.format = valueFormat.trim();
    settingsChanged(settings);
  }

  // Function to insert token at cursor position in the focused input
  function insertToken(token) {
    if (!focusedInput || !focusedInput.inputItem) {
      // If no input is focused, default to format input
      if (formatInput.inputItem) {
        formatInput.inputItem.focus = true;
        focusedInput = formatInput;
      }
    }

    if (focusedInput && focusedInput.inputItem) {
      var input = focusedInput.inputItem;
      var cursorPos = input.cursorPosition;
      var currentText = input.text;

      // Insert token at cursor position
      var newText = currentText.substring(0, cursorPos) + token + currentText.substring(cursorPos);
      input.text = newText + " ";

      // Move cursor after the inserted token
      input.cursorPosition = cursorPos + token.length + 1;

      // Ensure the input keeps focus
      input.focus = true;
      saveSettings();
    }
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Clock style"
    description: "Choose the clock display style."
    currentKey: valueClockStyle
    minimumWidth: 260 * Style.uiScaleRatio
    model: [
      {
        "key": "minimal",
        "name": "Minimal"
      },
      {
        "key": "digital",
        "name": "Digital"
      },
      {
        "key": "analog",
        "name": "Analog"
      },
      {
        "key": "binary",
        "name": "Binary"
      }
    ]
    onSelected: key => {
                  valueClockStyle = key;
                  saveSettings();
                }
  }

  NComboBox {
    label: "Select color"
    description: "Apply theme colors for emphasis."
    model: [
      {
        "name": "None",
        "key": "none"
      },
      {
        "key": "primary",
        "name": "Primary"
      },
      {
        "key": "secondary",
        "name": "Secondary"
      },
      {
        "key": "tertiary",
        "name": "Tertiary"
      },
      {
        "key": "error",
        "name": "Error"
      }
    ]
    currentKey: valueClockColor
    onSelected: key => {
                  valueClockColor = key;
                  saveSettings();
                }
    minimumWidth: 200
  }

  NToggle {
    Layout.fillWidth: true
    label: "Use custom font"
    description: "Override the default font selection with a custom font for the clock."
    checked: valueUseCustomFont
    onToggled: checked => {
                 valueUseCustomFont = checked;
                 saveSettings();
               }
  }

  NSearchableComboBox {
    Layout.fillWidth: true
    visible: valueUseCustomFont
    label: "Custom font"
    description: "Select a custom font for the clock display."
    model: FontService.availableFonts
    currentKey: valueCustomFont
    placeholder: "Select custom font..."
    searchPlaceholder: "Search fonts..."
    popupHeight: 420
    minimumWidth: 300
    onSelected: function (key) {
      valueCustomFont = key;
      saveSettings();
    }
  }

  NDivider {
    Layout.fillWidth: true
    visible: isMinimalMode
  }

  NHeader {
    visible: isMinimalMode
    label: "Clock display"
    description: "Customize your clock's display by adding tokens from the list below. To use the 12-hour format, you must include the 'AP' token."
  }

  // Format editor - only visible in minimal mode
  RowLayout {
    id: main
    visible: isMinimalMode
    spacing: Style.marginL
    Layout.fillWidth: true
    Layout.alignment: Qt.AlignHCenter | Qt.AlignTop

    ColumnLayout {
      spacing: Style.marginM
      Layout.fillWidth: true
      Layout.preferredWidth: 1
      Layout.alignment: Qt.AlignHCenter | Qt.AlignTop

      NTextInput {
        id: formatInput
        Layout.fillWidth: true
        label: "Format"
        description: "Tip: Use \\n to create a line break."
        placeholderText: "HH:mm\\nd MMMM yyyy"
        text: valueFormat
        onTextChanged: valueFormat = text
        onEditingFinished: settingsChanged(saveSettings())
        Component.onCompleted: {
          if (inputItem) {
            inputItem.onActiveFocusChanged.connect(function () {
              if (inputItem.activeFocus) {
                root.focusedInput = formatInput;
              }
            });
          }
        }
      }
    }

    // Preview
    ColumnLayout {
      Layout.alignment: Qt.AlignHCenter | Qt.AlignTop
      Layout.fillWidth: false

      NLabel {
        label: "Preview"
        Layout.alignment: Qt.AlignHCenter | Qt.AlignTop
      }

      Rectangle {
        Layout.preferredWidth: 320
        Layout.preferredHeight: 160
        color: Color.mSurfaceVariant
        radius: Style.radiusM
        border.color: Color.mSecondary
        border.width: Style.borderS

        Behavior on border.color {
          ColorAnimation {
            duration: Style.animationFast
          }
        }

        ColumnLayout {
          spacing: Style.marginM
          anchors.centerIn: parent

          ColumnLayout {
            spacing: -2
            Layout.alignment: Qt.AlignHCenter

            Repeater {
              Layout.topMargin: Style.marginM
              model: Qt.locale("en").toString(now, valueFormat.trim()).split("\\n")
              delegate: NText {
                visible: text !== ""
                text: modelData
                family: valueUseCustomFont && valueCustomFont ? valueCustomFont : Settings.data.ui.fontDefault
                pointSize: Style.fontSizeM
                font.weight: Style.fontWeightBold
                color: Color.resolveColorKey(valueClockColor)
                wrapMode: Text.WordWrap
                Layout.alignment: Qt.AlignHCenter | Qt.AlignVCenter

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

  NDivider {
    Layout.topMargin: Style.marginM
    Layout.bottomMargin: Style.marginM
    visible: isMinimalMode
  }

  NDateTimeTokens {
    Layout.fillWidth: true
    height: 200
    visible: isMinimalMode
    onTokenClicked: token => root.insertToken(token)
  }

  NDivider {
    Layout.fillWidth: true
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show background"
    description: "Show the background container for the clock widget."
    checked: valueShowBackground
    onToggled: checked => {
                 valueShowBackground = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    visible: valueShowBackground
    label: "Rounded corners"
    description: "Use rounded corners for the widget background."
    checked: valueRoundedCorners
    onToggled: checked => {
                 valueRoundedCorners = checked;
                 saveSettings();
               }
  }
}
