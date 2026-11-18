#!/usr/bin/env python3
import os, re, shutil, subprocess
import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk

CONF = os.path.expanduser("~/.config/caelestia/hypr-vars.lua")
APPS = [
    ("terminal", "Terminal", "foot"),
    ("browser", "Navegador", "librewolf"),
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
    ("kbMonitor", "Cambiar pantalla", "SUPER + P"),
]
MODS = ["SUPER", "SHIFT", "ALT", "CTRL"]
ALL = APPS + KEYS
MAX_CUSTOM = 20


def split_combo(s):
    parts = [p.strip() for p in s.split("+") if p.strip()]
    mods = [p.upper() for p in parts if p.upper() in MODS]
    keys = [p for p in parts if p.upper() not in MODS]
    return mods, (keys[0] if keys else "")


class Win(Gtk.Window):
    def __init__(self):
        super().__init__(title="Ajustes de Hyprland (Caelestia)")
        self.set_border_width(14)
        self.loaded, self.other, self.custom = {}, [], []
        self.read_conf()
        scroller = Gtk.ScrolledWindow()
        scroller.set_min_content_height(560)
        scroller.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.add(scroller)
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        scroller.add(box)

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

        box.pack_start(self.head("Atajos personalizados"), False, False, 0)
        self.custom_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.pack_start(self.custom_box, False, False, 0)
        self.custom_rows = []
        for label, mods, key, cmd in self.custom:
            self.add_custom_row(label, mods, key, cmd)
        addbtn = Gtk.Button(label="+ Agregar atajo")
        addbtn.connect("clicked", lambda _w: self.add_custom_row())
        box.pack_start(addbtn, False, False, 0)

        hint = Gtk.Label(xalign=0)
        hint.set_markup("<small>Teclas: letras (T), Return, Space, Print, Tab, Escape, F1...</small>")
        box.pack_start(hint, False, False, 0)
        btn = Gtk.Button(label="Guardar y aplicar")
        btn.connect("clicked", self.save)
        box.pack_start(btn, False, False, 0)
        self.status = Gtk.Label(xalign=0)
        self.status.set_line_wrap(True)
        box.pack_start(self.status, False, False, 0)

    def head(self, text):
        l = Gtk.Label(xalign=0)
        l.set_markup("<b>%s</b>" % text)
        return l

    def add_custom_row(self, label="", mods=None, key="", cmd=""):
        mods = mods or []
        row = Gtk.Grid(column_spacing=6, row_spacing=0)
        name_e = Gtk.Entry(); name_e.set_placeholder_text("Nombre (ej: Spotify)")
        name_e.set_text(label); name_e.set_width_chars(14)
        row.attach(name_e, 0, 0, 1, 1)
        checks = []
        for j, m in enumerate(MODS):
            c = Gtk.CheckButton(label=m)
            c.set_active(m in mods)
            row.attach(c, 1 + j, 0, 1, 1)
            checks.append(c)
        key_e = Gtk.Entry(); key_e.set_width_chars(8); key_e.set_text(key)
        row.attach(key_e, 5, 0, 1, 1)
        cmd_e = Gtk.Entry(); cmd_e.set_placeholder_text("comando (ej: spotify)")
        cmd_e.set_text(cmd); cmd_e.set_hexpand(True)
        row.attach(cmd_e, 6, 0, 1, 1)
        delbtn = Gtk.Button(label="x")
        row.attach(delbtn, 7, 0, 1, 1)
        self.custom_box.pack_start(row, False, False, 0)
        entry = {"row": row, "name": name_e, "checks": checks, "key": key_e, "cmd": cmd_e}
        self.custom_rows.append(entry)
        delbtn.connect("clicked", lambda _w: self.remove_custom_row(entry))
        row.show_all()

    def remove_custom_row(self, entry):
        entry["row"].destroy()
        self.custom_rows.remove(entry)

    def read_conf(self):
        known = {k for k, _, _ in ALL}
        try:
            lines = open(CONF, encoding="utf-8").read().splitlines()
        except OSError:
            return
        raw = {}
        for line in lines:
            m = re.match(r'^\s*([A-Za-z_]\w*)\s*=\s*(.*?)\s*,?\s*$', line)
            if not m:
                continue
            k, v = m.group(1), m.group(2)
            q = re.match(r'^"([^"]*)"', v)
            val = q.group(1) if q else None
            if k in known:
                if val is not None:
                    self.loaded[k] = val
            elif k.startswith("kbCustom") or k.startswith("cmdCustom"):
                raw[k] = val
            else:
                self.other.append(k + " = " + v + ",")
        for i in range(1, MAX_CUSTOM + 1):
            k, c = raw.get("kbCustom%d" % i), raw.get("cmdCustom%d" % i)
            if k and c:
                mods, key = split_combo(k)
                self.custom.append(("atajo%d" % i, mods, key, c))

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
        n = 0
        for entry in self.custom_rows:
            key = entry["key"].get_text().strip()
            cmd = entry["cmd"].get_text().strip()
            if not key or not cmd:
                continue
            n += 1
            mods = [m for m, c in zip(MODS, entry["checks"]) if c.get_active()]
            combo = " + ".join(mods + [key])
            out.append('  kbCustom%d = "%s",' % (n, combo.replace('"', "")))
            out.append('  cmdCustom%d = "%s",' % (n, cmd.replace('"', "")))
        out.append("}")
        os.makedirs(os.path.dirname(CONF), exist_ok=True)
        if os.path.exists(CONF):
            shutil.copy(CONF, CONF + ".bak")
        with open(CONF, "w", encoding="utf-8") as f:
            f.write("\n".join(out) + "\n")
        r = subprocess.run(["hyprctl", "reload"], capture_output=True, text=True)
        msg = (r.stdout + r.stderr).strip()
        self.status.set_text("Guardado y aplicado. " + msg)


w = Win()
w.connect("destroy", Gtk.main_quit)
w.show_all()
Gtk.main()