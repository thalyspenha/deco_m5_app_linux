# deco-linux

App Linux (GTK4 + libadwaita, Rust) para gerenciar redes mesh TP-Link Deco pela rede local: API HTTP (LuCI) e TMP (protocolo do app oficial, via SSH). Alvo final: Omarchy Linux (Hyprland). Desenvolvido no macOS e no Linux.

Antes de começar: ler `docs/ROADMAP.md` (feito/a fazer) e `docs/PROTOCOL.md` (protocolo, payloads e o que foi validado na rede real). Stack em `docs/DECISIONS.md`.

## Código

- Workspace Cargo: `crates/deco-api` (cliente/protocolo) e `crates/deco-app` (GUI).
- Duas APIs: HTTP LuCI (`Session`, leitura + Wi-Fi/reboot) e TMP (`tmp::TmpSession` + `tmp_config.rs`, SSH 20001, configuração). `tmp_probe` só aceita opcodes GET.
- `Session` faz logout e `TmpSession` libera token/fecha no drop. Sempre sessão curta: o Deco aceita só um admin logado e derruba o app oficial.
- Chamadas de rede rodam em `gio::spawn_blocking` via `run()` em `window.rs`; nunca bloquear a thread da UI. `run()` descarta o job se outro estiver em andamento (`app.busy`): páginas fazem retry no load e avisam no save.
- Páginas: `internet.rs`, `client.rs`, `deco.rs`, `blocked.rs`, empilhadas no `adw::NavigationView` (`app.nav`). Cabeçalho de detalhe: `detail_header()` (não usar `adw::StatusPage` dentro de `PreferencesPage`, é cortado).
- SETs do TMP exigem o envelope `params` + `config_version` (`TmpSession::set` já faz). Payload de WAN deve ser mínimo como o app (ver PROTOCOL.md).

## Credenciais

- `.env` (dev, gitignored; ver `.env.example`) ou keyring. Nunca commitar senha nem o e-mail do TP-Link ID.
- `DECO_HOST`, `DECO_PASSWORD` (senha local = senha da conta TP-Link), `DECO_USERNAME` (e-mail do TP-Link ID; o usuário SSH do TMP é o SHA-1 dele, calculado em `TmpSession::connect`).

## Regras com a rede real

- Leitura (`probe`, `tmp_probe`) pode rodar livre.
- Qualquer escrita (Wi-Fi, DNS, reserva, bloqueio, reboot, port forwarding, DHCP) só com confirmação do usuário, a cada vez.
- Testar escrita com aparelho dispensável (ex.: ar-condicionado Daikin) e reverter no mesmo teste; deixar o teste em `crates/deco-api/examples/` e registrar o resultado em PROTOCOL.md/ROADMAP.md.
- Nunca reiniciar o Deco principal em teste (`reboot_node_test` recusa).

## Rodar e ver a GUI

`DECO_OPEN=internet|blocked|client:<nome>|deco:<nome>` abre uma subpágina direto (cliques simulados não funcionam no Mac). A janela pode levar ~10–20 s para aparecer: fazer polling.

**Linux (Omarchy/Hyprland)**
- Deps: `sudo pacman -S rust gtk4 libadwaita openssl pkgconf grim jq`.
- Rodar: `cargo run -p deco-app` (ícones Adwaita já vêm do sistema).
- Screenshot da janela: `hyprctl clients -j | jq -r '.[] | select(.class=="io.github.thalys.DecoLinux") | "\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"'` e `grim -g "<geom>" <arquivo>.png`.

**macOS**
- cargo vem do rustup do brew: `export PATH=/opt/homebrew/opt/rustup/bin:$PATH`. Requer `brew install adwaita-icon-theme`.
- Rodar com `XDG_DATA_DIRS=/opt/homebrew/share ./target/debug/deco-app &` (Ghostty sobrescreve XDG_DATA_DIRS e os ícones quebram).
- Screenshot: `kCGWindowNumber` da janela de 460 px via `CGWindowListCopyWindowInfo(.optionAll, …)` (swift -e), depois `screencapture -x -o -l <id>` (não usar `-R`, captura o que estiver na frente). Se vier só wallpaper, falta permissão de gravação de tela.

## Engenharia reversa

- APK do app Deco (`com.tplink.tpm5`, APKMirror, bundle `.apkm`): `unzip` e `jadx --no-res -d <dir> base.apk` numa pasta temporária fora do repo. Opcodes em `com.tplink.libtpnetwork.mesh.global.p`, beans em `com.tplink.tpm5.component.more.*/repository/bo`.

## Git

- Commits com `36860610+thalyspenha@users.noreply.github.com` (configurar `git config user.email` em cada clone; a conta bloqueia push que expõe e-mail privado). Não usar gmail/outlook.
- Remote `origin`: `git@github.com-personal:thalyspenha/deco_m5_app_linux.git` no Mac (alias da chave pessoal); no Linux, o host que tiver a chave da conta `thalyspenha`.
