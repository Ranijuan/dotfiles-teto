use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use libadwaita as adw;
use adw::prelude::*;
use std::process::Command;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            while let Some(&c2) = chars.peek() {
                chars.next();
                if c2.is_ascii_alphabetic() { break; }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn run_cmd(args: &[&str]) -> String {
    match Command::new(args[0]).args(&args[1..]).output() {
        Ok(out) => {
            let mut s = String::from_utf8_lossy(&out.stdout).to_string();
            s.push_str(&String::from_utf8_lossy(&out.stderr));
            strip_ansi(&s)
        }
        Err(e) => format!("Error: {e}"),
    }
}

fn build_respaldos_page(output: &gtk::TextView) -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    page.set_margin_top(20);
    page.set_margin_bottom(20);
    page.set_margin_start(20);
    page.set_margin_end(20);

    let btn_crear = gtk::Button::with_label("Crear snapshot");
    let btn_listar = gtk::Button::with_label("Listar snapshots");
    let btn_volver = gtk::Button::with_label("Volver a uno (editar numero abajo)");
    let entry_num = gtk::Entry::new();
    entry_num.set_placeholder_text(Some("numero del snapshot"));

    let buf = output.buffer();

    {
        let buf = buf.clone();
        btn_crear.connect_clicked(move |_| {
            let res = run_cmd(&["sudo", "snapper", "-c", "root", "create", "-d", "manual teto-tools"]);
            buf.set_text(&res);
        });
    }
    {
        let buf = buf.clone();
        btn_listar.connect_clicked(move |_| {
            let res = run_cmd(&["snapper", "-c", "root", "list"]);
            buf.set_text(&res);
        });
    }
    {
        let buf = buf.clone();
        let entry_num = entry_num.clone();
        btn_volver.connect_clicked(move |_| {
            let num = entry_num.text().to_string();
            if num.trim().is_empty() {
                buf.set_text("Escribe el numero del snapshot primero.");
                return;
            }
            let res = run_cmd(&["sudo", "snapper", "-c", "root", "undochange", &format!("{num}..0")]);
            buf.set_text(&res);
        });
    }

    page.append(&btn_crear);
    page.append(&btn_listar);
    page.append(&entry_num);
    page.append(&btn_volver);
    page
}

enum Msg {
    Append(String),
    Done,
    Warn(String),
}

fn start_actualizacion(buf: gtk::TextBuffer, warning: gtk::Label, btn: gtk::Button) {
    btn.set_sensitive(false);
    btn.set_label("Trabajando...");
    warning.set_visible(false);
    buf.set_text("");

    let (tx, rx) = mpsc::channel::<Msg>();

    thread::spawn(move || {
        let send = |s: String| { let _ = tx.send(Msg::Append(s)); };

        send("== 1) Revisando que todo este bien ANTES ==\n".into());
        let check_antes = run_cmd(&["teto-check-shell"]);
        send(check_antes.clone());

        if check_antes.contains("AVISO") {
            send("\nSe detuvo: algo ya estaba roto antes de empezar. Arreglalo primero.\n".into());
            let _ = tx.send(Msg::Warn("Algo ya estaba roto ANTES de actualizar. No se hizo nada mas.".into()));
            let _ = tx.send(Msg::Done);
            return;
        }

        send("\n== 2) Creando snapshot de respaldo ==\n".into());
        send(run_cmd(&["sudo", "snapper", "-c", "root", "create", "-d", "antes de actualizar"]));

        send("\n== 3) pacman -Syu ==\n".into());
        send(run_cmd(&["sudo", "pacman", "-Syu", "--noconfirm"]));

        send("\n== 4) yay -Sua (AUR) ==\n".into());
        send(run_cmd(&["yay", "-Sua", "--noconfirm"]));

        send("\n== 5) checkrebuild ==\n".into());
        send(run_cmd(&["checkrebuild"]));

        send("\n== 6) Revisando que todo siga bien DESPUES ==\n".into());
        let check_despues = run_cmd(&["teto-check-shell"]);
        send(check_despues.clone());

        if check_despues.contains("AVISO") {
            let _ = tx.send(Msg::Warn("Caelestia quedo roto tras actualizar. Puedes volver atras con snapper (pestaña Respaldos).".into()));
        }
        let _ = tx.send(Msg::Done);
    });

    glib::timeout_add_local(Duration::from_millis(100), move || {
        let mut done = false;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Append(s) => {
                    let mut end = buf.end_iter();
                    buf.insert(&mut end, &s);
                }
                Msg::Warn(s) => {
                    warning.set_label(&s);
                    warning.add_css_class("error");
                    warning.set_visible(true);
                }
                Msg::Done => {
                    btn.set_sensitive(true);
                    btn.set_label("Actualizar sistema (revisar + respaldo + pacman + yay)");
                    done = true;
                }
            }
        }
        if done { glib::ControlFlow::Break } else { glib::ControlFlow::Continue }
    });
}

fn build_actualizar_page(output: &gtk::TextView, warning: &gtk::Label) -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    page.set_margin_top(20);
    page.set_margin_bottom(20);
    page.set_margin_start(20);
    page.set_margin_end(20);

    let btn = gtk::Button::with_label("Actualizar sistema (revisar + respaldo + pacman + yay)");

    let buf = output.buffer();
    let warning = warning.clone();

    btn.connect_clicked(move |b| {
        let buf = buf.clone();
        let warning = warning.clone();
        let btn = b.clone();

        let dialog = adw::MessageDialog::builder()
            .heading("¿Actualizar el sistema?")
            .body("Va a crear un respaldo y correr pacman -Syu + yay -Sua. Puede tardar varios minutos.")
            .build();
        if let Some(root) = b.root() {
            if let Some(window) = root.downcast_ref::<gtk::Window>() {
                dialog.set_transient_for(Some(window));
            }
        }
        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("confirm", "Actualizar");
        dialog.set_response_appearance("confirm", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("cancel"));

        dialog.connect_response(None, move |d, response| {
            if response == "confirm" {
                start_actualizacion(buf.clone(), warning.clone(), btn.clone());
            }
            d.close();
        });

        dialog.present();
    });

    page.append(&btn);
    page
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("com.teto.tools")
        .build();

    app.connect_activate(|app| {
        let output = gtk::TextView::new();
        output.set_editable(false);
        output.set_wrap_mode(gtk::WrapMode::Word);
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_child(Some(&output));
        scroll.set_vexpand(true);
        scroll.set_min_content_height(150);

        let warning = gtk::Label::new(None);
        warning.set_visible(false);
        warning.set_wrap(true);

        let stack = adw::ViewStack::new();
        stack.set_vexpand(false);
        stack.set_vhomogeneous(false);
        stack.add_titled(&build_respaldos_page(&output), Some("respaldos"), "Respaldos");
        stack.add_titled(&build_actualizar_page(&output, &warning), Some("actualizar"), "Actualizar");

        let switcher = adw::ViewSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_policy(adw::ViewSwitcherPolicy::Wide);

        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&switcher));

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.append(&header);
        content.append(&stack);
        content.append(&warning);
        content.append(&scroll);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Teto Tools")
            .default_width(500)
            .default_height(450)
            .content(&content)
            .build();

        window.present();
    });

    app.run()
}