import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.System
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NToggle {
    label: "Use App2Unit to launch applications"
    description: "Uses an alternative launch method to better manage app processes and prevent issues."
    checked: Settings.data.appLauncher.useApp2Unit && ProgramCheckerService.app2unitAvailable
    enabled: ProgramCheckerService.app2unitAvailable && !Settings.data.appLauncher.customLaunchPrefixEnabled
    opacity: ProgramCheckerService.app2unitAvailable ? 1.0 : 0.6
    onToggled: checked => {
                 if (ProgramCheckerService.app2unitAvailable) {
                   Settings.data.appLauncher.useApp2Unit = checked;
                   if (checked) {
                     Settings.data.appLauncher.customLaunchPrefixEnabled = false;
                   }
                 }
               }
    defaultValue: Settings.getDefaultValue("appLauncher.useApp2Unit")
  }

  NTextInput {
    label: "Terminal command"
    description: "Command to launch a terminal. E.g. 'kitty -e' or 'gnome-terminal --'."
    Layout.fillWidth: true
    text: Settings.data.appLauncher.terminalCommand
    onEditingFinished: {
      Settings.data.appLauncher.terminalCommand = text;
    }
  }

  NToggle {
    label: "Enable custom launch prefix"
    description: "Use a custom prefix for launching applications instead of the default method."
    checked: Settings.data.appLauncher.customLaunchPrefixEnabled
    enabled: !Settings.data.appLauncher.useApp2Unit
    onToggled: checked => {
                 Settings.data.appLauncher.customLaunchPrefixEnabled = checked;
                 if (checked) {
                   Settings.data.appLauncher.useApp2Unit = false;
                 }
               }
    defaultValue: Settings.getDefaultValue("appLauncher.customLaunchPrefixEnabled")
  }

  NTextInput {
    label: "Custom launch prefix"
    description: "Prefix commands with a custom launcher (e.g. 'runapp' for systemd integration)."
    Layout.fillWidth: true
    text: Settings.data.appLauncher.customLaunchPrefix
    enabled: Settings.data.appLauncher.customLaunchPrefixEnabled
    visible: Settings.data.appLauncher.customLaunchPrefixEnabled
    onEditingFinished: Settings.data.appLauncher.customLaunchPrefix = text
  }

  NTextInput {
    label: "Annotation tool"
    description: "Command to run when clicking the annotate button in clipboard history. The image will be piped to this command."
    Layout.fillWidth: true
    text: Settings.data.appLauncher.screenshotAnnotationTool
    placeholderText: "e.g. 'gradia', 'satty -f -'"
    onEditingFinished: Settings.data.appLauncher.screenshotAnnotationTool = text
  }
}
