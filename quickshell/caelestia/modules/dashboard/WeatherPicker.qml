import QtQuick
import Quickshell

Item {
    id: root
    property bool open: false

    z: 10
    anchors.left: parent.left
    anchors.top: parent.top
    anchors.leftMargin: 100
    anchors.topMargin: 12
    width: row.width
    height: root.open ? row.height : 32

    Grid {
        id: row
        columns: 4
        spacing: 8

        Rectangle {
            width: 32
            height: 32
            radius: 16
            color: "#80000000"

            Column {
                anchors.centerIn: parent
                spacing: 3
                Repeater {
                    model: 3
                    Rectangle { width: 14; height: 2; radius: 1; color: "white" }
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.open = !root.open
            }
        }

        Repeater {
            model: ["dia-claro", "dia-lluvia", "dia-nublado", "dia-niebla", "dia-tormenta", "noche-clara", "noche-lluvia", "noche-nublado", "noche-niebla", "noche-tormenta"]

            Rectangle {
                required property string modelData
                visible: root.open
                width: lbl.implicitWidth + 20
                height: 32
                radius: 16
                color: "#80000000"

                Text {
                    id: lbl
                    anchors.centerIn: parent
                    text: parent.modelData
                    color: "white"
                    font.pixelSize: 12
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.open = false
                        Quickshell.execDetached(["sh", "-c",
                            "f=$(zenity --file-selection --title='Elegir imagen jpg') && cp \"$f\" ~/.config/caelestia/weather-bg/" + parent.modelData + ".jpg"])
                    }
                }
            }
        }
    }
}
