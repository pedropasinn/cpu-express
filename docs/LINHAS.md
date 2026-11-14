# CPU Express — catálogo de conteúdo e linhas

Atualizado em 2026-11-14. Responsável: Claude Code. Código em `src/simulation/catalog.rs`, dados em `data/`, testes em `tests/catalog.rs`.

## O que é

O catálogo reúne as definições de conteúdo, separadas do estado da partida:

- `data/catalog/core.json`: vagões, cargas, pinturas, recompensas de serviço e locomotivas.
- `data/lines/*.json`: uma linha por arquivo, com estações, trechos, fontes e questões em aberto.

Os documentos são embutidos no executável na compilação; o jogo não lê arquivos nem usa rede para carregá-los. Saves e referências usam somente `id`. O nome exibido é conteúdo e pode mudar sem afetar um save.

**Estado nesta data:** o Circuito Aurora já roda com o conteúdo do catálogo: nomes das estações, comprimentos de base, variação por seed e biomas vêm de `data/lines/aurora-v1.json` (os valores antigos ficam só como reserva, com teste que os mantém idênticos). Vagões, cargas, pinturas, recompensas e locomotivas continuam definidos no código e espelhados no catálogo por testes de paridade. O motor de viagem ainda é o de **circuito com quatro estações**; a matemática para linhas com qualquer número de estações e de ponta a ponta existe em `src/simulation/itinerary.rs`, testada, mas ainda não está ligada à viagem, aos serviços, ao mapa nem ao desenho das estações.

## Regras de validação

- Identificadores: minúsculas, dígitos, `-` ou `_`, começando por letra, até 48 caracteres, únicos dentro de cada lista.
- Referências precisam existir: vagão de uma carga, recompensa que libera uma locomotiva, estações de um trecho, fontes citadas.
- Vagão transporta passageiros ou carga, nunca ambos nem nenhum.
- Conectividade: os trechos seguem as estações ativas na ordem. Em `circuit` o último trecho volta à primeira estação; em `shuttle` a linha vai de ponta a ponta e o trem retorna pelos mesmos trechos.
- Linha real: ao menos uma fonte. Para ser `playable` precisa também de época, bitola, `km` crescente a partir de zero, estações verificadas com fonte, `distance_km` coerente com os quilômetros e nenhuma questão em aberto.
- Cada trecho declara o grau de adaptação: `faithful`, `simplified` ou `invented`.
- Dados derivados de fonte licenciada exigem `data_license` e `attribution` na linha.

O núcleo inválido impede o carregamento do catálogo. Uma linha inválida fica fora da sessão, com o erro indicando linha, estação ou trecho; as demais continuam disponíveis. Linhas `draft` são validadas na estrutura e nunca aparecem para o jogador.

## Itens renomeados, desativados ou desconhecidos

Cada item aceita `aliases` (nomes antigos) e `retired`. A consulta devolve um de quatro resultados:

| Resultado | Significado para um save existente |
| --- | --- |
| `Current` | O identificador vale como está. |
| `Renamed` | Passa a usar o identificador novo, mantendo o que o save possuía. |
| `Retired` | Continua reconhecido; o que o save já tem é mantido, mas não é concedido de novo. |
| `Unknown` | O save veio de outro conteúdo: é preservado e não é sobrescrito. |

Um nome antigo nunca pode ser reutilizado por outro item. Esta política está implementada e testada no catálogo; ela ainda não é aplicada ao checkpoint, que hoje guarda enums fixos.

## Checkpoint v4 (SAV-02)

Implementado em 2026-11-14; é o formato gravado a partir desta fonte.

- Acrescenta `catalog_version` e `line` (identificador da linha) ao documento.
- v1 a v3 significam sempre o Circuito Aurora e migram com backup exato (`.v1.backup.json`, `.v2.backup.json`, `.v3.backup.json`).
- Um v4 cuja linha esta edição não sabe percorrer, ou com `catalog_version` maior que a compreendida, é recusado e preservado; o salvamento comum não o substitui.
- O executável 0.4.0-rc.2 não lê v4. Para voltar a ele, restaure o backup v3; o progresso posterior ao backup não existe na versão antiga.
- A posição em linha de ponta a ponta não precisa de campo próprio: o sentido de marcha decorre da distância percorrida.

Ainda não aplicado ao checkpoint: a consulta de identificadores renomeados ou desativados, porque estações e vagões seguem gravados como enums fixos.

## Percurso genérico e escala de tempo

