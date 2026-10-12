use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use deco_api::{Client, Network, Node, Session, Wan, WifiNetwork, WifiState};
use gtk::{gio, glib};

use crate::credentials::{self, Credentials};
use crate::internet;

/// Auto-refresh interval. Each refresh is a full login/logout, which briefly
/// kicks the official app, so it only runs while the window is focused.
const REFRESH_SECS: u32 = 15;

struct Snapshot {
    wan: Wan,
    wifi: WifiState,
    nodes: Vec<Node>,
    clients: Vec<Client>,
}

fn fetch(c: &Credentials) -> deco_api::Result<Snapshot> {
    let s = Session::login(&c.host, &c.password)?;
    let snap = Snapshot { wan: s.wan()?, wifi: s.wifi()?, nodes: s.nodes()?, clients: s.clients()? };
    s.logout()?;
    Ok(snap)
}

pub(crate) struct App {
    pub(crate) window: adw::ApplicationWindow,
    pub(crate) nav: adw::NavigationView,
    toasts: adw::ToastOverlay,
    stack: gtk::Stack,
    spinner: gtk::Spinner,
    internet_row: adw::ActionRow,
    nodes_group: adw::PreferencesGroup,
    clients_group: adw::PreferencesGroup,
    main_switch: adw::SwitchRow,
    guest_switch: adw::SwitchRow,
    login_host: adw::EntryRow,
    login_pass: adw::PasswordEntryRow,
    login_email: adw::EntryRow,
    dynamic_rows: RefCell<Vec<(adw::PreferencesGroup, adw::ActionRow)>>,
    pub(crate) creds: RefCell<Option<Credentials>>,
    pub(crate) busy: Cell<bool>,
    /// Set while the UI is updated from router data, so switch handlers don't fire writes.
    syncing: Cell<bool>,
}

