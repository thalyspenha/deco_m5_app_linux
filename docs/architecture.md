# Arquitetura

App desktop nativo (GTK4 + libadwaita, Rust) que gerencia uma rede mesh TP-Link Deco **pela rede local**, sem nuvem e sem servidor próprio. Não há backend: o processo da GUI fala direto com o Deco principal.

## Visão geral

```
┌──────────────────────── deco-app (binário) ────────────────────────┐
│ main.rs ─► window.rs (App, run(), refresh, tela principal, login)  │
│             ├─ internet.rs  (Conexão de internet / DNS)            │
│             ├─ client.rs    (detalhe do cliente, reserva, bloqueio)│
│             ├─ deco.rs      (detalhe do nó, reboot do nó)          │
│             └─ blocked.rs   (bloqueados, desbloquear)              │
│ credentials.rs (env/.env → keyring)                                │
└───────────────┬────────────────────────────────────────────────────┘
                │ chamadas bloqueantes em gio::spawn_blocking
┌───────────────▼──────────── deco-api (lib) ────────────────────────┐
│ lib.rs + models.rs  Session    → HTTP LuCI (RSA + AES-128-CBC)     │
│ crypto.rs           RsaKey, AesSession                             │
│ tmp.rs              TmpSession → SSH:20001 → direct-tcpip :20002   │
│ tmp_config.rs       API tipada sobre TMP (WAN, DHCP, Wi-Fi, ...)   │
└───────────────┬───────────────────────────────┬────────────────────┘
                │ HTTP :80 /cgi-bin/luci        │ SSH :20001
                ▼                               ▼
           Deco principal (firmware TP-Link, ex.: M5 1.9.5)
```

## Camadas

| Camada | Onde | Responsabilidade |
|---|---|---|
| Apresentação | `crates/deco-app/src/*.rs` | Widgets libadwaita, navegação (`adw::NavigationView`), diálogos de confirmação, toasts |
| Orquestração | `window.rs::run()` | Executa um job por vez numa thread de trabalho e devolve o resultado na thread da UI |
| Credenciais | `credentials.rs` | Carrega de variáveis de ambiente (`.env`) ou do keyring; salva/limpa no keyring |
| Cliente/protocolo | `crates/deco-api` | Sessões HTTP e TMP, criptografia, framing, mapeamento JSON → structs |

## Modelo de execução

- Thread da UI (GLib main loop) nunca faz rede.
- `run(app, job, done)` (`window.rs`): clona as credenciais, marca `app.busy`, roda `job` em `gio::spawn_blocking`, depois `done` na UI. Se `app.busy` já estiver ativo, **o job é descartado silenciosamente**.
  - Páginas tratam isso: no carregamento fazem retry a cada 300 ms; nas ações de escrita avisam "Aguarde a atualização terminar…".
- Auto-refresh: `glib::timeout_add_seconds_local(15 s)` só quando a janela está ativa e a tela visível é `main`.
- Sessões são curtas: cada job abre `Session::login` / `TmpSession::connect` e fecha no fim (o `Drop` também fecha).

## Navegação / telas

`window.rs` monta um `gtk::Stack` com `loading`, `login` e `main`, dentro de um `adw::NavigationView` (`app.nav`). Subpáginas são empilhadas com `app.nav.push(...)`:

- Internet (`internet::open`), Bloqueados (`blocked::open`), Cliente (`client::open`), Deco (`deco::open`).
- Atalho de desenvolvimento: `DECO_OPEN=internet|blocked|client:<nome>|deco:<nome>`.

## Fluxos principais

1. **Inicialização**: `main.rs` carrega `.env` (dotenvy), cria `adw::Application` (`io.github.thalys.DecoLinux`). `window::build` lê credenciais; se existirem → `refresh`, senão → tela de login.
2. **Login**: `submit_login` monta `Credentials` (host, senha, e-mail opcional), roda `fetch`; sucesso → salva no keyring e mostra `main`; falha → volta ao login com toast.
3. **Refresh** (`fetch`): `Session::login` → `wan()`, `wifi()`, `nodes()`, `clients()` → `logout()`; `show()` repopula as listas.
4. **Wi-Fi on/off**: switch → (confirmação se desligar a rede principal) → `TmpSession::set_wifi` → toast; erro reverte o switch.
5. **DNS da WAN**: página Internet (TMP) → validação IPv4 → confirmação → `set_wan_dns` → espera 6 s → relê `wan()`.
6. **Cliente**: descobre o Deco do cliente iterando `clients_of(node.mac)` (HTTP); estado de reserva via TMP; switch reserva/remove; botão bloquear com confirmação.
7. **Deco**: lista clientes do nó (`clients_of`); reiniciar o nó com confirmação (`reboot_node`).
8. **Reiniciar rede**: menu → confirmação → `reboot_all` (HTTP).
9. **Sair**: limpa o keyring e volta ao login.

## O que não existe

- Servidor/backend próprio, banco de dados, filas, Docker: **não existem** (ver `infrastructure.md`).
- Acesso pela nuvem TP-Link: fora do escopo (ver `PROTOCOL.md`).
