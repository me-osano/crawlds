import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL

  NToggle {
    label: "Lock on suspend"
    description: "Automatically lock the screen when suspending the system."
    checked: Settings.data.general.lockOnSuspend
    onToggled: checked => Settings.data.general.lockOnSuspend = checked
    defaultValue: Settings.getDefaultValue("general.lockOnSuspend")
  }

  NToggle {
    label: "Auto-start authentication"
    description: "Automatically starts fingerprint authentication without requiring a key press or button click."
    checked: Settings.data.general.autoStartAuth
    onToggled: checked => Settings.data.general.autoStartAuth = checked
    defaultValue: Settings.getDefaultValue("general.autoStartAuth")
  }

  NToggle {
    label: "Allow password login with fprintd"
    description: "When fprintd (fingerprint authentication) is active, this option lets you still login using your password instead of a fingerprint"
    checked: Settings.data.general.allowPasswordWithFprintd
    onToggled: checked => Settings.data.general.allowPasswordWithFprintd = checked
    defaultValue: Settings.getDefaultValue("general.allowPasswordWithFprintd")
  }

  NToggle {
    label: "Power controls"
    description: "Allow access to power settings from the lock screen."
    checked: Settings.data.general.showSessionButtonsOnLockScreen
    onToggled: checked => Settings.data.general.showSessionButtonsOnLockScreen = checked
    defaultValue: Settings.getDefaultValue("general.showSessionButtonsOnLockScreen")
  }

  NToggle {
    label: "Show hibernate"
    description: "Show the option 'hibernate' in the power controls."
    checked: Settings.data.general.showHibernateOnLockScreen
    onToggled: checked => Settings.data.general.showHibernateOnLockScreen = checked
    visible: Settings.data.general.showSessionButtonsOnLockScreen
    defaultValue: Settings.getDefaultValue("general.showSessionButtonsOnLockScreen")
  }

  NToggle {
    label: "Enable countdown timer"
    description: "Show a countdown timer before executing power actions."
    checked: Settings.data.general.enableLockScreenCountdown
    onToggled: checked => Settings.data.general.enableLockScreenCountdown = checked
    visible: Settings.data.general.showSessionButtonsOnLockScreen
    defaultValue: Settings.getDefaultValue("general.enableLockScreenCountdown")
  }

  NValueSlider {
    visible: Settings.data.general.showSessionButtonsOnLockScreen && Settings.data.general.enableLockScreenCountdown
    Layout.fillWidth: true
    label: "Countdown duration"
    description: "Set how long the countdown timer lasts before executing power actions."
    from: 1000
    to: 30000
    stepSize: 1000
    showReset: true
    value: Settings.data.general.lockScreenCountdownDuration
    onMoved: value => Settings.data.general.lockScreenCountdownDuration = value
    text: Math.round(Settings.data.general.lockScreenCountdownDuration / 1000) + "s"
    defaultValue: Settings.getDefaultValue("general.lockScreenCountdownDuration")
  }
}
