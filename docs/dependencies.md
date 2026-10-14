# Dependências

Versões declaradas nos `Cargo.toml`; versão travada no `Cargo.lock` entre parênteses.

## `deco-api`

| Crate | Declarada | Uso |
|---|---|---|
| `reqwest` | 0.12 (0.12.28), sem features padrão + `blocking`, `json` | HTTP LuCI |
| `ssh2` | 0.9 (0.9.6) → `libssh2-sys` 0.3.x, `openssl-sys` | SSH do TMP |
| `rsa` | 0.9 (0.9.10) | RSA PKCS#1 v1.5 (senha e assinatura) |
| `aes` | 0.8 (0.8.4) + `cbc` 0.1 (0.1.2, `alloc`) | AES-128-CBC do payload |
| `md-5` | 0.10 | hash `md5("admin"+senha)` na assinatura |
| `sha1` | 0.10 | usuário SSH do TMP |
| `crc32fast` | 1 | CRC32 do TMP e da camada app |
| `base64` | 0.22 | payload AES, nomes de clientes |
| `hex` | 0.4 | saída RSA, chaves AES, trace |
| `rand` | 0.8 | chave/IV AES, padding RSA |
| `form_urlencoded` | 1 | corpo `sign=&data=` |
| `serde` / `serde_json` | 1 | JSON (`serde_json::Value` é usado direto; não há derive de `Deserialize` nas structs) |
| `thiserror` | 2 (2.0.21) | `Error` |
| `dotenvy` (dev) | 0.15 | `.env` nos exemplos |

## `deco-app`

| Crate | Declarada | Uso |
|---|---|---|
| `gtk4` (como `gtk`) | 0.10 (0.10.3), feature `v4_12` | UI |
| `libadwaita` (como `adw`) | 0.8 (0.8.1), feature `v1_5` | UI (`AlertDialog`, `SwitchRow`, `NavigationView`, `ToolbarView`) |
| `keyring` | 3 (3.6.3), `apple-native`, `sync-secret-service`, `crypto-rust` | credenciais |
| `dotenvy` | 0.15 (0.15.7) | `.env` |
| `deco-api` | path | cliente |

## Dependências de sistema

- Build: Rust stable, `pkg-config`, OpenSSL (headers), GTK 4.12+, libadwaita 1.5+.
- Runtime (PKGBUILD): `gtk4`, `libadwaita`, `openssl`, `dbus` (Secret Service).
- macOS (dev): `adwaita-icon-theme` via brew.
