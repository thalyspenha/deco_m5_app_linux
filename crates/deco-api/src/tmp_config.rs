//! Typed configuration over TMP. Payloads follow the beans of the official
//! app; see docs/PROTOCOL.md for which ones were validated on real hardware.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::{json, Value};

use crate::models::{decode_name, list, s};
use crate::tmp::{op, TmpSession};
use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct WanConfig {
    pub dial_type: String,
    pub ip: String,
    pub gateway: String,
    pub auto_dns: bool,
    pub dns1: String,
    pub dns2: String,
    pub mtu: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DhcpConfig {
    pub start_ip: String,
    pub end_ip: String,
    pub gateway: String,
    /// Empty means the Deco hands out its own address as DNS.
    pub dns1: String,
    pub dns2: String,
    pub in_use: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reservation {
    pub ip: String,
    pub mac: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortForward {
    pub id: String,
    pub name: String,
    pub internal_ip: String,
    pub internal_port: String,
    pub external_port: String,
    /// `all`, `tcp` or `udp`.
    pub protocol: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockedClient {
    pub mac: String,
    pub name: String,
}

impl TmpSession {
    pub fn wan(&mut self) -> Result<WanConfig> {
        let r = self.get(op::IPV4_GET)?;
        let w = &r["wan"];
        Ok(WanConfig {
            dial_type: s(w, "dial_type"),
            ip: s(&w["ip_info"], "ip"),
            gateway: s(&w["ip_info"], "gateway"),
            auto_dns: w["enable_auto_dns"].as_bool().unwrap_or(true),
            dns1: s(&w["ip_info"], "dns1"),
            dns2: s(&w["ip_info"], "dns2"),
            mtu: w["mtu_size"].as_u64(),
        })
    }

    pub fn dhcp(&mut self) -> Result<DhcpConfig> {
        let r = self.get(op::DHCP_GET)?;
        Ok(DhcpConfig {
            start_ip: s(&r, "start_ip"),
            end_ip: s(&r, "end_ip"),
            gateway: s(&r, "gateway"),
            dns1: s(&r, "dns1"),
            dns2: s(&r, "dns2"),
            in_use: r["ip_amount_in_use"].as_u64().unwrap_or_default(),
        })
    }

    pub fn reservations(&mut self) -> Result<Vec<Reservation>> {
        let r = self.get(op::IP_RESERVATION_LIST_GET)?;
        Ok(list(&r, "reservation_list").map(|v| Reservation { ip: s(v, "ip"), mac: s(v, "mac") }).collect())
    }

    /// Validated. `mac` as `AA-BB-CC-DD-EE-FF`.
    pub fn add_reservation(&mut self, ip: &str, mac: &str) -> Result<()> {
        self.set(op::IP_RESERVATION_LIST_ADD, json!({"reservation_list": [{"ip": ip, "mac": mac}]}))?;
        Ok(())
    }

    /// Validated. Reservations are keyed by MAC.
    pub fn remove_reservation(&mut self, mac: &str) -> Result<()> {
        self.set(op::IP_RESERVATION_LIST_REMOVE, json!({"reservation_list": [mac]}))?;
        Ok(())
    }

    pub fn port_forwards(&mut self) -> Result<Vec<PortForward>> {
        let r = self.get(op::PORT_FORWARDING_LIST_GET)?;
        Ok(list(&r, "port_forwarding_list")
            .map(|v| PortForward {
                id: s(v, "port_forwarding_id"),
                name: decode_name(&s(v, "service_name")),
                internal_ip: s(v, "internal_ip"),
                internal_port: s(v, "internal_port"),
                external_port: s(v, "external_port"),
                protocol: s(v, "protocol"),
            })
            .collect())
    }

    /// Not validated yet. Ports are numbers or ranges (`"80-443"`).
    pub fn add_port_forward(&mut self, pf: &PortForward) -> Result<()> {
        self.set(
            op::PORT_FORWARDING_ADD,
            json!({
                "service_name": B64.encode(&pf.name),
                "service_type": "custom",
                "internal_ip": pf.internal_ip,
                "internal_port": pf.internal_port,
                "external_port": pf.external_port,
                "protocol": pf.protocol,
            }),
        )?;
        Ok(())
    }

    /// Not validated yet.
    pub fn delete_port_forward(&mut self, id: &str) -> Result<()> {
        self.set(op::PORT_FORWARDING_DELETE, json!({"port_forwarding_id": id}))?;
        Ok(())
    }

    pub fn blocked_clients(&mut self) -> Result<Vec<BlockedClient>> {
        let r = self.get(op::BLOCKED_LIST_GET)?;
        Ok(list(&r, "client_list")
            .map(|v| BlockedClient { mac: s(v, "mac"), name: decode_name(&s(v, "name")) })
            .collect())
    }

    /// Not validated yet. Disconnects the client from the whole mesh.
    pub fn block_client(&mut self, mac: &str, name: &str) -> Result<()> {
        self.set(op::CLIENT_LIST_BLOCK, json!({"client_list": [{"mac": mac, "name": B64.encode(name)}]}))?;
        Ok(())
    }

    /// Not validated yet.
    pub fn unblock_client(&mut self, mac: &str) -> Result<()> {
        self.set(op::BLOCKED_LIST_UNBLOCK, json!({"client_list": [mac]}))?;
        Ok(())
    }

    /// Not validated yet, and risky: a bad payload can drop the internet
    /// connection. Rewrites the full WAN object read from the Deco, changing
    /// only the DNS fields, so PPPoE credentials stay untouched.
    pub fn set_wan_dns(&mut self, auto: bool, dns1: &str, dns2: &str) -> Result<()> {
        let mut wan: Value = self.get(op::IPV4_GET)?["wan"].take();
        wan["enable_auto_dns"] = json!(auto);
        if !auto {
            wan["ip_info"]["dns1"] = json!(dns1);
            wan["ip_info"]["dns2"] = json!(dns2);
        }
        self.set(op::IPV4_SET, json!({"wan": wan}))?;
        Ok(())
    }

    /// Not validated yet. Changing the range can renumber clients.
    pub fn set_dhcp(&mut self, cfg: &DhcpConfig) -> Result<()> {
        self.set(
            op::DHCP_SET,
            json!({
                "start_ip": cfg.start_ip,
                "end_ip": cfg.end_ip,
                "gateway": cfg.gateway,
                "dns1": cfg.dns1,
                "dns2": cfg.dns2,
            }),
        )?;
        Ok(())
    }
}
