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
