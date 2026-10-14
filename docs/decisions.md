# Decisões

Decisões registradas e as que se veem no código. Datas só quando registradas; senão "Não identificado".

## GUI: Rust + GTK4 + libadwaita (decidido 2026-10-12)

Alvo: Omarchy Linux (Hyprland, Wayland).

- Nativo Wayland, encaixa bem em tiling; GTK é o toolkit da maioria dos apps do Omarchy.
- libadwaita entrega visual moderno pronto (listas, cards, dark mode, layout adaptativo).
- Rust escolhido pelo usuário: binário único, leve, sem runtime Python para empacotar (AUR simples).
- Bindings: `gtk4` 0.10 (feature `v4_12`), `libadwaita` 0.8 (feature `v1_5`).

## Backend: cliente próprio em Rust (`deco-api`)

Porte do protocolo implementado em `tplinkrouterc6u` (Python), que foi usado na prova de conceito (ver commit 2aea33a). Cliente HTTP bloqueante (`reqwest::blocking`) chamado em thread de trabalho via `gio::spawn_blocking`.

## Workspace com dois crates (data: Não identificado)

Protocolo (`deco-api`) separado da GUI (`deco-app`), permitindo exemplos/testes de rede sem GTK.

## TMP para configuração (2026-10-12/13)

O app oficial faz WAN/LAN/DHCP/reservas/bloqueio/Wi-Fi por TMP, e a escrita via HTTP falha (`code 1` em WAN; HTTP 500 em Wi-Fi no firmware 1.9.5). Implementado `TmpSession` (SSH via `ssh2`/libssh2) a partir do APK. `Session::set_wifi` (HTTP) foi removido; a GUI usa `TmpSession::set_wifi`. Leituras básicas (nós, clientes, WAN, estado do Wi-Fi) continuam em HTTP. Detalhes em [PROTOCOL.md](PROTOCOL.md).

## Payloads de escrita imitam o app oficial

SETs mínimos como os beans do app: WAN só com tipo/MTU/DNS (sem `user_info`, que faz o Deco ignorar a mudança); Wi-Fi com o objeto completo da rede alterada. Envelope `params` + `config_version` obrigatório.

## Sessões curtas

Login/logout (ou connect/close) por operação, com logout também no `Drop`, porque o Deco aceita um admin por vez.

## Um job de rede por vez

`run()` descarta jobs enquanto `app.busy`; evita sessões concorrentes no Deco. Páginas fazem retry no carregamento e avisam nas escritas.

## Credenciais no keyring

`keyring` crate: Secret Service no Linux (gnome-keyring/KeePassXC), Keychain no macOS. `.env` só para desenvolvimento.

## E-mail do TP-Link ID opcional

Sem ele o app funciona só com a visão geral (HTTP); funções TMP pedem o e-mail na hora (página Internet) ou ficam desativadas.

## Auto-refresh só com janela em foco

Cada refresh é login + logout, que derruba momentaneamente a sessão do app oficial. Atualiza a cada 15 s apenas enquanto a janela está ativa.

## Empacotamento AUR sem LTO

`options=('!lto')` no PKGBUILD: o LTO padrão do makepkg quebra o link do libssh2 embutido (commit 5b6bf1d).

## Alternativas descartadas

- Python + GTK4: reaproveitaria a lib pronta, mas usuário preferiu Rust.
- Tauri/Electron: pesado e destoa do sistema.
- Qt: visual menos alinhado ao ecossistema GNOME/Omarchy.
- Textual (TUI): não é app gráfico.
