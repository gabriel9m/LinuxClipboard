# Plano de Testes e Melhorias de Segurança

Este documento define os testes de vulnerabilidade, os critérios de sucesso e as melhorias necessárias para reduzir riscos no LinuxClipboard.

## Regra de Avanço

Cada etapa só deve avançar para a próxima quando:

- o teste da etapa atual foi executado;
- o resultado foi registrado;
- falhas críticas ou altas foram corrigidas;
- as correções passaram em `cargo test` e `cargo test --features desktop-gtk`;
- quando houver alteração de comportamento visual, o fluxo manual foi validado no Zorin OS.

## Etapa 1 - Permissões dos Dados Locais

### Risco

O histórico e as imagens podem conter senhas, tokens, documentos privados e dados pessoais. Atualmente os arquivos podem herdar permissões permissivas do `umask`.

### Testes

```bash
./scripts/install-local.sh
stat -c '%a %U:%G %n' ~/.local/share/clipboard-history ~/.local/share/clipboard-history/history.json ~/.local/share/clipboard-history/images ~/.config/autostart/linuxclipboard.desktop ~/.local/bin/linuxclipboard
find ~/.local/share/clipboard-history -maxdepth 2 -type f -printf '%m %u:%g %p\n'
```

### Critérios de Sucesso

- `~/.local/share/clipboard-history` deve ser `700`.
- `~/.local/share/clipboard-history/images` deve ser `700`.
- `history.json` deve ser `600`.
- arquivos de imagem salvos devem ser `600`.
- temporários criados durante salvamento devem ser `600`.
- `~/.local/bin/linuxclipboard` deve ser `755`.
- `~/.config/autostart/linuxclipboard.desktop` não deve ter escrita para grupo/outros.

### Melhorias Necessárias

- Criar diretórios sensíveis com `0700`.
- Salvar histórico, imagens e temporários com `0600`.
- Corrigir permissões de arquivos já existentes durante instalação ou inicialização.
- Ajustar instalador para criar arquivos com permissões explícitas.

## Etapa 2 - Persistência de Segredos do Clipboard

### Risco

Tudo que é copiado pode ser persistido em disco, incluindo senhas, tokens, chaves privadas e códigos temporários.

### Testes

```bash
printf 'SECRET_TOKEN_TEST_123\n' | wl-copy
sleep 2
grep -R 'SECRET_TOKEN_TEST_123' ~/.local/share/clipboard-history
```

### Critérios de Sucesso

- Conteúdo com padrão óbvio de segredo não deve ser persistido.
- O app deve continuar capturando textos comuns.
- O usuário deve ter opção clara para pausar captura ou limpar histórico.

### Melhorias Necessárias

- Implementar pausa de captura ou modo privado.
- Adicionar comando para limpar histórico e imagens.
- Adicionar filtros para padrões óbvios de segredo, como:
  - `BEGIN PRIVATE KEY`;
  - tokens `ghp_`;
  - tokens `sk-`;
  - JWTs;
  - textos com `password=`;
  - códigos curtos compatíveis com 2FA.

## Etapa 3 - Auto-paste em Janela Errada

### Risco

O app simula `Ctrl+V` globalmente. Se o foco mudar após selecionar um item, o conteúdo pode ser colado no aplicativo errado.

### Testes

```bash
~/.local/bin/linuxclipboard --toggle-popup
```

Passos manuais:

- abrir o popup;
- selecionar um item sensível;
- imediatamente trocar foco para outro aplicativo antes da colagem;
- observar onde o texto é colado.

### Critérios de Sucesso

- O app não deve colar em uma janela diferente da esperada.
- Se não for possível garantir foco seguro, o auto-paste deve ser opcional ou desativável.
- Fallback manual deve continuar funcionando: item selecionado fica no clipboard para `Ctrl+V`.

### Melhorias Necessárias

- Tornar auto-paste configurável.
- Documentar o risco do auto-paste.
- Quando possível, validar janela ativa antes e depois do delay.
- Considerar fallback manual como padrão seguro.

## Etapa 4 - Execução por PATH

### Risco

`ydotool`, `wtype`, `xdotool` e `gsettings` são resolvidos por `PATH`. Um ambiente contaminado pode executar binários maliciosos.

### Testes

```bash
tmpdir="$(mktemp -d)"
printf '#!/bin/sh\necho hijacked > /tmp/linuxclipboard-path-hijack\nexit 0\n' > "$tmpdir/ydotool"
chmod +x "$tmpdir/ydotool"
PATH="$tmpdir:$PATH" XDG_SESSION_TYPE=wayland ~/.local/bin/linuxclipboard
```

Depois de ativar um item no popup:

```bash
cat /tmp/linuxclipboard-path-hijack
```

### Critérios de Sucesso

- O app não deve executar binários inesperados em diretórios não confiáveis.
- Caminhos de ferramentas externas devem ser resolvidos e validados.
- O código não deve usar shell para verificar existência de comandos.

### Melhorias Necessárias

- Remover `sh -c` da detecção de comandos.
- Resolver caminhos absolutos de ferramentas externas.
- Preferir allowlist como:
  - `/usr/bin/ydotool`;
  - `/usr/bin/wtype`;
  - `/usr/bin/xdotool`;
  - `/usr/bin/gsettings`.
- Validar dono e permissões do executável encontrado.

## Etapa 5 - Logs

### Risco

O log atual usa caminho relativo. Isso pode criar arquivos em locais inesperados e seguir symlinks se o diretório de execução for controlado.

### Testes

```bash
tmpdir="$(mktemp -d)"
cd "$tmpdir"
ln -s ~/.local/share/clipboard-history/history.json clipboard-history-debug.log
~/.local/bin/linuxclipboard --quit || true
~/.local/bin/linuxclipboard
```