`Itinerary` localiza o trem a partir da distância total: estação de origem e destino, trecho, sentido, distância até a próxima parada e número de chegadas. Em circuito as estações se repetem em ordem; em linha de ponta a ponta o trem volta pelos mesmos trechos. Testes comparam o resultado com o motor atual em quatro seeds e centenas de distâncias, e conferem a sequência de paradas da Curitiba–Paranaguá nos dois sentidos.

**Escala confirmada pelo Pedro em 2026-11-14** (trechos de 1 a 7 minutos, sem demorar muito de uma cidade a outra): 480 unidades por quilômetro, isto é, 20 segundos de tempo ativo por quilômetro na velocidade do horário. A Curitiba–Paranaguá levaria cerca de 36 minutos de ponta a ponta, com trechos de 1 a 6 minutos. É decisão de balanceamento, não dado da ferrovia.

Falta para uma linha real rodar no jogo: viagem e serviços usarem o percurso genérico, identidade de estação que não seja o enum de quatro valores, mapa do caderno e arquitetura de estações para mais de quatro paradas, escolha de linha na interface e o crédito da fonte visível.

## Linha Curitiba–Paranaguá

Arquivo: `data/lines/curitiba-paranagua.json`. Decisões do Pedro em 2026-11-14: a ferrovia inteira, com Morretes no meio, na **época atual**. Dados coletados uma única vez nessa data; cópias, método e licenças em `data/sources/curitiba-paranagua/README.md`.

| Km | Altitude (m) | Ponto |
| --- | --- | --- |
| 0,0 | 8 | Paranaguá |
| 16,0 | 10 | Alexandra |
| 23,8 | 7 | Saquarema |
| 40,7 | 10 | Morretes |
| 50,6 | 231 | Porto de Cima |
| 55,9 | 377 | Engenheiro Lange |
| 59,8 | 489 | Marumbi |
| 63,9 | 632 | Cadeado |
| 66,8 | 705 | Véu de Noiva |
| 74,4 | 859 | Banhado |
| 80,5 | 952 | Roça Nova |
| 87,3 | 898 | Piraquara |
| 102,1 | 889 | Pinhais |
| 109,3 | 891 | Curitiba (Rodoferroviária) |

Bitola de 1000 mm em todo o traçado. Entre Porto de Cima e Roça Nova a linha sobe cerca de 720 m em 30 km, com 12 túneis e a maior parte das pontes e viadutos; o litoral e o planalto são quase planos. Os quilômetros foram conferidos contra quatro marcos quilométricos e contra a ponte sobre o Rio São João do acervo do Instituto Moreira Salles.

Origem dos dados: OpenStreetMap (trilhos, pontos, túneis e pontes), Open-Meteo/Copernicus (altitude) e Wikidata (conferência de nomes). A linha declara `ODbL-1.0` e o crédito "© OpenStreetMap contributors", que precisa aparecer no jogo junto com ela.

A linha continua como **rascunho** e não aparece no jogo. Os dados estão completos para a validação; o que a segura são as questões em aberto registradas no arquivo:

- O motor de viagem ainda não usa o percurso genérico: segue limitado ao circuito de quatro estações.
- Falta confirmar, em fonte do operador, quais pontos têm embarque de passageiros hoje.
- Demanda de passageiros e cargas por estação ainda não foram definidas.
- Os marcos visuais de cada trecho ainda não foram desenhados.

## Linha Vitória a Minas (rascunho inicial)

Arquivo: `data/lines/vitoria-minas.json`; fontes e método em `data/sources/vitoria-minas/README.md`. Coletada do OpenStreetMap em 2026-11-14, na época atual.

- 29 paradas do trem de passageiros, de Pedro Nolasco (Grande Vitória) a Belo Horizonte, em bitola de 1000 mm.
- Quilômetro medido para as 21 primeiras, até Ipaba (444,3 km). Entre as principais: Colatina 138,4; Aimorés 208,0; Governador Valadares 353,1.
- Os trilhos mapeados estão interrompidos entre Ipaba e Intendente Câmara e entre Rio Piracicaba e Dois Irmãos. As oito paradas seguintes ficam sem quilômetro; nada foi estimado.
- Faltam: conferência com a tabela do operador, altitude e cenário por trecho.

Está bem menos madura que a Curitiba–Paranaguá e não pode virar itinerário enquanto houver trecho sem medida; um teste garante isso.

## Locomotivas reais

O catálogo já aceita `kind: real` em locomotivas e exige fabricante, ano, arranjo de rodas e ao menos uma fonte. As três locomotivas atuais são fictícias. As reais entram quando houver os livros de referência; a arte é sempre original.
