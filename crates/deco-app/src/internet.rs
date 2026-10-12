//! "Conexão de internet" page, modeled after the official app's Internet
//! Connection screen. Data comes over TMP, which needs the TP-Link ID email.

use std::net::Ipv4Addr;
use std::rc::Rc;

use adw::prelude::*;
use deco_api::tmp::TmpSession;
use deco_api::WanConfig;
use gtk::glib;

use crate::credentials::{self, Credentials};
use crate::window::{run, toast, App};

fn connect(c: &Credentials) -> deco_api::Result<TmpSession> {
    let email = c.email.as_deref().ok_or(deco_api::Error::Protocol("e-mail do TP-Link ID não informado"))?;
    TmpSession::connect(&c.host, email, &c.password)
}

struct Page {
    stack: gtk::Stack,
    save_btn: gtk::Button,
    kind_row: adw::ActionRow,
    user_row: adw::ActionRow,
    ip_row: adw::ActionRow,
    mask_row: adw::ActionRow,
    gateway_row: adw::ActionRow,
    mac_row: adw::ActionRow,
    mtu_row: adw::ActionRow,
    auto_dns: adw::SwitchRow,
    dns1: adw::EntryRow,
    dns2: adw::EntryRow,
    loaded: std::cell::RefCell<Option<WanConfig>>,
}

