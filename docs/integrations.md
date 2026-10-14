# Integrações externas

| Integração | Como | Código | Observações |
|---|---|---|---|
| Deco — API HTTP LuCI | HTTP `:80`, `reqwest::blocking`, RSA + AES | `deco-api/src/lib.rs`, `models.rs`, `crypto.rs` | Leitura + reboot. Escrita de Wi-Fi via HTTP falha no firmware 1.9.5 (HTTP 500) |
| Deco — TMP | SSH `:20001` (crate `ssh2`/libssh2 com OpenSSL) + túnel para `:20002` | `deco-api/src/tmp.rs`, `tmp_config.rs` | Protocolo obtido por engenharia reversa do APK `com.tplink.tpm5` |
| Keyring do sistema | crate `keyring` 3 (`sync-secret-service` no Linux, `apple-native` no macOS) | `deco-app/src/credentials.rs` | Serviço `deco-linux` |
| `.env` | crate `dotenvy` | `deco-app/src/main.rs`, exemplos | Só no diretório atual; desenvolvimento |
| Desktop Linux | `.desktop` + ícone hicolor, `APP_ID` = class no Hyprland | `data/`, `main.rs` | — |
| AUR | `PKGBUILD` (`deco-linux-git`) clonando do GitHub | `packaging/aur/` | Ainda não publicado (ROADMAP) |

## Não utilizados

- **Nuvem TP-Link / TP-Link ID online**: não há chamada a serviços da TP-Link na internet. O e-mail do TP-Link ID só é usado localmente para derivar o usuário SSH (SHA-1).
- Telemetria, analytics, mensageria, serviços de pagamento ou e-mail: **não existem**.

## Compatibilidade

Testado somente com Deco M5 v3.2, firmware 1.9.5 (README, PROTOCOL.md). Outros modelos: não verificado.
