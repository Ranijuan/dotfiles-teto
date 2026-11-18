#!/usr/bin/env python3
import subprocess
import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk


def get_apps():
    out = subprocess.run(["pacman", "-Qe"], capture_output=True, text=True).stdout
    apps = []
    for line in out.splitlines():
        parts = line.split()
        if len(parts) >= 2:
            apps.append((parts[0], parts[1]))
    apps.sort()
    return apps


class Win(Gtk.Window):
    def __init__(self):
        super().__init__(title="Desinstalador de apps")
        self.set_default_size(420, 560)
        self.set_border_width(10)

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.add(box)

        search = Gtk.SearchEntry()
        search.connect("search-changed", self.filter_rows)
        box.pack_start(search, False, False, 0)
        self.search = search

        scroller = Gtk.ScrolledWindow()
        scroller.set_vexpand(True)
        box.pack_start(scroller, True, True, 0)
        self.list_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        scroller.add(self.list_box)

        self.rows = []
        for name, ver in get_apps():
            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            chk = Gtk.CheckButton()
            lbl = Gtk.Label(label="%s (%s)" % (name, ver), xalign=0)
            lbl.set_hexpand(True)
            row.pack_start(chk, False, False, 0)
            row.pack_start(lbl, True, True, 0)
            self.list_box.pack_start(row, False, False, 0)
            self.rows.append((name.lower(), row, chk))

        btn = Gtk.Button(label="Borrar seleccionadas")
        btn.connect("clicked", self.confirm_delete)
        box.pack_start(btn, False, False, 0)

        self.status = Gtk.Label(xalign=0)
        self.status.set_line_wrap(True)
        box.pack_start(self.status, False, False, 0)

    def filter_rows(self, _w):
        text = self.search.get_text().lower()
        for name, row, _chk in self.rows:
            row.set_visible(text in name)

    def confirm_delete(self, _w):
        selected = [name for name, _row, chk in self.rows if chk.get_active()]
        if not selected:
            self.status.set_text("No marcaste ninguna.")
            return
        d = Gtk.MessageDialog(
            transient_for=self, flags=0, message_type=Gtk.MessageType.WARNING,
            buttons=Gtk.ButtonsType.YES_NO,
            text="¿Borrar %d app(s)?\n%s" % (len(selected), ", ".join(selected)),
        )
        resp = d.run()
        d.destroy()
        if resp != Gtk.ResponseType.YES:
            return
        term = "foot" if subprocess.run(["which", "foot"], capture_output=True).returncode == 0 else "xterm"
        cmd = "sudo pacman -Rns %s; echo; read -p 'Enter para cerrar...'" % " ".join(selected)
        subprocess.Popen([term, "-e", "bash", "-c", cmd])
        self.status.set_text("Abrí la terminal, poné tu clave sudo ahí.")


w = Win()
w.connect("destroy", Gtk.main_quit)
w.show_all()
Gtk.main()