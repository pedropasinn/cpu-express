# Conferência do material rodante — Agente 1

Fonte desta rodada: `cpu-express-conteudo`, 2026-11-15. Arte procedural original, sem recursos externos, marcas ou fichas históricas. Os geradores usam o renderer real do projeto e não abrem o perfil do jogador nem consultam a rede.

## Estado da conferência

Aguardando resolver os adaptadores de enums nos arquivos protegidos. `adaptadores-agente1.patch` foi preparado e passou em `git apply --check`, **mas não foi aplicado**. Nenhum PNG foi gerado ou inspecionado ainda. Não tratar os desenhos novos como aprovados visualmente, nem a árvore como pronta para integração.

O patch substitui os nove matches incompatíveis por conversões/métodos públicos e troca a lista fixa da oficina por `CoachStyle::ALL`. Os arquivos originais de app e menus continuam intactos. Aplicá-lo exige exceção expressa à regra de território; a solicitação foi enviada ao Pedro durante a execução. O script `scripts/prepare-content-adapters.cjs` apenas gera esse arquivo de revisão, não altera fontes protegidas e recusa sobrescrever um patch existente.

## Exportação prevista depois da integração dos adaptadores

Usar `scripts/dev.ps1` com dois jobs, uma execução de Cargo por vez e `CARGO_NET_OFFLINE=true`. Em PowerShell com política de scripts restrita:

```powershell
$env:CARGO_NET_OFFLINE = 'true'
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','engine_gallery','-j','2','--','docs/conteudo/agente1-motores.ppm')"
node scripts/content-png.cjs docs/conteudo/agente1-motores.ppm docs/conteudo/agente1-motores.png 4
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','stock_gallery','-j','2','--','docs/conteudo/agente1-material.ppm')"
node scripts/content-png.cjs docs/conteudo/agente1-material.ppm docs/conteudo/agente1-material.png
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','art_review','-j','2','--','docs/conteudo/agente1-aurora.ppm')"
node scripts/content-png.cjs docs/conteudo/agente1-aurora.ppm docs/conteudo/agente1-aurora.png
```

As galerias de motores e material e o conversor recusam substituir arquivos existentes. Para uma nova revisão, usar nomes novos. `art_review` é o exportador existente: conferir que o caminho de saída é novo antes de executá-lo.

## Legenda das folhas

- **Motores:** seis linhas na ordem Vapor Aurora, Diesel do Porto, Elétrica da Serra, Vapor das Pedreiras, Manobreira do Pátio e Automotriz dos Campos; quatro colunas Verde Aurora, Azul do Porto, Cobre do Poente e Expresso Noturno. Recorte nativo de 60 × 29, ampliado 4× por nearest.
- **Material:** onze linhas na ordem `CoachStyle::ALL`: passageiros, dormitório, carga, tanque, panorâmico, restaurante, correio/bagagem, gôndola, graneleiro, toras e frigorífico. Colunas: dia, entardecer, noite e chuva. Cada célula tem 320 × 70; as locomotivas alternam na ordem acima.
- **Aurora:** exportação existente de quatro biomas e composições antigas para conferir continuidade visual. Não substitui os testes de hash.

As miniaturas de oficina usam o mesmo renderer, recortes de 42 × 28 e 60 × 29, nearest e upload somente na criação de `StockGallery`. A inspeção das imagens será headless; não comprova apresentação pelo compositor, faixa no desktop, DPI ou navegação nativa.

## Atualização de estado — Agente 2, 2026-11-15

O registro acima descreve a entrega histórica do Agente 1. A base recebida pelo Agente 2 já continha os adaptadores em `10073dd`. Após corrigir a expectativa antiga de `hopper` desconhecido no teste de catálogo, a suíte final desta rodada passou: 356 testes, zero falhas, dois exportadores opcionais ignorados; Clippy sem warnings e formatação aprovados. Não houve edição de arquivos protegidos pelo Agente 2. As galerias específicas de motores/vagões continuam com os comandos históricos acima, sem afirmar nova inspeção visual delas.

A documentação de cenário, legenda das linhas e instruções de integração estão em [CENARIO.md](CENARIO.md). Imagens finais geradas e inspecionadas nesta rodada:

- [12 etiquetas e quatro combinações, em quatro condições](agente2-cenarios-final.png).
- [13 trechos de Paranaguá–Curitiba com etiquetas e biomas do catálogo](agente2-trechos-final.png).
- [Circuito Aurora pelo exportador art_review](agente2-aurora-final.png).

As imagens sem o sufixo `-final` são prévias preservadas para histórico. O relato completo do Agente 2 está no fim de [RODADA_CONTEUDO.md](../RODADA_CONTEUDO.md).
