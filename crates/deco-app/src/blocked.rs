//! Blocked devices page (TMP): lists blocked clients and unblocks them.

use std::rc::Rc;

use adw::prelude::*;
use deco_api::tmp::TmpSession;
use deco_api::BlockedClient;
use gtk::glib;

use crate::credentials::Credentials;
use crate::window::{refresh, run, toast, App};

fn tmp(c: &Credentials) -> deco_api::Result<TmpSession> {
    let email = c.email.as_deref().ok_or(deco_api::Error::Protocol("e-mail do TP-Link ID não informado"))?;
    TmpSession::connect(&c.host, email, &c.password)
}

struct Page {
    stack: gtk::Stack,
    group: adw::PreferencesGroup,
    rows: std::cell::RefCell<Vec<adw::ActionRow>>,
}

pub fn open(app: &Rc<App>) {
    let group = adw::PreferencesGroup::new();
    let prefs = adw::PreferencesPage::new();
    prefs.add(&group);

    let empty = adw::StatusPage::builder()
        .icon_name("security-high-symbolic")
        .title("Nenhum dispositivo bloqueado")
        .build();
    let stack = gtk::Stack::builder().transition_type(gtk::StackTransitionType::Crossfade).build();
    stack.add_named(&adw::StatusPage::builder().title("Carregando…").build(), Some("loading"));
    stack.add_named(&empty, Some("empty"));
    stack.add_named(&prefs, Some("list"));

    let view = adw::ToolbarView::new();
    view.add_top_bar(&adw::HeaderBar::new());
    view.set_content(Some(&stack));
    app.nav.push(&adw::NavigationPage::builder().title("Dispositivos bloqueados").child(&view).build());

    let page = Rc::new(Page { stack, group, rows: Default::default() });
    load(app, &page);
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
    let page = page.clone();
    run(
        app,
        |c| {
            let mut t = tmp(c)?;
            let list = t.blocked_clients()?;
            t.close()?;
            Ok(list)
        },
        move |app, res| match res {
            Ok(list) => show(app, &page, &list),
            Err(e) => {
                toast(app, &e.to_string());
                app.nav.pop();
            }
        },
    );
}

fn show(app: &Rc<App>, page: &Rc<Page>, list: &[BlockedClient]) {
    for row in page.rows.take() {
        page.group.remove(&row);
    }
    for c in list {
        let row = adw::ActionRow::builder()
            .title(glib::markup_escape_text(&c.name))
            .subtitle(c.mac.as_str())
            .build();
        let btn = gtk::Button::builder().label("Desbloquear").valign(gtk::Align::Center).build();
        btn.connect_clicked(glib::clone!(#[strong] app, #[strong] page, #[strong(rename_to = mac)] c.mac, move |_| {
            unblock(&app, &page, &mac)
        }));
        row.add_suffix(&btn);
        page.group.add(&row);
        page.rows.borrow_mut().push(row);
    }
    page.stack.set_visible_child_name(if list.is_empty() { "empty" } else { "list" });
}

fn unblock(app: &Rc<App>, page: &Rc<Page>, mac: &str) {
    if app.busy.get() {
        toast(app, "Aguarde a atualização terminar e tente de novo.");
        return;
    }
    let mac = mac.to_owned();
    let page = page.clone();
    run(
        app,
        move |c| {
            let mut t = tmp(c)?;
            t.unblock_client(&mac)?;
            let list = t.blocked_clients()?;
            t.close()?;
            Ok(list)
        },
        move |app, res| match res {
            Ok(list) => {
                show(app, &page, &list);
                toast(app, "Dispositivo desbloqueado");
                refresh(app);
            }
            Err(e) => toast(app, &format!("Não foi possível desbloquear: {e}")),
        },
    );
}
