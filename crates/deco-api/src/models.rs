use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::{json, Value};

use crate::{Result, Session};

#[derive(Debug, Clone)]
pub struct Node {
    pub mac: String,
    pub name: String,
    pub model: String,
    pub ip: String,
    pub is_master: bool,
    pub parent_mac: Option<String>,
    pub online: bool,
    pub firmware: String,
    pub hardware: String,
    /// Backhaul signal per band (0 = none/wired or master, higher is better).
    pub signal_2g: u8,
    pub signal_5g: u8,
}

impl Node {
    /// Best backhaul signal across bands, `None` for the master node.
    pub fn signal(&self) -> Option<u8> {
        (!self.is_master).then(|| self.signal_2g.max(self.signal_5g))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Main,
    Guest,
    Iot,
    Wired,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub mac: String,
    pub name: String,
    pub ip: String,
    pub network: Network,
    /// "2.4 GHz" / "5 GHz" / "" for wired or unknown.
    pub band: String,
    pub down_kbps: u64,
    pub up_kbps: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Wan {
    pub ip: String,
    pub gateway: String,
    pub dial_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiNetwork {
    Main,
    Guest,
}

#[derive(Debug, Clone, Default)]
pub struct WifiState {
    pub main_2g: bool,
    pub main_5g: bool,
    pub guest_2g: bool,
    pub guest_5g: bool,
}

impl Session {
    pub fn nodes(&self) -> Result<Vec<Node>> {
        let data = self.request("admin/device?form=device_list", &json!({"operation": "read"}))?;
        Ok(list(&data, "device_list")
            .map(|d| Node {
                mac: s(d, "mac"),
                name: s(d, "nickname"),
                model: s(d, "device_model"),
                ip: s(d, "device_ip"),
                is_master: d["role"] == "master",
                parent_mac: Some(s(d, "previous")).filter(|p| !p.is_empty()),
                online: d["inet_status"] == "online",
                firmware: s(d, "software_ver"),
                hardware: s(d, "hardware_ver"),
                signal_2g: level(&d["signal_level"]["band2_4"]),
                signal_5g: level(&d["signal_level"]["band5"]),
            })
            .collect())
    }

    /// Online clients across the whole mesh.
    pub fn clients(&self) -> Result<Vec<Client>> {
        self.clients_of("default")
    }

    /// Online clients attached to one Deco (`node_mac`), or `"default"` for all.
    pub fn clients_of(&self, node_mac: &str) -> Result<Vec<Client>> {
        let data = self.request(
            "admin/client?form=client_list",
            &json!({"operation": "read", "params": {"device_mac": node_mac}}),
        )?;
        Ok(list(&data, "client_list")
            .filter(|c| c["online"].as_bool().unwrap_or(false))
            .map(|c| {
                let wired = c["wire_type"] == "wired";
                let network = match (wired, c["interface"].as_str()) {
                    (true, _) => Network::Wired,
                    (_, Some("main")) => Network::Main,
                    (_, Some("guest")) => Network::Guest,
                    (_, Some("iot")) => Network::Iot,
                    _ => Network::Unknown,
                };
                let band = match c["connection_type"].as_str() {
                    Some("band2_4") => "2.4 GHz",
                    Some("band5") => "5 GHz",
                    Some("band6") => "6 GHz",
                    _ => "",
                };
                Client {
                    mac: s(c, "mac"),
                    name: decode_name(&s(c, "name")),
                    ip: s(c, "ip"),
                    network,
                    band: band.into(),
                    down_kbps: c["down_speed"].as_u64().unwrap_or(0),
                    up_kbps: c["up_speed"].as_u64().unwrap_or(0),
                }
            })
            .collect())
    }

    pub fn wan(&self) -> Result<Wan> {
        let data = self.request("admin/network?form=wan_ipv4", &json!({"operation": "read"}))?;
        let ip_info = &data["wan"]["ip_info"];
        Ok(Wan {
            ip: s(ip_info, "ip"),
            gateway: s(ip_info, "gateway"),
            dial_type: s(&data["wan"], "dial_type"),
        })
    }

    pub fn wifi(&self) -> Result<WifiState> {
        let d = self.request("admin/wireless?form=wlan", &json!({"operation": "read"}))?;
        let on = |band: &str, net: &str| d[band][net]["enable"].as_bool().unwrap_or(false);
        Ok(WifiState {
            main_2g: on("band2_4", "host"),
            main_5g: on("band5_1", "host"),
            guest_2g: on("band2_4", "guest"),
            guest_5g: on("band5_1", "guest"),
        })
    }

    /// Enables or disables a Wi-Fi network on both 2.4 and 5 GHz.
    pub fn set_wifi(&self, network: WifiNetwork, enable: bool) -> Result<()> {
        let net = match network {
            WifiNetwork::Main => "host",
            WifiNetwork::Guest => "guest",
        };
        let params = json!({
            "band2_4": {net: {"enable": enable}},
            "band5_1": {net: {"enable": enable}},
        });
        self.request("admin/wireless?form=wlan", &json!({"operation": "write", "params": params}))?;
        Ok(())
    }

    /// Reboots a single Deco.
    pub fn reboot_node(&self, mac: &str) -> Result<()> {
        self.request(
            "admin/device?form=system",
            &json!({"operation": "reboot", "params": {"mac_list": [{"mac": mac}]}}),
        )?;
        Ok(())
    }

    /// Reboots every Deco in the mesh.
    pub fn reboot_all(&self) -> Result<()> {
        let macs: Vec<Value> = self.nodes()?.into_iter().map(|n| json!({"mac": n.mac})).collect();
        self.request(
            "admin/device?form=system",
            &json!({"operation": "reboot", "params": {"mac_list": macs}}),
        )?;
        Ok(())
    }
}

fn level(v: &Value) -> u8 {
    v.as_u64().or_else(|| v.as_str()?.parse().ok()).unwrap_or(0) as u8
}

pub(crate) fn list<'a>(data: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    data[key].as_array().into_iter().flatten()
}

pub(crate) fn s(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().to_owned()
}

/// Client names come base64-encoded; fall back to the raw value.
pub(crate) fn decode_name(raw: &str) -> String {
    B64.decode(raw)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_else(|| raw.to_owned())
}
