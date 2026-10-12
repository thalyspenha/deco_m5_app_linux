# Roadmap

## Feito

- [x] Prova de conceito em Python (tplinkrouterc6u) — substituída pelo Rust, ver commit 2aea33a
- [x] `deco-api` em Rust: login RSA/AES, nós, clientes, WAN, estado do Wi-Fi, set_wifi, reboot
- [x] GUI GTK4/libadwaita v0: login com keyring, status, topologia, clientes, switches de Wi-Fi, reiniciar

## A fazer

1. [ ] **Interface gráfica** (em andamento)
   - [ ] Validar visual no Mac e no Omarchy (screenshot via screencapture no Mac; no Omarchy pelo usuário)
   - [ ] Detalhe do cliente: renomear, bloquear, ver qual Deco está conectado
   - [ ] Detalhe do Deco: sinal, clientes por nó, reiniciar só um
   - [ ] Ícone e `.desktop`, pacote AUR
2. [ ] **Testar `wifi` on/off** na rede real (mexe na rede, precisa confirmação).
3. [ ] **Explorar a API local além do implementado**: bloquear cliente, reserva de IP, port forwarding, QoS. Usar `cargo run -p deco-api --example probe <path>` para dumps. Se faltar, investigar TMP (SSH porta 20001) via APK decompilado (jadx).

## Pendências conhecidas

- Clientes Daikin vêm sem `connection_type` (banda vazia).
- Aviso `g_settings_schema_source_lookup` no macOS (schemas GSettings do brew), inofensivo.
