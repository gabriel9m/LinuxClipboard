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
```

No Zorin OS 17.3, `libgtk-4-dev` fornece as bibliotecas nativas usadas pelo crate `gtk4`.
