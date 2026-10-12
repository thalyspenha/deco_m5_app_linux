//! Reboots a single Deco by name. Takes its clients offline for a few
//! minutes; run only with the user's consent.
//!
//!     cargo run -p deco-api --example reboot_node_test "<deco-name>"

use deco_api::Session;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let host = std::env::var("DECO_HOST").unwrap_or_else(|_| "192.168.68.1".into());
    let pass = std::env::var("DECO_PASSWORD")?;
    let name = std::env::args().nth(1).ok_or("usage: reboot_node_test <deco-name>")?;

    let s = Session::login(&host, &pass)?;
    let node = s.nodes()?.into_iter().find(|n| n.name == name).ok_or("deco not found")?;
    if node.is_master {
        return Err("refusing to reboot the main Deco from this test".into());
    }
    println!("rebooting {} ({}, {})", node.name, node.ip, node.mac);
    s.reboot_node(&node.mac)?;
    println!("reboot accepted");
    Ok(s.logout()?)
}
