# Fluxo Akita Reutilizável

Este é o fluxo padrão para iniciar e evoluir qualquer projeto neste repositório.

## 0. Briefing inicial

Entrada:

- o que você quer construir;
- para quem é;
- por que isso existe;
- restrições e preferências não técnicas.

Saída:

- `project-brief.md` preenchido;
- dúvidas abertas listadas;
- categorias técnicas a decidir.

## 1. Governança e ambiente

Objetivo:

- definir como o projeto será conduzido sem improviso.

Decisões típicas:

- estrutura do repositório;
- ferramenta de IA ou agente;
- sandbox e limites de execução;
- CI básico;
- convenções de trabalho.

Saída:

- `governanca-ambiente.md` preenchido para o projeto;
- `codex.md` ajustado para o projeto;
- ambiente inicial pronto;
- regras de trabalho claras.

## 2. Domínio e especificação

Objetivo:

- transformar o briefing em uma spec executável.

Decisões típicas:

- escopo da primeira fatia;
- casos de uso principais;
- fora de escopo;
- critérios de aceitação;
- entidades e dados principais.

Saída:

- spec da primeira entrega;
- critérios claros de pronto;
- riscos e dependências visíveis.

## 3. Testes e contratos

Objetivo:

- codificar o comportamento esperado antes da implementação.

Decisões típicas:

- casos felizes;
- casos de erro;
- bordas;
- mocks e integrações;
- ordem de validação.

Saída:

- `spec-testes-v1.md` definido para a primeira versão;
- testes falhando pelo motivo certo;
- contratos do sistema explicitados;
- base segura para implementar.

## 4. Implementação core

Objetivo:

- entregar a menor versão funcional que passe nos testes.

Decisões típicas:

- arquitetura mínima;
- linguagem e biblioteca escolhidas;
- separação de responsabilidades;
- dependências realmente necessárias.

Saída:

- funcionalidade principal pronta;
- testes verdes;
- commit pequeno e legível.

## 5. Poda e refatoração

Objetivo:

- reduzir acoplamento e dívida técnica sem mudar comportamento.

Decisões típicas:

- extração de funções e módulos;
- remoção de duplicação;
- simplificação de fluxos;
- reorganização de arquivos.

Saída:

- código mais simples;
- mesmos testes verdes;
- estrutura mais fácil de manter.

## 6. Segurança e hardening

Objetivo:

- tornar o sistema mais resistente a erro e uso real.

Decisões típicas:

- validação de entrada;
- tratamento de exceções;
- análise estática;
- revisão de segurança;
- observabilidade mínima.

Saída:

- falhas mais previsíveis;
- menos risco operacional;
- release mais confiável.

## 7. Interface e deploy

Objetivo:

- colocar o sistema em uso real com qualidade suficiente.

Decisões típicas:

- UI e experiência do usuário;
- estratégia de deploy;
- configuração de ambiente;
- monitoramento básico;
- roteiro de release.

Saída:

- projeto entregue ou publicável;
- próximo ciclo de melhoria definido.

## Regra de ouro

Cada etapa deve terminar com:

- decisão registrada;
- testes ou validação equivalentes;
- próximo passo claro.
- em qualquer ponto em que existir decisão técnica relevante, a IA deve apresentar opções comparadas com prós e contras detalhados antes de você decidir.

## Como reutilizar entre projetos

1. Reaproveite este fluxo sem mudar a lógica central.
2. Troque apenas o briefing, as escolhas técnicas e o domínio.
3. Mantenha as etapas e a ordem.
4. Se o projeto for pequeno, ainda siga as etapas, mas com entregas menores.
