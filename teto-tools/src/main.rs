use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use libadwaita as adw;
use adw::prelude::*;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
 
const PACMAN_LOCK: &str = "/var/lib/pacman/db.lck";
 
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
 
// Ejecuta un comando y devuelve (exito, salida)
fn run_ok(args: &[&str]) -> (bool, String) {
    match Command::new(args[0]).args(&args[1..]).output() {
        Ok(out) => {
            let mut s = String::from_utf8_lossy(&out.stdout).to_string();
            s.push_str(&String::from_utf8_lossy(&out.stderr));
            (out.status.success(), strip_ansi(&s))
        }
        Err(e) => (false, format!("Error: {e}")),
    }
}
 
fn run_cmd(args: &[&str]) -> String {
    run_ok(args).1
}
 
// Lista de paquetes pendientes (repos + AUR). Err si falta checkupdates.
fn pending_updates() -> Result<String, String> {
    match Command::new("checkupdates").output() {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
            if let Ok(a) = Command::new("yay").arg("-Qua").output() {
                s.push_str(&String::from_utf8_lossy(&a.stdout));
            }
            Ok(strip_ansi(&s))
        }
        Err(_) => Err("Falta checkupdates. Instala con: sudo pacman -S pacman-contrib".to_string()),
    }
}
 
// true si el kernel en uso ya no existe en /usr/lib/modules (hay que reiniciar)
fn reboot_needed() -> bool {
    let rel = run_cmd(&["uname", "-r"]);
    !Path::new(&format!("/usr/lib/modules/{}", rel.trim())).exists()
}
 
