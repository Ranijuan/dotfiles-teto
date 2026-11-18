#!/usr/bin/env python3
import os, subprocess
import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk

HOME = os.path.expanduser("~")
SERVICE_PATH = os.path.join(HOME, ".config/systemd/user/caelestia-wallpaper.service")
TIMER_PATH = os.path.join(HOME, ".config/systemd/user/caelestia-wallpaper.timer")
WALLPAPER_DIR = os.path.join(HOME, "Pictures/Wallpapers")
CACHE_DIR = os.path.join(HOME, ".cache/caelestia/wallpapers")

SERVICE_CONTENT = """[Unit]
Description=Cambiar wallpaper de Caelestia

[Service]
Type=oneshot
ExecStart=/usr/bin/caelestia wallpaper -r -n
"""

def timer_content(minutes):
    return f"""[Unit]
Description=Cambiar wallpaper cada {minutes} minutos

[Timer]
OnBootSec=2min
OnUnitActiveSec={minutes}min
Unit=caelestia-wallpaper.service

[Install]
WantedBy=timers.target
"""


class Win(Gtk.Window):
    def __init__(self):
        super().__init__(title="Wallpapers - Caelestia")
        self.set_border_width(14)
        self.set_default_size(420, 320)
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.add(box)

        box.pack_start(self.head("Carpeta de wallpapers"), False, False, 0)
        row = Gtk.Box(spacing=6)
        row.pack_start(Gtk.Label(label=WALLPAPER_DIR, xalign=0), True, True, 0)
        open_btn = Gtk.Button(label="Abrir carpeta")
        open_btn.connect("clicked", self.open_folder)
        row.pack_start(open_btn, False, False, 0)
        box.pack_start(row, False, False, 0)

        count_row = Gtk.Box(spacing=6)
        self.count_label = Gtk.Label(xalign=0)
        count_row.pack_start(self.count_label, True, True, 0)
        refresh_btn = Gtk.Button(label="Actualizar biblioteca")
        refresh_btn.connect("clicked", self.refresh_library)
        count_row.pack_start(refresh_btn, False, False, 0)
        box.pack_start(count_row, False, False, 0)

        box.pack_start(self.head("Cambio automático"), False, False, 0)
        timer_row = Gtk.Box(spacing=6)
        timer_row.pack_start(Gtk.Label(label="Cada"), False, False, 0)
        self.minutes = Gtk.SpinButton()
        self.minutes.set_adjustment(Gtk.Adjustment(value=10, lower=1, upper=1440, step_increment=1))
        timer_row.pack_start(self.minutes, False, False, 0)
        timer_row.pack_start(Gtk.Label(label="minutos"), False, False, 0)
        box.pack_start(timer_row, False, False, 0)

        btn_row = Gtk.Box(spacing=6, homogeneous=True)
        enable_btn = Gtk.Button(label="Activar")
        enable_btn.connect("clicked", self.enable_timer)
        btn_row.pack_start(enable_btn, True, True, 0)
        disable_btn = Gtk.Button(label="Desactivar")
        disable_btn.connect("clicked", self.disable_timer)
        btn_row.pack_start(disable_btn, True, True, 0)
        box.pack_start(btn_row, False, False, 0)

        now_btn = Gtk.Button(label="Cambiar fondo ahora")
        now_btn.connect("clicked", self.change_now)
        box.pack_start(now_btn, False, False, 0)

        self.status = Gtk.Label(xalign=0)
        self.status.set_line_wrap(True)
        box.pack_start(self.status, False, False, 0)

        self.refresh_library(None)

    def head(self, text):
        l = Gtk.Label(xalign=0)
        l.set_markup(f"<b>{text}</b>")
        return l

    def open_folder(self, _w):
        subprocess.Popen(["xdg-open", WALLPAPER_DIR])

    def refresh_library(self, _w):
        if not os.path.isdir(WALLPAPER_DIR):
            self.count_label.set_text("La carpeta no existe")
            return
        exts = (".jpg", ".jpeg", ".png", ".webp", ".tif", ".tiff", ".gif")
        files = [f for f in os.listdir(WALLPAPER_DIR) if f.lower().endswith(exts)]
        self.count_label.set_text(f"{len(files)} wallpapers encontrados")
        if os.path.isdir(CACHE_DIR):
            subprocess.run(["rm", "-rf", CACHE_DIR])
        self.status.set_text("Biblioteca actualizada, caché limpiado.")

    def enable_timer(self, _w):
        minutes = int(self.minutes.get_value())
        os.makedirs(os.path.dirname(SERVICE_PATH), exist_ok=True)
        with open(SERVICE_PATH, "w") as f:
            f.write(SERVICE_CONTENT)
        with open(TIMER_PATH, "w") as f:
            f.write(timer_content(minutes))
        subprocess.run(["systemctl", "--user", "daemon-reload"])
        r = subprocess.run(["systemctl", "--user", "enable", "--now", "caelestia-wallpaper.timer"],
                            capture_output=True, text=True)
        self.status.set_text(f"Activado cada {minutes} min. " + (r.stderr or "OK").strip())

    def disable_timer(self, _w):
        r = subprocess.run(["systemctl", "--user", "disable", "--now", "caelestia-wallpaper.timer"],
                            capture_output=True, text=True)
        self.status.set_text("Desactivado. " + (r.stderr or "OK").strip())

    def change_now(self, _w):
        r = subprocess.run(["caelestia", "wallpaper", "-r", "-n"], capture_output=True, text=True)
        self.status.set_text("Fondo cambiado." if r.returncode == 0 else "Error: " + r.stderr.strip())


w = Win()
w.connect("destroy", Gtk.main_quit)
w.show_all()
Gtk.main()