# Método Akita para este projeto

Este arquivo é a especificação viva do projeto. Ele existe para manter o trabalho alinhado com uma disciplina de engenharia forte, usando IA como par de programação e não como gerador autônomo de código.

## Objetivo

Construir software com:

- especificação clara antes da implementação;
- testes antes ou junto da implementação;
- commits pequenos e funcionais;
- refatoração contínua;
- segurança e validação em todo ciclo;
- revisão humana sempre no centro das decisões.

## Princípios

### IA é um acelerador, não a autoridade

- A IA pode escrever código, sugerir alternativas e explicar trade-offs.
- O humano define direção, valida decisões e faz o code review final.
- Se a IA escolher um caminho ruim, o ajuste principal deve ir para a spec, não para remendo manual solto.

### Papel do usuário neste projeto

- O usuário não é programador e não deve ser tratado como se precisasse definir a stack por conhecimento técnico prévio.
- O briefing inicial vem do usuário em linguagem de negócio ou de produto: o que ele quer construir, para quem e com quais restrições.
- A análise técnica vem da IA: linguagem, banco de dados, front-end, back-end, infraestrutura, testes e demais decisões relevantes.
- Para cada categoria técnica importante, a IA deve apresentar mais de uma opção, com prós e contras detalhados e uma recomendação contextualizada.
- O usuário decide com base nessa análise e pode pedir mais informações, novas alternativas ou refinamento da recomendação antes de bater o martelo.
- A IA não deve presumir que já existe uma decisão técnica fechada quando isso ainda não foi discutido.

### Pequenas entregas

- Cada mudança deve ser pequena o bastante para ser entendida e revertida com facilidade.
- Cada commit deve deixar o projeto em estado executável e testado.
- Feature grande demais deve ser fatiada até virar entregas independentes.

### TDD e feedback rápido

- Testes guiam a implementação.
- Falhas devem aparecer cedo.
- Refatoração só entra com rede de segurança suficiente.

### Refatoração contínua

- Não acumular dívida técnica esperando “a fase de limpeza”.
- Sempre que um trecho crescer demais, extrair responsabilidades e reduzir acoplamento.
- Preferir módulos pequenos e nomes claros a arquivos monolíticos.

### Segurança e robustez

- Validar entradas.
- Tratar erros explicitamente.
- Rodar checagens automatizadas sempre que possível.
- Não assumir que o caminho feliz é o único caminho real.

## Fluxo de trabalho

### Fluxo padrão reutilizável

- O fluxo detalhado e reaproveitável entre projetos está em [fluxo-akita.md](/home/gabriel/projetos/clipboard-history/fluxo-akita.md).
- Use esse arquivo como referência principal para iniciar projetos novos e para manter o ciclo de entrega do projeto atual.

### 1. Descoberta

- Ler o contexto existente.
- Entender o problema, o domínio e as restrições.
- Identificar riscos e dependências.

### 2. Especificação

- Escrever ou atualizar a spec antes do código.
- Definir objetivo, escopo, fora de escopo, critérios de aceitação e casos de borda.
- Se houver ambiguidade, resolver na spec primeiro.

### 3. Testes

- Criar testes para o comportamento esperado.
- Cobrir erro, borda e regressão relevante.
- Não avançar sem uma forma clara de validar a mudança.

### 4. Implementação

- Implementar o menor conjunto de mudanças para passar nos testes.
- Evitar overengineering.
- Preferir solução simples e explícita.

### 5. Refatoração

- Remover duplicação.
- Extrair funções, componentes ou módulos quando a responsabilidade ficar pesada.
- Manter o comportamento coberto pelos testes.

### 6. Validação

- Rodar testes e verificações relevantes.
- Corrigir falhas antes de considerar a entrega pronta.

### 7. Entrega

- Registrar o que foi feito.
- Se houver decisões relevantes, registrar também o motivo.

## Regras para trabalhar com agente de IA

- Dê contexto suficiente, mas não entregue um problema mal definido.
- Peça uma coisa por vez quando a tarefa for sensível.
- Interrompa quando a solução estiver ficando complexa sem necessidade.
- Peça alternativas quando houver trade-off importante.
- Não aceite código “que funciona” se ele cria dívida técnica óbvia.
- Em todas as etapas do trabalho, qualquer decisão técnica relevante deve vir da IA em formato de opções comparadas, com prós, contras e alternativa(s), antes da decisão final.
- Toda sugestão técnica deve vir com documentação suficiente para decisão: o que está sendo proposto, por que isso faz sentido, prós, contras e alternativas viáveis com seus respectivos prós e contras.
- Os prós e contras devem ser detalhados, não genéricos. Eles precisam explicar impacto prático, custo de manutenção, complexidade adicional, risco introduzido ou reduzido, e efeito no curto e no longo prazo.
- Quando houver uma recomendação, ela deve deixar claro em que cenário ela é melhor e em que cenário deixa de ser a melhor opção.
- Quando houver mais de um caminho razoável, a decisão final deve ser apresentada como escolha informada, não como imposição técnica.

## Formato esperado para sugestões técnicas

Use este nível de detalhe quando eu precisar decidir algo:

```md
Sugestão: <o que está sendo proposto>

Por que isso faz sentido:
- <razão 1>
- <razão 2>

Prós:
- <benefício prático 1>
- <benefício prático 2>
- <impacto no curto prazo>
- <impacto no longo prazo>

Contras:
- <limitação prática 1>
- <limitação prática 2>
- <custo de manutenção>
- <risco ou efeito colateral>

Alternativas:
1. <alternativa A>
   - Prós:
   - Contras:
   - Quando escolher:
2. <alternativa B>
   - Prós:
   - Contras:
   - Quando escolher:

Recomendação:
- <qual escolha eu recomendo>
- <em que cenário ela é melhor>
- <em que cenário eu não recomendaria>
```

## Definição de pronto

Uma entrega só está pronta quando:

- o comportamento pedido está implementado;
- os testes relevantes passam;
- o código está legível e coeso;
- o impacto foi documentado;
- o próximo passo do projeto ficou claro.

## Template de spec

Use este formato para qualquer tarefa nova:

```md
# Tarefa

## Problema

## Objetivo

## Fora de escopo

## Requisitos funcionais

## Requisitos não funcionais

## Casos de borda

## Critérios de aceitação

## Plano de testes

## Riscos

## Decisões registradas
```

## Ordem sugerida para iniciar o projeto

1. Definir o domínio do projeto e o primeiro fluxo útil.
2. Escrever a primeira spec pequena.
3. Criar testes do comportamento principal.
4. Implementar a menor versão funcional.
5. Refatorar o mínimo necessário.
6. Repetir o ciclo com a próxima fatia.

## Processo de escolha técnica

Quando o projeto novo começar, siga esta ordem:

1. O usuário descreve o que quer construir em nível de produto.
2. A IA devolve opções técnicas por categoria, sempre com pelo menos duas alternativas quando houver escolha real.
3. A IA detalha prós, contras, impacto de curto prazo, impacto de longo prazo, risco e custo de manutenção.
4. O usuário analisa, faz perguntas adicionais se quiser e escolhe o caminho.
5. Só depois disso a implementação começa.

Categorias em que a IA deve oferecer opções quando aplicável:

- linguagem de programação;
- banco de dados;
- front-end;
- back-end;
- autenticação;
- testes;
- deploy e infraestrutura;
- observabilidade;
- organização de pastas e módulos.

## Fontes de referência

- `bibliografia.txt`
- `referencia-teorica-akita.txt`
- `Guia Avançado_ Engenharia com Agentes de IA (Método Akita - V5).md`
