# Spec Técnica - Versão 1

## 1. Contexto

Este projeto é uma aplicação desktop para Zorin OS 17.3 Core 64 bits que reproduz, de forma leve e confiável, a experiência de histórico da área de transferência parecida com o `Win+V` do Windows, usando `Super+V` como atalho no Linux.

## 2. Objetivo da primeira versão

Entregar uma primeira versão funcional que:

- rode em segundo plano;
- capture texto e imagem copiados;
- armazene apenas os últimos 25 itens;
- abra um pop-up com `Super+V`;
- permita navegar pela lista com teclado;
- permita selecionar um item por clique ou `Enter`;
- tente colar automaticamente no aplicativo em foco quando o ambiente permitir;
- mantenha um fallback seguro para colagem manual via clipboard.

## 3. Escopo

### Dentro do escopo

- monitoramento do clipboard para texto;
- monitoramento do clipboard para imagem;
- histórico local com limite de 25 itens;
- popup com lista dos itens mais recentes;
- seleção por mouse;
- seleção por teclado com `seta para cima`, `seta para baixo` e `Enter`;
- colagem híbrida:
  - tentar colar automaticamente quando possível;
  - caso não seja possível, deixar o item pronto no clipboard;
- distribuição via AppImage;
- interface simples e direta;
- funcionamento como app de segundo plano.

### Fora do escopo

- sincronização com outros dispositivos;
- login de usuário;
- suporte a nuvem;
- OCR de imagens;
- busca no histórico na primeira versão;
- múltiplos perfis;
- histórico ilimitado;
- edição de itens copiados;
- categorias ou tags;
- suporte a outros tipos além de texto e imagem.

## 4. Decisões técnicas já definidas

- linguagem: Rust;
- interface: GTK4;
- armazenamento: JSON + arquivos locais para imagens;
- limite do histórico: 25 itens;
- distribuição: AppImage;
- atalho principal: `Super+V`;
- fluxo de trabalho: desenvolvimento no host com `git`;
- repositório remoto: privado, com escopo mínimo;
- branches: estratégia híbrida com `main` protegida.

### 4.1 Regras de interpretação

- esta spec é a fonte de verdade para a primeira versão;
- se algo não estiver descrito aqui, o agente deve perguntar antes de inventar comportamento;
- o agente não deve adicionar busca, tags, OCR, sync, nuvem ou edição por conta própria;
- `Win+V` é apenas a referência funcional do Windows; no Linux, o atalho desta versão é `Super+V`;
- o app deve ser útil sem depender de qualquer integração extra que não esteja descrita nesta spec.

### 4.2 Fronteiras do sistema

- `domain`: regras do histórico, tipos de item, limite de 25 e política de seleção;
- `storage`: leitura e escrita do JSON, persistência das imagens e limpeza dos arquivos antigos;
- `clipboard`: captura de texto e imagem, e atualização do clipboard na seleção;
- `paste`: tentativa de colar automaticamente no aplicativo em foco;
- `ui`: popup, lista, navegação e feedback visual.

### 4.3 Princípio de não autoalimento

- quando o app colocar um item no clipboard como resultado de uma seleção, essa ação não deve gerar um novo item no histórico;
- a implementação deve usar um sinal interno, janela de tempo curta, ou outro mecanismo explícito para impedir loop de captura;
- o usuário nunca deve ver o histórico duplicar o mesmo item só porque o app copiou algo para si mesmo.

## 5. Requisitos funcionais

### 5.1 Captura do clipboard

- o app deve observar mudanças relevantes no clipboard;
- ao copiar texto, o conteúdo deve ser registrado no histórico;
- ao copiar imagem, a imagem deve ser registrada no histórico;
- o app deve ignorar mudanças disparadas por ele mesmo ao atualizar o clipboard após uma seleção;
- itens duplicados podem ser preservados se ocorrerem em momentos diferentes, a menos que a implementação mostre risco de ruído excessivo;
- se o clipboard contiver conteúdo não suportado, o item deve ser ignorado sem quebrar o app;
- o app deve tratar texto como texto puro, sem tentar formatar ou interpretar HTML nesta versão.

### 5.2 Histórico

- o histórico deve manter apenas os 25 itens mais recentes;
- o item mais recente deve aparecer no topo;
- cada item deve armazenar:
  - tipo do conteúdo;
  - timestamp;
  - conteúdo de texto ou referência para imagem;
  - metadados mínimos para renderização;
