import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
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

  property string valueIcon: widgetData.icon !== undefined ? widgetData.icon : widgetMetadata.icon
  property bool valueTextStream: widgetData.textStream !== undefined ? widgetData.textStream : widgetMetadata.textStream
  property bool valueParseJson: widgetData.parseJson !== undefined ? widgetData.parseJson : widgetMetadata.parseJson
  property int valueMaxTextLengthHorizontal: widgetData?.maxTextLength?.horizontal ?? widgetMetadata?.maxTextLength?.horizontal
  property int valueMaxTextLengthVertical: widgetData?.maxTextLength?.vertical ?? widgetMetadata?.maxTextLength?.vertical
  property string valueHideMode: (widgetData.hideMode !== undefined) ? widgetData.hideMode : widgetMetadata.hideMode
  property bool valueShowIcon: (widgetData.showIcon !== undefined) ? widgetData.showIcon : widgetMetadata.showIcon
  property bool valueShowExecTooltip: widgetData.showExecTooltip !== undefined ? widgetData.showExecTooltip : (widgetMetadata.showExecTooltip !== undefined ? widgetMetadata.showExecTooltip : true)
  property bool valueShowTextTooltip: widgetData.showTextTooltip !== undefined ? widgetData.showTextTooltip : (widgetMetadata.showTextTooltip !== undefined ? widgetMetadata.showTextTooltip : true)
  property bool valueEnableColorization: widgetData.enableColorization || false
  property string valueColorizeSystemIcon: widgetData.colorizeSystemIcon !== undefined ? widgetData.colorizeSystemIcon : widgetMetadata.colorizeSystemIcon || "none"
  property string valueIpcIdentifier: widgetData.ipcIdentifier !== undefined ? widgetData.ipcIdentifier : widgetMetadata.ipcIdentifier || ""
  property string valueGeneralTooltipText: widgetData.generalTooltipText !== undefined ? widgetData.generalTooltipText : widgetMetadata.generalTooltipText || ""

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.icon = valueIcon;
    settings.leftClickExec = leftClickExecInput.text;
    settings.leftClickUpdateText = leftClickUpdateText.checked;
    settings.rightClickExec = rightClickExecInput.text;
    settings.rightClickUpdateText = rightClickUpdateText.checked;
    settings.middleClickExec = middleClickExecInput.text;
    settings.middleClickUpdateText = middleClickUpdateText.checked;
    settings.wheelMode = separateWheelToggle.internalChecked ? "separate" : "unified";
    settings.wheelExec = wheelExecInput.text;
    settings.wheelUpExec = wheelUpExecInput.text;
    settings.wheelDownExec = wheelDownExecInput.text;
    settings.wheelUpdateText = wheelUpdateText.checked;
    settings.wheelUpUpdateText = wheelUpUpdateText.checked;
    settings.wheelDownUpdateText = wheelDownUpdateText.checked;
    settings.textCommand = textCommandInput.text;
    settings.textCollapse = textCollapseInput.text;
    settings.textStream = valueTextStream;
    settings.parseJson = valueParseJson;
    settings.showIcon = valueShowIcon;
    settings.showExecTooltip = valueShowExecTooltip;
    settings.showTextTooltip = valueShowTextTooltip;
    settings.hideMode = valueHideMode;
    settings.maxTextLength = {
      "horizontal": valueMaxTextLengthHorizontal,
      "vertical": valueMaxTextLengthVertical
    };
    settings.textIntervalMs = parseInt(textIntervalInput.text || textIntervalInput.placeholderText, 10);
    settings.enableColorization = valueEnableColorization;
    settings.colorizeSystemIcon = valueColorizeSystemIcon;
    settings.ipcIdentifier = valueIpcIdentifier;
    settings.generalTooltipText = valueGeneralTooltipText;
    settingsChanged(settings);
  }

  RowLayout {
    spacing: Style.marginM

    NLabel {
      label: "Icon"
      description: "Select an icon from the library."
    }

    NIcon {
      Layout.alignment: Qt.AlignVCenter
      icon: valueIcon
      pointSize: Style.fontSizeXL
      visible: valueIcon !== ""
    }

    NButton {
      text: "Browse"
      onClicked: iconPicker.open()
    }
  }

  NIconPicker {
    id: iconPicker
    initialIcon: valueIcon
    onIconSelected: function (iconName) {
      valueIcon = iconName;
      saveSettings();
    }
  }

  NToggle {
    id: showIconToggle
    label: "Show icon"
    description: "Toggles the visibility of the widget's icon."
    checked: valueShowIcon
    onToggled: checked => {
                 valueShowIcon = checked;
                 saveSettings();
               }
    visible: textCommandInput.text !== ""
  }

  NToggle {
    label: "Enable colorization"
    description: "Enable colorization for the custom button icon and text, applying theme colors."
    checked: valueEnableColorization
    onToggled: checked => {
                 valueEnableColorization = checked;
                 saveSettings();
               }
  }

  NColorChoice {
    visible: valueEnableColorization
    label: "Select icon color"
    description: "Apply theme colors to icon and text."
    currentKey: valueColorizeSystemIcon
    onSelected: key => {
                  valueColorizeSystemIcon = key;
                  saveSettings();
                }
  }

  NTextInput {
    Layout.fillWidth: true
    label: "Custom tooltip text"
    description: "Custom text to display in the button's tooltip."
    placeholderText: "Enter tooltip"
    text: valueGeneralTooltipText
    onTextChanged: valueGeneralTooltipText = text
    onEditingFinished: saveSettings()
  }

  NToggle {
    id: showExecTooltipToggle
    label: "Show command tooltips"
    description: "Show tooltips with command details (left/right/middle click, wheel)."
    checked: valueShowExecTooltip
    onToggled: checked => {
                 valueShowExecTooltip = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showTextTooltipToggle
    label: "Show dynamic text tooltips"
    description: "Show tooltips with the output from the text command."
    checked: valueShowTextTooltip
    onToggled: checked => {
                 valueShowTextTooltip = checked;
                 saveSettings();
               }
  }

  NTextInput {
    Layout.fillWidth: true
    label: "IPC Identifier"
    description: "Unique identifier for IPC commands. Use this identifier with 'qs -c crawlds-shell ipc call cb [action] [identifier]' to control this button via IPC."
    placeholderText: "Enter unique identifier for IPC commands"
    text: valueIpcIdentifier
    onTextChanged: valueIpcIdentifier = text
    onEditingFinished: saveSettings()
  }

  RowLayout {
    spacing: Style.marginM

    NTextInput {
      id: leftClickExecInput
      Layout.fillWidth: true
      label: "Left click"
      description: "Command to execute when the button is left-clicked."
      placeholderText: "Enter command to execute (app or custom script)"
      text: widgetData?.leftClickExec || widgetMetadata.leftClickExec
      onEditingFinished: saveSettings()
    }

    NToggle {
      id: leftClickUpdateText
      enabled: !valueTextStream
      Layout.alignment: Qt.AlignRight | Qt.AlignBottom
      Layout.bottomMargin: Style.marginS
      onEntered: TooltipService.show(leftClickUpdateText, "Update displayed text on left-click")
      onExited: TooltipService.hide()
      checked: widgetData?.leftClickUpdateText ?? widgetMetadata.leftClickUpdateText
      onToggled: isChecked => {
                   checked = isChecked;
                   saveSettings();
                 }
    }
  }

  RowLayout {
    spacing: Style.marginM

    NTextInput {
      id: rightClickExecInput
      Layout.fillWidth: true
      label: "Right click"
      description: "Command to execute when the button is right-clicked."
      placeholderText: "Enter command to execute (app or custom script)"
      text: widgetData?.rightClickExec || widgetMetadata.rightClickExec
      onEditingFinished: saveSettings()
    }

    NToggle {
      id: rightClickUpdateText
      enabled: !valueTextStream
      Layout.alignment: Qt.AlignRight | Qt.AlignBottom
      Layout.bottomMargin: Style.marginS
      onEntered: TooltipService.show(rightClickUpdateText, "Update displayed text on right-click")
      onExited: TooltipService.hide()
      checked: widgetData?.rightClickUpdateText ?? widgetMetadata.rightClickUpdateText
      onToggled: isChecked => {
                   checked = isChecked;
                   saveSettings();
                 }
    }
  }

  RowLayout {
    spacing: Style.marginM

    NTextInput {
      id: middleClickExecInput
      Layout.fillWidth: true
      label: "Middle click"
      description: "Command to execute when the button is middle-clicked."
      placeholderText: "Enter command to execute (app or custom script)"
      text: widgetData.middleClickExec || widgetMetadata.middleClickExec
      onEditingFinished: saveSettings()
    }

    NToggle {
      id: middleClickUpdateText
      enabled: !valueTextStream
      Layout.alignment: Qt.AlignRight | Qt.AlignBottom
      Layout.bottomMargin: Style.marginS
      onEntered: TooltipService.show(middleClickUpdateText, "Update displayed text on middle-click")
      onExited: TooltipService.hide()
      checked: widgetData?.middleClickUpdateText ?? widgetMetadata.middleClickUpdateText
      onToggled: isChecked => {
                   checked = isChecked;
                   saveSettings();
                 }
    }
  }

  // Wheel command settings
  NToggle {
    id: separateWheelToggle
    Layout.fillWidth: true
    label: "Separate wheel commands"
    description: "Enable separate commands for wheel up and down."
    property bool internalChecked: (widgetData?.wheelMode || widgetMetadata?.wheelMode) === "separate"
    checked: internalChecked
    onToggled: checked => {
                 internalChecked = checked;
                 saveSettings();
               }
  }

  ColumnLayout {
    Layout.fillWidth: true

    RowLayout {
      id: unifiedWheelLayout
      visible: !separateWheelToggle.checked
      spacing: Style.marginM

      NTextInput {
        id: wheelExecInput
        Layout.fillWidth: true
        label: "Scroll wheel"
        description: "Command to execute when the scroll wheel is used.<br>Use $delta for the scroll wheel delta in the command."
        placeholderText: "Enter command to execute (app or custom script)"
        text: widgetData?.wheelExec || widgetMetadata?.wheelExec
        onEditingFinished: saveSettings()
      }

      NToggle {
        id: wheelUpdateText
        enabled: !valueTextStream
        Layout.alignment: Qt.AlignRight | Qt.AlignBottom
        Layout.bottomMargin: Style.marginS
        onEntered: TooltipService.show(wheelUpdateText, "Update displayed text on scroll")
        onExited: TooltipService.hide()
        checked: widgetData?.wheelUpdateText ?? widgetMetadata?.wheelUpdateText
        onToggled: isChecked => {
                     checked = isChecked;
                     saveSettings();
                   }
      }
    }

    ColumnLayout {
      id: separatedWheelLayout
      Layout.fillWidth: true
      visible: separateWheelToggle.checked

      RowLayout {
        spacing: Style.marginM

        NTextInput {
          id: wheelUpExecInput
          Layout.fillWidth: true
          label: "Wheel up command"
          description: "Command to execute when the scroll wheel is scrolled up."
          placeholderText: "Enter command to execute (app or custom script)"
          text: widgetData?.wheelUpExec || widgetMetadata?.wheelUpExec
          onEditingFinished: saveSettings()
        }

        NToggle {
          id: wheelUpUpdateText
          enabled: !valueTextStream
          Layout.alignment: Qt.AlignRight | Qt.AlignBottom
          Layout.bottomMargin: Style.marginS
          onEntered: TooltipService.show(wheelUpUpdateText, "Update displayed text on scroll")
          onExited: TooltipService.hide()
          checked: (widgetData?.wheelUpUpdateText !== undefined) ? widgetData.wheelUpUpdateText : widgetMetadata?.wheelUpUpdateText
          onToggled: isChecked => {
                       checked = isChecked;
                       saveSettings();
                     }
        }
      }

      RowLayout {
        spacing: Style.marginM

        NTextInput {
          id: wheelDownExecInput
          Layout.fillWidth: true
          label: "Wheel down command"
          description: "Command to execute when the scroll wheel is scrolled down."
          placeholderText: "Enter command to execute (app or custom script)"
          text: widgetData?.wheelDownExec || widgetMetadata?.wheelDownExec
          onEditingFinished: saveSettings()
        }

        NToggle {
          id: wheelDownUpdateText
          enabled: !valueTextStream
          Layout.alignment: Qt.AlignRight | Qt.AlignBottom
          Layout.bottomMargin: Style.marginS
          onEntered: TooltipService.show(wheelDownUpdateText, "Update displayed text on scroll")
          onExited: TooltipService.hide()
          checked: (widgetData?.wheelDownUpdateText !== undefined) ? widgetData.wheelDownUpdateText : widgetMetadata?.wheelDownUpdateText
          onToggled: isChecked => {
                       checked = isChecked;
                       saveSettings();
                     }
        }
      }
    }
  }

  NDivider {
    Layout.fillWidth: true
  }

  NHeader {
    label: "Dynamic text"
  }

  NSpinBox {
    label: "Max text length (horizontal)"
    description: "Maximum number of characters to show in horizontal bar (0 to hide text)."
    from: 0
    to: 100
    value: valueMaxTextLengthHorizontal
    onValueChanged: {
      valueMaxTextLengthHorizontal = value;
      saveSettings();
    }
  }

  NSpinBox {
    label: "Max text length (vertical)"
    description: "Maximum number of characters to show in vertical bar (0 to hide text)."
    from: 0
    to: 100
    value: valueMaxTextLengthVertical
    onValueChanged: {
      valueMaxTextLengthVertical = value;
      saveSettings();
    }
  }

  NToggle {
    id: textStreamInput
    label: "Stream"
    description: "Streamed lines from the command will be displayed as text on the button."
    checked: valueTextStream
    onToggled: checked => {
                 valueTextStream = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: parseJsonInput
    label: "Parse output as JSON"
    description: "Parse the command output as a JSON object to dynamically set text and icon."
    checked: valueParseJson
    onToggled: checked => {
                 valueParseJson = checked;
                 saveSettings();
               }
  }

  NTextInput {
    id: textCommandInput
    Layout.fillWidth: true
    label: "Display command output"
    description: valueTextStream ? "Enter a command to run continuously." : "Enter a command to run at a regular interval. The first line of its output will be displayed as text."
    placeholderText: "echo \"Hello World\""
    text: widgetData?.textCommand || widgetMetadata.textCommand
    onEditingFinished: saveSettings()
  }

  NTextInput {
    id: textCollapseInput
    Layout.fillWidth: true
    visible: valueTextStream
    label: "Collapse condition"
    description: "If the output text matches this value, the button will collapse."
    placeholderText: "e.g. 'nothing is playing'. Use /regex/ for patterns."
    text: widgetData?.textCollapse || widgetMetadata.textCollapse
    onEditingFinished: saveSettings()
  }

  NTextInput {
    id: textIntervalInput
    Layout.fillWidth: true
    visible: !valueTextStream
    label: "Refresh interval"
    description: "Interval in milliseconds."
    placeholderText: String(widgetMetadata.textIntervalMs)
    text: widgetData && widgetData.textIntervalMs !== undefined ? String(widgetData.textIntervalMs) : ""
    onEditingFinished: saveSettings()
  }

  NComboBox {
    id: hideModeComboBox
    label: "Hide mode"
    description: "Controls widget visibility when the command has no output."
    model: [
      {
        name: "Always expanded",
        key: "alwaysExpanded"
      },
      {
        name: "Expand when has output",
        key: "expandWithOutput"
      },
      {
        name: "Max expanded but transparent",
        key: "maxTransparent"
      }
    ]
    currentKey: valueHideMode
    onSelected: key => {
                  valueHideMode = key;
                  saveSettings();
                }
    visible: textCommandInput.text !== "" && valueTextStream == true
  }
}
