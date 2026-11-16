# Cenário E4 — bairros e construções

Arte original desenhada em Rust; **kind: fictional** para todos os 38 elementos. As etiquetas da linha real são escolhas editoriais de ambientação: nenhuma construção é apresentada como reprodução de um edifício real. Não há marcas, downloads, recursos externos, efeitos econômicos ou novo formato de save.

## Elementos

A galeria de elementos tem oito colunas, lidas da esquerda para a direita. A última linha tem seis desenhos e duas células vazias.

| Nº | Elemento | Nº | Elemento |
| --- | --- | --- | --- |
| 1 | Casa de madeira | 20 | Fábrica de chaminé |
| 2 | Casa de alvenaria | 21 | Fábrica de sheds |
| 3 | Sobrado geminado | 22 | Oficina de bairro |
| 4 | Casa com varanda | 23 | Armazém de carga |
| 5 | Casa de quatro águas | 24 | Silos geminados |
| 6 | Armazém de esquina | 25 | Reservatório industrial |
| 7 | Padaria | 26 | Olaria |
| 8 | Estufa de quintal | 27 | Pórtico de contêineres |
| 9 | Igreja com campanário | 28 | Galpão do cais |
| 10 | Mercado de toldos | 29 | Pilha de contêineres |
| 11 | Caixa-d'água elevada | 30 | Navio de dois mastros |
| 12 | Coreto | 31 | Farol de cais |
| 13 | Praça com fonte | 32 | Píer com cabeços |
| 14 | Edifício de três andares | 33 | Rua com iluminação |
| 15 | Apartamentos de cinco andares | 34 | Muro com portão |
| 16 | Torre de oito andares | 35 | Campinho |
| 17 | Cinema | 36 | Praça de brinquedos |
| 18 | Escola | 37 | Pomar cercado |
| 19 | Biblioteca | 38 | Quintal com varal |

Cada entrada tem desenho próprio, além da variação de cor, luz e disposição por semente. Elementos pequenos podem ficar atrás do trem, como o restante da paisagem. A galeria isolada permite conferir os detalhes que a composição encobre.

## Integração e contratos

`SegmentScenery` agora usa `u32` (17 bits ocupados). `town`, `city`, `industrial`, `suburb` e `village` foram acrescentadas; `port` já existia e foi reutilizada. O caminho vazio e o tema Legacy continuam preservados. O E5 já havia ligado `set_scenery_context`, `set_station_styles` e as opções de movimento; essa ligação foi mantida.

`SceneRenderer::set_settlement_context(SettlementContext { seed, density })` recebe apenas dados de apresentação. O renderer não consulta catálogo, Config, métricas, disco ou relógio. A semente escolhe lotes, construções, fachadas e janelas apagadas. Não há RNG mutável nem consumo da semente econômica. O mesmo estado sempre produz a mesma imagem.

`SettlementContext::along_segment(seed, progress, departure, arrival)` calcula uma curva suave no primeiro e último quarto do trecho. As flags indicam se os extremos são habitados. A densidade cai depois da partida, chega a zero no meio e aumenta antes da chegada elegível. Lotes têm limiares estáveis e revelação vertical curta para evitar que o edifício inteiro apareça de uma vez. A cidade não modifica a progressão lógica. A camada anterior de porto/urbanização permanece como ambientação de base; o controle de densidade governa os elementos E4.

No aplicativo atual, o adaptador identifica Vila Aurora e Porto do Sol como extremos habitados; Bosque e Serra permanecem naturais. Vila Aurora ganhou `village` e `town`; o porto usa `port`. Excursões continuam limpando as etiquetas. O catálogo da linha real ganhou indústria junto a Paranaguá, vila/centro junto a Morretes e cidade/subúrbio/indústria junto a Pinhais e Curitiba. Os trechos da serra conservaram floresta e suas obras anteriores. O status da linha e suas dúvidas editoriais não foram alterados.

