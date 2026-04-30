# LinuxClipboard

LinuxClipboard é um histórico de área de transferência para Linux, inspirado no fluxo do `Win+V` do Windows, com foco inicial em Zorin OS e ambientes GTK/Wayland.

O objetivo do projeto é oferecer uma experiência simples: copiar textos ou imagens, abrir o histórico com `Super+V`, escolher um item antigo e colar diretamente no aplicativo atual.

## Status do Projeto

O projeto está em estágio MVP validado manualmente no Zorin OS.

Versão marcada:

```text
v0.1.0
```

O MVP atual já suporta:

- daemon residente em segundo plano;
- autostart ao iniciar a sessão;
- atalho global configurável pelo usuário;
- histórico persistente de textos;
- histórico persistente de imagens;
- popup GTK com tema claro/escuro;
- navegação por teclado;
- seleção por mouse;
- colagem automática;
- itens fixados com pin;
- limpeza de histórico;
- hardening básico de permissões locais.

## Funcionalidades

### Histórico de Texto

O aplicativo monitora o clipboard e salva textos copiados no histórico.

Comportamento esperado:

- o item mais recente aparece no topo;
- cópias consecutivas iguais são ignoradas;
- textos vazios ou apenas com espaços são ignorados;
- textos muito grandes são ignorados;
- alguns padrões óbvios de segredo são ignorados.

Exemplos de textos que podem ser ignorados por segurança:

- chaves privadas;
- tokens iniciados com `ghp_`;
- tokens iniciados com `github_pat_`;
- tokens iniciados com `sk-`;
- JWTs simples;
- textos contendo `password=`;
- textos contendo `secret=`;
- textos contendo `api_key=`;
- códigos numéricos curtos compatíveis com 2FA.

Esse filtro é heurístico. Ele reduz risco, mas não substitui boas práticas de segurança.

### Histórico de Imagens

O aplicativo também captura imagens copiadas para o clipboard.

Comportamento esperado:

- imagens aparecem no popup como miniaturas;
- o histórico não mostra texto lateral para imagens;
- ao selecionar uma miniatura, a imagem é recolocada no clipboard;
- se o aplicativo atual aceitar imagem, a colagem automática pode inserir a imagem diretamente.

As imagens são persistidas como arquivos em disco.

### Popup de Histórico

O popup é uma janela GTK customizada.

Recursos da interface:

- visual integrado ao Zorin/GNOME;
- adaptação automática para tema claro e escuro;
- linhas com hover e seleção;
- miniaturas para imagens;
- botão de pin para fixar itens;
- cabeçalho compacto com instruções de uso;
- janela arrastável pelo cabeçalho.

### Navegação

Com o popup aberto:

```text
Seta para baixo  seleciona o próximo item
Seta para cima   seleciona o item anterior
Enter            ativa o item selecionado
Esc              fecha o popup
Mouse            seleciona e ativa um item com clique
```

### Pin

Itens pinados não são removidos pela retenção normal do histórico.

Exemplo:

- você copia um número importante;
- abre o popup;
- clica no ícone de pin;
- mesmo copiando muitos itens novos, o item pinado permanece no histórico.

Para remover o pin, clique novamente no mesmo ícone.

### Limite de Histórico

O histórico mantém até 25 itens, preservando os itens pinados.

Quando o limite é excedido:

- o item não pinado mais antigo é removido;
- imagens associadas ao item removido são apagadas;
- itens pinados são preservados.

### Limpeza de Histórico

O histórico pode ser limpo com:

```bash
linuxclipboard --clear-history
```

Esse comando remove os itens persistidos e as imagens associadas.

## Requisitos

### Sistema

Ambiente usado para validação:

```text
Zorin OS 17.3
GTK4
Wayland
```

O projeto deve funcionar em outras distribuições Linux com GTK4, mas o fluxo foi validado inicialmente no Zorin OS.

### Dependências de Build

Para compilar:

```bash
sudo apt-get update
sudo apt-get install -y pkg-config libgtk-4-dev
```

Também é necessário ter Rust instalado.

Se ainda não tiver Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Depois reinicie o terminal ou carregue o ambiente do Cargo:

```bash
source "$HOME/.cargo/env"
```

### Dependências Para Auto-paste

O LinuxClipboard sempre coloca o item selecionado de volta no clipboard.

Para colar automaticamente, ele tenta simular `Ctrl+V` com uma das ferramentas abaixo:

