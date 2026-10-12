# deco-linux

App Linux (GTK4 + libadwaita, Rust) para controlar rede mesh TP-Link Deco M5 via API HTTP local.

- Workspace Cargo: `crates/deco-api` (cliente/protocolo) e `crates/deco-app` (GUI).
- No Mac, cargo vem do rustup do brew: `export PATH=/opt/homebrew/opt/rustup/bin:$PATH`.
- `Session` (deco-api) faz logout no drop. Sempre sessão curta: o Deco aceita só um admin logado e derruba o app oficial.
- Chamadas de rede rodam em `gio::spawn_blocking` via `run()` em `window.rs`; nunca bloquear a thread da UI.
- Credenciais: `.env` (dev, gitignored) ou keyring. Nunca commitar senha.
- Docs: `docs/ROADMAP.md` (a fazer), `docs/DECISIONS.md` (stack), `docs/PROTOCOL.md` (protocolo).
- Alvo final: Omarchy Linux (Hyprland). Desenvolvimento no macOS.
- Não executar escrita (wifi, reboot) contra a rede real sem confirmação do usuário. `probe` é somente leitura.
- Screenshot da GUI no Mac (Ghostty com permissão de gravação de tela): rodar `./target/debug/deco-app &`, pegar bounds da janela via `CGWindowListCopyWindowInfo` (swift -e) e `screencapture -x -R x,y,w,h` no scratchpad. Se vier só wallpaper, permissão não está ativa.
- Git: identidade pessoal (36860610+thalyspenha@users.noreply.github.com), remote via host SSH `github.com-personal`.
