//! Router credentials: env vars (dev) first, then the system keyring
//! (Secret Service on Linux, Keychain on macOS).

const SERVICE: &str = "deco-linux";
const DEFAULT_HOST: &str = "192.168.68.1";

#[derive(Clone)]
pub struct Credentials {
    pub host: String,
    pub password: String,
}

fn entry(key: &str) -> Option<keyring::Entry> {
    keyring::Entry::new(SERVICE, key).ok()
}

pub fn load() -> Option<Credentials> {
    if let Ok(password) = std::env::var("DECO_PASSWORD") {
        let host = std::env::var("DECO_HOST").unwrap_or_else(|_| DEFAULT_HOST.into());
        return Some(Credentials { host, password });
    }
    let host = entry("host")?.get_password().ok()?;
    let password = entry("password")?.get_password().ok()?;
    Some(Credentials { host, password })
}

pub fn save(c: &Credentials) {
    if let Some(e) = entry("host") {
        let _ = e.set_password(&c.host);
    }
    if let Some(e) = entry("password") {
        let _ = e.set_password(&c.password);
    }
}

pub fn clear() {
    for key in ["host", "password"] {
        if let Some(e) = entry(key) {
            let _ = e.delete_credential();
        }
    }
}

pub fn default_host() -> &'static str {
    DEFAULT_HOST
}
