# Protocolo do Deco

## API HTTP local (usada hoje)

- Endpoint: `http://<deco>/cgi-bin/luci/;stok=/...`, login com a senha do admin local.
- **Uma sessão admin por vez**: logar derruba o app oficial e vice-versa. Sempre fazer logout logo após a operação (`Session` faz logout no drop).
- Implementação em Rust: `crates/deco-api`. Referência original: `tplinkrouterc6u.TPLinkDecoClient`.
- Cobre: firmware, nós mesh (topologia, papel, status de internet), clientes (IP, MAC, banda, velocidade), status WAN, liga/desliga Wi-Fi host/guest/IoT, reboot.

### Endpoints explorados (leitura, 2026-10-12)

| Form | Conteúdo |
|---|---|
| `admin/network?form=wan_ipv4` | `dial_type`, `enable_auto_dns`, `ip_info` (ip, gateway, dns1/2, mac), usuário PPPoE; LAN ip/mask |
| `admin/network?form=lan_ipv4` | IP/MAC/máscara da LAN |
| `admin/network?form=internet` | status IPv4/IPv6, `link_status` |
| `admin/network?form=mac_clone` | `enable` |
| `admin/network?form=performance` | `cpu_usage`, `mem_usage` |
| `admin/wireless?form=wlan` | Wi-Fi por banda (canal, SSID, host/guest) |

Sem sucesso (`code 1`, `-1` ou resposta não-JSON): `wan_ipv6`, `ipv4`, `lan`, `dhcp`, `dhcp_server`,
`reservation`, `address_reservation`, `port_forwarding`, `iptv`, `fast_roaming`, `ipv6`,
`admin/dhcps?form=setting|reservation`, `admin/dhcp?form=reservation`, `admin/nat?form=vs|port_forwarding`,
`admin/system?form=sysmode|ledctrl`.

**Escrita de DNS:** `write` parcial em `wan_ipv4` (`{"wan":{"enable_auto_dns":false,"ip_info":{"dns1","dns2"}}}`)
retorna `code 1`, nada muda. Falta descobrir o payload aceito (talvez exige objeto completo com
credenciais PPPoE, ou é feito via TMP). Próximo passo: payload do app oficial via jadx no APK.

## TMP (TP-Link Mesh Protocol)

- SSH na porta 20001, usado pelo app oficial para parte das configurações.
- Sem documentação pública. Não aparece em mitmproxy (não é HTTP).
- Caminho realista: decompilar o APK Android com jadx.

### Achados no APK (com.tplink.tpm5 3.10.645, jadx)

- O app **não usa** os forms HTTP para WAN/LAN/DHCP; tudo de configuração vai por TMP.
- Transporte: JSch (SSH) com auth de senha customizada (`com.tplink.ssh2.TPUserAuthPasswordMethods`,
  usuário no formato `user@<algo>`) e KEX customizado (`TPDHG256`, diffie-hellman-group14-sha256).
  Abre port forwarding local para a porta **20002** do Deco (`com.tplink.ssh2.m`) e canais `exec`.
- Opcodes: tabela em `com.tplink.libtpnetwork.mesh.global.p` (`TMP_APPV2_OP_*`). Payload é JSON (Gson).
- Envio: `postRequestForSet(opcode, params)`; WAN IPv4 manda `{"wan": IPv4WanBean}`
  (campos `dns1`, `dns2`, `auto_config`, `ipv4_mask_len`, ...).

| Opcode | Nome |
|---|---|
| 0x4004 / 0x4005 | IPV4_GET / IPV4_SET (WAN, inclui DNS) |
| 0x4211 / 0x4212 | LAN_IP_GET / SET |
| 0x4213 / 0x4214 | DHCP_GET / SET |
| 0x40C0..0x40C3 | IP_RESERVATION list/add/modify/remove |
| 0x40B0..0x40B3 | PORT_FORWARDING list/add/modify/delete |
| 0x4017 / 0x4018 / 0x4019 | CLIENT_LIST_BLOCK / BLOCKED_LIST_GET / UNBLOCK |
| 0x401A / 0x401B | AUTO_LED_GET / SET |
| 0x4036 / 0x4037 | QOS_MODE_GET / SET |
| 0x40D0 / 0x40D1 | DDNS_GET / SET |
| 0x424A / 0x424B | UPNP_GET / SET |

