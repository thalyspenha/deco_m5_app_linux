# Decisões

## GUI: Rust + GTK4 + libadwaita (decidido 2026-10-12)

Alvo: Omarchy Linux (Hyprland, Wayland).

- Nativo Wayland, encaixa bem em tiling; GTK é o toolkit da maioria dos apps do Omarchy.
- libadwaita entrega visual moderno pronto (listas, cards, dark mode, layout adaptativo).
- Rust escolhido pelo usuário: binário único, leve, sem runtime Python para empacotar (AUR simples).
- Bindings: `gtk4` 0.10 (feature `v4_12`), `libadwaita` 0.8 (feature `v1_5`).

## Backend: cliente próprio em Rust (`deco-api`)

Porte do protocolo implementado em `tplinkrouterc6u` (Python), que foi usado na prova de conceito (ver commit 2aea33a). Cliente HTTP bloqueante (`reqwest::blocking`) chamado em thread de trabalho via `gio::spawn_blocking`.

## Credenciais no keyring

`keyring` crate: Secret Service no Linux (gnome-keyring/KeePassXC), Keychain no macOS. `.env` só para desenvolvimento.

## Auto-refresh só com janela em foco

Cada refresh é login + logout, que derruba momentaneamente a sessão do app oficial. Atualiza a cada 15 s apenas enquanto a janela está ativa.

## Alternativas descartadas

- Python + GTK4: reaproveitaria a lib pronta, mas usuário preferiu Rust.
- Tauri/Electron: pesado e destoa do sistema.
- Qt: visual menos alinhado ao ecossistema GNOME/Omarchy.
- Textual (TUI): não é app gráfico.
