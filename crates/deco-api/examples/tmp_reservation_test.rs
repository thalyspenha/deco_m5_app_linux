//! First TMP write test: reserves the IP a client already has, verifies it,
//! then removes the reservation. Changes the network briefly; run only with
//! the user's consent.
//!
//!     cargo run -p deco-api --example tmp_reservation_test [client-name]

use deco_api::tmp::TmpSession;
use deco_api::Session;


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
    println!("before: {:?}", s.reservations()?);
    s.add_reservation(&client.ip, &client.mac)?;
    println!("after add: {:?}", s.reservations()?);
    s.remove_reservation(&client.mac)?;
    println!("after remove: {:?}", s.reservations()?);
    Ok(s.close()?)
}