pub fn build(application: &adw::Application) {
    let toasts = adw::ToastOverlay::new();
    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("Deco")
        .default_width(460)
        .default_height(780)
        .content(&toasts)
        .build();

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new("Deco", "Rede mesh")));
    let refresh_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.set_tooltip_text(Some("Atualizar"));
    let spinner = gtk::Spinner::new();
    header.pack_start(&refresh_btn);
    header.pack_start(&spinner);

    let menu = gio::Menu::new();
    menu.append(Some("Reiniciar rede"), Some("win.reboot"));
    menu.append(Some("Sair"), Some("win.logout"));
    let menu_btn = gtk::MenuButton::builder().icon_name("open-menu-symbolic").menu_model(&menu).build();
    header.pack_end(&menu_btn);

    // Main page
    let page = adw::PreferencesPage::new();

    let status_group = adw::PreferencesGroup::new();
    let internet_row = adw::ActionRow::builder().title("Internet").activatable(true).build();
    internet_row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
    internet_row.add_prefix(&gtk::Image::from_icon_name("network-transmit-receive-symbolic"));
    status_group.add(&internet_row);
    page.add(&status_group);

    let wifi_group = adw::PreferencesGroup::builder().title("Wi-Fi").build();
    let main_switch = adw::SwitchRow::builder()
        .title("Rede principal")
        .subtitle("2.4 GHz e 5 GHz")
        .build();
    let guest_switch = adw::SwitchRow::builder()
        .title("Rede de convidados")
        .subtitle("Acesso isolado da rede principal")
        .build();
    wifi_group.add(&main_switch);
    wifi_group.add(&guest_switch);
    page.add(&wifi_group);

    let nodes_group = adw::PreferencesGroup::builder().title("Decos").build();
    page.add(&nodes_group);
    let clients_group = adw::PreferencesGroup::builder().title("Clientes").build();
    page.add(&clients_group);

    // Login page
    let login_host = adw::EntryRow::builder().title("Endereço do Deco").build();
    login_host.set_text(credentials::default_host());
    let login_pass = adw::PasswordEntryRow::builder().title("Senha do administrador").build();
    let login_email = adw::EntryRow::builder().title("E-mail do TP-Link ID (opcional)").build();
    login_email.set_tooltip_text(Some("Necessário para configurações avançadas, como DNS"));
    let login_btn = gtk::Button::builder()
        .label("Conectar")
        .halign(gtk::Align::Center)
        .css_classes(["suggested-action", "pill"])
        .margin_top(24)
        .build();
    let login_group = adw::PreferencesGroup::new();
    login_group.add(&login_host);
    login_group.add(&login_pass);
    login_group.add(&login_email);
    let login_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    login_box.append(&login_group);
    login_box.append(&login_btn);
    let login_clamp = adw::Clamp::builder().maximum_size(400).child(&login_box).build();
    let login_page = adw::StatusPage::builder()
        .icon_name("network-wireless-symbolic")
        .title("Conectar ao Deco")
        .description("Use a senha local definida no app Deco.")
        .child(&login_clamp)
        .build();

    let loading_page = adw::StatusPage::builder().title("Conectando…").build();

    let stack = gtk::Stack::builder().transition_type(gtk::StackTransitionType::Crossfade).build();
    stack.add_named(&loading_page, Some("loading"));
    stack.add_named(&login_page, Some("login"));
    stack.add_named(&page, Some("main"));

    let view = adw::ToolbarView::new();
    view.add_top_bar(&header);
    view.set_content(Some(&stack));
    let nav = adw::NavigationView::new();
    nav.add(&adw::NavigationPage::builder().title("Deco").tag("home").child(&view).build());
    toasts.set_child(Some(&nav));

    let app = Rc::new(App {
        window: window.clone(),
        nav,
        toasts,
        stack,
        spinner,
        internet_row,
        nodes_group,
        clients_group,
        main_switch,
        guest_switch,
        login_host,
        login_pass,
        login_email,
        dynamic_rows: RefCell::default(),
        creds: RefCell::new(credentials::load()),
        busy: Cell::new(false),
        syncing: Cell::new(false),
    });

    refresh_btn.connect_clicked(glib::clone!(#[strong] app, move |_| refresh(&app)));
    app.internet_row.connect_activated(glib::clone!(#[strong] app, move |_| internet::open(&app)));
    login_btn.connect_clicked(glib::clone!(#[strong] app, move |_| submit_login(&app)));
    app.login_pass.connect_entry_activated(glib::clone!(#[strong] app, move |_| submit_login(&app)));
    app.main_switch.connect_active_notify(glib::clone!(#[strong] app, move |row| {
        on_wifi_toggled(&app, row, WifiNetwork::Main)
    }));
    app.guest_switch.connect_active_notify(glib::clone!(#[strong] app, move |row| {
        on_wifi_toggled(&app, row, WifiNetwork::Guest)
    }));

    let reboot = gio::SimpleAction::new("reboot", None);
    reboot.connect_activate(glib::clone!(#[strong] app, move |_, _| confirm_reboot(&app)));
    window.add_action(&reboot);
    let logout = gio::SimpleAction::new("logout", None);
    logout.connect_activate(glib::clone!(#[strong] app, move |_, _| {
        credentials::clear();
        app.creds.replace(None);
        app.stack.set_visible_child_name("login");
    }));
    window.add_action(&logout);

    glib::timeout_add_seconds_local(REFRESH_SECS, glib::clone!(#[strong] app, move || {
        if app.window.is_active() && app.stack.visible_child_name().as_deref() == Some("main") {
            refresh(&app);
        }
        glib::ControlFlow::Continue
    }));

    if app.creds.borrow().is_some() {
        refresh(&app);
        // Dev aid for screenshots: open a subpage directly.
        if std::env::var("DECO_OPEN").as_deref() == Ok("internet") {
            internet::open(&app);
        }
    } else {
        app.stack.set_visible_child_name("login");
    }
    window.present();
}

/// Runs `job` with the current credentials on a worker thread, then `done` on the UI thread.
pub(crate) fn run<T: Send + 'static>(
    app: &Rc<App>,
    job: impl FnOnce(&Credentials) -> deco_api::Result<T> + Send + 'static,
    done: impl FnOnce(&Rc<App>, deco_api::Result<T>) + 'static,
) {
    let Some(creds) = app.creds.borrow().clone() else { return };
    if app.busy.replace(true) {
        return;
    }
    app.spinner.start();
    let app = app.clone();
    glib::spawn_future_local(async move {
        let res = gio::spawn_blocking(move || job(&creds))
            .await
            .unwrap_or(Err(deco_api::Error::Protocol("worker thread panicked")));
        app.busy.set(false);
        app.spinner.stop();
        done(&app, res);
    });
}

pub(crate) fn toast(app: &App, msg: &str) {
    app.toasts.add_toast(adw::Toast::new(msg));
}

fn refresh(app: &Rc<App>) {
    run(app, fetch, |app, res| match res {
        Ok(snap) => {
            show(app, &snap);
            app.stack.set_visible_child_name("main");
        }
        Err(deco_api::Error::Login) if app.stack.visible_child_name().as_deref() != Some("main") => {
            app.stack.set_visible_child_name("login");
            toast(app, "Não foi possível entrar. Senha errada?");
        }
        Err(e) => {
            if app.stack.visible_child_name().as_deref() == Some("loading") {
                app.stack.set_visible_child_name("login");
            }
            toast(app, &e.to_string());
        }
    });
}

fn submit_login(app: &Rc<App>) {
    let creds = Credentials {
        host: app.login_host.text().trim().to_owned(),
        password: app.login_pass.text().to_string(),
        email: Some(app.login_email.text().trim().to_owned()).filter(|e| !e.is_empty()),
    };
    if creds.host.is_empty() || creds.password.is_empty() {
        return;
    }
    app.creds.replace(Some(creds.clone()));
    app.stack.set_visible_child_name("loading");
    run(app, fetch, move |app, res| match res {
        Ok(snap) => {
            credentials::save(&creds);
            app.login_pass.set_text("");
            show(app, &snap);
            app.stack.set_visible_child_name("main");
        }
        Err(e) => {
            app.creds.replace(None);
            app.stack.set_visible_child_name("login");
            toast(app, &e.to_string());
        }
    });
}

fn show(app: &App, snap: &Snapshot) {
    app.syncing.set(true);

    let online = snap.nodes.iter().find(|n| n.is_master).is_some_and(|n| n.online);
    app.internet_row.set_subtitle(&format!(
        "{} · {} ({})",
        if online { "Conectado" } else { "Sem conexão" },
        snap.wan.ip,
        snap.wan.dial_type.to_uppercase()
    ));

    app.main_switch.set_active(snap.wifi.main_2g || snap.wifi.main_5g);
    app.guest_switch.set_active(snap.wifi.guest_2g || snap.wifi.guest_5g);

    for (group, row) in app.dynamic_rows.take() {
        group.remove(&row);
    }
    let mut rows = Vec::new();

    let mut nodes = snap.nodes.clone();
    nodes.sort_by_key(|n| !n.is_master);
    for node in &nodes {
        let parent = node
            .parent_mac
            .as_ref()
            .and_then(|p| snap.nodes.iter().find(|n| &n.mac == p))
            .map(|n| n.name.as_str());
        let role = match parent {
            None => "Principal".to_owned(),
            Some(p) => format!("via {p}"),
        };
        let row = adw::ActionRow::builder()
            .title(&node.name)
            .subtitle(format!("{role} · {} · Deco {} v{}", node.ip, node.model, node.hardware))
            .build();
        row.add_prefix(&gtk::Image::from_icon_name("network-workgroup-symbolic"));
        let state = gtk::Label::new(Some(if node.online { "Online" } else { "Offline" }));
        state.add_css_class(if node.online { "success" } else { "error" });
        row.add_suffix(&state);
        app.nodes_group.add(&row);
        rows.push((app.nodes_group.clone(), row));
    }
    app.nodes_group.set_description(Some(&format!(
        "Firmware {}",
        nodes.first().map(|n| n.firmware.split(' ').next().unwrap_or("")).unwrap_or("")
    )));

    let mut clients = snap.clients.clone();
    clients.sort_by(|a, b| (b.down_kbps + b.up_kbps).cmp(&(a.down_kbps + a.up_kbps)).then(a.name.cmp(&b.name)));
    app.clients_group.set_title(&format!("Clientes ({})", clients.len()));
    for c in &clients {
        let row = adw::ActionRow::builder()
            .title(glib::markup_escape_text(&c.name))
            .subtitle(client_subtitle(c))
            .build();
        row.add_prefix(&gtk::Image::from_icon_name(client_icon(c)));
        let speed = gtk::Label::new(Some(&format!("↓ {}  ↑ {}", speed(c.down_kbps), speed(c.up_kbps))));
        speed.add_css_class("dim-label");
        speed.add_css_class("numeric");
        row.add_suffix(&speed);
        app.clients_group.add(&row);
        rows.push((app.clients_group.clone(), row));
    }

    app.dynamic_rows.replace(rows);
    app.syncing.set(false);
}

fn client_subtitle(c: &Client) -> String {
    let net = match c.network {
        Network::Main => "",
        Network::Guest => "Convidados · ",
        Network::Iot => "IoT · ",
        Network::Wired => "Cabo · ",
        Network::Unknown => "",
    };
    let band = match (c.band.is_empty(), c.network) {
        (false, _) => format!(" · {}", c.band),
        (true, Network::Wired) => String::new(),
        (true, _) => " · Wi-Fi".to_string(),
    };
    format!("{net}{}{band}", c.ip)
}

fn client_icon(c: &Client) -> &'static str {
    let name = c.name.to_lowercase();
    let has = |keys: &[&str]| keys.iter().any(|k| name.contains(k));
    if c.network == Network::Wired {
        "network-wired-symbolic"
    } else if has(&["iphone", "android", "galaxy", "redmi", "s2", "pixel", "phone", "moto"]) {
        "phone-symbolic"
    } else if has(&["mac", "laptop", "pc", "desktop", "notebook", "thinkpad"]) {
        "computer-symbolic"
    } else if has(&["tv", "tizen", "roku", "chromecast", "fire"]) {
        "video-display-symbolic"
    } else {
        "network-wireless-symbolic"
    }
}

fn speed(kbps: u64) -> String {
    if kbps >= 1024 {
        format!("{:.1} MB/s", kbps as f64 / 1024.0)
    } else {
        format!("{kbps} KB/s")
    }
}

fn on_wifi_toggled(app: &Rc<App>, row: &adw::SwitchRow, network: WifiNetwork) {
    if app.syncing.get() {
        return;
    }
    let enable = row.is_active();
    if network == WifiNetwork::Main && !enable {
        confirm_main_off(app, row.clone());
        return;
    }
    apply_wifi(app, row.clone(), network, enable);
}

fn apply_wifi(app: &Rc<App>, row: adw::SwitchRow, network: WifiNetwork, enable: bool) {
    run(
        app,
        move |c| {
            let s = Session::login(&c.host, &c.password)?;
            s.set_wifi(network, enable)?;
            s.logout()
        },
        move |app, res| match res {
            Ok(()) => toast(app, &format!("{} {}", row.title(), if enable { "ligada" } else { "desligada" })),
            Err(e) => {
                revert(app, &row, !enable);
                toast(app, &e.to_string());
            }
        },
    );
}

fn revert(app: &App, row: &adw::SwitchRow, value: bool) {
    app.syncing.set(true);
    row.set_active(value);
    app.syncing.set(false);
}

fn confirm_main_off(app: &Rc<App>, row: adw::SwitchRow) {
    let dialog = adw::AlertDialog::new(
        Some("Desligar rede principal?"),
        Some("Todos os dispositivos no Wi-Fi principal serão desconectados, incluindo este computador se estiver no Wi-Fi."),
    );
    dialog.add_response("cancel", "Cancelar");
    dialog.add_response("off", "Desligar");
    dialog.set_response_appearance("off", adw::ResponseAppearance::Destructive);
    dialog.set_close_response("cancel");
    dialog.connect_response(None, glib::clone!(#[strong] app, move |_, resp| {
        if resp == "off" {
            apply_wifi(&app, row.clone(), WifiNetwork::Main, false);
        } else {
            revert(&app, &row, true);
        }
    }));
    dialog.present(Some(&app.window));
}

fn confirm_reboot(app: &Rc<App>) {
    let dialog = adw::AlertDialog::new(
        Some("Reiniciar a rede?"),
        Some("Todos os Decos serão reiniciados. A internet ficará fora por alguns minutos."),
    );
    dialog.add_response("cancel", "Cancelar");
    dialog.add_response("reboot", "Reiniciar");
    dialog.set_response_appearance("reboot", adw::ResponseAppearance::Destructive);
    dialog.set_close_response("cancel");
    dialog.connect_response(None, glib::clone!(#[strong] app, move |_, resp| {
        if resp != "reboot" {
            return;
        }
        run(
            &app,
            |c| {
                let s = Session::login(&c.host, &c.password)?;
                s.reboot_all()?;
                s.logout()
            },
            |app, res| match res {
                Ok(()) => toast(app, "Reiniciando todos os Decos…"),
                Err(e) => toast(app, &e.to_string()),
            },
        );
    }));
    dialog.present(Some(&app.window));
}
