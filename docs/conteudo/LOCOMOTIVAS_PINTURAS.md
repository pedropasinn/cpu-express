# Locomotivas e pinturas — segunda leva

Agente 7, 2026-11-15. Conteúdo `kind: fictional`, original e desenhado em código na grade do renderer. Nomes, cores e formas são autorais; não representam modelos, fabricantes ou ferrovias reais. Não há ficha histórica, imagem externa ou logotipo.

## Locomotivas

| Nome / identificador estável | Silhueta e detalhes | Obtenção provisória | Raridade proposta |
| --- | --- | --- | --- |
| Expresso das Andorinhas / `express_steam` | Vapor expresso, caldeira baixa afilada, três rodas motoras maiores, biela contínua e pequeno eixo traseiro. | Postal Companhia de viagem: 20 passageiros entregues. | Raro: desenho de expresso e rodas grandes como escolha de coleção. |
| Articulada dos Vales / `articulated_steam` | Vapor articulada, cabine alta, caldeira estreita elevada, dois conjuntos separados de bielas e cilindros. | Postal Correio sobre trilhos: 20 unidades de carga entregues. | Épico: articulação ilustrada pelo mecanismo duplo; sem multiplicador de produção. |
| Diesel Dois Horizontes / `twin_diesel` | Duas cabines altas nas extremidades, compartimento central rebaixado, duas ventoinhas e grelhas. | Postal Correio sobre trilhos: 20 unidades de carga entregues. | Incomum: alternativa visual de linha, utilitária e sem superioridade sobre a manobreira. |
| Cremalheira do Mirante / `rack_electric` | Elétrica compacta, cabine deslocada, nariz em rampa, coletor de teto e roda dentada lateral. | Postal Companhia de viagem: 20 passageiros entregues. | Raro: tema de ferrovia de montanha; não cria trilho dentado nem bônus em rampa. |

São quatro opções visuais, sem alterar velocidade lógica, capacidade, cargas, progresso, postais ou serviços. A articulação é uma representação lateral; não há movimento independente de bogies em curvas. O expresso conserva os centros horizontais das rodas e o contato em y=57, elevando o centro vertical das três rodas grandes para y=53. A fase continua vindo de `Train`, sem relógio novo. As duas máquinas a vapor usam a boca existente em (220,32); diesel e elétrica não desenham fumaça, mesmo quando há partículas no World.

## Pinturas

| Nome / identificador estável | Corpo / luz / sombra (RGB) | Chegadas | Raridade proposta e motivo |
| --- | --- | ---: | --- |
| Cereja da Estação / `cherry` | 149,52,76 / 208,108,115 / 89,37,60 | 16 | Comum: esmalte quente de uso cotidiano. |
| Marfim da Manhã / `ivory` | 213,202,165 / 245,232,197 / 143,132,107 | 20 | Incomum: composição clara, mantendo contornos e ferragens escuros. |
| Turquesa da Lagoa / `lagoon` | 36,128,139 / 102,187,181 / 27,77,94 | 24 | Comum: alternativa fria e aquática ao azul antigo. |
| Urze dos Montes / `heather` | 128,78,130 / 191,139,184 / 77,48,88 | 28 | Raro: esmalte floral de coleção. |
| Girassol do Campo / `sunflower` | 196,148,44 / 239,203,104 / 121,83,37 | 36 | Incomum: amarelo terroso distinto do cobre, sem brilho intermitente. |
| Ardósia da Chuva / `slate` | 79,99,113 / 144,165,176 / 43,59,76 | 48 | Comum: opção sóbria, sem perder os recortes claros. |

Os valores, marcos e raridades são propostas para Pedro confirmar. Raridade não determina desempenho ou tempo de desbloqueio nesta implementação. Não há moeda, saldo, caixa, sorteio ou campo persistido novo. As pinturas usam o mecanismo existente: trocam três tons do esmalte da **locomotiva**, preservando vagões, cenário, vidros, luzes, ferragens, rodas e bielas. Não há nova animação decorativa. No tema Legacy, pinturas continuam ignoradas como antes; a escolha da locomotiva é respeitada.

