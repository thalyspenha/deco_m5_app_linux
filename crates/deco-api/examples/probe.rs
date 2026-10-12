//! Read-only smoke test against a real Deco. Reads DECO_HOST / DECO_PASSWORD from .env.
//!
//!     cargo run -p deco-api --example probe [raw-path [params-json]]
//!
//! With a path (e.g. `admin/client?form=client_list`) it dumps the raw JSON
//! of a read operation, useful for exploring endpoints.

use deco_api::Session;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let host = std::env::var("DECO_HOST").unwrap_or_else(|_| "192.168.68.1".into());
    let password = std::env::var("DECO_PASSWORD")?;
    let session = Session::login(&host, &password)?;

    if let Some(path) = std::env::args().nth(1) {
        let mut body = json!({"operation": "read"});
        if let Some(params) = std::env::args().nth(2) {
            body["params"] = serde_json::from_str(&params)?;
        }
        let raw = session.request(&path, &body)?;
        println!("{}", serde_json::to_string_pretty(&raw)?);
        return Ok(session.logout()?);
    }

    println!("{:?}", session.wan()?);
    println!("{:?}", session.wifi()?);
    for n in session.nodes()? {
        println!("{n:?}");
    }
    for c in session.clients()? {
        println!("{c:?}");
    }
    Ok(session.logout()?)
}
