# CPU Express — catálogo de conteúdo e linhas

Atualizado em 2026-11-14. Responsável: Claude Code. Código em `src/simulation/catalog.rs`, dados em `data/`, testes em `tests/catalog.rs`.

## O que é

O catálogo reúne as definições de conteúdo, separadas do estado da partida:

- `data/catalog/core.json`: vagões, cargas, pinturas, recompensas de serviço e locomotivas.
- `data/lines/*.json`: uma linha por arquivo, com estações, trechos, fontes e questões em aberto.

Os documentos são embutidos no executável na compilação; o jogo não lê arquivos nem usa rede para carregá-los. Saves e referências usam somente `id`. O nome exibido é conteúdo e pode mudar sem afetar um save.

**Estado nesta data:** o catálogo é validado e espelha as regras atuais, com testes de paridade contra o código (capacidades, cargas por vagão, pinturas, metas dos postais, locomotivas e o Circuito Aurora em várias seeds). O motor de viagem **ainda não lê a linha do catálogo**: continua usando o circuito fixo de quatro estações. Essa troca é a próxima etapa e exige o checkpoint v4 descrito abaixo.

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

## Definição proposta para o checkpoint v4 (SAV-02)

Ainda não implementada. O v3 continua sendo o formato gravado.

- Acrescentar `catalog_version` e `line` (identificador da linha) ao documento.
- v1 a v3 migram com `line = "aurora-v1"` e backup exato, como nas migrações anteriores.
- Na leitura, cada identificador passa pela consulta acima. `Unknown` ou `catalog_version` maior que a compreendida recusam o carregamento e preservam o arquivo.
- A posição em linhas `shuttle` precisa registrar o sentido de marcha; em `circuit` permanece como hoje.

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

- O motor de viagem ainda não percorre linhas de ponta a ponta nem com mais de quatro estações.
- Falta confirmar, em fonte do operador, quais pontos têm embarque de passageiros hoje.
- Demanda de passageiros e cargas por estação ainda não foram definidas.
- Os marcos visuais de cada trecho ainda não foram desenhados.

## Locomotivas reais

O catálogo já aceita `kind: real` em locomotivas e exige fabricante, ano, arranjo de rodas e ao menos uma fonte. As três locomotivas atuais são fictícias. As reais entram quando houver os livros de referência; a arte é sempre original.
