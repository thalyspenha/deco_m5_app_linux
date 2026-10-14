# Desenvolvimento

## Rodar e ver a GUI

`DECO_OPEN=internet|blocked|client:<nome>|deco:<nome>` abre uma subpágina direto (cliques simulados não funcionam no Mac). A janela pode levar ~10–20 s para aparecer: fazer polling.

### Linux (Omarchy/Hyprland)

- Deps: `sudo pacman -S rust gtk4 libadwaita openssl pkgconf grim jq`.
- Rodar: `cargo run -p deco-app` (ícones Adwaita já vêm do sistema).
- Screenshot da janela: `hyprctl clients -j | jq -r '.[] | select(.class=="io.github.thalys.DecoLinux") | "\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"'` e `grim -g "<geom>" <arquivo>.png`.

### NixOS

- `shell.nix` traz Rust (cargo, rustc, clippy, rustfmt, rust-analyzer), pkg-config, GTK4, libadwaita, OpenSSL, dbus e ícones Adwaita; exporta `XDG_DATA_DIRS` com schemas GSettings e ícones.
- Com direnv: `direnv allow` uma vez (`.envrc` = `use nix`). Sem direnv: `nix-shell`.
- Rodar: `cargo run -p deco-app`.

### macOS

- cargo vem do rustup do brew: `export PATH=/opt/homebrew/opt/rustup/bin:$PATH`. Requer `brew install adwaita-icon-theme`.
- Rodar com `XDG_DATA_DIRS=/opt/homebrew/share ./target/debug/deco-app &` (Ghostty sobrescreve XDG_DATA_DIRS e os ícones quebram).
- Screenshot: `kCGWindowNumber` da janela de 460 px via `CGWindowListCopyWindowInfo(.optionAll, …)` (swift -e), depois `screencapture -x -o -l <id>` (não usar `-R`, captura o que estiver na frente). Se vier só wallpaper, falta permissão de gravação de tela.

## Depurar o TMP

`TMP_DEBUG=1` imprime cada pacote TMP enviado/recebido em hex no stderr.

## Engenharia reversa

APK do app Deco (`com.tplink.tpm5`, APKMirror, bundle `.apkm`): `unzip` (o `.apkm` é um zip com `base.apk`) e `jadx --no-res -d <dir> base.apk` numa pasta temporária fora do repo. Wi-Fi em `component/more/wireless/repository`. Opcodes em `com.tplink.libtpnetwork.mesh.global.p`, beans em `com.tplink.tpm5.component.more.*/repository/bo`.

## Empacotamento

Ver [infrastructure.md](infrastructure.md). Após mudar o PKGBUILD, regenerar `.SRCINFO` com `makepkg --printsrcinfo > .SRCINFO`. O PKGBUILD compila do GitHub: só vê o que foi pushado.
