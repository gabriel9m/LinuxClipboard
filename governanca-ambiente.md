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

- Para este projeto, a opção adotada é **repositório privado + SSH deploy key limitada ao repositório**.
- Motivo: o acesso necessário neste estágio é apenas `git push`/`git pull`, então a deploy key dá escopo menor do que uma credencial de conta.
- A chave privada deve ficar fora do repositório, em `~/.ssh`, com permissão `600`.
- A chave pública deve ser cadastrada no GitHub como deploy key do repositório, com permissão de escrita somente quando o agente precisar fazer `push`.

### 3.1 Persistência segura da chave SSH entre sessões

Problema:
- novos chats ou novas execuções de CLI podem perder configuração local temporária, mas não devem exigir uma nova chave SSH a cada sessão;
- a chave privada também não pode ser versionada nem copiada para arquivos do projeto.

Decisão:
- manter uma chave dedicada por repositório em `~/.ssh`;
- configurar um alias persistente em `~/.ssh/config`;
- apontar o remoto Git para esse alias;
- nunca salvar chave privada no repositório.

Configuração adotada neste host:

```sshconfig
Host github-linuxclipboard
  HostName github.com
  User git
  IdentityFile /home/gabriel/.ssh/linuxclipboard_deploy_key
  IdentitiesOnly yes
  AddKeysToAgent yes
```

Remoto Git adotado:

```bash
git@github-linuxclipboard:gabriel9m/LinuxClipboard.git
```

Prós:
- a mesma chave é reutilizada em novos chats/CLIs na mesma máquina;
- o escopo continua limitado ao repositório;
- não depende de token pessoal de conta;
- a chave privada permanece fora do projeto e fora do Git.

Contras:
- se a máquina for perdida, a chave precisa ser revogada no GitHub;
- se o projeto for clonado em outra máquina, será necessário copiar a chave com segurança ou criar uma nova deploy key;
- deploy key é boa para Git, mas não substitui GitHub App ou token para automações avançadas da API.

Procedimento de recuperação em novo clone na mesma máquina:

```bash
git remote set-url origin git@github-linuxclipboard:gabriel9m/LinuxClipboard.git
ssh -T github-linuxclipboard
```

Regra de segurança:
- a chave privada `/home/gabriel/.ssh/linuxclipboard_deploy_key` não deve ser enviada para chats, commits, tickets ou documentação;
- apenas a chave pública `.pub` pode ser cadastrada no GitHub;
- ao encerrar o projeto ou suspeitar de exposição, revogar a deploy key no GitHub e gerar outra.

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

## 6. Dependências nativas de desktop

A integração GTK4 deve permanecer opcional durante o desenvolvimento inicial.

Decisão:
- manter o crate `gtk4` atrás da feature Cargo `desktop-gtk`;
- manter os testes de domínio, storage, clipboard, paste e UI pura sem dependências nativas;
- validar GTK/GDK separadamente com `cargo check --features desktop-gtk`.

Pacotes necessários no Zorin OS 17.3:

```bash
sudo apt-get update
sudo apt-get install -y pkg-config libgtk-4-dev
```

Motivo:
- `pkg-config` é necessário para os crates `*-sys` encontrarem GObject/GTK;
- `libgtk-4-dev` fornece headers e arquivos `.pc` do GTK4;
- sem esses pacotes, `cargo check --features desktop-gtk` falha antes de compilar o adaptador do projeto.

Regra:
- não tornar GTK uma dependência obrigatória enquanto a lógica principal ainda estiver sendo estabilizada;
- a feature padrão deve continuar compilando e testando sem bibliotecas de desktop.
