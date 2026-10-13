//! TMP (TP-Link Mesh Protocol), used by the official app for configuration
//! the HTTP API does not expose (WAN/DNS, LAN, DHCP, reservations, ...).
//!
//! Stack, reverse engineered from the Android app (see docs/PROTOCOL.md):
//! SSH to port 20001 (TP-Link ID email + password) → direct-tcpip channel to
//! 127.0.0.1:20002 → TMP framing (16-byte header, CRC32) → app layer v2
//! (20-byte header, opcode, 8156-byte fragments) → JSON.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::Value;
use sha1::{Digest, Sha1};

use crate::{Error, Result};

const SSH_PORT: u16 = 20001;
const TMP_PORT: u16 = 20002;

const TMP_VERSION: u8 = 1;
const TMP_SUBVERSION: u8 = 1;
const TMP_ASSOC_REQ: u8 = 1;
const TMP_ASSOC_ACCEPT: u8 = 2;
const TMP_HELLO: u8 = 4;
const TMP_DATA: u8 = 5;
const TMP_CLOSE: u8 = 6;
const TMP_CRC_SEED: u32 = 0x5A6B_7C8D;

const APP_VERSION: u8 = 1;
const APP_LAYER: u8 = 2;
const APP_PUSH: u8 = 2;
const APP_PUSH_ACK: u8 = 3;
const APP_PULL: u8 = 4;
const APP_PULL_ACK: u8 = 5;
const APP_FRAGMENT: usize = 8156;

const OP_TOKEN_ALLOC: u16 = 1;
const OP_TOKEN_FREE: u16 = 3;

pub mod op {
    pub const IPV4_GET: u16 = 0x4004;
    pub const LAN_IP_GET: u16 = 0x4211;
    pub const DHCP_GET: u16 = 0x4213;
    pub const IP_RESERVATION_LIST_GET: u16 = 0x40C0;
    pub const PORT_FORWARDING_LIST_GET: u16 = 0x40B0;
    pub const BLOCKED_LIST_GET: u16 = 0x4018;
    pub const WIRELESS_GET: u16 = 0x4009;
    pub const IPV4_SET: u16 = 0x4005;
    pub const DHCP_SET: u16 = 0x4214;
    pub const IP_RESERVATION_LIST_ADD: u16 = 0x40C1;
    pub const IP_RESERVATION_LIST_REMOVE: u16 = 0x40C3;
    pub const PORT_FORWARDING_ADD: u16 = 0x40B1;
    pub const PORT_FORWARDING_DELETE: u16 = 0x40B3;
    pub const CLIENT_LIST_BLOCK: u16 = 0x4017;
    pub const BLOCKED_LIST_UNBLOCK: u16 = 0x4019;
    pub const WIRELESS_SET: u16 = 0x4008;
}

pub struct TmpSession {
    _ssh: ssh2::Session,
    chan: ssh2::Channel,
    seq: u32,
    tx: u16,
    closed: bool,
}

impl TmpSession {
    /// Opens SSH + TMP association and allocates an app-layer token.
    /// `email` is the TP-Link ID that owns the network; the SSH user is its
    /// SHA-1 hex digest, the password is the account password.
    pub fn connect(host: &str, email: &str, password: &str) -> Result<Self> {
        let username = hex::encode(Sha1::digest(email.trim().as_bytes()));
        let tcp = TcpStream::connect((host, SSH_PORT))?;
        tcp.set_read_timeout(Some(Duration::from_secs(15)))?;
        let mut ssh = ssh2::Session::new()?;
        ssh.set_tcp_stream(tcp);
        ssh.set_timeout(15_000);
        ssh.handshake()?;
        ssh.userauth_password(&username, password)?;
        if !ssh.authenticated() {
            return Err(Error::Login);
        }
        let chan = ssh.channel_direct_tcpip("127.0.0.1", TMP_PORT, None)?;
        let mut s = TmpSession { _ssh: ssh, chan, seq: 1, tx: 0, closed: false };

        // Three-way association: request, device accepts, client confirms.
        s.write_short(TMP_ASSOC_REQ)?;
        let (kind, _) = s.read_tmp()?;
        if kind != TMP_ASSOC_ACCEPT {
            return Err(Error::Protocol("TMP association refused"));
        }
        s.write_short(TMP_ASSOC_ACCEPT)?;
        s.call_raw(OP_TOKEN_ALLOC, &[])?;
        Ok(s)
    }

