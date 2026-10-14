# Testes

## Testes automatizados

Apenas dois testes unitários, em `crates/deco-api/src/crypto.rs`:

- `aes_roundtrip`: cifra/decifra com `AesSession`.
- `key_string_is_hex`: formato de `k=<16>&i=<16>`.

Rodar: `cargo test -p deco-api`.

Não há testes de `Session`, `TmpSession` (framing, CRC, fragmentação), mapeamento JSON → structs, nem da GUI. Não há mocks/fixtures do Deco nem CI (ver `infrastructure.md`).

## Testes manuais contra a rede real (`crates/deco-api/examples/`)

Leem `.env` (`DECO_HOST`, `DECO_PASSWORD`, `DECO_USERNAME`).

| Exemplo | Tipo | O que faz |
|---|---|---|
| `probe [path [params-json]]` | leitura (HTTP) | Imprime WAN, Wi-Fi, nós, clientes; ou JSON cru de um endpoint |
| `tmp_probe [typed \| <opcode hex>]` | leitura (TMP) | GETs; recusa opcodes fora da lista de GET |
| `tmp_reservation_test [cliente]` | escrita, reverte | Reserva o IP atual do cliente e remove (padrão `MacBookAir`) |
| `tmp_block_test <cliente>` | escrita, reverte | Bloqueia e desbloqueia |
| `tmp_dns_test` | escrita, reverte | Troca DNS para 8.8.8.8/8.8.4.4 e restaura |
| `wifi_test [guest\|main]` | escrita, reverte | Inverte uma rede Wi-Fi e restaura (`main` só de máquina no cabo) |
| `reboot_node_test "<deco>"` | escrita | Reinicia um satélite; recusa o principal |

Regra: escrita só com confirmação do usuário a cada vez; resultado registrado em `PROTOCOL.md`/`ROADMAP.md`.

## Verificação da GUI

Manual, por screenshot (instruções em `CLAUDE.md`), com `DECO_OPEN` para abrir subpáginas.

## Lacunas e inconsistências observadas no código

Verificação cruzada feita ao escrever esta documentação (sem alterar código):

1. `tmp_config.rs`: os doc-comments de `block_client` e `unblock_client` dizem "Not validated yet", mas `PROTOCOL.md` e `ROADMAP.md` registram block/unblock como validados em 2026-10-12.
2. `window.rs::apply_wifi` e `confirm_reboot` não checam `app.busy`; durante um refresh o pedido é descartado por `run()` sem aviso, e o switch de Wi-Fi fica na posição nova sem ser revertido. As páginas de detalhe tratam esse caso.
3. A rede Wi-Fi principal via TMP ainda não foi validada na rede real (ROADMAP), mas o switch já está ativo na GUI.
4. A descrição do crate `deco-api` (`Cargo.toml`: "Client for the TP-Link Deco local HTTP API") não menciona o TMP.
