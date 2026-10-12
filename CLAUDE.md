# deco-linux

App Linux para controlar rede mesh TP-Link Deco M5 via API HTTP local.

- Python 3.13, gerenciado com `uv` (`uv sync`, `uv run deco ...`).
- Backend: `src/deco_linux/core.py` — `session()` faz login/logout. Sempre usar sessão curta: o Deco aceita só um admin logado e derruba o app oficial.
- CLI: `src/deco_linux/cli.py`.
- Credenciais em `.env` (gitignored). Nunca commitar senha.
- Docs: `docs/ROADMAP.md` (a fazer), `docs/DECISIONS.md` (stack), `docs/PROTOCOL.md` (protocolo).
- Alvo final: Omarchy Linux (Hyprland). Desenvolvimento no macOS.
- Não executar `wifi` ou `reboot` contra a rede real sem confirmação do usuário.
- Git: identidade pessoal (36860610+thalyspenha@users.noreply.github.com), remote via host SSH `github.com-personal`.