fn confirm<F: Fn() + 'static>(
    anchor: &gtk::Button,
    heading: &str,
    body: &str,
    accept: &str,
    destructive: bool,
    on_accept: F,
) {
    let dialog = adw::MessageDialog::builder()
        .heading(heading)
        .body(body)
        .build();
    if let Some(root) = anchor.root() {
        if let Some(window) = root.downcast_ref::<gtk::Window>() {
            dialog.set_transient_for(Some(window));
        }
    }
    dialog.add_response("cancel", "Cancelar");
    dialog.add_response("confirm", accept);
    dialog.set_response_appearance(
        "confirm",
        if destructive {
            adw::ResponseAppearance::Destructive
        } else {
            adw::ResponseAppearance::Suggested
        },
    );
    dialog.set_default_response(Some("cancel"));
    dialog.connect_response(None, move |d, response| {
        if response == "confirm" {
            on_accept();
        }
        d.close();
    });
    dialog.present();
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
        btn_volver.connect_clicked(move |b| {
            let num = entry_num.text().to_string().trim().to_string();
            if num.is_empty() || num == "0" || !num.chars().all(|c| c.is_ascii_digit()) {
                buf.set_text("Escribe un numero de snapshot valido (solo digitos, mayor que 0).");
                return;
            }
            let body = format!(
                "Se deshacen los cambios del sistema desde el snapshot {num}. Antes se crea un snapshot nuevo por si te arrepientes."
            );
            let buf = buf.clone();
            confirm(b, "¿Volver a este snapshot?", &body, "Volver", true, move || {
                let _ = run_ok(&["sudo", "snapper", "-c", "root", "create", "-d", "antes de volver"]);
                let res = run_cmd(&["sudo", "snapper", "-c", "root", "undochange", &format!("{num}..0")]);
                buf.set_text(&res);
            });
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
    Warn(String),
    Updates(bool),
    Done,
}
 
#[derive(Clone, Copy)]
enum Task {
    Search,
    Update,
}
 
fn run_search(tx: mpsc::Sender<Msg>) {
    let send = |s: String| { let _ = tx.send(Msg::Append(s)); };
 
    if Path::new(PACMAN_LOCK).exists() {
        let _ = tx.send(Msg::Warn("Hay otro pacman en marcha (db.lck). Espera a que termine.".into()));
        let _ = tx.send(Msg::Updates(false));
        let _ = tx.send(Msg::Done);
        return;
    }
 
    send("Buscando actualizaciones (repos + AUR)...\n".into());
    match pending_updates() {
        Err(e) => {
            let _ = tx.send(Msg::Warn(e));
            let _ = tx.send(Msg::Updates(false));
        }
        Ok(list) => {
            if list.trim().is_empty() {
                send("\nEl sistema ya esta al dia. No hace falta actualizar.\n".into());
                let _ = tx.send(Msg::Updates(false));
            } else {
                let n = list.lines().filter(|l| !l.trim().is_empty()).count();
                send(format!("\nHay {n} paquetes por actualizar:\n\n{list}"));
                let _ = tx.send(Msg::Updates(true));
            }
        }
    }
    let _ = tx.send(Msg::Done);
}
 
fn run_update(tx: mpsc::Sender<Msg>) {
    let send = |s: String| { let _ = tx.send(Msg::Append(s)); };
    let abort = |msg: &str| {
        let _ = tx.send(Msg::Warn(msg.to_string()));
        let _ = tx.send(Msg::Updates(true));
        let _ = tx.send(Msg::Done);
    };
 
    if Path::new(PACMAN_LOCK).exists() {
        abort("Hay otro pacman en marcha (db.lck). No se hizo nada.");
        return;
    }
 
    send("== 1) Revisando que todo este bien ANTES ==\n".into());
    let check_antes = run_cmd(&["teto-check-shell"]);
    send(check_antes.clone());
    if check_antes.contains("AVISO") {
        send("\nSe detuvo: algo ya estaba roto antes de empezar. Arreglalo primero.\n".into());
        abort("Algo ya estaba roto ANTES de actualizar. No se hizo nada mas.");
        return;
    }
 
    send("\n== 2) Creando snapshot de respaldo ==\n".into());
    let (snap_ok, snap_out) = run_ok(&["sudo", "snapper", "-c", "root", "create", "-d", "antes de actualizar"]);
    send(snap_out);
    if !snap_ok {
        send("\nNo se pudo crear el snapshot. Se cancela para no actualizar sin respaldo.\n".into());
        abort("Fallo el snapshot. No se actualizo nada.");
        return;
    }
 
    send("\n== 3) pacman -Syu ==\n".into());
    let (pac_ok, pac_out) = run_ok(&["sudo", "pacman", "-Syu", "--noconfirm"]);
    send(pac_out);
    if !pac_ok {
        abort("pacman fallo. Revisa el texto de arriba; puedes volver atras con snapper.");
        return;
    }
 
    send("\n== 4) yay -Sua (AUR) ==\n".into());
    let (aur_ok, aur_out) = run_ok(&["yay", "-Sua", "--noconfirm"]);
    send(aur_out);
 
    send("\n== 5) checkrebuild ==\n".into());
    send(run_cmd(&["checkrebuild"]));
 
    send("\n== 6) Revisando que todo siga bien DESPUES ==\n".into());
    let check_despues = run_cmd(&["teto-check-shell"]);
    send(check_despues.clone());
 
    let mut avisos: Vec<String> = Vec::new();
    if !aur_ok {
        avisos.push("yay (AUR) dio errores, revisa el texto de arriba.".to_string());
    }
    if check_despues.contains("AVISO") {
        avisos.push("Caelestia quedo roto tras actualizar. Puedes volver atras con snapper (pestaña Respaldos).".to_string());
    }
    if reboot_needed() {
        avisos.push("Se actualizo el kernel: reinicia el PC cuando puedas.".to_string());
    }
    if !avisos.is_empty() {
        let _ = tx.send(Msg::Warn(avisos.join("\n")));
    } else {
        send("\nListo, todo salio bien.\n".into());
    }
    let _ = tx.send(Msg::Updates(false));
    let _ = tx.send(Msg::Done);
}
 
fn start_task(
    task: Task,
    buf: gtk::TextBuffer,
    warning: gtk::Label,
    btn_buscar: gtk::Button,
    btn_act: gtk::Button,
) {
    btn_buscar.set_sensitive(false);
    btn_buscar.set_label("Trabajando...");
    btn_act.set_sensitive(false);
    warning.set_visible(false);
    buf.set_text("");
 
    let (tx, rx) = mpsc::channel::<Msg>();
 
    thread::spawn(move || match task {
        Task::Search => run_search(tx),
        Task::Update => run_update(tx),
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
                Msg::Updates(found) => {
                    btn_act.set_sensitive(found);
                }
                Msg::Done => {
                    btn_buscar.set_sensitive(true);
                    btn_buscar.set_label("Buscar actualizaciones");
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
 
    let btn_buscar = gtk::Button::with_label("Buscar actualizaciones");
    let btn_act = gtk::Button::with_label("Actualizar sistema (respaldo + pacman + yay)");
    btn_act.set_sensitive(false);
 
    let buf = output.buffer();
 
    {
        let buf = buf.clone();
        let warning = warning.clone();
        let btn_act = btn_act.clone();
        btn_buscar.connect_clicked(move |b| {
            start_task(Task::Search, buf.clone(), warning.clone(), b.clone(), btn_act.clone());
        });
    }
    {
        let buf = buf.clone();
        let warning = warning.clone();
        let btn_buscar = btn_buscar.clone();
        btn_act.connect_clicked(move |b| {
            let buf = buf.clone();
            let warning = warning.clone();
            let buscar = btn_buscar.clone();
            let act = b.clone();
            confirm(
                b,
                "¿Actualizar el sistema?",
                "Crea un respaldo y corre pacman -Syu + yay -Sua. Puede tardar varios minutos. No apagues el PC.",
                "Actualizar",
                false,
                move || {
                    start_task(Task::Update, buf.clone(), warning.clone(), buscar.clone(), act.clone());
                },
            );
        });
    }
 
    page.append(&btn_buscar);
    page.append(&btn_act);
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
 