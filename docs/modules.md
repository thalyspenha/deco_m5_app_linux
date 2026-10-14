# Módulos

Workspace Cargo (`Cargo.toml` raiz, resolver 2, edição 2021, versão 0.1.0, licença MIT) com dois crates.

## `crates/deco-api` (biblioteca)

| Arquivo | Visibilidade | Conteúdo |
|---|---|---|
| `src/lib.rs` | público | `Session` (login/request/logout HTTP), `Error`, `Result`; reexporta `models::*` e tipos de `tmp_config` |
| `src/crypto.rs` | privado | `RsaKey` (RSA PKCS#1 v1.5, blocos de 53 bytes, saída hex), `AesSession` (AES-128-CBC/PKCS7, base64). Contém os únicos testes unitários |
| `src/models.rs` | privado (tipos reexportados) | `Node`, `Network`, `Client`, `Wan`, `WifiNetwork`, `WifiState`; métodos de `Session`: `nodes`, `clients`, `clients_of`, `wan`, `wifi`, `reboot_node`, `reboot_all`; helpers `list`, `s`, `decode_name` |
| `src/tmp.rs` | público (`deco_api::tmp`) | `TmpSession` (SSH + associação TMP + camada app v2), módulo `op` com opcodes, `call`/`get`/`set`/`close`; trace hex com `TMP_DEBUG` |
| `src/tmp_config.rs` | privado (tipos reexportados) | `WanConfig`, `DhcpConfig`, `Reservation`, `PortForward`, `BlockedClient`; métodos de `TmpSession`: `wifi`, `set_wifi`, `wan`, `dhcp`, `reservations`, `add/remove_reservation`, `port_forwards`, `add/delete_port_forward`, `blocked_clients`, `block/unblock_client`, `set_wan_dns`, `set_dhcp` |

### Erros (`deco_api::Error`)

`Http` (reqwest), `Protocol(&'static str)`, `Login`, `Router { code, path }` (para TMP, `path` = `tmp:0x....`), `Json`, `Io`, `Ssh`, `Tmp(u8)` (status do TMP/app).

### Exemplos (`crates/deco-api/examples/`)

Usados como testes manuais contra a rede real (ver `testing.md`): `probe`, `tmp_probe`, `tmp_reservation_test`, `tmp_block_test`, `tmp_dns_test`, `wifi_test`, `reboot_node_test`.

## `crates/deco-app` (binário `deco-app`)

| Arquivo | Conteúdo |
|---|---|
| `src/main.rs` | `APP_ID = "io.github.thalys.DecoLinux"`, carrega `.env`, ícone padrão, `window::build` |
| `src/window.rs` | struct `App` (estado compartilhado em `Rc`), tela principal, login, `run()`, `refresh()`, `toast()`, `fetch()`, Wi-Fi switches, reboot geral, helpers de UI (`detail_header`, `connection_label`, `client_subtitle`, `client_icon`, `speed`) |
| `src/credentials.rs` | `Credentials { host, password, email }`, `Credentials::tmp()`, `load`/`save`/`clear` no keyring (serviço `deco-linux`, chaves `host`, `password`, `email`), host padrão `192.168.68.1` |
| `src/internet.rs` | Página "Conexão de internet": dados WAN via TMP, edição de DNS, sub-tela de e-mail do TP-Link ID |
| `src/client.rs` | Página do cliente: IP/MAC/rede, Deco conectado, switch de reserva de IP, bloquear |
| `src/deco.rs` | Página do nó: papel, Deco anterior, sinal de backhaul, modelo/IP/MAC/firmware, clientes do nó, reiniciar nó |
| `src/blocked.rs` | Página de dispositivos bloqueados com "Desbloquear" |

### Estado compartilhado (`App`)

- `creds: RefCell<Option<Credentials>>`, `busy: Cell<bool>` (um job por vez), `syncing: Cell<bool>` (evita que switches disparem escrita ao serem atualizados pelo refresh), `dynamic_rows` (linhas recriadas a cada refresh), `open_client` (atalho `DECO_OPEN`).
- Páginas de detalhe têm structs `Page` próprias em `Rc`, com `syncing` local quando há switches.

## Outros diretórios

| Caminho | Conteúdo |
|---|---|
| `data/` | `io.github.thalys.DecoLinux.desktop` e ícone SVG (hicolor/scalable) |
| `packaging/aur/` | `PKGBUILD` e `.SRCINFO` do pacote `deco-linux-git` |
| `docs/` | Esta documentação, `PROTOCOL.md`, `ROADMAP.md` |
