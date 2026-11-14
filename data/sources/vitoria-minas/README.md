# Fontes da linha Vitória a Minas

Coleta única feita em 2026-11-14, a pedido do Pedro. O jogo não acessa esses serviços; os arquivos são a cópia do que foi consultado e o caminho para refazer `data/lines/vitoria-minas.json`.

## Licença e crédito

Todos os arquivos derivam do **OpenStreetMap**: © OpenStreetMap contributors, Open Database License (ODbL) 1.0, https://www.openstreetmap.org/copyright. São distribuídos sob a mesma licença, separados do código e da arte do jogo.

## Arquivos

| Arquivo | Conteúdo |
| --- | --- |
| `overpass-*.overpassql` | As quatro consultas feitas: a relação do trem de passageiros (3331630), as relações de volta e da infraestrutura (7831016, 7754552), os trilhos do corredor e todos os trilhos nas três áreas com falha. |
| `osm-extracts.zip` | As quatro respostas, sem alteração. |
| `derive_path.py` | Monta o grafo dos trilhos e mede a distância entre paradas consecutivas. Espera os quatro extratos descompactados ao lado. |
| `derived-path.json` | Paradas com quilômetro e distância do trecho, obras de arte e o traçado medido. |
| `build_line.py` | Escreve o arquivo da linha a partir do resultado. |

## O que se sabe e o que não se sabe

- A relação do trem de passageiros lista 28 paradas em ordem, de Pedro Nolasco a Dois Irmãos; a estação de Belo Horizonte foi acrescentada a partir dos nós do corredor. A própria relação traz um aviso de que está incompleta.
- Foram medidos 647 km pelos trilhos mapeados, quase todos em bitola de 1000 mm, com 14 túneis e 90 pontes no caminho medido.
- **Os trilhos mapeados estão interrompidos em dois pontos:** entre Ipaba e Intendente Câmara, e entre Rio Piracicaba e Dois Irmãos. Por isso o quilômetro acumulado só é confiável até Ipaba (444,3 km); depois dela a linha guarda apenas a distância dos trechos que puderam ser medidos, e nenhum valor foi estimado.
- Altitude, bioma e cenário por trecho não foram levantados; os valores do arquivo da linha são provisórios e iguais para todos os trechos.
- Nada aqui foi conferido com a tabela do operador.
