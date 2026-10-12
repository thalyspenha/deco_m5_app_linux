//! Client for the TP-Link Deco local HTTP API (LuCI, RSA + AES).
//!
//! The Deco accepts a single admin session at a time: logging in kicks the
//! official app and vice versa. Keep sessions short — [`Session`] logs out on
//! drop, so open one per operation.

mod crypto;
mod models;
pub mod tmp;

use std::time::Duration;

use md5::{Digest, Md5};
use reqwest::blocking::Client as Http;
use serde_json::{json, Value};

use crypto::{AesSession, RsaKey};
pub use models::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("network error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unexpected response: {0}")]
    Protocol(&'static str),
    #[error("login failed (wrong password or another admin session active?)")]
    Login,
    #[error("router returned error {code} for {path}")]
    Router { code: i64, path: String },
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SSH error: {0}")]
    Ssh(#[from] ssh2::Error),
    #[error("TMP error status {0}")]
    Tmp(u8),
}

pub type Result<T> = std::result::Result<T, Error>;

const USERNAME: &str = "admin";

pub struct Session {
    http: Http,
    base: String,
    stok: String,
    sysauth: String,
    seq: u64,
    sign_key: RsaKey,
    aes: AesSession,
    hash: String,
    logged_out: bool,
}

impl Session {
    /// Logs in to the Deco at `host` (IP or hostname, no scheme).
    pub fn login(host: &str, password: &str) -> Result<Self> {
        let http = Http::builder()
            .timeout(Duration::from_secs(15))
            .build()?;
        let base = format!("http://{host}/cgi-bin/luci/;stok=");

        let keys = read_plain(&http, &format!("{base}/login?form=keys"))?;
        let pwd_key = rsa_from(&keys["result"]["password"])?;

        let auth = read_plain(&http, &format!("{base}/login?form=auth"))?;
        let sign_key = rsa_from(&auth["result"]["key"])?;
        let seq = as_u64(&auth["result"]["seq"]).ok_or(Error::Protocol("missing seq"))?;

        let hash = hex::encode(Md5::digest(format!("{USERNAME}{password}")));
        let mut session = Session {
            http,
            base,
            stok: String::new(),
            sysauth: String::new(),
            seq,
            sign_key,
            aes: AesSession::random(),
            hash,
            logged_out: false,
        };

        let crypted = pwd_key.encrypt_hex(password.as_bytes())?;
        let body = json!({"params": {"password": crypted}, "operation": "login"});
        let url = format!("{}/login?form=login", session.base);
        let resp = session.post(&url, &body, true)?;

        let sysauth = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find_map(|c| c.strip_prefix("sysauth="))
            .and_then(|c| c.split(';').next())
            .map(str::to_owned)
            .ok_or(Error::Login)?;
        let data = session.decrypt_body(resp)?;
        if data["error_code"].as_i64() != Some(0) {
            return Err(Error::Login);
        }
        session.stok = data["result"]["stok"].as_str().ok_or(Error::Login)?.to_owned();
        session.sysauth = sysauth;
        Ok(session)
    }

    /// Sends an encrypted request and returns its `result` block.
    pub fn request(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}/{}", self.base, self.stok, path);
        let resp = self.post(&url, body, false)?;
        let mut data = self.decrypt_body(resp)?;
        match data["error_code"].as_i64() {
            Some(0) => Ok(data["result"].take()),
            code => Err(Error::Router { code: code.unwrap_or(-1), path: path.to_owned() }),
        }
    }

    pub fn logout(mut self) -> Result<()> {
        self.do_logout()
    }

    fn do_logout(&mut self) -> Result<()> {
        if self.logged_out {
            return Ok(());
        }
        self.logged_out = true;
        let url = format!("{}{}/admin/system?form=logout", self.base, self.stok);
        self.post(&url, &json!({"operation": "logout"}), false)?;
        Ok(())
    }

    fn post(&self, url: &str, body: &Value, login: bool) -> Result<reqwest::blocking::Response> {
        let data = self.aes.encrypt(&body.to_string());
        let mut to_sign = format!("h={}&s={}", self.hash, self.seq + data.len() as u64);
        if login {
            to_sign = format!("{}&{}", self.aes.key_string(), to_sign);
        }
        let sign = self.sign_key.encrypt_chunked_hex(&to_sign)?;
        let form = form_urlencoded::Serializer::new(String::new())
            .append_pair("sign", &sign)
            .append_pair("data", &data)
            .finish();

        let mut req = self.http.post(url).header("Content-Type", "application/json").body(form);
        if !self.sysauth.is_empty() {
            req = req.header("Cookie", format!("sysauth={}", self.sysauth));
        }
        Ok(req.send()?)
    }

    fn decrypt_body(&self, resp: reqwest::blocking::Response) -> Result<Value> {
        let outer: Value = resp.json()?;
        let enc = outer["data"].as_str().ok_or(Error::Protocol("missing data field"))?;
        Ok(serde_json::from_str(&self.aes.decrypt(enc)?)?)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.do_logout();
    }
}

fn read_plain(http: &Http, url: &str) -> Result<Value> {
    Ok(http.post(url).query(&[("operation", "read")]).send()?.json()?)
}

fn rsa_from(pair: &Value) -> Result<RsaKey> {
    match (pair[0].as_str(), pair[1].as_str()) {
        (Some(n), Some(e)) => RsaKey::from_hex(n, e),
        _ => Err(Error::Protocol("missing RSA key")),
    }
}

fn as_u64(v: &Value) -> Option<u64> {
    v.as_u64().or_else(|| v.as_str()?.parse().ok())
}