    /// Sends `opcode` with an optional JSON body and returns the decoded
    /// response (`{"error_code": .., "result": ..}`).
    pub fn call(&mut self, opcode: u16, params: Option<&Value>) -> Result<Value> {
        let body = params.map(|p| p.to_string().into_bytes()).unwrap_or_default();
        let resp = self.call_raw(opcode, &body)?;
        if resp.is_empty() {
            return Ok(Value::Null);
        }
        let v: Value = serde_json::from_slice(&resp)?;
        match v["error_code"].as_i64() {
            Some(0) | None => Ok(v),
            Some(code) => Err(Error::Router { code, path: format!("tmp:{opcode:#06x}") }),
        }
    }

    /// Sends a SET opcode. Writes need the `params` + `config_version`
    /// envelope; without it the Deco answers `error_code: 1`.
    pub fn set(&mut self, opcode: u16, params: Value) -> Result<Value> {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or_default();
        self.call(opcode, Some(&serde_json::json!({"params": params, "config_version": ms})))
    }

    /// GET returning the `result` block.
    pub fn get(&mut self, opcode: u16) -> Result<Value> {
        Ok(self.call(opcode, None)?["result"].take())
    }

    pub fn close(mut self) -> Result<()> {
        self.do_close()
    }

    fn do_close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        let _ = self.call_raw(OP_TOKEN_FREE, &[]);
        let _ = self.write_tmp(TMP_CLOSE, &[]);
        let _ = self.chan.close();
        Ok(())
    }

    fn call_raw(&mut self, opcode: u16, body: &[u8]) -> Result<Vec<u8>> {
        self.tx = self.tx.wrapping_add(1);
        let tx = self.tx;
        let crc = crc32fast::hash(body);
        let total = body.len() as u32;

        let chunks: Vec<&[u8]> = if body.is_empty() { vec![&[]] } else { body.chunks(APP_FRAGMENT).collect() };
        let mut ack = None;
        for (i, chunk) in chunks.iter().enumerate() {
            let offset = (i * APP_FRAGMENT) as u32;
            self.write_app(opcode, APP_PUSH, tx, crc, total, offset, chunk)?;
            ack = Some(self.read_app(tx)?);
        }
        let (hdr, data) = ack.ok_or(Error::Protocol("no app ack"))?;
        if hdr.kind == APP_PULL_ACK {
            return finish(&hdr, data);
        }
        if hdr.kind != APP_PUSH_ACK {
            return Err(Error::Protocol("unexpected app packet"));
        }

        // Response is pulled, possibly in several fragments.
        self.write_app(opcode, APP_PULL, tx, 0, 0, 0, &[])?;
        let (first, mut out) = self.read_app(tx)?;
        if first.kind != APP_PULL_ACK {
            return Err(Error::Protocol("expected pull ack"));
        }
        let total = first.total as usize;
        while out.len() < total {
            self.write_app(opcode, APP_PULL, tx, 0, 0, out.len() as u32, &[])?;
            let (_, more) = self.read_app(tx)?;
            if more.is_empty() {
                return Err(Error::Protocol("short TMP response"));
            }
            out.extend(more);
        }
        finish(&first, out)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_app(&mut self, opcode: u16, kind: u8, tx: u16, crc: u32, total: u32, offset: u32, data: &[u8]) -> Result<()> {
        let mut p = Vec::with_capacity(20 + data.len());
        p.extend([APP_VERSION, APP_LAYER]);
        p.extend(opcode.to_be_bytes());
        p.extend([kind, 0]);
        p.extend(tx.to_be_bytes());
        p.extend(crc.to_be_bytes());
        p.extend(total.to_be_bytes());
        p.extend(offset.to_be_bytes());
        p.extend(data);
        self.write_tmp(TMP_DATA, &p)
    }

    fn read_app(&mut self, tx: u16) -> Result<(AppHeader, Vec<u8>)> {
        loop {
            let (kind, payload) = self.read_tmp()?;
            if kind != TMP_DATA || payload.len() < 20 {
                continue;
            }
            let hdr = AppHeader {
                kind: payload[4],
                status: payload[5],
                tx: u16::from_be_bytes([payload[6], payload[7]]),
                crc: u32::from_be_bytes(payload[8..12].try_into().unwrap()),
                total: u32::from_be_bytes(payload[12..16].try_into().unwrap()),
            };
            if hdr.tx != tx {
                continue;
            }
            if hdr.status != 0 {
                return Err(Error::Tmp(hdr.status));
            }
            return Ok((hdr, payload[20..].to_vec()));
        }
    }

    /// Header-only control packet (association).
    fn write_short(&mut self, kind: u8) -> Result<()> {
        let p = [TMP_VERSION, TMP_SUBVERSION, kind, 0];
        trace("send", &p);
        self.chan.write_all(&p)?;
        Ok(self.chan.flush()?)
    }

    fn write_tmp(&mut self, kind: u8, payload: &[u8]) -> Result<()> {
        let mut p = Vec::with_capacity(16 + payload.len());
        p.extend([TMP_VERSION, TMP_SUBVERSION, kind, 0]);
        p.extend((payload.len() as u16).to_be_bytes());
        p.extend([0, 0]);
        p.extend(self.seq.to_be_bytes());
        p.extend(TMP_CRC_SEED.to_be_bytes());
        p.extend(payload);
        let crc = crc32fast::hash(&p);
        p[12..16].copy_from_slice(&crc.to_be_bytes());
        self.seq = self.seq.wrapping_add(1);
        trace("send", &p);
        self.chan.write_all(&p)?;
        Ok(self.chan.flush()?)
    }

    /// Reads one TMP packet; answers keep-alives transparently.
    fn read_tmp(&mut self) -> Result<(u8, Vec<u8>)> {
        loop {
            let mut head = [0u8; 4];
            self.chan.read_exact(&mut head)?;
            trace("recv", &head);
            let kind = head[2];
            if !(TMP_HELLO..=TMP_CLOSE).contains(&kind) {
                return Ok((kind, Vec::new()));
            }
            let mut rest = [0u8; 12];
            self.chan.read_exact(&mut rest)?;
            let len = u16::from_be_bytes([rest[0], rest[1]]) as usize;
            if rest[3] != 0 {
                return Err(Error::Tmp(rest[3]));
            }
            let mut payload = vec![0u8; len];
            self.chan.read_exact(&mut payload)?;
            trace("recv", &[&rest[..], &payload[..]].concat());
            match kind {
                TMP_HELLO => continue,
                TMP_CLOSE => return Err(Error::Protocol("TMP closed by device")),
                _ => return Ok((kind, payload)),
            }
        }
    }
}

impl Drop for TmpSession {
    fn drop(&mut self) {
        let _ = self.do_close();
    }
}

struct AppHeader {
    kind: u8,
    status: u8,
    tx: u16,
    crc: u32,
    total: u32,
}

fn finish(hdr: &AppHeader, data: Vec<u8>) -> Result<Vec<u8>> {
    if !data.is_empty() && crc32fast::hash(&data) != hdr.crc {
        return Err(Error::Protocol("TMP response CRC mismatch"));
    }
    Ok(data)
}

fn trace(dir: &str, bytes: &[u8]) {
    if std::env::var_os("TMP_DEBUG").is_some() {
        eprintln!("{dir} {}", hex::encode(bytes));
    }
}
