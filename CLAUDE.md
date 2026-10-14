# deco-linux

## Contexto

App desktop Linux para gerenciar redes mesh TP-Link Deco pela rede local, sem nuvem e sem o app oficial. Alvo final: Omarchy Linux (Hyprland). Desenvolvido no Linux (NixOS: `shell.nix` + direnv) e no macOS. Testado só com Deco M5 v3.2, firmware 1.9.5.

## Stack

Rust (workspace Cargo), GTK4 0.10 + libadwaita 0.8, `reqwest` blocking (HTTP), `ssh2`/libssh2 (TMP), `rsa`/`aes`/`cbc` (cripto do LuCI), `keyring` (credenciais). Empacotado para o AUR (`deco-linux-git`).

## Arquitetura resumida

- `crates/deco-api`: cliente. `Session` (HTTP LuCI: leitura + reboot; escrita de Wi-Fi quebrada no firmware 1.9.5) e `tmp::TmpSession` + `tmp_config.rs` (TMP via SSH 20001: WAN/DNS, reservas, bloqueio, Wi-Fi on/off).
- `crates/deco-app`: GUI. `window.rs` (estado `App`, tela principal, login, `run()`), páginas `internet.rs`, `client.rs`, `deco.rs`, `blocked.rs` empilhadas no `adw::NavigationView` (`app.nav`); `credentials.rs` (env/`.env` → keyring; `Credentials::tmp()` abre o TMP).
- Sem backend, banco, filas ou Docker.

## Regras importantes

- Sempre sessão curta: o Deco aceita um admin por vez e derruba o app oficial. `Session` faz logout e `TmpSession` libera token/fecha no drop.
- Rede só em `gio::spawn_blocking` via `run()`; nunca bloquear a thread da UI. `run()` descarta o job se `app.busy`: páginas fazem retry no load e avisam no save.
- SETs do TMP exigem o envelope `params` + `config_version` (`TmpSession::set` já faz). Payloads de escrita mínimos como os do app oficial (ver PROTOCOL.md).
- Com a rede real: leitura (`probe`, `tmp_probe`) livre; **qualquer escrita só com confirmação do usuário, a cada vez**. Testar escrita com aparelho dispensável (ex.: ar-condicionado Daikin), reverter no mesmo teste, deixar o teste em `crates/deco-api/examples/` e registrar em PROTOCOL.md/ROADMAP.md. Nunca reiniciar o Deco principal.
- Credenciais: `.env` (dev, gitignored; ver `.env.example`) ou keyring. Nunca commitar senha nem o e-mail do TP-Link ID. `DECO_PASSWORD` = senha local = senha da conta; `DECO_USERNAME` = e-mail do TP-Link ID (usuário SSH = SHA-1 dele).
- Instalado, o app não lê `.env` (só no diretório atual); login vai pelo keyring.

## Convenções de desenvolvimento

- Textos da UI e docs em português; código e comentários em inglês.
- Cabeçalho de páginas de detalhe com `detail_header()` (não usar `adw::StatusPage` dentro de `PreferencesPage`, é cortado).
- Switches atualizados por dados do Deco usam a flag `syncing` para não disparar escrita.
- Ações destrutivas passam por `adw::AlertDialog`; erros viram toast.
- `tmp_probe` só aceita opcodes GET. `DECO_OPEN=internet|blocked|client:<nome>|deco:<nome>` abre uma subpágina direto.
- `data/` (`.desktop`, ícone) usa o id `io.github.thalys.DecoLinux` = `APP_ID` = class no Hyprland. PKGBUILD precisa de `options=('!lto')`; regenerar `.SRCINFO` após mudá-lo.
- Git: commits com `36860610+thalyspenha@users.noreply.github.com` (configurar `git config user.email` em cada clone; não usar gmail/outlook). Remote `origin`: `git@github.com-personal:thalyspenha/deco_m5_app_linux.git`.

## Documentação

Antes de começar: `docs/ROADMAP.md` (feito/a fazer) e `docs/PROTOCOL.md` (protocolo, payloads, o que foi validado).

- `docs/architecture.md`, `docs/modules.md`, `docs/api.md`, `docs/business-rules.md`
- `docs/integrations.md`, `docs/infrastructure.md`, `docs/dependencies.md`, `docs/database.md`
- `docs/testing.md` (inclui inconsistências conhecidas), `docs/decisions.md`
- `docs/development.md`: rodar/screenshot no Linux e macOS, `TMP_DEBUG`, engenharia reversa do APK
