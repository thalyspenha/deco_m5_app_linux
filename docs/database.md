# Banco de dados

**Não há banco de dados** no projeto: nenhuma dependência de SQL/ORM/armazenamento embarcado no `Cargo.lock`, nenhum arquivo de schema ou migração.

## Persistência existente

| O quê | Onde | Código |
|---|---|---|
| Credenciais (`host`, `password`, `email`) | Keyring do sistema, serviço `deco-linux` (Secret Service no Linux, Keychain no macOS) | `crates/deco-app/src/credentials.rs` |
| Credenciais de desenvolvimento | `.env` no diretório atual (`DECO_HOST`, `DECO_PASSWORD`, `DECO_USERNAME`), gitignored | `main.rs` (`dotenvy::dotenv()`), `credentials::load` |

Variáveis de ambiente têm precedência: se `DECO_PASSWORD` existir, o keyring não é lido.

Todo o resto (nós, clientes, WAN, reservas etc.) é lido do Deco a cada operação e mantido só em memória.

## Entidades (estruturas em memória, `deco-api`)

Não são tabelas; são structs preenchidas a partir do JSON do Deco.

| Struct | Origem | Campos |
|---|---|---|
| `Node` | HTTP `admin/device?form=device_list` | `mac`, `name`, `model`, `ip`, `is_master`, `parent_mac`, `online`, `firmware`, `hardware`, `signal_2g`, `signal_5g` |
| `Client` | HTTP `admin/client?form=client_list` | `mac`, `name` (base64 decodificado), `ip`, `network` (`Network`), `band`, `down_kbps`, `up_kbps` |
| `Wan` | HTTP `admin/network?form=wan_ipv4` | `ip`, `gateway`, `dial_type` |
| `WifiState` | HTTP `admin/wireless?form=wlan` ou TMP `0x4009` | `main_2g`, `main_5g`, `guest_2g`, `guest_5g` |
| `WanConfig` | TMP `0x4004` | `dial_type`, `ip`, `mask`, `gateway`, `mac`, `username`, `auto_dns`, `dns1`, `dns2`, `mtu` |
| `DhcpConfig` | TMP `0x4213` | `start_ip`, `end_ip`, `gateway`, `dns1`, `dns2`, `in_use` |
| `Reservation` | TMP `0x40C0` | `ip`, `mac` |
| `PortForward` | TMP `0x40B0` | `id`, `name`, `internal_ip`, `internal_port`, `external_port`, `protocol` |
| `BlockedClient` | TMP `0x4018` | `mac`, `name` |

## Relacionamentos (lógicos)

- `Node.parent_mac` → `Node.mac` do Deco anterior (topologia em árvore/cadeia; `None` no principal).
- `Client` ↔ `Node`: não há campo no cliente; a associação é obtida chamando `clients_of(node.mac)` para cada nó (`client.rs`).
- `Reservation.mac`, `BlockedClient.mac` ↔ `Client.mac` (formato `AA-BB-CC-DD-EE-FF`).
