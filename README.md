# LinuxClipboard

Aplicação desktop para histórico da área de transferência no Linux, com foco inicial em Zorin OS 17.3.

## Desenvolvimento

### Testes padrão

```bash
cargo test
```

Os testes padrão não exigem GTK instalado, porque as integrações desktop ficam atrás de feature opcional.

### Integração GTK4

Para compilar a feature desktop:

```bash
sudo apt-get update
sudo apt-get install -y pkg-config libgtk-4-dev
cargo check --features desktop-gtk
cargo run --features desktop-gtk
```

No Zorin OS 17.3, `libgtk-4-dev` fornece as bibliotecas nativas usadas pelo crate `gtk4`.

### Execução residente

O comando padrão inicia o daemon oculto, mantendo o monitor do clipboard ativo:

```bash
cargo run --features desktop-gtk
```

Em outro terminal, use o comando remoto abaixo para mostrar ou esconder o popup da instância já aberta:

```bash
cargo run --features desktop-gtk -- --toggle-popup
```

Para abrir diretamente sem alternar:

```bash
cargo run --features desktop-gtk -- --show-popup
```

Para encerrar a instância residente:

```bash
cargo run --features desktop-gtk -- --quit
```

No Wayland, o atalho global `Super+V` deve ser configurado no ambiente desktop para executar o comando `--toggle-popup`.

No Zorin OS, crie um atalho personalizado em Configurações > Teclado > Atalhos personalizados:

Nome:

```text
LinuxClipboard
```

Comando:

```bash
bash -lc 'cd /home/gabriel/projetos/clipboard-history && cargo run --features desktop-gtk -- --toggle-popup'
```

Atalho:

```text
Super+V
```

Esse fluxo foi validado no ambiente de desenvolvimento: o daemon fica residente, e `Super+V` alterna a visibilidade do popup.

### Auto-paste

Ao ativar um item com clique ou `Enter`, o app sempre escreve o conteúdo selecionado no clipboard. Em seguida, ele tenta colar automaticamente disparando `Ctrl+V` por uma ferramenta disponível no sistema:

- Wayland: tenta `ydotool` primeiro e `wtype` depois.
- X11: tenta `xdotool`.
- Sem ferramenta compatível: mantém o fallback manual, ou seja, o item fica no clipboard e pode ser colado com `Ctrl+V`.

No ambiente atual de desenvolvimento, a sessão é Wayland. Para auto-paste real no Wayland/GNOME, a opção mais provável é instalar e habilitar `ydotool`.

Na versão `ydotool 0.1.8`, o comando usado para simular `Ctrl+V` é:

```bash
ydotool key ctrl+v
```