### Critérios de Sucesso

- Logs devem ser gravados em caminho controlado.
- Logs não devem seguir symlinks para arquivos sensíveis.
- Logs não devem conter conteúdo bruto do clipboard.

### Melhorias Necessárias

- Mover log para `~/.local/state/clipboard-history/debug.log`.
- Criar diretório de log com `0700`.
- Criar arquivo de log com `0600`.
- Ativar log detalhado apenas por variável de ambiente ou flag de debug.

## Etapa 6 - Limites de Tamanho

### Risco

Textos ou imagens muito grandes podem consumir memória, disco e travar o daemon.

### Testes

Texto grande:

```bash
python3 -c 'print("A"*50000000)' | wl-copy
sleep 3
du -h ~/.local/share/clipboard-history/history.json
```

Imagem grande:

```bash
watch -n1 'du -sh ~/.local/share/clipboard-history/images; pgrep -a linuxclipboard'
```

### Critérios de Sucesso

- Textos acima do limite definido não devem ser persistidos.
- Imagens acima do limite definido não devem ser persistidas.
- O daemon não deve travar nem crescer indefinidamente em memória/disco.
- O usuário deve receber comportamento previsível, mesmo que silencioso inicialmente.

### Melhorias Necessárias

- Definir tamanho máximo de texto.
- Definir tamanho máximo de imagem por bytes.
- Definir dimensão máxima de imagem quando possível.
- Aplicar retenção por tamanho total em disco, não só por número de itens.

## Etapa 7 - JSON Corrompido

### Risco

Se `history.json` estiver corrompido, o app pode carregar histórico vazio e sobrescrever estado sem preservar o arquivo problemático.

### Testes

```bash
cp ~/.local/share/clipboard-history/history.json /tmp/history.backup.json
printf '{not-json' > ~/.local/share/clipboard-history/history.json
~/.local/bin/linuxclipboard --quit || true
~/.local/bin/linuxclipboard
```

### Critérios de Sucesso

- O arquivo corrompido deve ser preservado.
- O app deve criar backup com timestamp ou extensão clara.
- O app não deve sobrescrever silenciosamente o histórico anterior.
- O daemon deve continuar iniciando.

### Melhorias Necessárias

- Ao falhar parse do JSON, mover arquivo para `history.json.corrupt.<timestamp>`.
- Iniciar com histórico vazio apenas depois de preservar o corrompido.
- Registrar erro em log controlado.

## Etapa 8 - Remoção de Imagens

### Risco

Se o JSON for adulterado, o cleanup pode tentar remover arquivos fora do diretório controlado de imagens.

### Testes

```bash
tmpfile="$(mktemp)"
echo keep > "$tmpfile"
```

Depois:

- inserir manualmente um item de imagem em `history.json` apontando `content.path` ou `preview` para `$tmpfile`;
- forçar retenção com mais de 25 itens;
- verificar se `$tmpfile` foi removido.

### Critérios de Sucesso

- O cleanup deve remover apenas arquivos dentro de `~/.local/share/clipboard-history/images`.
- Caminhos absolutos ou externos vindos do JSON devem ser ignorados.
- Symlinks não devem permitir remoção fora do diretório controlado.

### Melhorias Necessárias

- Validar caminho canônico antes de remover.
- Remover apenas se o caminho estiver dentro de `images_dir`.
- Ignorar paths externos ou inválidos vindos do JSON.

## Etapa 9 - Desinstalação e Limpeza de Dados

### Risco

O desinstalador remove binário e autostart, mas mantém histórico e imagens sensíveis.

### Testes

```bash
./scripts/uninstall-local.sh
test -e ~/.local/bin/linuxclipboard; echo "binary=$?"
test -e ~/.config/autostart/linuxclipboard.desktop; echo "autostart=$?"
test -e ~/.local/share/clipboard-history/history.json; echo "history=$?"
```

### Critérios de Sucesso

- Desinstalação padrão deve remover binário e autostart.
- Deve existir opção explícita para remover dados sensíveis.
- Remoção de dados deve exigir confirmação clara.

### Melhorias Necessárias

- Adicionar opção `--purge-data` ao script de uninstall.
- Documentar que uninstall padrão preserva histórico.
- Confirmar antes de remover `~/.local/share/clipboard-history`.

## Etapa 10 - Dependências e Lint

### Risco

Vulnerabilidades em crates ou bibliotecas nativas podem entrar sem detecção automatizada.

### Testes

```bash
cargo install cargo-audit
cargo audit
cargo tree --features desktop-gtk --duplicates
```

```bash
rustup component add clippy
cargo clippy --features desktop-gtk --all-targets -- -D warnings
```

### Critérios de Sucesso

- `cargo audit` sem vulnerabilidades críticas ou altas não tratadas.
- Duplicações relevantes na árvore de dependências avaliadas.
- `cargo clippy` sem warnings.
- Critérios documentados no README ou em fluxo de CI.

### Melhorias Necessárias

- Adicionar `cargo audit` ou `cargo deny` ao fluxo de validação.
- Adicionar `cargo clippy` ao fluxo antes de release.
- Avaliar dependências nativas GTK/GDK no sistema operacional.

## Ordem Recomendada de Implementação

1. Hardening de permissões.
2. Comando de limpeza de histórico e imagens.
3. Modo privado ou pausa de captura.
4. Filtro básico para segredos óbvios.
5. Auto-paste configurável ou com fallback seguro.
6. Validação de caminhos externos e remoção segura de imagens.
7. Log em caminho controlado.
8. Limites de tamanho.
9. Tratamento de JSON corrompido.
10. Auditoria de dependências e lint no fluxo de release.
