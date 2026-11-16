# Menus E6 — loja, remessas e coleção

Implementação de desenvolvimento de 16/11/2026. Usa a economia E1b e a central de 704 × 676 pontos da E5. Não altera checkpoint, preços, catálogo, probabilidades ou distribuição rc.3.

## Fluxos

- **Loja → Remessas:** cinco objetos próprios, preço, garantia restante e chances efetivas da próxima abertura. A remessa inicial do Pátio usa o botão Abrir gratuitamente. `i` abre uma página focável de informações, com conteúdo paginado, regra de novidade e reembolso. A garantia é normalizada: Garagem no limite mostra 86,7% raro e 13,3% épico. Plataforma e Ateliê mostram 0% épico.
- **Abrir:** `progress::economy_action` grava débito, inventário, RNG, garantia e resultado antes da apresentação. A animação dura 1,2 s: tremor, tampa levantada e revelação. Menos movimento mostra a peça imediatamente. O resultado fica pendente até Continuar confirmar sua apresentação em disco. Fechar o painel ou o programa antes disso preserva a mesma peça; a próxima abertura da central direciona à Loja. Uma falha de gravação não consome a caixa nem permite repetir o sorteio.
- **Resultado:** miniatura do StockGallery, moldura e palavra de raridade, nome, Nova! ou Repetida · +N moedas. Equipar permite escolher uma das três posições para vagões e respeita ocupação/carga. Continuar não equipa automaticamente. Reembolso usa o valor já salvo, inclusive compras antigas de preço diferente.
- **Compra direta:** três categorias e quatro modelos por página. Modelos possuídos não são vendidos novamente. A compra é imediata e salva; a tela seguinte permite equipar. A posse vale para repetir o modelo em várias posições.
- **Coleção:** vagões, motores e pinturas mostram total possuído por categoria e por raridade. Postais preservam a coleção de estações, excursões e serviços. Postais não têm raridade no domínio: não foi inventada uma classificação. Os atalhos levam à página correta do Trem ou à compra direta.
- **Museu:** após os 39 modelos, mostra os selos numerados de 1 a N e compra o próximo por 120 réis, pela transação do domínio. Postais não bloqueiam selos.
- **Recompensas:** `+N` sai da locomotiva na faixa e na prévia somente após observar renda de chegada confirmada; leituras da UI não pagam nada. Dura dois segundos, fica estático em Menos movimento e desaparece no Modo tranquilo. Chegadas que só acumulam frações não mostram `+0`. A posição acompanha a origem do trem também em faixas largas/estreitas.
- **Resumo:** ao abrir a central, o rodapé informa moedas creditadas desde a última visita **nesta sessão** e a quantidade atual de ordens prontas para recolher. Usa o total durável de créditos, portanto inclui prêmios e reembolsos e não confunde gasto com renda. Não é renda offline nem histórico entre execuções. A informação é de apresentação, sem campo novo no save. Erros de gravação têm prioridade no rodapé.
- **Trem → Máquina:** espaço reservado com explicação de indisponibilidade. `src/simulation/machine.rs` não existia nesta entrega. A E2 deve substituir esse conteúdo pelos medidores reais; não há porcentagens ou efeitos simulados apresentados como reais.

Compras em demonstração sem caminho de save ficam indisponíveis com motivo visível. Os exportadores usam esse modo; os testes dos botões habilitados usam diretórios temporários próprios. Pinturas continuam exemplificadas no Vapor Aurora pelo StockGallery, sem trocar o modelo equipado.

## Arte e reprodução

Oito SVGs novos sem fundo em `assets/icons/`: `moeda`, `pilha-moedas`, `saco-moedas`, `caixa-patio`, `caixa-oficina`, `caixa-plataforma`, `caixa-garagem`, `caixa-atelie`. A roda na moeda distingue Réis de Ferro da condecoração de pontos. A versão grande `grandes/moeda-48.svg` amplia a geometria por três, sem suavização. `draw_economy.cjs` conserva a geometria autoral; não lê imagens externas.

O gerador canônico continua `assets/icons/build_sprites.py`. A tentativa de executá-lo com o Python 3.13 instalado retornou **Acesso negado**. Não houve instalação ou alteração de permissões. `build_sprites.cjs` é uma alternativa local sem dependências que reproduz a mesma ordenação, parser de retângulos e formato de tabela; gerou `src/ui_sprites.rs`, depois formatado pelo Cargo. Não foi possível comparar as duas execuções nesta máquina. O validador PowerShell foi ampliado para os 51 sprites da pasta e conferiu grade, paleta, contorno, transparência e ausência de sobreposições.

```powershell
node assets/icons/draw_economy.cjs
py -3.13 assets/icons/build_sprites.py # gerador canônico, requer Python executável
# Alternativa local usada nesta entrega:
node assets/icons/build_sprites.cjs
powershell -NoProfile -ExecutionPolicy Bypass -File assets/icons/validar.ps1
```

As telas E6 têm prefixo `floating_e6_`; os arquivos E5 permanecem como histórico. São triângulos e texturas reais do egui rasterizados localmente, sem janela nativa. Índices 0–4 são vagões, 5–7 motores e 10–12 pinturas. Na coleção, 15–16 são postais. Índices ausentes evitam duplicar páginas limitadas pelo paginador.

Com a toolchain local configurada como em `scripts/dev.ps1`, Cargo offline e `CPU_EXPRESS_CONFIG` temporário:

```powershell
$env:CARGO_NET_OFFLINE = 'true'
$env:CPU_EXPRESS_CONFIG = Join-Path $env:TEMP 'cpu-express-e6-validation/config.json'
$env:CPU_EXPRESS_VISUAL_DIR = [IO.Path]::GetFullPath('docs/expansao/telas')
cargo test --locked --test ux_visual -j 2 export_economy_menus -- --ignored --nocapture
Get-ChildItem docs/expansao/telas/floating_e6_*.json | ForEach-Object {
    node docs/ux_rasterizar.cjs $_.FullName
}
```

## Limites

O resumo entre execuções exigiria persistir um marcador de leitura; ficou fora para manter o v5 intacto. Os testes de clique são headless, não teclado/Narrador, DPI ou compositor. Motor e pintura usam a gravação protegida de preferências existente; composição de vagões é gravada antes de publicar a troca. Não há promessa de atomicidade entre os dois arquivos de preferências/jornada em queda física de energia. Compra, inventário, reembolso e revelação compartilham a transação econômica existente.

A revisão E7 ainda deve conferir o fluxo interativo no Windows, foco/teclado, contraste, escalas fracionárias, animação vista em movimento e fechamento real do aplicativo. Não foram executados release, empacotamento, benchmark ou uso prolongado.
