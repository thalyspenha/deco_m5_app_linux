# deco-linux

App GTK4/libadwaita para gerenciar redes mesh TP-Link Deco pela rede local, sem depender do app oficial Android/iOS. Feito para Omarchy Linux (Hyprland), roda em qualquer desktop com GTK4.

Fala direto com os Decos, sem nuvem: a API HTTP local (a mesma da interface web) e o **TMP**, o protocolo que o app oficial usa para configurações, implementado aqui a partir do app Android.

## Recursos

**Visão geral**
- Status da internet (IP WAN, tipo de conexão)
- Topologia mesh: cada Deco, a quem está ligado, online/offline
- Clientes conectados com rede, banda e velocidade, atualizando a cada 15 s com a janela em foco
- Liga/desliga Wi-Fi principal e de convidados
- Reiniciar toda a rede

**Conexão de internet**
- Tipo de conexão, usuário PPPoE, IP, máscara, gateway, MAC, MTU
- DNS automático ou manual, com validação e confirmação

**Detalhe do cliente**
- IP, MAC, rede/banda e em qual Deco está conectado
- Reservar o endereço IP atual
- Bloquear o dispositivo; página de dispositivos bloqueados para desbloquear

**Detalhe do Deco**
- Papel (principal/satélite), Deco anterior e sinal do backhaul por banda
- Modelo, IP, MAC, firmware
- Clientes ligados a esse Deco
- Reiniciar só esse Deco

Credenciais ficam no keyring do sistema (Secret Service no Linux, Keychain no macOS).

## Compatibilidade

| Equipamento | Situação |
|---|---|
| Deco M5 v3.2, firmware 1.9.5 | Testado: tudo acima validado na rede real |
| Outros Decos (M4, X20, X50, X55, E4…) | Esperado funcionar: mesma API local e mesmo TMP da linha mesh. Campos ausentes aparecem como "—". Não testado |
| Roteadores Archer (app Tether) | Não funciona: endpoints HTTP diferentes e login do TMP diferente. Exigiria um backend próprio |

Com outro modelo Deco, os testes somente leitura abaixo mostram rápido o que funciona. Relatos são bem-vindos.

## Acesso

- **Senha**: a senha local do Deco, que é a mesma da conta TP-Link ID dona da rede.
- **E-mail do TP-Link ID** (opcional): necessário para as configurações via TMP (conexão de internet, reserva de IP, bloqueio). Sem ele o app funciona só com a visão geral.

O Deco aceita **uma sessão de administrador por vez**: cada atualização do app derruba momentaneamente a sessão do app oficial, e vice-versa. Por isso o app faz login e logout a cada operação e só atualiza com a janela em foco.

## Build

Dependências: Rust stable, GTK 4.12+, libadwaita 1.5+, OpenSSL e pkg-config (para o cliente SSH do TMP).

```sh
# Arch / Omarchy
sudo pacman -S rust gtk4 libadwaita openssl pkgconf
# macOS (desenvolvimento)
brew install rustup gtk4 libadwaita adwaita-icon-theme pkg-config

cargo run -p deco-app --release
```

No macOS, rode com `XDG_DATA_DIRS=/opt/homebrew/share` se os ícones aparecerem quebrados (alguns terminais, como o Ghostty, sobrescrevem essa variável).

## Desenvolvimento

Copie `.env.example` para `.env` e preencha: com `DECO_PASSWORD` (e `DECO_USERNAME` para o TMP) o app pula a tela de login.

```sh
# Somente leitura
cargo run -p deco-api --example probe                       # nós, clientes, WAN, Wi-Fi (HTTP)
cargo run -p deco-api --example probe "admin/...?form=..."  # JSON cru de um endpoint HTTP
cargo run -p deco-api --example tmp_probe typed             # WAN, DHCP, reservas, port forwarding, bloqueados (TMP)
cargo run -p deco-api --example tmp_probe 4004              # JSON cru de um opcode GET do TMP

# Alteram a rede (revertem no fim, mas use com cuidado)
cargo run -p deco-api --example tmp_reservation_test [cliente]
cargo run -p deco-api --example tmp_block_test <cliente>
cargo run -p deco-api --example tmp_dns_test
cargo run -p deco-api --example reboot_node_test "<deco satélite>"
```

`DECO_OPEN=internet|blocked|client:<nome>|deco:<nome>` abre uma subpágina direto ao iniciar (útil para screenshots).

## Estrutura

- `crates/deco-api` — cliente dos Decos
  - `lib.rs`, `models.rs`: API HTTP local (login RSA + AES, nós, clientes, Wi-Fi, reboot)
  - `tmp.rs`: TMP (SSH na porta 20001, framing TMP, camada de app v2)
  - `tmp_config.rs`: configurações tipadas sobre o TMP (WAN/DNS, DHCP, reservas, port forwarding, bloqueio)
- `crates/deco-app` — app GTK4/libadwaita: janela principal e páginas de internet, cliente, Deco e bloqueados

## Docs

- [docs/ROADMAP.md](docs/ROADMAP.md) — o que já foi feito e o que falta
- [docs/DECISIONS.md](docs/DECISIONS.md) — decisões de stack
- [docs/PROTOCOL.md](docs/PROTOCOL.md) — protocolo: API HTTP, TMP, opcodes e payloads validados

## Aviso

Projeto independente, sem relação com a TP-Link. Usa protocolos não documentados que podem mudar com atualizações de firmware. Mudanças de rede (DNS, bloqueio, reboot) pedem confirmação no app; use por sua conta.

## Licença

MIT
