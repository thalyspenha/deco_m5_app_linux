# deco-linux

App GTK4/libadwaita para controlar redes mesh TP-Link Deco pela rede local, sem depender do app oficial Android/iOS. Feito para Omarchy Linux (Hyprland), roda em qualquer desktop com GTK4.

Testado com Deco M5 v3.2, firmware 1.9.5.

## Recursos

- Status da internet (IP WAN, tipo de conexão)
- Topologia mesh: cada Deco, quem está ligado em quem, online/offline
- Clientes conectados com banda e velocidade, atualizando a cada 15 s com a janela em foco
- Liga/desliga Wi-Fi principal e de convidados
- Reiniciar toda a rede
- Senha guardada no keyring do sistema (Secret Service / Keychain)

## Build

Dependências: Rust stable, GTK 4.12+, libadwaita 1.5+.

```sh
# Arch / Omarchy
sudo pacman -S rust gtk4 libadwaita
# macOS (desenvolvimento)
brew install rustup gtk4 libadwaita pkg-config

cargo run -p deco-app --release
```

Para desenvolvimento, `DECO_HOST` e `DECO_PASSWORD` num `.env` pulam a tela de login (veja `.env.example`).

## Estrutura

- `crates/deco-api` — cliente da API HTTP local do Deco (login RSA + AES). `cargo run -p deco-api --example probe` faz um teste somente leitura.
- `crates/deco-app` — app GTK4/libadwaita.

## Docs

- [docs/ROADMAP.md](docs/ROADMAP.md) — o que falta fazer
- [docs/DECISIONS.md](docs/DECISIONS.md) — decisões de stack
- [docs/PROTOCOL.md](docs/PROTOCOL.md) — notas sobre o protocolo do Deco