```text
Wayland: /usr/bin/ydotool
Wayland: /usr/bin/wtype
X11:     /usr/bin/xdotool
```

No Wayland, a opção mais comum é `ydotool`.

Instalação:

```bash
sudo apt-get install -y ydotool
```

Dependendo da distribuição, pode ser necessário configurar `uinput` e permissões do grupo `input` para o `ydotool` funcionar corretamente.

Se nenhuma ferramenta compatível estiver disponível:

- o item selecionado ainda será copiado para o clipboard;
- a colagem automática não acontecerá;
- você poderá usar `Ctrl+V` manualmente.

## Instalação

O fluxo recomendado é usar o instalador local do projeto.

Clone o repositório:

```bash
git clone https://github.com/gabriel9m/LinuxClipboard.git
cd LinuxClipboard
```

Se você preferir SSH, use a URL SSH equivalente do repositório no GitHub.

Execute:

```bash
./scripts/install-local.sh
```

O instalador faz:

- compila o binário em modo `release`;
- instala o executável em `~/.local/bin/linuxclipboard`;
- cria autostart em `~/.config/autostart/linuxclipboard.desktop`;
- cria/corrige diretórios de dados;
- aplica permissões locais mais restritas;
- mostra o comando recomendado para o atalho `Super+V`.

Depois da instalação, inicie ou reinicie o daemon:

```bash
~/.local/bin/linuxclipboard --quit || true
setsid ~/.local/bin/linuxclipboard >/tmp/linuxclipboard.log 2>&1 < /dev/null &
```

## Configuração do Atalho Super+V

O LinuxClipboard não registra atalho global sozinho. O atalho deve ser configurado no ambiente desktop.

No Zorin OS:

```text
Configurações > Teclado > Atalhos personalizados
```

Crie um novo atalho:

```text
Nome: LinuxClipboard
Comando: /home/seu-usuario/.local/bin/linuxclipboard --toggle-popup
Atalho: Super+V
```

Se `~/.local/bin` estiver no seu `PATH`, o comando também pode ser:

```bash
linuxclipboard --toggle-popup
```

## Uso Diário

### Iniciar Daemon

```bash
linuxclipboard
```

Normalmente você não precisa executar isso manualmente depois da instalação, porque o autostart já inicia o daemon na sessão.

### Abrir ou Fechar Popup

```bash
linuxclipboard --toggle-popup
```

Esse é o comando recomendado para o atalho `Super+V`.

### Abrir Popup Diretamente

```bash
linuxclipboard --show-popup
```

### Encerrar Daemon

```bash
linuxclipboard --quit
```

### Limpar Histórico

```bash
linuxclipboard --clear-history
```

### Desativar Auto-paste

Por padrão, ao selecionar um item, o app tenta colar automaticamente.

Para iniciar o daemon com auto-paste desativado:

```bash
LINUXCLIPBOARD_AUTO_PASTE=0 linuxclipboard
```

Com auto-paste desativado:

- selecionar um item coloca o conteúdo no clipboard;
- você cola manualmente com `Ctrl+V`.

Valores aceitos para desativar:

```text
0
false
off
disabled
```

## Onde os Dados Ficam

Histórico:

```text
~/.local/share/clipboard-history/history.json
```

Imagens:

```text
~/.local/share/clipboard-history/images
```

Log:

```text
~/.local/state/clipboard-history/debug.log
```

Autostart:

```text
~/.config/autostart/linuxclipboard.desktop
```

Binário:

```text
~/.local/bin/linuxclipboard
```

## Segurança e Privacidade

Um gerenciador de clipboard lida com dados sensíveis por natureza.

O LinuxClipboard aplica algumas proteções:

- histórico com permissão `0600`;
- diretório de dados com permissão `0700`;
- imagens com permissão `0600`;
- log com permissão `0600`;
- filtro básico para segredos óbvios;
- limite de tamanho para textos;
- limite de tamanho para imagens;
- preservação de JSON corrompido antes de recriar histórico;
- cleanup de imagens restrito ao diretório controlado;
- auto-paste desativável por variável de ambiente;
- ferramentas externas de auto-paste chamadas por caminho absoluto.

Mesmo assim, existem limitações:

- o filtro de segredos é heurístico;
- se você copiar um segredo com formato desconhecido, ele ainda pode ser salvo;
- auto-paste pode colar em local inesperado se o foco mudar no momento errado;
- qualquer pessoa com acesso à sua sessão de usuário pode abrir o histórico;
- imagens copiadas são salvas em disco.

