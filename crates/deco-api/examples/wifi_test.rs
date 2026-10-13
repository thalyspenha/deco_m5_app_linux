//! Validates TmpSession::set_wifi (TMP 0x4008): flips one Wi-Fi network, checks it, then
//! restores the original state. Changes the network; run only with the
//! user's consent.
//!
//!     cargo run -p deco-api --example wifi_test [guest|main]
//!
//! `guest` (default) affects nobody while it is off. `main` drops every
//! Wi-Fi client for a few seconds; run it only from a wired machine.

use std::{thread, time::Duration};

use deco_api::{tmp::TmpSession, WifiNetwork, WifiState};

fn enabled(w: &WifiState, net: WifiNetwork) -> (bool, bool) {
    match net {
        WifiNetwork::Main => (w.main_2g, w.main_5g),
        WifiNetwork::Guest => (w.guest_2g, w.guest_5g),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let host = std::env::var("DECO_HOST").unwrap_or_else(|_| "192.168.68.1".into());
    let user = std::env::var("DECO_USERNAME")?;
    let pass = std::env::var("DECO_PASSWORD")?;
    let net = match std::env::args().nth(1).as_deref() {
        None | Some("guest") => WifiNetwork::Guest,
        Some("main") => WifiNetwork::Main,
        Some(other) => return Err(format!("unknown network {other}").into()),
    };

    let mut s = TmpSession::connect(&host, &user, &pass)?;
    let (orig_2g, orig_5g) = enabled(&s.wifi()?, net);
    println!("before: 2.4={orig_2g} 5={orig_5g}");
    let target = !(orig_2g || orig_5g);
    println!("set {target}: {:?}", s.set_wifi(net, target));
    s.close()?;

    thread::sleep(Duration::from_secs(10));
    let mut s = TmpSession::connect(&host, &user, &pass)?;
    println!("10s later: {:?}", enabled(&s.wifi()?, net));

    // Restore; set_wifi sets both bands, so mixed states come back as "on".
    s.set_wifi(net, orig_2g || orig_5g)?;
    s.close()?;

    thread::sleep(Duration::from_secs(10));
    let mut s = TmpSession::connect(&host, &user, &pass)?;
    println!("restored: {:?}", enabled(&s.wifi()?, net));
    Ok(s.close()?)
}