**Para E3:** substituir o índice Aurora no adaptador pelos extremos do itinerário ativo, invertendo partida/chegada na volta. Reutilizar `along_segment` com o progresso do trecho ativo e a semente da jornada; não persistir densidade. Esta entrega prepara e ilustra as etiquetas da linha real, mas não habilita sua viagem.

Três camadas usam bases y=41/47/53 e velocidades 1/32, 1/8 e 1/4. O tile de 960 colunas fecha nos períodos 30.720/7.680/3.840, todos divisores de 184.320. A largura não altera identidade nem frequência dos lotes. O desenho termina antes de y=54; trilhos, estações e trem são pintados depois. O recorte vertical do Canvas não altera seus defaults.

Janelas acendem gradualmente pela iluminação existente, com algumas apagadas por semente. Fumaça da padaria/fábrica/olaria responde ao vento recebido; guindaste, balanço e varal usam apenas tempo ativo. Movimento reduzido e Low fixam essas poses; fumaça é omitida em Low. Chuva acrescenta reflexo na rua e recolhe o balanço do varal, neve marca o chão dos lotes. Véus de chuva/neblina e partículas continuam no renderer existente. Não se coleta clima no desenho.

## Reprodução

Imagens finais geradas e abertas para inspeção nesta entrega:

| Imagem | Dimensão | Conteúdo |
| --- | --- | --- |
| [Elementos de dia](cenario-elementos-dia-final.png) | 1024 × 700 | 38 desenhos, ampliação nearest 2× |
| [Elementos à noite](cenario-elementos-noite-final.png) | 1024 × 700 | Mesma ordem, janelas e iluminação noturnas |
| [Paisagens](cenario-paisagens-final.png) | 2560 × 420 | Seis famílias, quatro condições, células 640 × 70 |
| [Aproximação](cenario-aproximacao-final.png) | 2560 × 420 | Densidades 0 / 0,35 / 0,70 / 1 |
| [Faixa de 60 px](cenario-faixa60-final.png) | 2560 × 60 | Cidade em quatro condições, recorte sem reescala |
| [Trechos da linha real](cenario-linha-final.png) | 1280 × 910 | 13 trechos do catálogo, na ordem Paranaguá → Curitiba |

As duas imagens com sufixo `-revisao` foram preservadas como etapa intermediária; os arquivos `-final` acima são a referência desta entrega. Os PPMs e logs ficam locais, ignorados pelas regras existentes do Git.

Usar Cargo offline, toolchain local e no máximo dois jobs. Os exportadores recusam sobrescrever saídas; escolher outro nome ao repetir. Não executam o app nem abrem o perfil.

```powershell
$env:CARGO_NET_OFFLINE = 'true'
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','scenery_gallery','-j','2','--','docs/expansao/nova-galeria.ppm','--expansao')"
node scripts/content-png.cjs docs/expansao/nova-galeria.ppm docs/expansao/nova-galeria.png
```

Modos novos: `--elementos`, `--elementos-noite`, `--expansao`, `--aproximacao`, `--faixa60`. Modos anteriores padrão/`--linha`/`--vida`/`--estatico` continuam disponíveis. A expansão tem seis linhas: vila, centro, cidade, indústria, porto e subúrbio. As quatro colunas mostram dia, entardecer, noite e chuva; na aproximação mostram densidades 0/0,35/0,70/1, em luz fixa. `--faixa60` exporta a linha de cidade com 60 px de altura física simulada: linhas 10–69 da imagem original, sem reescala, exatamente o recorte superior empregado pela faixa de 70 linhas em uma área de 60 px.

As imagens são renderizações headless da fonte desta entrega. Não provam compositor, DPI, animação interativa, desempenho/energia ou legibilidade observada na barra real. A conferência visual e os gates efetivamente executados estão no relato E4 ao fim de `docs/PLANO_EXPANSAO.md`.
