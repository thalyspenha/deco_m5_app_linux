# Roadmap

## Feito

- [x] Prova de conceito em Python (tplinkrouterc6u) — substituída pelo Rust, ver commit 2aea33a
- [x] `deco-api` em Rust: login RSA/AES, nós, clientes, WAN, estado do Wi-Fi, set_wifi, reboot
- [x] GUI GTK4/libadwaita v0: login com keyring, status, topologia, clientes, switches de Wi-Fi, reiniciar
- [x] Cliente TMP (`deco_api::tmp`): SSH 20001 + framing TMP + app v2, GETs validados (WAN/DNS, LAN, DHCP, reservas, port forwarding, bloqueados)

## A fazer

1. [ ] **Interface gráfica** (em andamento)
   - [x] Validar visual no Mac
   - [x] Validar visual no Omarchy (2026-10-13, PC no cabo aparece como "Cabo")
   - [ ] Renomear cliente (descobrir opcode)
   - [x] Detalhe do Deco: papel, Deco anterior, sinal do backhaul, modelo/IP/MAC/firmware, clientes do nó
   - [x] "Reiniciar este Deco" validado na rede real (~2 min fora, só o nó)
   - [x] Ícone e `.desktop` (`data/`), `PKGBUILD` em `packaging/aur/`
   - [ ] Publicar no AUR (`deco-linux-git`; requer repo público no GitHub)
2. [ ] **Wi-Fi on/off** via TMP (`0x4008`; HTTP quebrado no firmware 1.9.5)
   - [x] Rede de convidados validada na rede real
   - [ ] Rede principal (derruba clientes Wi-Fi; testar só de máquina no cabo)
   - [ ] Switches na GUI validados (exigem e-mail do TP-Link ID)
3. [ ] **Configuração via TMP** (opcodes em `docs/PROTOCOL.md`)
   - [x] Primeiro SET de baixo risco: criar/remover reserva de IP (envelope `params` + `config_version`)
   - [x] API tipada (`tmp_config.rs`); bloqueio validado, port forwarding pendente
   - [x] Tela "Conexão de internet" (tipo, IPv4, DNS editável, MTU); salvar DNS validado na rede real
   - [x] Detalhe do cliente: IP, MAC, rede, Deco conectado, reservar IP, bloquear
   - [x] Página de dispositivos bloqueados com desbloquear
   - [ ] Port forwarding na GUI (validar SET antes)
   - [x] DNS da WAN (validado)
   - [ ] LAN/DHCP (`set_dhcp` não validado; risco de renumerar clientes)
   - [x] `DECO_USERNAME` (e-mail TP-Link ID) no login da GUI/keyring

## Pendências conhecidas

- Clientes Daikin vêm sem `connection_type` (banda vazia); GUI mostra "Wi-Fi".
- HTTP (`admin/...?form=`) não aceita escrita de WAN/DNS (`code 1`); usar TMP.
- Aviso `g_settings_schema_source_lookup` no macOS (schemas GSettings do brew), inofensivo.
