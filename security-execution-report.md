# Relatório de Execução do Plano de Segurança

Data: 2026-04-29

## Resumo

Foram executadas as etapas não visuais do `security-test-plan.md` e aplicadas correções de hardening para reduzir exposição local de dados sensíveis, integridade do storage e riscos de execução externa.

## Alterações Implementadas

### Permissões de dados locais

- Diretórios de dados sensíveis agora são criados/corrigidos com `0700`.
- `history.json`, imagens salvas e temporários agora são criados/corrigidos com `0600`.
- A inicialização da aplicação chama hardening de permissões para dados existentes.
- O instalador local corrige permissões do diretório de dados, imagens, histórico e autostart.

### Segredos no clipboard

- Adicionado filtro básico para ignorar textos com padrões óbvios de segredo:
  - chaves privadas;
  - tokens GitHub `ghp_` e `github_pat_`;
  - tokens `sk-`;
  - JWTs;
  - `password=`, `passwd=`, `secret=`, `api_key=`;
  - códigos numéricos curtos compatíveis com 2FA.
- Textos comuns continuam sendo capturados.

### Limpeza de histórico

- Adicionado comando:

```bash
linuxclipboard --clear-history
```

- O comando limpa o histórico persistido e remove imagens associadas.
- README atualizado com o novo comando.

### Auto-paste

- Auto-paste continua habilitado por padrão para preservar a UX validada.
- Adicionada configuração para desativar:

```bash
LINUXCLIPBOARD_AUTO_PASTE=0 linuxclipboard
```

- Quando desativado, o item ainda é escrito no clipboard e pode ser colado manualmente com `Ctrl+V`.

### Execução por PATH

- Removido uso de `sh -c command -v`.
- Auto-paste agora usa caminhos absolutos allowlisted:
  - `/usr/bin/ydotool`;
  - `/usr/bin/wtype`;
  - `/usr/bin/xdotool`.

### Logs

- Log movido de caminho relativo para:

```text
~/.local/state/clipboard-history/debug.log
```

- Diretório de log criado com `0700`.
- Arquivo de log criado/corrigido com `0600`.
- O app evita gravar se o caminho final do log for symlink.

### Limites de tamanho

- Texto acima de `1_000_000` bytes não é persistido.
- Imagem acima de `10_000_000` bytes não é persistida.

### JSON corrompido

- `history.json` corrompido agora é preservado como:

```text
history.json.corrupt.<timestamp>
```

- O daemon continua iniciando com histórico vazio após preservar o arquivo corrompido.

### Remoção segura de imagens

- Cleanup de imagens agora remove apenas arquivos dentro do diretório controlado de imagens.
- Caminhos externos vindos do JSON são ignorados.
- Symlinks apontando para fora do diretório de imagens são ignorados.

### Desinstalação

- `./scripts/uninstall-local.sh` continua removendo binário e autostart.
- Dados persistidos são preservados por padrão.
- Adicionada opção explícita:

```bash
./scripts/uninstall-local.sh --purge-data
```

## Testes Executados

### Automatizados

```bash
cargo fmt --check
cargo test
cargo test --features desktop-gtk
cargo check --features desktop-gtk --bin linuxclipboard
bash -n scripts/install-local.sh
bash -n scripts/uninstall-local.sh
```

Resultados:

- `cargo test`: 70 testes passaram.
- `cargo test --features desktop-gtk`: 80 testes passaram.
- `cargo check --features desktop-gtk --bin linuxclipboard`: passou.
- `bash -n` dos scripts: passou.

### Instalação e permissões

```bash
./scripts/install-local.sh
stat -c '%a %U:%G %n' ~/.local/share/clipboard-history ~/.local/share/clipboard-history/history.json ~/.local/share/clipboard-history/images ~/.config/autostart/linuxclipboard.desktop ~/.local/bin/linuxclipboard
find ~/.local/share/clipboard-history -maxdepth 2 -type f -printf '%m %u:%g %p\n'
```

Resultado observado:

```text
700 gabriel:gabriel /home/gabriel/.local/share/clipboard-history
600 gabriel:gabriel /home/gabriel/.local/share/clipboard-history/history.json
700 gabriel:gabriel /home/gabriel/.local/share/clipboard-history/images
600 gabriel:gabriel /home/gabriel/.config/autostart/linuxclipboard.desktop
755 gabriel:gabriel /home/gabriel/.local/bin/linuxclipboard
```

Arquivos em `images` também ficaram com `600`.

### Log controlado

```bash
stat -c '%a %U:%G %n' ~/.local/state/clipboard-history ~/.local/state/clipboard-history/debug.log
```

Resultado observado:

```text
700 gabriel:gabriel /home/gabriel/.local/state/clipboard-history
600 gabriel:gabriel /home/gabriel/.local/state/clipboard-history/debug.log
```

### Desinstalação em HOME temporário

Comando padrão:

```bash
tmp_home=$(mktemp -d)
HOME="$tmp_home" ./scripts/uninstall-local.sh
```

Resultado:

- remove caminhos esperados de binário/autostart;
- preserva dados por padrão.

Com purge:

```bash
tmp_home=$(mktemp -d)
mkdir -p "$tmp_home/.local/share/clipboard-history"
touch "$tmp_home/.local/share/clipboard-history/history.json"
HOME="$tmp_home" ./scripts/uninstall-local.sh --purge-data
test ! -e "$tmp_home/.local/share/clipboard-history"
```

Resultado: passou.

## Testes Não Executados

### `cargo clippy`

Não executado porque `cargo-clippy` não está instalado:

```text
error: 'cargo-clippy' is not installed for the toolchain 'stable-x86_64-unknown-linux-gnu'
```

Comando para habilitar:

```bash
rustup component add clippy
cargo clippy --features desktop-gtk --all-targets -- -D warnings
```

### `cargo audit`

Não executado porque `cargo-audit` não está instalado:

```text
error: no such command: `audit`
```

Comando para habilitar:

```bash
cargo install cargo-audit
cargo audit
```

### Testes com clipboard real

O comando `wl-copy` não está disponível no ambiente atual, então o teste de persistência via clipboard real não foi executado.

## Testes Manuais Pendentes

- Confirmar que o daemon instalado continua abrindo com `Super+V`.
- Confirmar que textos comuns ainda aparecem no histórico.
- Confirmar que tokens/senhas óbvios não aparecem no histórico.
- Confirmar que `--clear-history` limpa o popup e remove imagens associadas.
- Confirmar que auto-paste segue funcionando por padrão.
- Confirmar que `LINUXCLIPBOARD_AUTO_PASTE=0` mantém apenas fallback manual com `Ctrl+V`.
- Confirmar que imagens grandes demais são ignoradas sem travar a aplicação.
- Confirmar que a UX não foi afetada pelo hardening.

## Critério Atual de Avanço

As etapas automatizáveis foram aprovadas. Para avançar com segurança para uma próxima versão, ainda é necessário:

- instalar `clippy` e `cargo-audit`;
- executar os testes manuais pendentes;
- decidir se auto-paste deve continuar habilitado por padrão ou virar opt-in em uma versão futura.
