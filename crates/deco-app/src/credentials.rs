//! Router credentials: env vars (dev) first, then the system keyring
//! (Secret Service on Linux, Keychain on macOS).

const SERVICE: &str = "deco-linux";
const DEFAULT_HOST: &str = "192.168.68.1";

#[derive(Clone)]
pub struct Credentials {
    pub host: String,
    pub password: String,
    /// TP-Link ID email, needed for TMP (advanced settings). Optional.
    pub email: Option<String>,
}

impl Credentials {
    /// Opens a TMP session; needs the TP-Link ID email.
    pub fn tmp(&self) -> deco_api::Result<deco_api::tmp::TmpSession> {
        let email = self.email.as_deref().ok_or(deco_api::Error::Protocol("e-mail do TP-Link ID não informado"))?;
        deco_api::tmp::TmpSession::connect(&self.host, email, &self.password)
    }
}

fn entry(key: &str) -> Option<keyring::Entry> {
    keyring::Entry::new(SERVICE, key).ok()
}

pub fn load() -> Option<Credentials> {
    if let Ok(password) = std::env::var("DECO_PASSWORD") {
        let host = std::env::var("DECO_HOST").unwrap_or_else(|_| DEFAULT_HOST.into());
        let email = std::env::var("DECO_USERNAME").ok().filter(|e| !e.is_empty());
        return Some(Credentials { host, password, email });
    }
    let host = entry("host")?.get_password().ok()?;
    let password = entry("password")?.get_password().ok()?;
    let email = entry("email").and_then(|e| e.get_password().ok());
    Some(Credentials { host, password, email })
}

pub fn save(c: &Credentials) {
    if let Some(e) = entry("host") {
        let _ = e.set_password(&c.host);
    }
    if let Some(e) = entry("password") {
        let _ = e.set_password(&c.password);
    }
    if let Some(e) = entry("email") {
        let _ = match &c.email {
            Some(email) => e.set_password(email),
            None => e.delete_credential(),
        };
    }
}

pub fn clear() {
    for key in ["host", "password", "email"] {
        if let Some(e) = entry(key) {
            let _ = e.delete_credential();
        }
    }
}

pub fn default_host() -> &'static str {
    DEFAULT_HOST
}
