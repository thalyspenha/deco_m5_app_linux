//! First TMP write test: reserves the IP a client already has, verifies it,
//! then removes the reservation. Changes the network briefly; run only with
//! the user's consent.
//!
//!     cargo run -p deco-api --example tmp_reservation_test [client-name]

use deco_api::tmp::{op, TmpSession};
use deco_api::Session;
use serde_json::json;

const IP_RESERVATION_LIST_ADD: u16 = 0x40C1;
const IP_RESERVATION_LIST_REMOVE: u16 = 0x40C3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let host = std::env::var("DECO_HOST").unwrap_or_else(|_| "192.168.68.1".into());
    let user = std::env::var("DECO_USERNAME")?;
    let pass = std::env::var("DECO_PASSWORD")?;
    let name = std::env::args().nth(1).unwrap_or_else(|| "MacBookAir".into());

    let client = {
        let http = Session::login(&host, &pass)?;
        let c = http.clients()?.into_iter().find(|c| c.name == name).ok_or("client not found")?;
        http.logout()?;
        c
    };
    println!("client {} ip={} mac={}", client.name, client.ip, client.mac);

    let mut s = TmpSession::connect(&host, &user, &pass)?;
    let list = |s: &mut TmpSession| s.call(op::IP_RESERVATION_LIST_GET, None).map(|v| v["result"]["reservation_list"].clone());

    println!("before: {}", list(&mut s)?);
    let wrap = |params: serde_json::Value| json!({"params": params, "config_version": now_ms()});
    let add = wrap(json!({"reservation_list": [{"ip": client.ip, "mac": client.mac}]}));
    println!("add: {:?}", s.call(IP_RESERVATION_LIST_ADD, Some(&add)));
    println!("after add: {}", list(&mut s)?);

    let remove = wrap(json!({"reservation_list": [client.mac]}));
    println!("remove: {:?}", s.call(IP_RESERVATION_LIST_REMOVE, Some(&remove)));
    println!("after remove: {}", list(&mut s)?);
    Ok(s.close()?)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
}
