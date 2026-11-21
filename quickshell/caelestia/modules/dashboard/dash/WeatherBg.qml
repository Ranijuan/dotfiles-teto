import QtQuick
import Quickshell.Widgets
import qs.services

ClippingRectangle {
    id: root

    anchors.fill: parent
    color: "transparent"
    radius: parent.radius

    Image {
        anchors.fill: parent
        source: "file:/home/ranijuan/.config/caelestia/weather-bg/" + Weather.bgFallback
        fillMode: Image.PreserveAspectCrop
    }

    Image {
        anchors.fill: parent
        source: "file:/home/ranijuan/.config/caelestia/weather-bg/" + Weather.bgImage
        fillMode: Image.PreserveAspectCrop
    }

    Rectangle {
        anchors.fill: parent
        color: "#66000000"
    }
}
