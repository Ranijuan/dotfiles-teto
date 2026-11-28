use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use libadwaita as adw;
use adw::prelude::*;
use std::collections::HashSet;
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

// (paquetes de repos, paquetes del AUR). Err si falta checkupdates o falla.
fn pending_updates() -> Result<(String, String), String> {
    match Command::new("checkupdates").output() {
        Ok(o) => {
            if o.status.code() == Some(1) {
                return Err(format!(
                    "checkupdates fallo: {}",
                    strip_ansi(&String::from_utf8_lossy(&o.stderr))
                ));
            }
            let repo = strip_ansi(&String::from_utf8_lossy(&o.stdout));
            let aur = match Command::new("yay").arg("-Qua").output() {
                Ok(a) => strip_ansi(&String::from_utf8_lossy(&a.stdout)),
                Err(_) => String::new(),
            };
            Ok((repo, aur))
        }
        Err(_) => Err("Falta checkupdates. Instala con: sudo pacman -S pacman-contrib".to_string()),
    }
}

fn count_lines(s: &str) -> usize {
    s.lines().filter(|l| !l.trim().is_empty()).count()
}

fn foreign_pkgs() -> HashSet<String> {
    let (_, out) = run_ok(&["pacman", "-Qmq"]);
    out.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

// Para cada paquete de repos pendiente, que paquetes del AUR dependen de el
fn impact(repo_pending: &str) -> String {
    let foreign = foreign_pkgs();
    if foreign.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for line in repo_pending.lines() {
        let name = line.split_whitespace().next().unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let (ok, deps) = run_ok(&["pactree", "-r", "-u", name]);
        if !ok {
            continue;
        }
        let hit: Vec<&str> = deps
            .lines()
            .map(|l| l.trim())
            .filter(|l| *l != name && foreign.contains(*l))
            .collect();
        if !hit.is_empty() {
            out.push_str(&format!("  {} -> {}\n", name, hit.join(", ")));
        }
    }
    out
}

// Paquetes que quedaron desfasados y hay que reconstruir
fn stale_packages() -> Vec<String> {
    let (_, out) = run_ok(&["checkrebuild"]);
    if out.trim_start().starts_with("Error") {
        return Vec::new();
    }
    let mut v: Vec<String> = out
        .lines()
        .filter_map(|l| l.split_whitespace().last())
        .map(|s| s.to_string())
        .collect();
    v.sort();
    v.dedup();
    v
}

fn arch_news() -> String {
    let (_, out) = run_ok(&["yay", "-Pw"]);
    out.lines().take(12).collect::<Vec<_>>().join("\n")
}

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
    Rebuild(bool),
    Done,
}

#[derive(Clone, Copy)]
enum Task {
    Search,
    Update,
    Rebuild,
}

fn run_search(tx: mpsc::Sender<Msg>) {
    let send = |s: String| { let _ = tx.send(Msg::Append(s)); };

    if Path::new(PACMAN_LOCK).exists() {
        let _ = tx.send(Msg::Warn("Hay otro pacman en marcha (db.lck). Espera a que termine.".into()));
        let _ = tx.send(Msg::Updates(false));
        let _ = tx.send(Msg::Rebuild(false));
        let _ = tx.send(Msg::Done);
        return;
    }

    send("Buscando actualizaciones (repos + AUR)...\n".into());
    let (repo, aur) = match pending_updates() {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.send(Msg::Warn(e));
            let _ = tx.send(Msg::Updates(false));
            let _ = tx.send(Msg::Rebuild(false));
            let _ = tx.send(Msg::Done);
            return;
        }
    };
    let n_repo = count_lines(&repo);
    let n_aur = count_lines(&aur);

    send("Revisando paquetes desfasados...\n".into());
    let stale = stale_packages();

    if n_repo + n_aur == 0 && stale.is_empty() {
        send("\nEl sistema ya esta al dia y no hay nada desfasado. No hace falta hacer nada.\n".into());
        let _ = tx.send(Msg::Updates(false));
        let _ = tx.send(Msg::Rebuild(false));
        let _ = tx.send(Msg::Done);
        return;
    }

    if n_repo > 0 {
        send(format!("\n== {n_repo} paquetes de los repos por actualizar ==\n{repo}"));
        send("\nRevisando si eso afecta paquetes del AUR...\n".into());
        let imp = impact(&repo);
        if imp.is_empty() {
            send("Nada del AUR depende de lo que se va a actualizar.\n".into());
        } else {
            send(format!(
                "Estos paquetes del AUR dependen de lo que se va a actualizar; despues quiza haya que reconstruirlos (el boton lo hace):\n{imp}"
            ));
        }
    }
    if n_aur > 0 {
        send(format!("\n== {n_aur} paquetes del AUR por actualizar ==\n{aur}"));
    }
    if !stale.is_empty() {
        send(format!(
            "\n== Paquetes desfasados (hay que reconstruirlos) ==\n{}\n",
            stale.join("\n")
        ));
    }

    let news = arch_news();
    if !news.trim().is_empty() {
        send(format!(
            "\n== Noticias de Arch (revisa si piden algo manual) ==\n{news}\n"
        ));
    }

    let _ = tx.send(Msg::Updates(n_repo + n_aur > 0));
    let _ = tx.send(Msg::Rebuild(!stale.is_empty()));
    let _ = tx.send(Msg::Done);
}

