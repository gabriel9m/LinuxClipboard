# LinuxClipboard

Aplicação desktop para histórico da área de transferência no Linux, com foco inicial em Zorin OS 17.3.

## Status

MVP `v0.1.0` validado manualmente no Zorin OS com:

- instalação local em `~/.local/bin/linuxclipboard`;
- daemon residente com autostart de sessão;
- atalho `Super+V`;
- histórico de textos e imagens;
- navegação por teclado e mouse;
- colagem automática;
- itens fixados com pin;
- tema claro/escuro.

## Instalação Local

O fluxo recomendado para uso diário é instalar o binário localmente, em vez de depender de `cargo run`:

```bash
./scripts/install-local.sh
```

Esse script:

- compila o app em modo `release` com a feature GTK;
- instala o executável em `~/.local/bin/linuxclipboard`;
- cria o autostart em `~/.config/autostart/linuxclipboard.desktop`;
- mostra o comando correto para configurar o atalho `Super+V`.

Depois da instalação, inicie ou reinicie o daemon:

```bash
~/.local/bin/linuxclipboard --quit || true
setsid ~/.local/bin/linuxclipboard >/tmp/linuxclipboard.log 2>&1 < /dev/null &
```

Para remover a instalação local:

```bash
./scripts/uninstall-local.sh
```

O histórico fica em:

```text
~/.local/share/clipboard-history/history.json
```

As imagens ficam em:

```text
~/.local/share/clipboard-history/images
```

## Uso Diário

O comando padrão inicia o daemon oculto, mantendo o monitor do clipboard ativo:

```bash
linuxclipboard
```

Para mostrar ou esconder o popup da instância residente:

```bash
linuxclipboard --toggle-popup
```

Para abrir diretamente sem alternar:

```bash
linuxclipboard --show-popup
```

Para encerrar a instância residente:

```bash
linuxclipboard --quit
```

No Zorin OS, crie um atalho personalizado em Configurações > Teclado > Atalhos personalizados:

Nome:

```text
LinuxClipboard
```

Comando:

```bash
/home/gabriel/.local/bin/linuxclipboard --toggle-popup
```

Atalho:

```text
Super+V
```

Se `~/.local/bin` estiver no `PATH`, o comando também pode ser:

```bash
linuxclipboard --toggle-popup
```

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

### Execução residente em desenvolvimento

Durante desenvolvimento, ainda é possível rodar sem instalar:

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

No Wayland, o atalho global `Super+V` deve ser configurado no ambiente desktop para executar o comando `--toggle-popup`. Para uso diário, prefira o binário instalado em `~/.local/bin/linuxclipboard`.

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

### Imagens

O domínio e a integração GTK suportam imagens no histórico:

- quando o clipboard não contém texto, o app tenta ler uma textura GTK;
- texturas copiadas são convertidas para PNG e persistidas em `~/.local/share/clipboard-history/images`;
- itens de imagem aparecem no popup apenas como miniatura;
- ao selecionar uma imagem, o app carrega o PNG salvo, coloca a textura no clipboard e tenta o auto-paste.

### Interface

O popup GTK usa CSS de aplicação com cores do tema ativo do Zorin/GNOME:

- cabeçalho compacto com instrução de uso;
- cantos arredondados e bordas discretas;
- linhas com hover, foco e seleção visíveis;
- miniaturas de imagem sem texto lateral;
- suporte natural a tema claro/escuro via cores do tema GTK.
