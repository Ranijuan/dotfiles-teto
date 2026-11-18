import QtQuick
import QtQuick.Layouts
import Caelestia.Config
import qs.components
import qs.services

ColumnLayout {
    id: root

    readonly property string title: Players.active?.trackTitle ?? ""
    readonly property string artist: Players.active?.trackArtist ?? ""
    property var lines: []
    property string plain: ""

    readonly property string currentLine: {
        if (lines.length === 0) return "";
        const p = Players.active?.position ?? 0;
        let cur = "";
        for (const l of lines) {
            if (l.t <= p) cur = l.text; else break;
        }
        return cur;
    }

    function parse(lrc) {
        const out = [];
        lrc.split("\n").forEach(l => {
            const m = l.match(/^\[(\d+):(\d+(?:\.\d+)?)\](.*)$/);
            if (m) out.push({ t: parseInt(m[1]) * 60 + parseFloat(m[2]), text: m[3].trim() });
        });
        return out;
    }

    function load() {
        lines = [];
        plain = "";
        if (!title) return;
        const xhr = new XMLHttpRequest();
        xhr.onreadystatechange = function () {
            if (xhr.readyState !== XMLHttpRequest.DONE || xhr.status !== 200) return;
            try {
                const arr = JSON.parse(xhr.responseText);
                const d = arr.length > 0 ? arr[0] : null;
                if (!d) return;
                if (d.syncedLyrics) root.lines = root.parse(d.syncedLyrics);
                else root.plain = d.plainLyrics ?? "";
            } catch (e) {}
        };
        xhr.open("GET", "https://lrclib.net/api/search?track_name=" + encodeURIComponent(title)
                 + "&artist_name=" + encodeURIComponent(artist));
        xhr.send();
    }

    onTitleChanged: load()
    Component.onCompleted: load()

    StyledText {
        Layout.fillWidth: true
        Layout.fillHeight: true
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        wrapMode: Text.WordWrap
        elide: Text.ElideRight
        color: Colours.palette.m3onSurface
        font: Tokens.font.title.small
        text: root.lines.length > 0 ? root.currentLine : root.plain
    }
}