pub fn open(app: &Rc<App>) {
    let header = adw::HeaderBar::new();
    let save_btn = gtk::Button::builder().label("Salvar").css_classes(["suggested-action"]).sensitive(false).build();
    header.pack_end(&save_btn);

    let prefs = adw::PreferencesPage::new();

    let conn = adw::PreferencesGroup::builder().title("Conexão").build();
    let kind_row = property_row("Tipo de conexão");
    let user_row = property_row("Usuário PPPoE");
    conn.add(&kind_row);
    conn.add(&user_row);
    prefs.add(&conn);

    let ipv4 = adw::PreferencesGroup::builder().title("IPv4").build();
    let ip_row = property_row("Endereço IP");
    let mask_row = property_row("Máscara de sub-rede");
    let gateway_row = property_row("Gateway padrão");
    let mac_row = property_row("Endereço MAC");
    for r in [&ip_row, &mask_row, &gateway_row, &mac_row] {
        ipv4.add(r);
    }
    prefs.add(&ipv4);

    let dns = adw::PreferencesGroup::builder()
        .title("DNS")
        .description("Servidores usados pelo Deco para resolver nomes.")
        .build();
    let auto_dns = adw::SwitchRow::builder().title("DNS automático").subtitle("Usar os servidores do provedor").build();
    let dns1 = adw::EntryRow::builder().title("DNS primário").build();
    let dns2 = adw::EntryRow::builder().title("DNS secundário").build();
    dns.add(&auto_dns);
    dns.add(&dns1);
    dns.add(&dns2);
    prefs.add(&dns);

    let adv = adw::PreferencesGroup::builder().title("Avançado").build();
    let mtu_row = property_row("Tamanho do MTU");
    adv.add(&mtu_row);
    prefs.add(&adv);

    let stack = gtk::Stack::builder().transition_type(gtk::StackTransitionType::Crossfade).build();
    stack.add_named(&adw::StatusPage::builder().title("Carregando…").build(), Some("loading"));
    stack.add_named(&email_page(app), Some("email"));
    stack.add_named(&prefs, Some("main"));

    let view = adw::ToolbarView::new();
    view.add_top_bar(&header);
    view.set_content(Some(&stack));
    app.nav.push(&adw::NavigationPage::builder().title("Conexão de internet").child(&view).build());

    let page = Rc::new(Page {
        stack,
        save_btn,
        kind_row,
        user_row,
        ip_row,
        mask_row,
        gateway_row,
        mac_row,
        mtu_row,
        auto_dns,
        dns1,
        dns2,
        loaded: Default::default(),
    });

    let changed = glib::clone!(#[weak] page, move || update_save(&page));
    page.auto_dns.connect_active_notify(glib::clone!(#[strong] changed, move |_| changed()));
    page.dns1.connect_changed(glib::clone!(#[strong] changed, move |_| changed()));
    page.dns2.connect_changed(move |_| changed());
    page.save_btn.connect_clicked(glib::clone!(#[strong] app, #[strong] page, move |_| confirm_save(&app, &page)));

    // The email page reloads through this hook once the user fills it in.
    page.stack.connect_visible_child_name_notify(glib::clone!(#[strong] app, #[strong] page, move |stack| {
        if stack.visible_child_name().as_deref() == Some("loading") {
            load(&app, &page);
        }
    }));
    load(app, &page);
}

fn property_row(title: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(title).subtitle("—").subtitle_selectable(true).build();
    row.add_css_class("property");
    row
}

fn email_page(app: &Rc<App>) -> adw::StatusPage {
    let entry = adw::EntryRow::builder().title("E-mail do TP-Link ID").build();
    let group = adw::PreferencesGroup::new();
    group.add(&entry);
    let btn = gtk::Button::builder()
        .label("Continuar")
        .halign(gtk::Align::Center)
        .css_classes(["suggested-action", "pill"])
        .margin_top(24)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&group);
    content.append(&btn);
    let status = adw::StatusPage::builder()
        .icon_name("dialog-password-symbolic")
        .title("Conta TP-Link necessária")
        .description("As configurações de internet usam o mesmo acesso do app Deco: o e-mail da conta dona da rede.")
        .child(&adw::Clamp::builder().maximum_size(400).child(&content).build())
        .build();
    btn.connect_clicked(glib::clone!(#[strong] app, #[weak] status, move |_| {
        let email = entry.text().trim().to_owned();
        if email.is_empty() {
            return;
        }
        let creds = app.creds.borrow().clone();
        if let Some(mut c) = creds {
            c.email = Some(email);
            credentials::save(&c);
            app.creds.replace(Some(c));
        }
        if let Some(stack) = status.parent().and_downcast::<gtk::Stack>() {
            stack.set_visible_child_name("loading");
        }
    }));
    status
}

fn load(app: &Rc<App>, page: &Rc<Page>) {
    if app.creds.borrow().as_ref().is_none_or(|c| c.email.is_none()) {
        page.stack.set_visible_child_name("email");
        return;
    }
    page.stack.set_visible_child_name("loading");
    // `run` drops jobs while another one (e.g. auto-refresh) is in flight.
    if app.busy.get() {
        glib::timeout_add_local_once(
            std::time::Duration::from_millis(300),
            glib::clone!(#[strong] app, #[strong] page, move || load(&app, &page)),
        );
        return;
    }
    let page = page.clone();
    run(
        app,
        |c| {
            let mut s = connect(c)?;
            let wan = s.wan()?;
            s.close()?;
            Ok(wan)
        },
        move |app, res| match res {
            Ok(wan) => {
                show(&page, &wan);
                page.stack.set_visible_child_name("main");
            }
            Err(deco_api::Error::Ssh(_) | deco_api::Error::Login) => {
                page.stack.set_visible_child_name("email");
                toast(app, "Não foi possível entrar com esse e-mail e a senha atual.");
            }
            Err(e) => {
                toast(app, &e.to_string());
                app.nav.pop();
            }
        },
    );
}

fn show(page: &Page, wan: &WanConfig) {
    page.kind_row.set_subtitle(&connection_label(&wan.dial_type));
    page.user_row.set_subtitle(or_dash(&wan.username));
    page.user_row.set_visible(wan.dial_type == "pppoe");
    page.ip_row.set_subtitle(or_dash(&wan.ip));
    page.mask_row.set_subtitle(or_dash(&wan.mask));
    page.gateway_row.set_subtitle(or_dash(&wan.gateway));
    page.mac_row.set_subtitle(or_dash(&wan.mac));
    page.mtu_row.set_subtitle(&wan.mtu.map(|m| m.to_string()).unwrap_or_else(|| "—".into()));
    page.loaded.replace(Some(wan.clone()));
    page.auto_dns.set_active(wan.auto_dns);
    page.dns1.set_text(&wan.dns1);
    page.dns2.set_text(&wan.dns2);
    update_save(page);
}

/// Enables Save only for a valid change; manual fields follow the switch.
fn update_save(page: &Page) {
    let auto = page.auto_dns.is_active();
    page.dns1.set_sensitive(!auto);
    page.dns2.set_sensitive(!auto);
    let (d1, d2) = (page.dns1.text(), page.dns2.text());
    let valid = |t: &str| t.trim().parse::<Ipv4Addr>().is_ok();
    for (row, ok) in [(&page.dns1, auto || valid(&d1)), (&page.dns2, auto || d2.is_empty() || valid(&d2))] {
        if ok {
            row.remove_css_class("error");
        } else {
            row.add_css_class("error");
        }
    }
    let changed = page.loaded.borrow().as_ref().is_some_and(|w| {
        w.auto_dns != auto || (!auto && (w.dns1 != d1.trim() || w.dns2 != d2.trim()))
    });
    let ok = auto || (valid(&d1) && (d2.is_empty() || valid(&d2)));
    page.save_btn.set_sensitive(changed && ok);
}

fn confirm_save(app: &Rc<App>, page: &Rc<Page>) {
    let auto = page.auto_dns.is_active();
    let (d1, d2) = (page.dns1.text().trim().to_owned(), page.dns2.text().trim().to_owned());
    let body = if auto {
        "O Deco passará a usar o DNS do provedor.".to_owned()
    } else {
        format!("O Deco passará a usar {d1}{}.", if d2.is_empty() { String::new() } else { format!(" e {d2}") })
    };
    let dialog = adw::AlertDialog::new(
        Some("Alterar DNS?"),
        Some(&format!("{body} A internet pode cair por alguns segundos enquanto a conexão é refeita.")),
    );
    dialog.add_response("cancel", "Cancelar");
    dialog.add_response("save", "Alterar");
    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    dialog.set_close_response("cancel");
    dialog.connect_response(None, glib::clone!(#[strong] app, #[strong] page, move |_, resp| {
        if resp != "save" {
            return;
        }
        if app.busy.get() {
            toast(&app, "Aguarde a atualização terminar e tente de novo.");
            return;
        }
        let (d1, d2) = (d1.clone(), d2.clone());
        let page = page.clone();
        run(
            &app,
            move |c| {
                let mut s = connect(c)?;
                s.set_wan_dns(auto, &d1, &d2)?;
                // The Deco applies WAN changes asynchronously (~5 s).
                std::thread::sleep(std::time::Duration::from_secs(6));
                let wan = s.wan()?;
                s.close()?;
                Ok(wan)
            },
            move |app, res| match res {
                Ok(wan) => {
                    show(&page, &wan);
                    toast(app, "DNS alterado");
                }
                Err(e) => toast(app, &format!("Não foi possível alterar o DNS: {e}")),
            },
        );
    }));
    dialog.present(Some(&app.window));
}

fn connection_label(dial_type: &str) -> String {
    match dial_type {
        "pppoe" => "PPPoE".into(),
        "dynamic_ip" | "dhcp" => "IP dinâmico".into(),
        "static_ip" | "static" => "IP estático".into(),
        "l2tp" => "L2TP".into(),
        "pptp" => "PPTP".into(),
        "" => "—".into(),
        other => other.to_owned(),
    }
}

fn or_dash(s: &str) -> &str {
    if s.is_empty() { "—" } else { s }
}