- itens antigos devem ser removidos automaticamente quando o limite for excedido.
- itens de texto devem exibir uma prévia de uma linha com truncamento visual;
- itens de imagem devem exibir uma miniatura;
- o item selecionado por padrão ao abrir o popup deve ser o mais recente, se houver itens;
- o histórico vazio deve ser representado explicitamente na UI com uma mensagem simples de estado vazio.

### 5.3 Popup

- `Super+V` deve abrir a janela do histórico;
- o popup deve aparecer rapidamente e sem travar o desktop;
- o popup deve abrir como uma janela flutuante pequena, não como tela cheia;
- a lista deve exibir texto e miniatura de imagem quando aplicável;
- o popup deve permitir navegação por teclado;
- o popup deve fechar ao selecionar um item ou ao cancelar.
- a navegação com `seta para cima` e `seta para baixo` não deve ultrapassar os limites da lista;
- `Enter` ativa o item selecionado;
- `Esc` fecha o popup sem alterar o item atualmente selecionado no histórico;
- clicar em um item deve equivaler a ativá-lo;
- o popup não deve exigir interação com mouse para funcionar.

### 5.4 Seleção e colagem

- clicar em um item deve selecionar o conteúdo;
- pressionar `Enter` em um item selecionado deve selecionar o conteúdo;
- o app deve tentar colar automaticamente no aplicativo em foco quando o sistema permitir;
- se o colar automático não for possível, o item deve permanecer disponível no clipboard;
- o comportamento deve ser previsível e seguro.
- a atualização do clipboard deve acontecer antes da tentativa de colar;
- a tentativa de colar automático deve ser tratada como esforço máximo, não como garantia;
- se a colagem automática falhar, o popup pode fechar mesmo assim, desde que o conteúdo continue no clipboard;
- o comportamento de clique e de `Enter` deve ser o mesmo;
- a aplicação não deve exigir o uso de atalhos adicionais depois da seleção em cenários suportados.

### 5.5 Execução em segundo plano

- o app deve iniciar e permanecer em background;
- a interface principal não precisa ficar aberta o tempo todo;
- o app deve ter uma forma simples de sair ou encerrar quando necessário;
- idealmente deve haver ícone de tray ou equivalente, se viável no GTK4/Zorin sem complexidade excessiva;
- se tray não for viável sem elevar muito a complexidade, o app pode operar sem ícone visível, desde que continue acessível pelo atalho e tenha uma forma clara de encerrar.

## 6. Requisitos não funcionais

- leveza;
- baixa complexidade operacional;
- manutenção simples;
- segurança básica;
- robustez contra falhas do clipboard;
- comportamento estável no Zorin OS;
- consumo moderado de memória e CPU;
- base pequena o suficiente para evoluir sem virar monólito.

## 7. Modelo de dados

### 7.1 Estrutura de item do histórico

Cada item pode seguir, conceitualmente, este formato:

```json
{
  "id": "uuid-ou-id-estavel",
  "kind": "text|image",
  "created_at": "timestamp_iso",
  "preview": "texto-resumido-ou-caminho-da-miniatura",
  "content": {
    "text": "conteudo"
  }
}
```

Para imagens:

```json
{
  "id": "uuid-ou-id-estavel",
  "kind": "image",
  "created_at": "timestamp_iso",
  "preview": "caminho-para-miniatura",
  "content": {
    "path": "caminho-local-da-imagem"
  }
}
```

### 7.2 Persistência

- um arquivo JSON deve guardar o índice do histórico;
- imagens devem ficar em arquivos locais dentro de uma pasta controlada pelo app;
- o JSON deve referenciar esses arquivos;
- ao remover itens antigos, os arquivos de imagem associados também devem ser limpos.
- a escrita do JSON deve ser atômica ou equivalente para reduzir risco de corrupção;
- se a leitura do JSON falhar, o app deve cair para um histórico vazio e seguir funcionando;
- se um arquivo de imagem estiver ausente, o item correspondente deve continuar visível com fallback de conteúdo ausente, ou ser ignorado de forma segura, desde que isso seja consistente.

## 8. Fluxo de execução

### 8.1 Inicialização

1. o app carrega o estado salvo;
2. o app observa o clipboard;
3. o app fica em segundo plano;
4. o atalho `Super+V` é registrado;
5. o histórico fica pronto para abertura imediata.

### 8.2 Ao copiar algo

