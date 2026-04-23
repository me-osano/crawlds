pragma Singleton

import QtQuick
import Quickshell

import qs.Common
import qs.Services.UI

Singleton {
    id: root

    // ── Profile State ────────────────────────────────────────────────────
    // 0 = Balanced, 1 = PowerSaver, 2 = Performance
    property int profile: 0
    readonly property bool available: true
    readonly property bool hasPerformanceProfile: false
    property bool initialized: false

       
    // Track previous profile to detect changes
    property int _previousProfile: 0

    // ── Init ─────────────────────────────────────────────────
    function init() {
        Logger.i("PowerProfileService", "Service started - using CrawlDS backend")
        _fetchProfile()
    }

    Component.onCompleted: {
        init()
    }

    function _fetchProfile() {
        CrawlDSService.jsonCmd({ cmd: "PowerProfile" }, function(data) {
            if (data && data.profile !== undefined) {
                root.profile = data.profile
                root._previousProfile = data.profile
                if (data.name === "performance") {
                    root.hasPerformanceProfile = true
                } else {
                    root.hasPerformanceProfile = false
                }
            }
            Qt.callLater(function() { root.initialized = true })
        })
    }

    // ── Helpers ─────────────────────────────────────────────────
    function getName(p) {
        if (!available)
            return "Unknown"

        const prof = (p !== undefined) ? p : profile

        switch (prof) {
        case 2: return "Performance"
        case 1: return "Power saver"
        case 0: return "Balanced"
        default: return "Balanced"
        }
    }

    function getIcon(p) {
        if (!available)
            return "balanced"

        const prof = (p !== undefined) ? p : profile

        switch (prof) {
        case 2: return "performance"
        case 1: return "powersaver"
        case 0: return "balanced"
        default: return "balanced"
        }
    }

    function isDefault() {
        return profile === 0
    }

    // ── Profile Control ─────────────────────────────────────────────
    function setProfile(p) {
        if (!available) return

        const validProfile = (p === 0 || p === 1 || p === 2)
        if (!validProfile) return

        if (p === 2 && !hasPerformanceProfile) return

        CrawlDSService.setPowerProfile(p)
        root.profile = p
    }

    function cycleProfile() {
        if (!available) return

        let next = profile
        if (hasPerformanceProfile) {
            if (profile === 2) next = 1
            else if (profile === 1) next = 0
            else next = 2
        } else {
            next = profile === 1 ? 0 : 1
        }

        setProfile(next)
    }

    function cycleProfileReverse() {
        if (!available) return

        let prev = profile
        if (hasPerformanceProfile) {
            if (profile === 2) prev = 0
            else if (profile === 1) prev = 2
            else prev = 1
        } else {
            prev = profile === 1 ? 0 : 1
        }

        setProfile(prev)
    }
    
    // ── CrawlDS Service Connections ─────────────────────────────────
    Connections {
        target: CrawlDSService
        function onBatteryStateChanged() {
            // Re-fetch profile on power state changes
            _fetchProfile()
        }
    }

    onProfileChanged: {
        if (initialized && profile !== _previousProfile) {
            const profileName = getName()
            if (profileName !== "Unknown") {
                ToastService.showNotice(profileName, "Power profile changed", profileName.toLowerCase().replace(" ", ""))
            }
            _previousProfile = profile
        }
    }
}