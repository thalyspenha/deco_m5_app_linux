# Regras de negócio

Regras extraídas do código (arquivo de origem entre parênteses).

## Sessão e concorrência

- Sessões sempre curtas: login → operação → logout/close em cada job; `Drop` garante logout (`lib.rs`, `tmp.rs`). Motivo: o Deco aceita um admin por vez e derruba o app oficial.
- Um job de rede por vez: `run()` ignora o pedido se `app.busy` (`window.rs`).
  - Carregamento de páginas reagenda a cada 300 ms enquanto ocupado (`internet.rs`, `client.rs`, `deco.rs`, `blocked.rs`).
  - Escritas em páginas de detalhe avisam "Aguarde a atualização terminar e tente de novo." (`client.rs`, `deco.rs`, `blocked.rs`, `internet.rs`).
  - Exceções: switches de Wi-Fi e "Reiniciar rede" (`window.rs`) não checam `busy`; se ocupado, o pedido é descartado sem aviso (ver inconsistências em `testing.md`).
- Auto-refresh a cada 15 s somente com janela ativa e tela `main` visível (`window.rs`, `REFRESH_SECS`).

## Credenciais

- Ordem: `DECO_PASSWORD` no ambiente → usa env (`DECO_HOST` padrão `192.168.68.1`, `DECO_USERNAME` opcional, vazio = ausente); senão keyring (`credentials.rs`).
- Login só é tentado com host e senha não vazios; credenciais vão ao keyring só após um `fetch` bem-sucedido (`window.rs::submit_login`).
- E-mail informado depois (página Internet) é salvo imediatamente no keyring (`internet.rs::email_page`).
- "Sair" apaga as três chaves do keyring (`credentials::clear`).
- Na página Internet, erro `Ssh`/`Login` volta à tela de e-mail ("Não foi possível entrar com esse e-mail e a senha atual.").

## Confirmações obrigatórias (diálogo `adw::AlertDialog`)

| Ação | Confirma? | Arquivo |
|---|---|---|
| Desligar rede Wi-Fi principal | sim | `window.rs::confirm_main_off` |
| Ligar Wi-Fi principal / ligar ou desligar convidados | não | `window.rs::on_wifi_toggled` |
| Reiniciar toda a rede | sim | `window.rs::confirm_reboot` |
| Reiniciar um Deco (texto diferente se for o principal) | sim | `deco.rs::confirm_reboot` |
| Alterar DNS da WAN | sim | `internet.rs::confirm_save` |
| Bloquear cliente | sim | `client.rs::confirm_block` |
| Reservar/remover reserva de IP | não (switch direto) | `client.rs::set_reservation` |
| Desbloquear | não | `blocked.rs::unblock` |

## Wi-Fi

- Switch "ligado" se qualquer banda estiver ligada (`main_2g || main_5g`) (`window.rs::show`).
- `set_wifi` aplica o mesmo estado nas bandas `band2_4` e `band5_1`, enviando o objeto completo da rede (como lido do GET) só com `enable` trocado; bandas sem objeto são ignoradas (`tmp_config.rs`).
- Falha → o switch é revertido (`window.rs::revert`).

## DNS da WAN

- Salvar habilitado só quando há mudança e os valores são válidos: DNS primário IPv4 válido; secundário vazio ou IPv4 válido; com "DNS automático" os campos ficam desativados (`internet.rs::update_save`).
- Payload mínimo: `dial_type`, `support_mtu_size`, `enable_auto_dns`, `mtu_size`, `ip_info{dns1,dns2}` só se manual, `service_info` se existir; **nunca** `user_info` (`tmp_config.rs::set_wan_dns`).
- Após o SET espera 6 s antes de reler (aplicação assíncrona no Deco) (`internet.rs`).

## Clientes

- Só clientes `online` são listados (`models.rs::clients_of`).
- Rede: `wire_type == "wired"` → Cabo; senão por `interface` (`main`/`guest`/`iot`), outro → `Unknown` (exibido como "Principal").
- Banda: `band2_4`/`band5`/`band6` → "2.4/5/6 GHz"; vazio exibido como "Wi-Fi" (exceto cabo).
- Nomes vêm em base64; se não decodificar, usa o valor cru (`decode_name`).
- Lista principal ordenada por tráfego (↓+↑) decrescente, depois nome; no detalhe do Deco, por nome.
- Ícone escolhido por palavras-chave no nome (`client_icon`).
- Velocidade: ≥ 1024 KB/s mostrada em MB/s.
- Reserva usa o IP atual do cliente; remoção é por MAC.
- Bloqueio envia o nome em base64; após bloquear, volta à tela anterior e faz refresh.

## Decos (nós)

- Principal = `role == "master"`; listado primeiro.
- Online = `inet_status == "online"`; internet "Conectado" na tela principal = principal online.
- Sinal do backhaul: maior entre bandas; `None` no principal. Rótulos: 0 "—", 1 "Fraco", 2 "Regular", ≥3 "Bom" (escala inferida, ver PROTOCOL.md).
- Firmware exibido no grupo "Decos" = primeira palavra do firmware do primeiro nó (o principal).

## Testes contra a rede real (regra do projeto, `CLAUDE.md`)

- Leitura livre; qualquer escrita só com confirmação do usuário, a cada vez; reverter no mesmo teste.
- `reboot_node_test` recusa o Deco principal; `tmp_probe` só aceita opcodes GET.
