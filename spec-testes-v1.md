# Spec de Testes - Versão 1

## 1. Objetivo

Definir os testes que validam a primeira versão do app de histórico de clipboard para Zorin OS, antes da implementação do código funcional.

## 2. Fonte de verdade

Esta spec deve ser lida junto com:

- [spec-tecnica-v1.md](/home/gabriel/projetos/clipboard-history/spec-tecnica-v1.md)
- [fluxo-akita.md](/home/gabriel/projetos/clipboard-history/fluxo-akita.md)
- [governanca-ambiente.md](/home/gabriel/projetos/clipboard-history/governanca-ambiente.md)

Se houver conflito, a ordem de prioridade é:

1. `spec-tecnica-v1.md`
2. `governanca-ambiente.md`
3. `fluxo-akita.md`
4. esta spec de testes

## 3. Princípios de teste

- cada requisito funcional importante precisa ter pelo menos um teste associado;
- os testes devem cobrir sucesso, falha e borda quando isso fizer sentido;
- a implementação não deve ser escrita antes de existir um caminho claro de validação;
- o foco é garantir comportamento, não reproduzir detalhes internos desnecessários;
- onde o comportamento depender de desktop real, o teste deve ser de integração ou manual;
- o que for puro domínio ou persistência deve ser coberto por teste automatizado.

## 4. Camadas de teste

### 4.1 Testes de unidade

Devem cobrir:

- regras do histórico;
- limite de 25 itens;
- ordenação do item mais recente no topo;
- serialização e desserialização do JSON;
- remoção de itens antigos;
- persistência de referências para imagens;
- prevenção de loop de recaptura;
- normalização de texto;
- tratamento de conteúdo inválido.

### 4.2 Testes de integração

Devem cobrir:

- leitura do estado salvo;
- escrita atômica ou equivalente do JSON;
- captura de texto;
- captura de imagem;
- carregamento do popup com dados persistidos;
- atualização do clipboard ao selecionar um item;
- fluxo híbrido de colagem;
- limpeza de arquivos de imagem antigos.

### 4.3 Testes manuais

Devem cobrir:

- abertura do popup com `Super+V`;
- navegação com teclado;
- seleção por clique;
- seleção por `Enter`;
- comportamento em um editor de texto;
- comportamento em um aplicativo web;
- colagem manual quando a colagem automática não funcionar;
- estabilidade geral no Zorin OS.

## 5. Casos de teste por funcionalidade

### 5.1 Captura de texto

#### Teste 5.1.1

Nome: registrar texto copiado no histórico

Pré-condições:

- o app está em execução;
- o histórico está vazio.

Passos:

1. copiar um texto qualquer;
2. aguardar o app detectar a mudança;
3. abrir o popup.

Resultado esperado:

- o texto aparece como item mais recente;
- o item é persistido no JSON;
- o histórico contém 1 item.

#### Teste 5.1.2

Nome: ignorar texto inválido ou não suportado

Passos:

1. simular conteúdo não suportado no clipboard;
2. acionar a leitura do clipboard.

Resultado esperado:

- o app não quebra;
- nenhum item inválido é salvo;
- o histórico continua acessível.

### 5.2 Captura de imagem

#### Teste 5.2.1

Nome: registrar imagem copiada no histórico

Pré-condições:

- o app está em execução;
- existe uma imagem copiada para o clipboard.

Passos:

1. copiar uma imagem;
2. aguardar o app detectar a mudança;
3. abrir o popup.

Resultado esperado:

- a imagem aparece como item no histórico;
- existe referência local persistida;
- a miniatura pode ser exibida.

#### Teste 5.2.2

Nome: remover arquivo de imagem antigo ao exceder limite

Passos:

1. adicionar mais de 25 itens, incluindo imagens;
2. forçar a política de retenção;
3. verificar os arquivos locais.

Resultado esperado:

- apenas 25 itens permanecem no histórico;
- os arquivos das imagens removidas também são excluídos.

### 5.3 Limite e ordenação do histórico

#### Teste 5.3.1

Nome: manter no máximo 25 itens

Passos:

1. inserir 26 itens diferentes;
2. consultar o histórico.

Resultado esperado:

- o histórico mantém 25 itens;
- o item mais antigo é descartado;
- o item mais recente permanece no topo.

#### Teste 5.3.2

Nome: manter o item mais recente no topo

Passos:

1. inserir um item;
2. inserir outro item depois;
3. consultar a ordem.

Resultado esperado:

- o segundo item aparece antes do primeiro.

### 5.4 Persistência

#### Teste 5.4.1

Nome: salvar e carregar o histórico do JSON

Passos:

