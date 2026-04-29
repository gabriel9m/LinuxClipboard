# Changelog

## v0.1.0 - 2026-04-29

Primeira versão mínima usável validada no Zorin OS.

### Adicionado

- Aplicação GTK4 residente para histórico da área de transferência.
- Comando instalado como `linuxclipboard`.
- Popup acionável por `--toggle-popup`, `--show-popup` e `--quit`.
- Histórico persistente de textos em `~/.local/share/clipboard-history/history.json`.
- Suporte a imagens com miniaturas e arquivos em `~/.local/share/clipboard-history/images`.
- Navegação por teclado com setas, `Enter` e `Esc`.
- Ativação por clique do mouse.
- Colagem automática usando backend disponível no sistema.
- Itens fixados com pin, preservados na retenção do histórico.
- Interface adaptada para tema claro/escuro do Zorin/GNOME.
- Script de instalação local com autostart de sessão.
- Script de desinstalação local.

### Validado

- Instalação local.
- Atalho `Super+V`.
- Popup abrindo e fechando.
- Captura de textos e imagens.
- Ordenação do histórico por item mais recente.
- Navegação por teclado e mouse.
- Colagem automática em editor de texto.
- Pin visual e persistência de itens fixados.
- Troca de paleta entre tema claro e escuro.
- Autostart após login.
