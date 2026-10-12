# deco-linux

App Linux (GTK4 + libadwaita, Rust) para controlar rede mesh TP-Link Deco M5 via API HTTP local.

- Workspace Cargo: `crates/deco-api` (cliente/protocolo) e `crates/deco-app` (GUI).
- No Mac, cargo vem do rustup do brew: `export PATH=/opt/homebrew/opt/rustup/bin:$PATH`.
- `Session` (deco-api) faz logout no drop. Sempre sessão curta: o Deco aceita só um admin logado e derruba o app oficial.
- Chamadas de rede rodam em `gio::spawn_blocking` via `run()` em `window.rs`; nunca bloquear a thread da UI.
- Credenciais: `.env` (dev, gitignored) ou keyring. Nunca commitar senha. `.env`: `DECO_HOST`, `DECO_PASSWORD`, `DECO_USERNAME` (e-mail TP-Link ID, usado no TMP).
- Duas APIs: HTTP LuCI (`Session`, leitura + Wi-Fi/reboot) e TMP (`tmp::TmpSession`, SSH 20001, configuração). Ver `docs/PROTOCOL.md`. `tmp_probe` só aceita opcodes GET.
- APK decompilado (jadx) fica no scratchpad, não no repo; regenerar do `.apkm` em ~/Downloads se preciso.
- Docs: `docs/ROADMAP.md` (a fazer), `docs/DECISIONS.md` (stack), `docs/PROTOCOL.md` (protocolo).
- Alvo final: Omarchy Linux (Hyprland). Desenvolvimento no macOS.
- Não executar escrita (wifi, reboot) contra a rede real sem confirmação do usuário. `probe` é somente leitura.
- Screenshot da GUI no Mac (Ghostty com permissão de gravação de tela): rodar `XDG_DATA_DIRS=/opt/homebrew/share ./target/debug/deco-app &` (Ghostty sobrescreve XDG_DATA_DIRS e GTK não acha o tema Adwaita: ícones quebrados), pegar o `kCGWindowNumber` da janela via `CGWindowListCopyWindowInfo` (swift -e) e `screencapture -x -o -l <id>` no scratchpad (`-R` captura o que estiver na frente). Se vier só wallpaper, permissão não está ativa. Requer `brew install adwaita-icon-theme`. Cliques simulados (CGEvent) não chegam; para abrir subpágina use `DECO_OPEN=internet`.
- Git: identidade pessoal (36860610+thalyspenha@users.noreply.github.com), remote via host SSH `github.com-personal`.
