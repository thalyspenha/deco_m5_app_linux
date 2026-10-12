//! Read-only TMP smoke test. Needs DECO_USERNAME (TP-Link ID email) and
//! DECO_PASSWORD in .env.
//!
//!     cargo run -p deco-api --example tmp_probe [get-opcode-hex | typed]

use deco_api::tmp::{op, TmpSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let host = std::env::var("DECO_HOST").unwrap_or_else(|_| "192.168.68.1".into());
    let user = std::env::var("DECO_USERNAME")?;
    let pass = std::env::var("DECO_PASSWORD")?;
    let mut s = TmpSession::connect(&host, &user, &pass)?;

    if std::env::args().nth(1).as_deref() == Some("typed") {
        println!("{:?}", s.wan()?);
        println!("{:?}", s.dhcp()?);
        println!("{:?}", s.reservations()?);
        println!("{:?}", s.port_forwards()?);
        println!("{:?}", s.blocked_clients()?);
        return Ok(s.close()?);
    }

    // Only *_GET opcodes: this example must never change the network.
    let gets = [
        op::IPV4_GET,
        op::LAN_IP_GET,
        op::DHCP_GET,
        op::IP_RESERVATION_LIST_GET,
        op::PORT_FORWARDING_LIST_GET,
        op::BLOCKED_LIST_GET,
    ];
    let ops: Vec<u16> = match std::env::args().nth(1) {
        Some(a) => {
            let code = u16::from_str_radix(a.trim_start_matches("0x"), 16)?;
            if !gets.contains(&code) {
                return Err(format!("{code:#06x} is not a known GET opcode").into());
            }
            vec![code]
        }
        None => gets[..4].to_vec(),
    };
    for code in ops {
        match s.call(code, None) {
            Ok(v) => println!("{code:#06x}: {}", serde_json::to_string_pretty(&v)?),
            Err(e) => println!("{code:#06x}: error {e}"),
        }
    }
    Ok(s.close()?)
}
