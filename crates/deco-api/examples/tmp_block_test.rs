//! Validates block/unblock over TMP: blocks a client, checks the blocked
//! list, then unblocks it. Disconnects the device briefly; run only with the
//! user's consent.
//!
//!     cargo run -p deco-api --example tmp_block_test <client-name>

use deco_api::tmp::TmpSession;
use deco_api::Session;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let host = std::env::var("DECO_HOST").unwrap_or_else(|_| "192.168.68.1".into());
    let user = std::env::var("DECO_USERNAME")?;
    let pass = std::env::var("DECO_PASSWORD")?;
    let name = std::env::args().nth(1).ok_or("usage: tmp_block_test <client-name>")?;

    let client = {
        let http = Session::login(&host, &pass)?;
        let c = http.clients()?.into_iter().find(|c| c.name == name).ok_or("client not found")?;
        http.logout()?;
        c
    };
    println!("client {} mac={}", client.name, client.mac);

    let mut s = TmpSession::connect(&host, &user, &pass)?;
    println!("blocked before: {:?}", s.blocked_clients()?);
    let res = s.block_client(&client.mac, &client.name);
    println!("block: {res:?}");
    println!("blocked after: {:?}", s.blocked_clients()?);

    // Always try to unblock, even if the listing above failed.
    let res = s.unblock_client(&client.mac);
    println!("unblock: {res:?}");
    println!("blocked final: {:?}", s.blocked_clients()?);
    Ok(s.close()?)
}
