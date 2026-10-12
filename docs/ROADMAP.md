# Roadmap

## Feito

- [x] Prova de conceito: login na API HTTP local, leitura de firmware, nós mesh e clientes
- [x] CLI `deco` (`status`, `nodes`, `clients`, `wifi`, `reboot`)
- [x] Sessão curta (login → ação → logout) para não derrubar o app oficial

## A fazer

1. [ ] **Interface gráfica** (prioridade atual) — app nativo para Omarchy/Hyprland. Ver decisão de stack em [DECISIONS.md](DECISIONS.md).
2. [ ] **Testar `wifi guest on/off`** na rede real (ainda não executado; mexe na rede).
3. [ ] **Explorar a API local além da lib**: bloquear cliente, reserva de IP, port forwarding, QoS. Se faltar, investigar TMP (SSH porta 20001) via APK decompilado (jadx).

## Pendências conhecidas

- Os clientes Daikin aparecem com banda `unknown` (talvez rede IoT não mapeada pela lib).
- `clients_total` do roteador (4) diverge da lista de dispositivos (6); a CLI conta pela lista.
