# Infraestrutura

App desktop local. **Não há** servidor, Docker/docker-compose, Kubernetes, IaC, banco de dados, filas ou workers em processo separado.

## Background / concorrência

- Não há fila nem worker dedicado. Trabalho de rede roda em `gio::spawn_blocking` (pool de threads do GIO), um job por vez (`app.busy`).
- Timer periódico: `glib::timeout_add_seconds_local(15)` para auto-refresh (só com janela em foco).
- Timers pontuais: `timeout_add_local_once(300 ms)` para retry de carregamento; `thread::sleep(6 s)` no worker após salvar DNS.

## Configuração

| Variável | Uso | Onde |
|---|---|---|
| `DECO_HOST` | IP/host do Deco (padrão `192.168.68.1`) | `credentials.rs`, exemplos |
| `DECO_PASSWORD` | Senha do admin local; se definida, ignora o keyring | idem |
| `DECO_USERNAME` | E-mail do TP-Link ID (TMP) | idem |
| `DECO_OPEN` | Abre subpágina ao iniciar (`internet`, `blocked`, `client:<nome>`, `deco:<nome>`) | `window.rs` |
| `TMP_DEBUG` | Se presente, imprime pacotes TMP em hex no stderr | `tmp.rs` |

Modelo: `.env.example`. `.env` e `/target` estão no `.gitignore`.

## Build

- `cargo build -p deco-app [--release]` / `cargo run -p deco-app`.
- Requisitos de sistema: Rust stable, GTK ≥ 4.12, libadwaita ≥ 1.5, OpenSSL, pkg-config (README). No macOS também `adwaita-icon-theme` e `XDG_DATA_DIRS=/opt/homebrew/share`.
- Versão mínima de Rust: **Não identificado** (sem `rust-version` nem `rust-toolchain`).

## Empacotamento (Arch/AUR)

`packaging/aur/PKGBUILD` (`deco-linux-git`, `x86_64`/`aarch64`):

- `depends`: `gtk4`, `libadwaita`, `openssl`, `dbus`; `makedepends`: `cargo`, `git`.
- `options=('!lto')` (LTO do makepkg quebra o link do libssh2 embutido).
- `prepare`: `cargo fetch --locked`; `build`: `cargo build --frozen --release -p deco-app`.
- Instala `/usr/bin/deco-app`, o `.desktop`, o ícone SVG e a licença.
- `.SRCINFO` regenerado com `makepkg --printsrcinfo` (o `pkgver` nele, `0.1.0.r17.ge420b1f`, é o da última geração; o real é calculado no build).

## CI/CD

**Não identificado**: não há `.github/workflows` nem outro pipeline no repositório.

## Rede

O app precisa estar na mesma LAN do Deco e alcançar as portas TCP 80 (HTTP) e 20001 (SSH/TMP) do Deco principal.
