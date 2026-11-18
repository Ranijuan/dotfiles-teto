import re

path = "/home/ranijuan/.config/quickshell/caelestia/modules/dashboard/WeatherTab.qml"
with open(path, encoding="utf-8") as f:
    content = f.read()

props_block = '''
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
'''

anchor1 = 'readonly property var today: Weather.forecast && Weather.forecast.length > 0 ? Weather.forecast[0] : null'
content = content.replace(anchor1, anchor1 + "\n" + props_block, 1)

image_block = '''
    Image {
        anchors.fill: parent
        source: "file:/home/ranijuan/.config/caelestia/weather-bg/" + root.bgImage
        fillMode: Image.PreserveAspectCrop
        opacity: 0.35
        z: -1
    }
'''

anchor2 = 'Component.onCompleted: Weather.reload()'
content = content.replace(anchor2, anchor2 + "\n" + image_block, 1)

with open(path, "w", encoding="utf-8") as f:
    f.write(content)

print("listo")