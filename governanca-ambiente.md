# Governança e Ambiente

Esta etapa define como o projeto será conduzido com segurança antes da spec do produto.

## Objetivo

- proteger o trabalho;
- manter o histórico das mudanças;
- limitar risco de perda de código;
- controlar acesso ao repositório remoto;
- evitar um ambiente de desenvolvimento que atrapalhe o comportamento real do app.

## 1. Isolamento do desenvolvimento na máquina

### Opção A: desenvolvimento direto no host, com repositório separado e git

Prós:
- é a opção mais simples;
- não atrapalha o acesso ao clipboard, atalho global e janela do desktop;
- evita a complexidade de container ou VM para um app que depende do ambiente gráfico real;
- facilita depuração;
- reduz atrito no dia a dia.

Contras:
- isola menos a máquina do que um container ou uma VM;
- exige disciplina de organização de arquivos;
- depende mais de boas práticas e menos de barreiras técnicas.

Quando escolher:
- quando o app precisa interagir diretamente com o desktop do Zorin;
- quando a prioridade é funcionalidade e velocidade de validação.

### Opção B: container de desenvolvimento

Prós:
- maior isolamento do sistema host;
- reduz risco de contaminar o ambiente principal com dependências;
- facilita padronizar versões de ferramentas.

Contras:
- costuma atrapalhar aplicações de desktop que precisam de clipboard, atalho global e integração visual;
- aumenta a complexidade operacional;
- pode criar uma diferença artificial entre o ambiente de desenvolvimento e o ambiente real do app.

Quando escolher:
- quando o projeto é mais backend, serviço ou CLI;
- quando o comportamento não depende fortemente do desktop.

### Opção C: máquina virtual

Prós:
- maior isolamento;
- mais fácil descartar um ambiente inteiro se algo der errado;
- separa bem desenvolvimento e sistema principal.

Contras:
- mais pesado;
- pior para validar comportamento de interface e integração com o desktop;
- aumenta custo de uso e manutenção.

Quando escolher:
- quando você quer isolamento forte e aceita a perda de praticidade;
- quando o comportamento do app não exige interação fina com o desktop real.

### Recomendação

- Para este projeto, a melhor opção é **desenvolvimento direto no host com repositório separado e git**.
- Motivo: o app depende de atalho global, clipboard e popup no desktop real. Container e VM tendem a atrapalhar a validação do comportamento principal.

## 2. Versionamento de código

### Opção A: git local obrigatório, com commits pequenos

Prós:
- permite voltar atrás com segurança;
- registra a evolução do projeto;
- combina com o método Akita;
- facilita revisão e refatoração.

Contras:
- exige disciplina;
- adiciona um pouco de ritual ao fluxo de trabalho.

Quando escolher:
- sempre, para este projeto.

### Opção B: trabalhar sem git no início

Prós:
- zero atrito inicial.

Contras:
- alto risco de perder trabalho;
- dificulta revisão;
- prejudica commits pequenos e reversão segura;
- não combina com o método.

Quando escolher:
- não recomendado para este projeto.

### Recomendação

- **git é obrigatório** desde o início.
- Commits devem ser pequenos, atômicos e legíveis.

## 3. Integração com GitHub

### Opção A: repositório privado + fine-grained personal access token limitado ao repositório

Prós:
- escopo restrito;
- bom equilíbrio entre segurança e simplicidade;
- permite controle de acesso ao projeto específico;
- é mais fácil de revogar ou expirar depois.

Contras:
- precisa cuidar da validade do token;
- se o token for mal configurado, pode ficar mais permissivo do que o desejado;
- a gestão do segredo exige atenção.

Quando escolher:
- quando você quer usar GitHub com acesso restrito ao projeto e sem complicar demais o fluxo.

### Opção B: repositório privado + SSH key/deploy key limitada ao repositório

Prós:
- escopo por repositório;
- boa segurança para acesso só ao projeto;
- funciona bem para git push/pull.

Contras:
- gerenciamento menos intuitivo para quem não usa SSH com frequência;
- pode ser mais chato de configurar e depurar;
- é menos flexível para integrações futuras além de git.

Quando escolher:
- quando o foco é apenas push/pull no repositório e você quer uma credencial bem restrita.

### Opção C: GitHub App ou outro mecanismo mais sofisticado de integração

Prós:
- pode ser muito bem restringido;
- é mais elegante para automações.

Contras:
- mais complexo;
- desnecessário para o estágio atual;
- adiciona burocracia sem benefício proporcional neste momento.

Quando escolher:
- só quando houver automações mais avançadas ou necessidade de integração formal.

### Recomendação

- Para este projeto, a melhor opção inicial é **repositório privado + fine-grained personal access token limitado ao repositório**.
- Se a configuração ficar simples e você preferir uma credencial ainda mais “por repositório”, a alternativa de SSH/deploy key também é válida.

## 4. Estratégia de branches e commits

### Opção A: `main` protegido + branch de trabalho

Prós:
- reduz chance de quebrar a base principal;
- incentiva commits pequenos;
- facilita retorno para um estado conhecido bom.

Contras:
- adiciona um pouco de processo;
- exige mais disciplina para criar branch e mesclar mudanças.

Quando escolher:
- quando você quer controle e segurança desde o início.

### Opção B: trabalhar direto em `main`

Prós:
- mais simples;
- menos etapas.

Contras:
- maior risco de bagunçar a linha principal;
- mais difícil reverter mudanças grandes;
- menos alinhado ao método.

Quando escolher:
- não recomendado.

### Recomendação

- Use uma estratégia **híbrida**:
  - `main` fica protegida;
  - mudanças de risco ou escopo maior vão para branch curta de trabalho;
  - mudanças pequenas e seguras podem ser feitas diretamente na branch de trabalho atual, sem criar branches desnecessárias;
  - o merge para `main` só acontece depois de validação.
- O objetivo é evitar burocracia sem abrir mão de controle.

## 5. Definição de pronto para esta etapa

Esta etapa só está pronta quando:

- o nível de isolamento de desenvolvimento foi decidido;
- o método de versionamento foi definido;
- a forma segura de usar GitHub foi escolhida;
- a estratégia de branch e commit está clara;
- o próximo passo é escrever a spec do produto.
