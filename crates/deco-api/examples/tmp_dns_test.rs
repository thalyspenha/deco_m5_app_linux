//! Validates TmpSession::set_wan_dns: switches WAN DNS to Google, checks it,
//! then restores the original values. Changes the network; run only with
//! the user's consent.
//!
//!     cargo run -p deco-api --example tmp_dns_test

use std::{thread, time::Duration};

use deco_api::tmp::TmpSession;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let host = std::env::var("DECO_HOST").unwrap_or_else(|_| "192.168.68.1".into());
    let user = std::env::var("DECO_USERNAME")?;
    let pass = std::env::var("DECO_PASSWORD")?;

    let mut s = TmpSession::connect(&host, &user, &pass)?;
    let orig = s.wan()?;
    println!("before: auto={} {} {}", orig.auto_dns, orig.dns1, orig.dns2);

    let res = s.set_wan_dns(false, "8.8.8.8", "8.8.4.4");
    println!("set google: {res:?}");
    let now = s.wan()?;
    println!("after set: auto={} {} {} ip={}", now.auto_dns, now.dns1, now.dns2, now.ip);
    s.close()?;

    thread::sleep(Duration::from_secs(5));
    let mut s = TmpSession::connect(&host, &user, &pass)?;
    let mid = s.wan()?;
    println!("5s later: auto={} {} {} ip={}", mid.auto_dns, mid.dns1, mid.dns2, mid.ip);

    s.set_wan_dns(orig.auto_dns, &orig.dns1, &orig.dns2)?;
    let back = s.wan()?;
    println!("restored: auto={} {} {} ip={}", back.auto_dns, back.dns1, back.dns2, back.ip);
    Ok(s.close()?)
}
