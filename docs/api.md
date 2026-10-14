# APIs

O projeto **não expõe** nenhuma API (sem servidor HTTP, CLI pública ou D-Bus próprio). Ele **consome** duas APIs do Deco. Detalhes de protocolo, payloads e histórico de validação: [PROTOCOL.md](PROTOCOL.md).

## 1. HTTP LuCI (`deco_api::Session`)

- Base: `http://<host>/cgi-bin/luci/;stok=<stok>/<path>`; timeout 15 s; usuário fixo `admin`.
- Login (`Session::login`):
  1. `POST login?form=keys` (`operation=read`, sem cripto) → chave RSA de senha.
  2. `POST login?form=auth` → chave RSA de assinatura + `seq`.
  3. `POST login?form=login` com `{"params":{"password":<RSA hex>},"operation":"login"}` cifrado; resposta traz `stok` e cookie `sysauth`.
- Requisições: corpo `sign=<RSA hex>&data=<AES base64>`; assinatura sobre `[k=..&i=..&]h=md5("admin"+senha)&s=<seq+len(data)>`. Resposta `{"data": <AES>}` → `{"error_code":0,"result":...}`; código ≠ 0 vira `Error::Router`.
- Logout: `admin/system?form=logout` (`operation: logout`), também no `Drop`.

### Endpoints usados pelo código

| Método Rust | Path | Body | Uso na GUI |
|---|---|---|---|
| `nodes()` | `admin/device?form=device_list` | `{"operation":"read"}` | refresh, reboot geral |
| `clients()` / `clients_of(mac)` | `admin/client?form=client_list` | `{"operation":"read","params":{"device_mac": mac \| "default"}}`; filtra `online` | refresh, página do cliente/Deco |
| `wan()` | `admin/network?form=wan_ipv4` | read | linha "Internet" |
| `wifi()` | `admin/wireless?form=wlan` | read | estado dos switches de Wi-Fi |
| `reboot_node(mac)` | `admin/device?form=system` | `{"operation":"reboot","params":{"mac_list":[{"mac"}]}}` | reiniciar este Deco |
| `reboot_all()` | `admin/device?form=system` | idem com todos os MACs | menu "Reiniciar rede" |
| `request(path, body)` | qualquer | livre | só `examples/probe.rs` |

## 2. TMP (`deco_api::tmp::TmpSession`)

- SSH (libssh2) em `<host>:20001`, usuário = SHA-1 hex do e-mail do TP-Link ID (`trim`), senha = senha da conta; canal `direct-tcpip` para `127.0.0.1:20002`; timeouts 15 s.
- Associação TMP em 3 passos, aloca token (opcode 1); `close`/`Drop` libera token (opcode 3) e envia close.
- `get(op)` → `result`; `set(op, params)` envolve em `{"params":..., "config_version": <epoch ms>}`; `call(op, body)` cru. `error_code` ≠ 0 → `Error::Router { path: "tmp:0x...." }`.

### Opcodes definidos em `tmp::op`

| Constante | Valor | Método tipado | Validado na rede real (PROTOCOL.md) | Usado na GUI |
|---|---|---|---|---|
| `IPV4_GET` | 0x4004 | `wan()` | sim | Internet |
| `IPV4_SET` | 0x4005 | `set_wan_dns()` | sim | Internet (salvar DNS) |
| `LAN_IP_GET` | 0x4211 | — (só `tmp_probe`) | sim (GET) | não |
| `DHCP_GET` | 0x4213 | `dhcp()` | sim | não |
| `DHCP_SET` | 0x4214 | `set_dhcp()` | **não** | não |
| `IP_RESERVATION_LIST_GET` | 0x40C0 | `reservations()` | sim | Cliente |
| `IP_RESERVATION_LIST_ADD` | 0x40C1 | `add_reservation()` | sim | Cliente |
| `IP_RESERVATION_LIST_REMOVE` | 0x40C3 | `remove_reservation()` | sim | Cliente |
| `PORT_FORWARDING_LIST_GET` | 0x40B0 | `port_forwards()` | sim | não |
| `PORT_FORWARDING_ADD` | 0x40B1 | `add_port_forward()` | **não** | não |
| `PORT_FORWARDING_DELETE` | 0x40B3 | `delete_port_forward()` | **não** | não |
| `CLIENT_LIST_BLOCK` | 0x4017 | `block_client()` | sim | Cliente |
| `BLOCKED_LIST_GET` | 0x4018 | `blocked_clients()` | sim | Bloqueados |
| `BLOCKED_LIST_UNBLOCK` | 0x4019 | `unblock_client()` | sim | Bloqueados |
| `WIRELESS_GET` | 0x4009 | `wifi()` | sim | indireto (`set_wifi` lê antes) |
| `WIRELESS_SET` | 0x4008 | `set_wifi()` | guest sim; main **não** | switches de Wi-Fi |

Outros opcodes listados em PROTOCOL.md (LED, QoS, DDNS, UPnP, LAN SET, port forwarding modify) **não estão implementados**.

## Autenticação / autorização

- HTTP: senha local do admin (= senha da conta TP-Link). Usuário fixo `admin`.
- TMP: e-mail do TP-Link ID (opcional na GUI) + mesma senha. Sem e-mail, `Credentials::tmp()` falha com `Protocol("e-mail do TP-Link ID não informado")` e as funções TMP ficam indisponíveis (botão bloquear desativado, reserva oculta, Internet pede e-mail).
- O Deco aceita **uma sessão admin por vez**; não há papéis/permissões no app.
