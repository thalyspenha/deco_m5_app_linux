# Roadmap

## Feito

- [x] Prova de conceito em Python (tplinkrouterc6u) — substituída pelo Rust, ver commit 2aea33a
- [x] `deco-api` em Rust: login RSA/AES, nós, clientes, WAN, estado do Wi-Fi, set_wifi, reboot
- [x] GUI GTK4/libadwaita v0: login com keyring, status, topologia, clientes, switches de Wi-Fi, reiniciar
- [x] Cliente TMP (`deco_api::tmp`): SSH 20001 + framing TMP + app v2, GETs validados (WAN/DNS, LAN, DHCP, reservas, port forwarding, bloqueados)

## A fazer

1. [ ] **Interface gráfica** (em andamento)
   - [x] Validar visual no Mac
   - [ ] Validar visual no Omarchy (pelo usuário)
   - [ ] Renomear cliente (descobrir opcode)
   - [x] Detalhe do Deco: papel, Deco anterior, sinal do backhaul, modelo/IP/MAC/firmware, clientes do nó
   - [ ] Validar "Reiniciar este Deco" (`Session::reboot_node`) na rede real
   - [ ] Ícone e `.desktop`, pacote AUR
2. [ ] **Testar `wifi` on/off** na rede real (mexe na rede, precisa confirmação).
3. [ ] **Configuração via TMP** (opcodes em `docs/PROTOCOL.md`)
   - [x] Primeiro SET de baixo risco: criar/remover reserva de IP (envelope `params` + `config_version`)
   - [x] API tipada (`tmp_config.rs`); validar SETs de port forwarding e bloqueio
   - [x] Tela "Conexão de internet" (tipo, IPv4, DNS editável, MTU); salvar DNS validado na rede real
   - [x] Detalhe do cliente: IP, MAC, rede, Deco conectado, reservar IP, bloquear
   - [x] Página de dispositivos bloqueados com desbloquear
   - [ ] Port forwarding na GUI (validar SET antes)
   - [ ] DNS/WAN e LAN/DHCP (risco de cortar rede; testar com cuidado)
   - [ ] `DECO_USERNAME` (e-mail TP-Link ID) no login da GUI/keyring

## Pendências conhecidas

- Clientes Daikin vêm sem `connection_type` (banda vazia); GUI mostra "Wi-Fi".
- HTTP (`admin/...?form=`) não aceita escrita de WAN/DNS (`code 1`); usar TMP.
- Aviso `g_settings_schema_source_lookup` no macOS (schemas GSettings do brew), inofensivo.
