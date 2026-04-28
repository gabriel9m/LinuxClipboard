# **Guia Avançado: Engenharia com Agentes de IA (Método Akita \- V5)**

Este documento estabelece a metodologia de engenharia rigorosa para o desenvolvimento de sistemas complexos utilizando agentes de IA. O foco é a aplicação de Extreme Programming (XP) acelerada por IA, priorizando disciplina cirúrgica para evitar dívida técnica e o "Vibe Coding".

## **1\. A Filosofia: IA como Espelho e Multiplicador**

A IA não substitui o programador; ela atua como um multiplicador de competência (ou incompetência). Como afirma Akita: *"A IA é seu espelho... Se for incompetente, vai produzir coisas ruins mais rápido. Se for competente, vai produzir coisas boas mais rápido."*

* **O Adulto na Sala:** O agente de IA é um executor incansável que nunca diz "não". O desenvolvedor humano é o arquiteto, o revisor e o responsável pela segurança.  
* **Dívida Técnica Exponencial:** Sem intervenção constante, agentes tendem a empilhar código funcional mas redundante, criando monólitos de difícil manutenção.

## **2\. O CLAUDE.md: A Especificação Viva**

O arquivo de contexto (CLAUDE.md ou codex.md) é o cérebro compartilhado do projeto. Ele deve ser atualizado continuamente para guiar o agente sem alucinações.

### **Elementos Obrigatórios:**

* **Arquitetura de Memória:** Diretrizes de como o sistema deve ser construído (ex: "Usar Dry-RB para validações", "Mantenha controllers Stimulus curtos").  
* **Decisões de Design:** Registro do porquê certas tecnologias foram escolhidas para manter a IA dentro da stack definida.  
* **Contexto de Sessão:** Orientações específicas para tarefas longas e complexas.

## **3\. Segurança e Governança: O Protocolo ai-jail (Sandbox)**

O uso de agentes exige um ambiente de contenção real para proteger a máquina host e os dados sensíveis:

* **Isolamento em Containers:** O agente deve operar dentro de um ambiente Docker onde comandos destrutivos não afetem o sistema real.  
* **Revisão de Commits:** Pequenos lançamentos protegidos por CI (Continuous Integration) automático. Se houver erro, reverte-se apenas um commit atômico.

## **4\. Fluxo de Trabalho: Do Zero à Produção em 7 Etapas**

Este cronograma substitui a intuição por um processo de engenharia estruturado:

| Etapa | Foco Principal | Atividades Chave   |
| :---- | :---- | :---- |
| **Etapa 1** | Governança e Ambiente | Setup do *ai-jail*, Docker e ferramentas de agente (Aider, Claude Code). |
| **Etapa 2** | Fundação e Domínio | Criação do CLAUDE.md, histórias de usuário e definição do esquema de dados. |
| **Etapa 3** | Infraestrutura e TDD | Escrita da suíte de testes unitários e mocks. Nada de código funcional ainda. |
| **Etapa 4** | Implementação Core | O agente escreve o código funcional para passar nos testes. Humano faz code review linha a linha. |
| **Etapa 5** | Poda e Refatoração | Extração de *concerns*, simplificação de lógica e redução de linhas de código desnecessárias. |
| **Etapa 6** | Segurança e Hardening | Auditoria via IA, execução de linters e ferramentas de análise estática (ex: Brakeman). |
| **Etapa 7** | Interface e Deploy | Implementação da UI e deploy resiliente (ex: usando Kamal e SQLite). |

## **5\. Lições de Ouro do Método**

* **Refactoring Contínuo:** A poda regular do código é o que impede a base de se tornar um monólito indomável.  
* **Small Releases:** Cada commit deve ser funcional e passar no CI. No projeto real, a média foi de 34 commits por dia.  
* **Desapego ao Código:** Se a IA errar, não corrija manualmente no arquivo. Explique o erro, ajuste a spec e mande a IA refazer.

---

*Documento V5: Atualizado para refletir o fluxo por etapas em conformidade com as melhores práticas de engenharia acelerada.*