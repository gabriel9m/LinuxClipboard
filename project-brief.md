# Project Brief

## O que quero construir

Uma aplicação para utilizar no Zorin OS 17.3 Core 64 bits que ofereça uma experiência parecida com o `Win+V` do Windows 10/11, mas acionada por `Super+V` no Linux: ao pressionar o atalho, abrir uma janela com o histórico da área de transferência do computador, incluindo textos e imagens copiadas.

## Problema que isso resolve

No Zorin OS, sinto falta de um histórico fácil e rápido da área de transferência para recuperar itens copiados anteriormente, sem precisar depender de copiar novamente ou perder conteúdo importante.

## Para quem é

Para uso pessoal neste computador com Zorin OS.

## Objetivo principal

Ter acesso rápido ao histórico da área de transferência com um atalho de teclado global, em uma interface simples e confiável.

## Funcionalidades desejadas

- capturar textos copiados;
- capturar imagens copiadas;
- manter um histórico local;
- abrir a janela do histórico com atalho de teclado;
- permitir selecionar um item do histórico para reutilizar;
- usar `Super+V` para abrir o pop-up com os últimos 25 itens;
- navegar pela lista com seta para cima, seta para baixo e `Enter`.

## O que não deve fazer

- não precisa ser um produto para várias pessoas agora;
- não precisa depender de internet;
- não precisa sincronizar com outros dispositivos neste momento.

## Restrições

- rodar no Zorin OS 17.3 Core 64 bits;
- ser prático de usar no desktop;
- funcionar de forma confiável como ferramenta de uso diário.
- priorizar leveza, funcionalidade, manutenabilidade e segurança;
- limitar o histórico aos últimos 25 itens copiados;
- usar Rust + GTK4;
- usar JSON + arquivos locais para imagens;
- distribuição inicial via AppImage;
- evitar uma base tecnológica pesada sem necessidade.

## Preferências não técnicas

- experiência parecida com o `Win+V` do Windows;
- acesso rápido;
- simplicidade de uso.
- fluxo direto: abrir com atalho, navegar com setas, confirmar com `Enter`, ou clicar no item e colar.

## Prazo ou urgência

Não definido.

## Critério de sucesso

Pressionar o atalho e ver o histórico da área de transferência, com textos e imagens, funcionando de forma estável no Zorin OS.

## Dúvidas em aberto

- deve existir busca no histórico já na primeira versão?
- deve haver suporte a múltiplos tipos além de texto e imagem no futuro?

## Observações adicionais

O foco inicial é reproduzir a experiência do `Win+V` de forma confiável no Linux, começando pelo uso pessoal.

Comportamento esperado do usuário:

- `Ctrl+C` continua funcionando normalmente;
- a aplicação roda em segundo plano e armazena automaticamente o conteúdo copiado;
- `Super+V` abre o pop-up com os últimos 25 itens;
- ao clicar em um item, o conteúdo deve ser colado automaticamente no aplicativo que estiver em foco, quando a plataforma permitir;
- ao navegar com `seta para cima`, `seta para baixo` e `Enter`, o item escolhido também deve ser colado automaticamente no app em foco, quando isso for possível.

Decisão técnica atual:

- a interação de colar seguirá uma abordagem híbrida;
- o app tentará colar automaticamente no aplicativo em foco quando a plataforma permitir;
- quando isso não for possível, o item continuará disponível no clipboard para colagem manual.
