   wl-paste > ~/ajustes-hypr.py


#!/usr/bin/env python3
import os, re, shutil, subprocess
import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk

CONF = os.path.expanduser("~/.config/caelestia/hypr-vars.lua")
APPS = [
    ("terminal", "Terminal", "foot"),
    ("browser", "Navegador", "firefox"),
    ("editor", "Editor", "codium"),
    ("fileExplorer", "Archivos", "thunar"),
]
KEYS = [
    ("kbTerminal", "Abrir terminal", "SUPER + T"),
    ("kbBrowser", "Abrir navegador", "SUPER + W"),
    ("kbEditor", "Abrir editor", "SUPER + C"),
    ("kbFileExplorer", "Abrir archivos", "SUPER + E"),
    ("kbScreenshot", "Captura de pantalla", "Print"),
    ("kbLock", "Bloquear pantalla", "SUPER + L"),
]
MODS = ["SUPER", "SHIFT", "ALT", "CTRL"]
ALL = APPS + KEYS


def split_combo(s):
    parts = [p.strip() for p in s.split("+") if p.strip()]
    mods = [p.upper() for p in parts if p.upper() in MODS]
    keys = [p for p in parts if p.upper() not in MODS]
    return mods, (keys[0] if keys else "")


class Win(Gtk.Window):
    def __init__(self):
        super().__init__(title="Ajustes de Hyprland (Caelestia)")
        self.set_border_width(14)
        self.loaded, self.other = {}, []
        self.read_conf()
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.add(box)

        box.pack_start(self.head("Aplicaciones predeterminadas"), False, False, 0)
        grid = Gtk.Grid(column_spacing=10, row_spacing=6)
        self.app_entries = {}
        for i, (k, label, d) in enumerate(APPS):
            grid.attach(Gtk.Label(label=label, xalign=0), 0, i, 1, 1)
            e = Gtk.Entry()
            e.set_text(self.loaded.get(k, d))
            e.set_hexpand(True)
            grid.attach(e, 1, i, 1, 1)
            self.app_entries[k] = e
        box.pack_start(grid, False, False, 0)

        box.pack_start(self.head("Atajos de teclado"), False, False, 0)
        grid2 = Gtk.Grid(column_spacing=10, row_spacing=6)
        for j, m in enumerate(MODS):
            grid2.attach(Gtk.Label(label=m), 1 + j, 0, 1, 1)
        grid2.attach(Gtk.Label(label="Tecla"), 5, 0, 1, 1)
        self.key_checks, self.key_entries = {}, {}
        for i, (k, label, d) in enumerate(KEYS, start=1):
            mods, key = split_combo(self.loaded.get(k, d))
            grid2.attach(Gtk.Label(label=label, xalign=0), 0, i, 1, 1)
            checks = []
            for j, m in enumerate(MODS):
                c = Gtk.CheckButton()
                c.set_active(m in mods)
                grid2.attach(c, 1 + j, i, 1, 1)
                checks.append(c)
            e = Gtk.Entry()
            e.set_text(key)
            e.set_width_chars(10)
            grid2.attach(e, 5, i, 1, 1)
            self.key_checks[k], self.key_entries[k] = checks, e
        box.pack_start(grid2, False, False, 0)

        hint = Gtk.Label(xalign=0)
        hint.set_markup("<small>Teclas: letras (T), Return, Space, Print, Tab, Escape, F1...</small>")
        box.pack_start(hint, False, False, 0)
        btn = Gtk.Button(label="Guardar y aplicar")
        btn.connect("clicked", self.save)
        box.pack_start(btn, False, False, 0)
        self.status = Gtk.Label(xalign=0)
        box.pack_start(self.status, False, False, 0)

    def head(self, text):
        l = Gtk.Label(xalign=0)
        l.set_markup("<b>%s</b>" % text)
        return l

    def read_conf(self):
        known = {k for k, _, _ in ALL}
        try:
            lines = open(CONF, encoding="utf-8").read().splitlines()
        except OSError:
            return
        for line in lines:
            m = re.match(r'^\s*([A-Za-z_]\w*)\s*=\s*(.*?)\s*,?\s*$', line)
            if not m:
                continue
            k, v = m.group(1), m.group(2)
            if k in known:
                q = re.match(r'^"([^"]*)"', v)
                if q:
                    self.loaded[k] = q.group(1)
            else:
                self.other.append(k + " = " + v + ",")

    def save(self, _w):
        vals = {}
        for k, _, _ in APPS:
            vals[k] = self.app_entries[k].get_text().strip()
        for k, _, _ in KEYS:
            key = self.key_entries[k].get_text().strip()
            if key:
                mods = [m for m, c in zip(MODS, self.key_checks[k]) if c.get_active()]
                vals[k] = " + ".join(mods + [key])
        defaults = {k: d for k, _, d in ALL}
        out = ["return {"] + ["  " + l for l in self.other]
        for k, _, _ in ALL:
            v = vals.get(k)
            if v and (v != defaults[k] or k in self.loaded):
                out.append('  %s = "%s",' % (k, v.replace('"', "")))
        out.append("}")
        os.makedirs(os.path.dirname(CONF), exist_ok=True)
        if os.path.exists(CONF):
            shutil.copy(CONF, CONF + ".bak")
        with open(CONF, "w", encoding="utf-8") as f:
            f.write("\n".join(out) + "\n")
        b = vals.get("browser", "")
        if b and os.path.exists("/usr/share/applications/%s.desktop" % b):
            subprocess.run(["xdg-settings", "set", "default-web-browser", b + ".desktop"])
        r = subprocess.run(["hyprctl", "reload"], capture_output=True, text=True)
        msg = (r.stdout + r.stderr).strip()
        self.status.set_text("Guardado y aplicado. " + msg)


w = Win()
w.connect("destroy", Gtk.main_quit)
w.show_all()
Gtk.main()