1. inserir itens;
2. encerrar o app;
3. iniciar o app novamente.

Resultado esperado:

- o histórico anterior é restaurado;
- a ordem é preservada;
- imagens continuam referenciadas corretamente.

#### Teste 5.4.2

Nome: recuperar-se de JSON corrompido

Passos:

1. corromper o arquivo JSON de propósito;
2. iniciar o app.

Resultado esperado:

- o app não quebra;
- o histórico vazio ou fallback seguro é carregado;
- o popup ainda funciona.

#### Teste 5.4.3

Nome: escrita segura do JSON

Passos:

1. simular falha de escrita;
2. tentar salvar um novo item.

Resultado esperado:

- o app não perde o estado inteiro;
- o arquivo final não fica em estado inconsistente, ou a falha é tratada de forma segura;
- a aplicação permanece utilizável.

### 5.5 Popup e navegação

#### Teste 5.5.1

Nome: abrir popup com `Super+V`

Passos:

1. executar o app;
2. pressionar `Super+V`.

Resultado esperado:

- o popup aparece;
- o foco vai para a lista;
- o item mais recente fica selecionado por padrão, se houver itens.

#### Teste 5.5.2

Nome: navegar com setas

Passos:

1. abrir o popup;
2. pressionar `seta para baixo`;
3. pressionar `seta para cima`.

Resultado esperado:

- a seleção muda corretamente;
- a navegação não ultrapassa os limites da lista.

#### Teste 5.5.3

Nome: fechar com `Esc`

Passos:

1. abrir o popup;
2. pressionar `Esc`.

Resultado esperado:

- o popup fecha;
- nenhum item é ativado;
- o histórico não sofre alteração indevida.

### 5.6 Seleção e colagem híbrida

#### Teste 5.6.1

Nome: selecionar item por clique

Passos:

1. abrir o popup;
2. clicar em um item.

Resultado esperado:

- o item é selecionado;
- o conteúdo vai para o clipboard;
- o app tenta colar automaticamente;
- o popup fecha;
- o item não é recapturado como novo.

#### Teste 5.6.2

Nome: selecionar item por `Enter`

Passos:

1. abrir o popup;
2. navegar até um item;
3. pressionar `Enter`.

Resultado esperado:

- o comportamento é o mesmo do clique;
- o conteúdo vai para o clipboard;
- o app tenta colar automaticamente;
- o popup fecha.

#### Teste 5.6.3

Nome: fallback quando paste automático falhar

Passos:

1. simular ambiente em que o paste automático não funciona;
2. selecionar um item.

Resultado esperado:

- o conteúdo permanece no clipboard;
- o popup não trava;
- o usuário consegue colar manualmente depois.

#### Teste 5.6.4

Nome: evitar loop de recaptura

Passos:

1. selecionar um item no popup;
2. observar a atualização do clipboard feita pelo app;
3. verificar a leitura subsequente.

Resultado esperado:

- o item não volta como novo registro;
- o histórico não duplica o item selecionado.

### 5.7 Estado vazio

#### Teste 5.7.1

Nome: popup com histórico vazio

Passos:

1. iniciar com histórico vazio;
2. abrir o popup.

Resultado esperado:

- o popup mostra estado vazio;
- não há erro;
- a interface continua navegável.

## 6. Critérios de cobertura mínima

A primeira implementação só deve ser considerada pronta para integrar quando existir cobertura automatizada para:

- limite de histórico;
- persistência do JSON;
- limpeza de imagens antigas;
- prevenção de loop de recaptura;
- captura de texto;
- captura de imagem;
- navegação de domínio;
- seleção e persistência do item escolhido.

## 7. Estratégia de falha esperada

- testes de unidade devem falhar no comportamento do domínio antes da implementação;
- testes de integração devem falhar quando persistência ou fluxo estiverem incompletos;
- testes manuais devem validar o comportamento específico de desktop que não é confiável em teste isolado;
- se o ambiente impedir automação de alguma parte do desktop, isso deve ser documentado e coberto manualmente.

## 8. Ordem recomendada de execução

1. testar o domínio do histórico;
2. testar persistência JSON;
3. testar retenção de 25 itens;
4. testar prevenção de loop;
5. testar captura de texto;
6. testar captura de imagem;
7. testar popup e navegação;
8. testar seleção e colagem híbrida;
9. validar manualmente no Zorin.

## 9. Definição de pronto

Esta spec de testes só está pronta quando:

- cada requisito principal da spec técnica tiver ao menos um teste associado;
- os testes estiverem organizados por camada;
- o caminho de implementação estiver claro;
- a próxima etapa puder ser iniciar a codificação dos módulos principais.