Implementado em `crates/deco-api/src/tmp.rs` (`TmpSession`), validado com GETs em 2026-10-12
(`cargo run -p deco-api --example tmp_probe`, precisa `DECO_USERNAME` = e-mail do TP-Link ID no `.env`).

- **SSH**: dropbear na 20001. Usuário = **SHA-1 hex (minúsculo) do e-mail do TP-Link ID**; senha = senha
  da conta em texto puro (mesma do admin local). `admin` e o e-mail puro são recusados.
- **Túnel**: `direct-tcpip` para `127.0.0.1:20002`.
- **TMP**: associação em 3 passos, pacotes de 4 bytes `[1,1,tipo,0]`: cliente 1 → Deco 2 → cliente 2.
  Dados (tipo 5): header 16 bytes BE `[1,1,5,0, len u16, 0, status u8, seq u32, crc32]`; o CRC32 cobre
  header+payload com o campo de CRC = `0x5A6B7C8D`. Tipo 4 = keepalive, 6 = close.
- **App v2** (payload do TMP): header 20 bytes BE `[1,2, opcode u16, tipo u8, status u8, tx u16,
  crc32(corpo) u32, tamanho total u32, offset u32]` + corpo. Tipos: 2 push, 3 push_ack, 4 pull, 5 pull_ack.
  Fragmentos de 8156 bytes. Respostas pequenas chegam direto como pull_ack.
- **Token**: opcode 1 (alloc, corpo vazio) logo após associar; opcode 3 (free) ao fechar.
- **JSON**: GETs vão sem corpo; resposta `{"error_code":0,"result":{...}}`. SETs **exigem** o envelope
  `{"params": {...}, "config_version": <epoch ms>}`; sem ele o Deco responde `error_code: 1`.

GETs validados: `0x4004` WAN (DNS, MTU, PPPoE), `0x4211` LAN, `0x4213` DHCP (faixa, DNS),
`0x40C0` reservas, `0x40B0` port forwarding, `0x4018` bloqueados.

SETs validados (2026-10-12, `examples/tmp_reservation_test.rs`): `0x40C1` add reserva
`{"reservation_list":[{"ip","mac"}]}` e `0x40C3` remove `{"reservation_list":["<MAC>"]}`
(MAC no formato `AA-BB-CC-DD-EE-FF`, como vem da lista de clientes).

API tipada em `crates/deco-api/src/tmp_config.rs`: `wan`, `dhcp`, `reservations`, `port_forwards`,
`blocked_clients` (leitura validada) e `add/remove_reservation` (validado). **Não validados**:
`add/delete_port_forward`, `set_dhcp`.

`block/unblock_client` **validados** (2026-10-12, `examples/tmp_block_test.rs`): block `0x4017`
`{"client_list":[{"mac","name":<base64>}]}`, unblock `0x4019` `{"client_list":["<MAC>"]}`; efeito imediato.

Deco de cada cliente: HTTP `admin/client?form=client_list` com `params.device_mac` = MAC do nó lista só
os clientes daquele Deco (`Session::clients_of`); `"default"` lista todos. `access_host` não é o nó.

Nós (`admin/device?form=device_list`): `signal_level` = `{"band2_4","band5"}` em string, nível do backhaul
até o Deco anterior (0 no principal; o app usa o maior entre bandas, maior é melhor; GUI mostra
1 Fraco, 2 Regular, 3+ Bom, escala inferida). `previous` = MAC do Deco anterior. Reiniciar um só:
`admin/device?form=system` `operation: reboot` com `mac_list: [{"mac"}]` (mesmo do reboot geral; não validado
com um nó só).

`set_wan_dns` **validado** (2026-10-12, `examples/tmp_dns_test.rs`, PPPoE): payload mínimo como o app,
`{"wan":{"dial_type","support_mtu_size","enable_auto_dns","mtu_size","ip_info":{"dns1","dns2"}}}`, sem
`user_info`. Mandar o objeto `wan` inteiro (com `user_info.username` sem senha) retorna `error_code 0`
mas é ignorado. A mudança aplica de forma assíncrona (~5 s, sem redial PPPoE); ler logo após o SET
ainda mostra o valor antigo. Usuário PPPoE vem em base64.

## Nuvem TP-Link

- Usada pelo app para acesso remoto (TP-Link ID). Fora do escopo.

## Rede de referência

- Deco M5 v3.2, firmware 1.9.5 Build 20260602.
- 3 nós: Bedroom (master, 192.168.68.1) → Quarto mãe → Living Room (cadeia).