fn run_update(tx: mpsc::Sender<Msg>) {
    let send = |s: String| { let _ = tx.send(Msg::Append(s)); };
    let abort = |msg: &str| {
        let _ = tx.send(Msg::Warn(msg.to_string()));
        let _ = tx.send(Msg::Done);
    };

    if Path::new(PACMAN_LOCK).exists() {
        abort("Hay otro pacman en marcha (db.lck). No se hizo nada.");
        return;
    }

    match pending_updates() {
        Ok((r, a)) => {
            if r.trim().is_empty() && a.trim().is_empty() {
                send("El sistema ya esta al dia. No se hizo nada.\n".into());
                let _ = tx.send(Msg::Updates(false));
                let _ = tx.send(Msg::Done);
                return;
            }
        }
        Err(e) => {
            abort(&e);
            return;
        }
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

    send("\n== 5) Buscando paquetes desfasados ==\n".into());
    let stale = stale_packages();
    if stale.is_empty() {
        send("Nada desfasado.\n".into());
    } else {
        send(format!("{}\n", stale.join("\n")));
    }

    send("\n== 6) Revisando que todo siga bien DESPUES ==\n".into());
    let check_despues = run_cmd(&["teto-check-shell"]);
    send(check_despues.clone());

    let mut avisos: Vec<String> = Vec::new();
    if !aur_ok {
        avisos.push("yay (AUR) dio errores, revisa el texto de arriba.".to_string());
    }
    if !stale.is_empty() {
        avisos.push(format!(
            "Hay {} paquetes desfasados: usa el boton Reconstruir desfasados.",
            stale.len()
        ));
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
    let _ = tx.send(Msg::Rebuild(!stale.is_empty()));
    let _ = tx.send(Msg::Done);
}

fn run_rebuild(tx: mpsc::Sender<Msg>) {
    let send = |s: String| { let _ = tx.send(Msg::Append(s)); };
    let abort = |msg: &str| {
        let _ = tx.send(Msg::Warn(msg.to_string()));
        let _ = tx.send(Msg::Done);
    };

    if Path::new(PACMAN_LOCK).exists() {
        abort("Hay otro pacman en marcha (db.lck). No se hizo nada.");
        return;
    }

    send("Buscando paquetes desfasados...\n".into());
    let stale = stale_packages();
    if stale.is_empty() {
        send("No hay nada desfasado. No se hizo nada.\n".into());
        let _ = tx.send(Msg::Rebuild(false));
        let _ = tx.send(Msg::Done);
        return;
    }
    send(format!("Se van a reconstruir:\n{}\n", stale.join("\n")));

    send("\n== Creando snapshot de respaldo ==\n".into());
    let (snap_ok, snap_out) = run_ok(&["sudo", "snapper", "-c", "root", "create", "-d", "antes de reconstruir"]);
    send(snap_out);
    if !snap_ok {
        abort("Fallo el snapshot. No se reconstruyo nada.");
        return;
    }

    send("\n== Reconstruyendo con yay (puede tardar) ==\n".into());
    let mut args: Vec<&str> = vec!["yay", "-S", "--rebuild", "--noconfirm"];
    for p in &stale {
        args.push(p.as_str());
    }
    let (ok, out) = run_ok(&args);
    send(out);
    if !ok {
        abort("La reconstruccion dio errores. Revisa el texto de arriba; puedes volver atras con snapper.");
        return;
    }

    let still = stale_packages();
    let _ = tx.send(Msg::Rebuild(!still.is_empty()));
    if still.is_empty() {
        send("\nListo. Reinicia la barra o la sesion para usar las versiones nuevas.\n".into());
    } else {
        let _ = tx.send(Msg::Warn(format!(
            "Siguen desfasados: {}",
            still.join(", ")
        )));
    }
    let _ = tx.send(Msg::Done);
}

fn start_task(
    task: Task,
    buf: gtk::TextBuffer,
    warning: gtk::Label,
    btn_buscar: gtk::Button,
    btn_act: gtk::Button,
    btn_reb: gtk::Button,
) {
    let mut act_state = btn_act.is_sensitive();
    let mut reb_state = btn_reb.is_sensitive();

    btn_buscar.set_sensitive(false);
    btn_buscar.set_label("Trabajando...");
    btn_act.set_sensitive(false);
    btn_reb.set_sensitive(false);
    warning.set_visible(false);
    buf.set_text("");

    let (tx, rx) = mpsc::channel::<Msg>();

    thread::spawn(move || match task {
        Task::Search => run_search(tx),
        Task::Update => run_update(tx),
        Task::Rebuild => run_rebuild(tx),
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
                    act_state = found;
                }
                Msg::Rebuild(found) => {
                    reb_state = found;
                }
                Msg::Done => {
                    btn_buscar.set_sensitive(true);
                    btn_buscar.set_label("Buscar actualizaciones");
                    btn_act.set_sensitive(act_state);
                    btn_reb.set_sensitive(reb_state);
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
    let btn_reb = gtk::Button::with_label("Reconstruir desfasados");
    btn_act.set_sensitive(false);
    btn_reb.set_sensitive(false);

    let buf = output.buffer();

    {
        let buf = buf.clone();
        let warning = warning.clone();
        let act = btn_act.clone();
        let reb = btn_reb.clone();
        btn_buscar.connect_clicked(move |b| {
            start_task(Task::Search, buf.clone(), warning.clone(), b.clone(), act.clone(), reb.clone());
        });
    }
    {
        let buf = buf.clone();
        let warning = warning.clone();
        let buscar = btn_buscar.clone();
        let reb = btn_reb.clone();
        btn_act.connect_clicked(move |b| {
            let buf = buf.clone();
            let warning = warning.clone();
            let buscar = buscar.clone();
            let reb = reb.clone();
            let act = b.clone();
            confirm(
                b,
                "¿Actualizar el sistema?",
                "Crea un respaldo y corre pacman -Syu + yay -Sua. Puede tardar varios minutos. No apagues el PC.",
                "Actualizar",
                false,
                move || {
                    start_task(Task::Update, buf.clone(), warning.clone(), buscar.clone(), act.clone(), reb.clone());
                },
            );
        });
    }
    {
        let buf = buf.clone();
        let warning = warning.clone();
        let buscar = btn_buscar.clone();
        let act = btn_act.clone();
        btn_reb.connect_clicked(move |b| {
            let buf = buf.clone();
            let warning = warning.clone();
            let buscar = buscar.clone();
            let act = act.clone();
            let reb = b.clone();
            confirm(
                b,
                "¿Reconstruir paquetes desfasados?",
                "Crea un respaldo y reconstruye con yay los paquetes que quedaron desfasados. Puede tardar varios minutos.",
                "Reconstruir",
                false,
                move || {
                    start_task(Task::Rebuild, buf.clone(), warning.clone(), buscar.clone(), act.clone(), reb.clone());
                },
            );
        });
    }

    page.append(&btn_buscar);
    page.append(&btn_act);
    page.append(&btn_reb);
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
