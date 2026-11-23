import QtQuick
import Quickshell.Services.UPower
import Caelestia.Config
import qs.components
import qs.services

Item {
    id: root

    required property color colour

    readonly property real pct: UPower.displayDevice.percentage
    readonly property bool charging: [UPowerDeviceState.Charging, UPowerDeviceState.FullyCharged, UPowerDeviceState.PendingCharge].includes(UPower.displayDevice.state)
    readonly property color level: UPower.onBattery && pct <= 0.2 ? Colours.palette.m3error : charging ? Colours.palette.m3primary : colour

    visible: UPower.displayDevice.isLaptopBattery
    implicitWidth: 36
    implicitHeight: 20

    Rectangle {
        id: body

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width - 3
        height: parent.height
        radius: 5
        color: "transparent"
        border.width: 1.5
        border.color: root.level

        Rectangle {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 2.5
            width: Math.max(0, (parent.width - 5) * root.pct)
            radius: 3
            color: root.level
            opacity: 0.35
        }

        StyledText {
            anchors.centerIn: parent
            text: Math.round(root.pct * 100) + "%"
            color: root.colour
            font: Tokens.font.body.builders.small.size(8).weight(Font.DemiBold).build()
        }
    }

    Rectangle {
        anchors.left: body.right
        anchors.verticalCenter: body.verticalCenter
        width: 3
        height: 8
        radius: 1.5
        color: root.level
    }
}