## Compatibilidade e integração

Enums e listas `ALL` apenas acrescentam variantes ao final. O catálogo mantém versão 1 e o checkpoint permanece v4. `LiveryDef.kind` é metadado de catálogo, com default fictício para documentos anteriores; não é campo do perfil. Configurações anteriores mantêm defaults e IDs antigos.

A validação de serviço continua exigindo as quatro pinturas originais em saves que alcançaram seus marcos. Um v4 anterior com muitas chegadas pode não possuir as seis novas: ele continua válido, conserva sua propriedade ao carregar e recebe as adições elegíveis **na próxima chegada**, pela transação existente do World. Reabrir, pausar ou ler menus não concede nada. Migrações legadas derivam somente as quatro pinturas originais das chegadas, preservando o comportamento anterior; também recebem as novas na chegada seguinte. Um executável antigo que desconhece os novos enums pode recusar configurações/checkpoints com esses valores; esta entrega não promete leitura reversa.

As listas da oficina já percorrem `ALL`; somente o cache e o índice das miniaturas de pinturas foram ampliados em `src/ui_stock.rs`. Não houve alteração de layout ou de `src/app*`. O integrador deve confirmar os marcos/raridades, explicar a concessão na próxima chegada para perfis avançados e conferir as listas maiores em 480 × 480. A amostra de pintura da oficina continua no Vapor Aurora; mostrar o motor equipado é melhoria de UI separada. Os postais atuais liberam vários modelos ao mesmo tempo; obter cada locomotiva separadamente requer projeto de propriedade/persistência futuro.

## Conferência local

O exemplo `engine_gallery` usa o renderer real, sem ler o perfil nem acessar a rede. O modo padrão percorre dez locomotivas × dez pinturas. A ordem de linhas é Vapor Aurora, Diesel do Porto, Elétrica da Serra, Vapor das Pedreiras, Manobreira do Pátio, Automotriz dos Campos e os quatro modelos da tabela acima. Colunas: Verde Aurora, Azul do Porto, Cobre do Poente, Expresso Noturno e as seis pinturas acima.

`--clima` mostra os quatro modelos novos, nas primeiras quatro pinturas novas, com colunas dia, entardecer, noite e chuva. `--cenas` mostra as mesmas combinações na cena completa. As miniaturas mantêm o recorte 60 × 29, incluindo rodas e engates. A filtragem da ampliação é nearest inteira.

```powershell
$env:CARGO_NET_OFFLINE = 'true'
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','engine_gallery','-j','2','--','docs/conteudo/agente7-matriz-final-v2.ppm')"
node scripts/content-png.cjs docs/conteudo/agente7-matriz-final-v2.ppm docs/conteudo/agente7-matriz-final-v2.png 4
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','engine_gallery','-j','2','--','docs/conteudo/agente7-clima-final-v2.ppm','--clima')"
node scripts/content-png.cjs docs/conteudo/agente7-clima-final-v2.ppm docs/conteudo/agente7-clima-final-v2.png 4
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','engine_gallery','-j','2','--','docs/conteudo/agente7-cenas-final-v2.ppm','--cenas')"
node scripts/content-png.cjs docs/conteudo/agente7-cenas-final-v2.ppm docs/conteudo/agente7-cenas-final-v2.png 2
```

Execute uma compilação por vez, no máximo dois jobs. Os exportadores recusam sobrescrever imagens: escolha novos nomes para outra revisão. Os resultados dos gates e os limites da inspeção ficam no relato do Agente 7 ao fim de `docs/RODADA_CONTEUDO.md`. Imagens headless não demonstram o compositor, DPI, movimento interativo ou leitura na faixa real de 60 pixels físicos.