Para limpar dados:

```bash
linuxclipboard --clear-history
```

Para desinstalar removendo também os dados:

```bash
./scripts/uninstall-local.sh --purge-data
```

## Desinstalação

Para remover o binário e o autostart:

```bash
./scripts/uninstall-local.sh
```

Por padrão, esse comando preserva histórico e imagens.

Para remover também dados persistidos:

```bash
./scripts/uninstall-local.sh --purge-data
```

## Desenvolvimento

### Testes

Testes padrão:

```bash
cargo test
```

Testes com integração desktop GTK:

```bash
cargo test --features desktop-gtk
```

Checagem do binário GTK:

```bash
cargo check --features desktop-gtk --bin linuxclipboard
```

### Execução Sem Instalar

Durante desenvolvimento, é possível executar via Cargo.

Iniciar daemon:

```bash
cargo run --features desktop-gtk
```

Alternar popup:

```bash
cargo run --features desktop-gtk -- --toggle-popup
```

Abrir popup:

```bash
cargo run --features desktop-gtk -- --show-popup
```

Encerrar daemon:

```bash
cargo run --features desktop-gtk -- --quit
```

Limpar histórico:

```bash
cargo run --features desktop-gtk -- --clear-history
```

### Estrutura do Projeto

```text
src/domain.rs        regras de domínio, histórico, itens, limites e filtros
src/storage.rs       persistência do histórico e imagens
src/clipboard.rs     captura, deduplicação e escrita no clipboard
src/paste.rs         fluxo de ativação e colagem
src/ui.rs            estado e ações da interface
src/app.rs           coordenação da aplicação e storage
src/desktop/gtk.rs   integração GTK, popup, daemon, auto-paste e logs
src/desktop/paths.rs caminhos padrão de dados
scripts/             instalação e desinstalação local
```

### Relatórios de Segurança

O projeto contém documentos de apoio:

```text
security-test-plan.md
security-execution-report.md
```

Esses arquivos descrevem testes de vulnerabilidade, critérios de sucesso e correções já aplicadas.

## Troubleshooting

### O popup só mostra o último item copiado

Verifique se o daemon está rodando antes de copiar os itens:

```bash
linuxclipboard --quit || true
setsid ~/.local/bin/linuxclipboard >/tmp/linuxclipboard.log 2>&1 < /dev/null &
```

Depois copie os itens novamente e abra com `Super+V`.

### O popup não abre com Super+V

Teste o comando diretamente:

```bash
~/.local/bin/linuxclipboard --toggle-popup
```

Se funcionar, revise o atalho do sistema.

O comando do atalho deve ser:

```bash
/home/seu-usuario/.local/bin/linuxclipboard --toggle-popup
```

### O item selecionado não cola automaticamente

Verifique se existe uma ferramenta de automação instalada:

```bash
command -v ydotool
command -v wtype
command -v xdotool
```

No Wayland, instale `ydotool`:

```bash
sudo apt-get install -y ydotool
```

Se auto-paste não estiver disponível, o fallback manual ainda funciona:

```text
1. selecione o item no LinuxClipboard
2. pressione Ctrl+V no aplicativo desejado
```

### O histórico não captura uma senha ou token

Isso pode ser esperado.

O app ignora alguns padrões óbvios de segredo para reduzir risco de persistir dados sensíveis.

### Quero resetar tudo

```bash
linuxclipboard --quit || true
./scripts/uninstall-local.sh --purge-data
./scripts/install-local.sh
setsid ~/.local/bin/linuxclipboard >/tmp/linuxclipboard.log 2>&1 < /dev/null &
```

## Roadmap

Ideias para próximas versões:

- busca textual no histórico;
- botão visual para limpar histórico;
- remoção individual de item;
- configuração gráfica;
- auto-paste opt-in;
- criptografia local do histórico;
- suporte mais robusto a múltiplos monitores;
- empacotamento `.deb` ou Flatpak;
- CI com `cargo clippy` e `cargo audit`.

## Contribuição

Contribuições são bem-vindas.

Antes de abrir pull request:

```bash
cargo fmt --check
cargo test
cargo test --features desktop-gtk
cargo check --features desktop-gtk --bin linuxclipboard
```

Se disponíveis:

```bash
cargo clippy --features desktop-gtk --all-targets -- -D warnings
cargo audit
```

Para mudanças de interface, descreva também o teste manual realizado no Zorin OS ou no ambiente GTK usado.
