# Fontes da linha Curitiba–Paranaguá

Coleta única feita em 2026-11-14, a pedido do Pedro. Estes arquivos são a cópia do que foi consultado e o caminho para refazer `data/lines/curitiba-paranagua.json`. O jogo não acessa nenhum desses serviços: tudo é processado aqui e o resultado vai embutido.

## Licença e crédito

- `osm-railways.json`, `osm-points.json`, `derived-path.json` e o arquivo da linha derivam do **OpenStreetMap**: © OpenStreetMap contributors, Open Database License (ODbL) 1.0, https://www.openstreetmap.org/copyright. Base de dados do OSM em 2026-11-14T13:24:21Z. Esses arquivos são distribuídos sob a mesma licença, separados do código e da arte do jogo.
- `wikidata-stations.json`: Wikidata, CC0.
- `open-meteo-elevation.json`: Open-Meteo Elevation API, com dados do Copernicus DEM GLO-90.

## Arquivos

| Arquivo | Conteúdo |
| --- | --- |
| `overpass-railways.overpassql`, `osm-railways.json` | Consulta e resposta: 543 vias férreas e as estações da região. |
| `overpass-points.overpassql`, `osm-points.json` | Consulta e resposta: pontos operacionais, marcos quilométricos e prédios de estação. |
| `wikidata-stations.sparql`, `wikidata-stations.json` | Estações com coordenadas, para conferir nomes e posições. |
| `open-meteo-elevation.json` | Pedido e resposta de altitude dos catorze pontos. |
| `derive_path.py` | Monta o grafo dos trilhos, acha o traçado principal de Paranaguá a Curitiba e mede cada ponto, túnel e ponte ao longo dele. Gera `derived-path.json`. |
| `derived-path.json` | Traçado (3 614 pontos com latitude, longitude e quilômetro), pontos projetados e obras de arte. Serve de base para o mapa do caderno. |
| `build_line.py` | Escolhe os catorze pontos, aplica a calibração e escreve o arquivo da linha. Refaz o pedido de altitude ao rodar. |

## Método e conferência

- O traçado é o caminho mais curto pelos trilhos mapeados, com penalidade para pátios, desvios e ramais industriais. Resultado: 109,2 km, com toda a extensão em bitola de 1000 mm.
- **Calibração:** os quatro marcos quilométricos do OSM (68, 73, 77 e 81) ficaram em 67,88, 72,89, 76,91 e 80,88 no traçado calculado. Somou-se a média da diferença, 0,11 km, para que os quilômetros da linha coincidam com a quilometragem da ferrovia.
- **Conferência externa:** o viaduto de 125 m aparece no km 62,3; o acervo do Instituto Moreira Salles registra a ponte sobre o Rio São João no km 62,21. A extensão de 109,3 km até a Rodoferroviária é compatível com os 110 km das fontes históricas, cuja estação original ficava adiante. O ponto mais alto, Roça Nova, deu 952 m.
- **Diferenças para as fontes de texto:** o OSM tem 12 túneis (1 940 m somados) e 39 pontes ou viadutos no traçado; a Agência Estadual de Notícias do Paraná fala em 13 túneis e 30 pontes e viadutos. A contagem por trecho na linha usa o que está mapeado.

## Limites

- A altitude vem de um modelo de 90 m de resolução, na coordenada do ponto; em encosta pode divergir alguns metros da cota oficial.
- Em Paranaguá os trilhos mapeados terminam cerca de 0,8 km antes do prédio da estação; o km 0 da linha é o fim dos trilhos, ajustado pela calibração.
- Pontos só operacionais (Dom Pedro II, Quilômetro 5, Quilômetro 70, Vila Oficinas, Iguaçu) e o ramal de Antonina ficaram fora da linha do jogo, mas estão nos extratos.
- Biomas e etiquetas de cenário são leitura editorial desses dados, não um dado da fonte.