1. o app detecta a mudança;
2. identifica se é texto ou imagem;
3. cria o item do histórico;
4. salva o item no JSON e, se necessário, em arquivo local;
5. aplica a política de limite de 25 itens;
6. atualiza o estado interno.

### 8.3 Ao pressionar `Super+V`

1. o popup aparece;
2. a lista mostra os itens mais recentes;
3. o usuário navega com teclado ou mouse;
4. ao confirmar, o item é copiado para o clipboard;
5. o app tenta colar automaticamente no destino em foco;
6. se a tentativa falhar, o conteúdo continua disponível no clipboard.

### 8.4 Ao selecionar um item

1. o item escolhido vira o item ativo;
2. o conteúdo é copiado para o clipboard;
3. a gravação desse clipboard não gera um novo item;
4. o app tenta colar automaticamente se a plataforma permitir;
5. o popup fecha;
6. o histórico permanece intacto, apenas com atualização de seleção se necessário.

## 9. Segurança e confiabilidade

- o app não deve expor dados do clipboard fora do sistema local;
- imagens devem ser gravadas em caminho controlado pelo app;
- a leitura e escrita do JSON devem ser tratadas com cuidado para não corromper o histórico;
- falhas de persistência não devem quebrar o popup;
- conteúdo inválido ou inesperado do clipboard deve ser ignorado ou tratado com fallback seguro;
- o app não deve assumir que a tentativa de paste automático sempre vai funcionar.

## 10. UI e experiência

- popup simples, objetivo e rápido;
- lista visual com itens recentes;
- destaque do item selecionado;
- texto legível;
- miniatura para imagens quando possível;
- comportamento de teclado previsível;
- fechar com `Esc` ou após seleção, se isso não prejudicar o fluxo.
- o popup não precisa ter campos de edição;
- o popup não precisa ter busca na primeira versão;
- o popup não precisa ter configuração de tema nesta versão;
- o conteúdo textual exibido na lista deve ser apenas uma prévia, não o texto completo quando isso prejudicar a leitura.

## 11. Estratégia de testes

### Testes de unidade

- limite de 25 itens;
- serialização e desserialização do histórico;
- remoção de itens antigos;
- tratamento de texto e imagem;
- fallback quando o conteúdo do clipboard não for suportado.

### Testes de integração

- carregamento do estado salvo;
- gravação e leitura do JSON;
- fluxo de captura de texto;
- fluxo de captura de imagem;
- abertura do popup;
- seleção de item;
- comportamento híbrido de colagem.

### Testes manuais

- copiar texto e ver o item aparecer;
- copiar imagem e ver o item aparecer;
- abrir com `Super+V`;
- navegar com teclas;
- selecionar por clique;
- selecionar por `Enter`;
- validar comportamento em um editor de texto e em um app web.

## 12. Riscos

- integração de atalho global no desktop Linux;
- comportamento de colagem automática variar conforme a sessão do sistema;
- diferença entre ambientes Wayland e X11;
- empacotamento AppImage com dependências de GTK4;
- gestão correta de arquivos de imagem antigos;
- sincronização entre estado interno, JSON e clipboard.
- loops de recaptura quando o próprio app alterar o clipboard;
- corrupção de arquivo de persistência em falhas de escrita;
- diferença entre visualização e comportamento real de colagem em apps diferentes.

## 13. Critérios de aceitação

A primeira versão será considerada pronta quando:

- copiar texto registrar no histórico;
- copiar imagem registrar no histórico;
- `Super+V` abrir o popup;
- o histórico respeitar o limite de 25 itens;
- clicar ou pressionar `Enter` em um item funcionar;
- o conteúdo selecionado entrar no clipboard;
- a colagem automática funcionar quando a plataforma permitir;
- a colagem manual continuar possível quando o automático não funcionar;
- o app rodar de forma estável no Zorin OS.
- o app não recapturar como novo item aquilo que ele mesmo acabou de colocar no clipboard;
- um JSON corrompido não impedir o app de abrir;
- itens de imagem antigos serem removidos junto com seus arquivos locais.

## 14. Próximo passo

Com esta spec aprovada, a próxima etapa é:

1. estruturar o projeto em Rust;
2. criar a base GTK4;
3. implementar persistência JSON;
4. implementar monitoramento do clipboard;
5. implementar o popup;
6. validar o atalho global;
7. fechar o fluxo híbrido de seleção e colagem.
