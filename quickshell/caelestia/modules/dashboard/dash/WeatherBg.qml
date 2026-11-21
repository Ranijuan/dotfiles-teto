import QtQuick
import Quickshell.Widgets
import qs.services

ClippingRectangle {
    id: root

    anchors.fill: parent
    color: "transparent"
    radius: parent.radius

    readonly property bool isRaining: {
        if (!Weather.cc) return false;
        const code = String(Weather.cc.weatherCode);
        return ["51","53","55","56","57","61","63","65","66","67","80","81","82","95","96","99"].includes(code);
    }
    readonly property bool isDay: Weather.cc ? Weather.cc.isDay === 1 : true
    readonly property string bgImage: {
        if (isDay) return isRaining ? "dia-lluvia.jpg" : "dia-claro.jpg";
        return isRaining ? "noche-lluvia.jpg" : "noche-clara.jpg";
    }

    Image {
        anchors.fill: parent
        source: "file:/home/ranijuan/.config/caelestia/weather-bg/" + root.bgImage
        fillMode: Image.PreserveAspectCrop
    }

    Rectangle {
        anchors.fill: parent
        color: "#66000000"
    }
}
