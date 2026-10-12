# deco-linux

Controle local de redes mesh TP-Link Deco (testado com Deco M5 v3.2, firmware 1.9.5) a partir do Linux, sem depender do app oficial Android/iOS.

Fala direto com a API HTTP local do Deco (login RSA/AES) via [`tplinkrouterc6u`](https://pypi.org/project/tplinkrouterc6u/).

## Setup

```sh
uv sync
cp .env.example .env   # preencha DECO_HOST e DECO_PASSWORD
```

## CLI

```sh
uv run deco status              # resumo da rede
uv run deco nodes               # lista os decos e a topologia mesh
uv run deco clients             # clientes conectados, banda e velocidade
uv run deco wifi guest on|off   # host / guest / iot
uv run deco reboot --yes        # reinicia toda a rede mesh
```

## Docs

- [docs/ROADMAP.md](docs/ROADMAP.md) — o que falta fazer
- [docs/PROTOCOL.md](docs/PROTOCOL.md) — notas sobre como o Deco se comunica
