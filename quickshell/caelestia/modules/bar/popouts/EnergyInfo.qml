import QtQuick
import Quickshell
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.services
import Caelestia.Services
import qs.utils

Column {
    id: root

    property real chargeFull: 0
    property real chargeDesign: 0
    property int limit: 80
    property string disk: "..."

    readonly property int health: chargeDesign > 0 ? Math.round(chargeFull / chargeDesign * 100) : 0

    function refresh(): void {
        fullFile.reload();
        designFile.reload();
        limitFile.reload();
        dfProc.running = true;
    }

    function toggleLimit(): void {
        const target = limit >= 100 ? 80 : 100;
        Quickshell.execDetached(["sh", "-c", "echo " + target + " > /sys/class/power_supply/BAT0/charge_control_end_threshold"]);
        limit = target;
    }

    spacing: Tokens.spacing.small
    width: parent.width

    StyledText {
        text: "Salud de la batería: " + (root.health > 0 ? root.health + "%" : "--")
    }

    StyledText {
        text: "Disco: " + (Storage.primaryDisk ? Units.formatKibUsage(Storage.primaryDisk.used, Storage.primaryDisk.total) : "--")
    }

    StyledRect {
        anchors.horizontalCenter: parent.horizontalCenter

        implicitWidth: limitRow.implicitWidth + Tokens.padding.large * 2
        implicitHeight: limitRow.implicitHeight + Tokens.padding.small * 2

        color: root.limit >= 100 ? Colours.palette.m3primary : Colours.tPalette.m3surfaceContainer
        radius: Tokens.rounding.full

        StateLayer {
            radius: Tokens.rounding.full
            color: root.limit >= 100 ? Colours.palette.m3onPrimary : Colours.palette.m3onSurface
            onClicked: root.toggleLimit()
        }

        Row {
            id: limitRow

            anchors.centerIn: parent
            spacing: Tokens.spacing.small

            MaterialIcon {
                anchors.verticalCenter: parent.verticalCenter
                text: "battery_charging_full"
                color: root.limit >= 100 ? Colours.palette.m3onPrimary : Colours.palette.m3onSurfaceVariant
            }

            StyledText {
                anchors.verticalCenter: parent.verticalCenter
                text: "Límite de carga: " + root.limit + "%"
                color: root.limit >= 100 ? Colours.palette.m3onPrimary : Colours.palette.m3onSurface
            }
        }
    }

    FileView {
        id: fullFile

        path: "/sys/class/power_supply/BAT0/charge_full"
        printErrors: false
        onLoaded: root.chargeFull = parseFloat(text())
    }

    FileView {
        id: designFile

        path: "/sys/class/power_supply/BAT0/charge_full_design"
        printErrors: false
        onLoaded: root.chargeDesign = parseFloat(text())
    }

    FileView {
        id: limitFile

        path: "/sys/class/power_supply/BAT0/charge_control_end_threshold"
        printErrors: false
        onLoaded: root.limit = parseInt(text())
    }

    Process {
        id: dfProc

        command: ["sh", "-c", "df -h --output=avail,size / | tail -1"]
        running: true

        stdout: StdioCollector {
            onStreamFinished: {
                const p = text.trim().split(/\s+/);
                root.disk = p.length >= 2 ? p[0] + " de " + p[1] : "--";
            }
        }
    }

    Timer {
        interval: 5000
        running: true
        repeat: true
        onTriggered: root.refresh()
    }
}