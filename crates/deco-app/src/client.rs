//! Client detail page: addresses, which Deco it is attached to, IP
//! reservation (TMP) and blocking (TMP).

use std::rc::Rc;

use adw::prelude::*;
use deco_api::tmp::TmpSession;
use deco_api::{Client, Session};
use gtk::glib;

use crate::credentials::Credentials;
use crate::window::{detail_header, client_icon, connection_label, refresh, run, speed, toast, App};

fn tmp(c: &Credentials) -> deco_api::Result<TmpSession> {
    let email = c.email.as_deref().ok_or(deco_api::Error::Protocol("e-mail do TP-Link ID não informado"))?;
    TmpSession::connect(&c.host, email, &c.password)
}

/// Data loaded when the page opens.
struct Details {
    deco: Option<String>,
    /// `None` when TMP is unavailable (no email): reservation row is hidden.
    reserved: Option<bool>,
}

struct Page {
    client: Client,
    deco_row: adw::ActionRow,
    reserve_row: adw::SwitchRow,
    manage: adw::PreferencesGroup,
    /// Set while the switch is updated from router data, so it doesn't write back.
    syncing: std::cell::Cell<bool>,
}

pub fn open(app: &Rc<App>, client: &Client) {
    let prefs = adw::PreferencesPage::new();

    let head = detail_header(client_icon(client), &client.name, &format!("↓ {}  ↑ {}", speed(client.down_kbps), speed(client.up_kbps)));
    let head_group = adw::PreferencesGroup::new();
    head_group.add(&head);
    prefs.add(&head_group);

    let info = adw::PreferencesGroup::builder().title("Conexão").build();
    info.add(&property_row("Endereço IP", &client.ip));
    info.add(&property_row("Endereço MAC", &client.mac));
    info.add(&property_row("Rede", &connection_label(client)));
    let deco_row = property_row("Conectado ao Deco", "…");
    info.add(&deco_row);
    prefs.add(&info);

    let manage = adw::PreferencesGroup::builder().title("Gerenciar").build();
    let reserve_row = adw::SwitchRow::builder()
        .title("Reservar endereço IP")
        .subtitle(format!("Sempre usar {}", client.ip))
        .build();
    // Shown once the reservation state is known (needs TMP).
    manage.set_visible(false);
    manage.add(&reserve_row);
    let block_btn = gtk::Button::builder()
        .label("Bloquear dispositivo")
        .css_classes(["destructive-action", "pill"])
        .halign(gtk::Align::Center)
        .margin_top(12)
        .build();
    prefs.add(&manage);
    let block_group = adw::PreferencesGroup::new();
    block_group.add(&block_btn);
    prefs.add(&block_group);

    let view = adw::ToolbarView::new();
    view.add_top_bar(&adw::HeaderBar::new());
    view.set_content(Some(&prefs));
    app.nav.push(&adw::NavigationPage::builder().title(client.name.as_str()).child(&view).build());
    // The action button grabs focus on map, which would scroll to the bottom.
    glib::idle_add_local_once(glib::clone!(#[weak] prefs, move || prefs.scroll_to_top()));

    let page = Rc::new(Page { client: client.clone(), deco_row, reserve_row, manage, syncing: Default::default() });
    page.reserve_row.connect_active_notify(glib::clone!(#[strong] app, #[strong] page, move |row| {
        if !page.syncing.get() {
            set_reservation(&app, &page, row.is_active());
        }
    }));
    block_btn.connect_clicked(glib::clone!(#[strong] app, #[strong] page, move |_| confirm_block(&app, &page)));
    block_btn.set_sensitive(app.creds.borrow().as_ref().is_some_and(|c| c.email.is_some()));
    load(app, &page);
}

fn property_row(title: &str, value: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(if value.is_empty() { "—" } else { value })
        .subtitle_selectable(true)
        .build();
    row.add_css_class("property");
    row
}

fn load(app: &Rc<App>, page: &Rc<Page>) {
    // `run` drops jobs while another one (e.g. auto-refresh) is in flight.
    if app.busy.get() {
        glib::timeout_add_local_once(
            std::time::Duration::from_millis(300),
            glib::clone!(#[strong] app, #[strong] page, move || load(&app, &page)),
        );
        return;
    }
    let mac = page.client.mac.clone();
    let page = page.clone();
    run(
        app,
        move |c| {
            let s = Session::login(&c.host, &c.password)?;
            let mut deco = None;
            for node in s.nodes()? {
                if s.clients_of(&node.mac)?.iter().any(|cl| cl.mac == mac) {
                    deco = Some(node.name);
                    break;
                }
            }
            s.logout()?;
            let reserved = match c.email {
                Some(_) => {
                    let mut t = tmp(c)?;
                    let r = t.reservations()?.iter().any(|r| r.mac == mac);
                    t.close()?;
                    Some(r)
                }
                None => None,
            };
            Ok(Details { deco, reserved })
        },
        move |app, res| match res {
            Ok(d) => {
                page.deco_row.set_subtitle(d.deco.as_deref().unwrap_or("—"));
                if let Some(r) = d.reserved {
                    page.syncing.set(true);
                    page.reserve_row.set_active(r);
                    page.manage.set_visible(true);
                    page.syncing.set(false);
                }
            }
            Err(e) => toast(app, &e.to_string()),
        },
    );
}

fn set_reservation(app: &Rc<App>, page: &Rc<Page>, reserve: bool) {
    let revert = glib::clone!(#[strong] page, move || {
        page.syncing.set(true);
        page.reserve_row.set_active(!reserve);
        page.syncing.set(false);
    });
    if app.busy.get() {
        revert();
        toast(app, "Aguarde a atualização terminar e tente de novo.");
        return;
    }
    let (ip, mac) = (page.client.ip.clone(), page.client.mac.clone());
    run(
        app,
        move |c| {
            let mut t = tmp(c)?;
            if reserve {
                t.add_reservation(&ip, &mac)?;
            } else {
                t.remove_reservation(&mac)?;
            }
            t.close()
        },
        move |app, res| match res {
            Ok(()) => toast(app, if reserve { "Endereço IP reservado" } else { "Reserva removida" }),
            Err(e) => {
                revert();
                toast(app, &e.to_string());
            }
        },
    );
}

fn confirm_block(app: &Rc<App>, page: &Rc<Page>) {
    let dialog = adw::AlertDialog::new(
        Some(&format!("Bloquear {}?", page.client.name)),
        Some("O dispositivo será desconectado de toda a rede e não conseguirá se reconectar até ser desbloqueado."),
    );
    dialog.add_response("cancel", "Cancelar");
    dialog.add_response("block", "Bloquear");
    dialog.set_response_appearance("block", adw::ResponseAppearance::Destructive);
    dialog.set_close_response("cancel");
    dialog.connect_response(None, glib::clone!(#[strong] app, #[strong] page, move |_, resp| {
        if resp != "block" {
            return;
        }
        if app.busy.get() {
            toast(&app, "Aguarde a atualização terminar e tente de novo.");
            return;
        }
        let (mac, name) = (page.client.mac.clone(), page.client.name.clone());
        run(
            &app,
            move |c| {
                let mut t = tmp(c)?;
                t.block_client(&mac, &name)?;
                t.close()
            },
            |app, res| match res {
                Ok(()) => {
                    app.nav.pop();
                    toast(app, "Dispositivo bloqueado");
                    refresh(app);
                }
                Err(e) => toast(app, &format!("Não foi possível bloquear: {e}")),
            },
        );
    }));
    dialog.present(Some(&app.window));
}
