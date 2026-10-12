# Protocolo do Deco

## API HTTP local (usada hoje)

- Endpoint: `http://<deco>/cgi-bin/luci/;stok=/...`, login com a senha do admin local.
- **Uma sessão admin por vez**: logar derruba o app oficial e vice-versa. Sempre fazer logout logo após a operação (`Session` faz logout no drop).
- Implementação em Rust: `crates/deco-api`. Referência original: `tplinkrouterc6u.TPLinkDecoClient`.
- Cobre: firmware, nós mesh (topologia, papel, status de internet), clientes (IP, MAC, banda, velocidade), status WAN, liga/desliga Wi-Fi host/guest/IoT, reboot.

## TMP (TP-Link Mesh Protocol)

- SSH na porta 20001, usado pelo app oficial para parte das configurações.
- Sem documentação pública. Não aparece em mitmproxy (não é HTTP).
- Caminho realista: decompilar o APK Android com jadx.

## Nuvem TP-Link

- Usada pelo app para acesso remoto (TP-Link ID). Fora do escopo.

## Rede de referência

- Deco M5 v3.2, firmware 1.9.5 Build 20260602.
- 3 nós: Bedroom (master, 192.168.68.1) → Quarto mãe → Living Room (cadeia).
