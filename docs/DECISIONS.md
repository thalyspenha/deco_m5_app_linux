# Decisões

## Backend em Python + tplinkrouterc6u

A lib já resolve a criptografia do login e suporta o Deco M5 v3. Reaproveitada pela CLI e pela GUI.

## Stack da GUI — proposta: Python + GTK4 + libadwaita (PyGObject)

Status: **aguardando confirmação**.

Alvo: Omarchy Linux (Hyprland, Wayland).

- Nativo Wayland, encaixa bem em tiling (Hyprland) e já é o toolkit da maioria dos apps do Omarchy.
- libadwaita entrega visual moderno pronto (cards, listas, dark mode automático, layout responsivo).
- Mesma linguagem do backend: zero ponte/IPC, reaproveita `deco_linux.core`.
- Distribuição: pacote AUR / Flatpak.
- Dá para desenvolver no Mac (`brew install gtk4 libadwaita pygobject3`).

Alternativas avaliadas:
- Rust + gtk4-rs/relm4: mesmo visual, binário rápido, mas reescreveria o backend.
- Tauri/Electron (web UI): visual livre, mas pesado e destoa do sistema.
- Qt/PySide6: maduro, mas visual menos alinhado ao ecossistema GNOME/Omarchy.
- Textual (TUI): rápido de fazer, combina com Omarchy (muito terminal), mas não é "app gráfico".
