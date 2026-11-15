# Mapas para linhas de qualquer tamanho

Agente 3, 15/11/2026. Desenho para integração no caderno; nenhuma tela foi editada. `simulation::map_layout::MapLayout` recebe uma `LineDef` e o `Itinerary` efetivo, valida identidade/topologia/ordem e calcula geometria sem egui, relógio, disco ou progresso próprio. Rascunhos podem ser conferidos por ferramentas, sem torná-los jogáveis.

## Duas escalas, duas funções

O **esquema de paradas** usa espaçamento regular para que estações próximas possam ser selecionadas. Em linhas `shuttle`, sete colunas em serpentina: primeira fila esquerda→direita, segunda direita→esquerda e assim por diante. Quatorze paradas ocupam duas filas; 29, cinco. O trem retorna pelos mesmos pontos, sem duplicar estações ou inverter o desenho quando muda de sentido. No circuito, um anel liga a última parada à primeira. Nenhuma dessas posições representa latitude/longitude nem a curva real da ferrovia.

O **perfil de altitude** mantém distância proporcional no eixo x e elevação no y. Estações próximas podem se sobrepor visualmente no perfil; seleção principal continua no esquema/lista, sem espaçar artificialmente os quilômetros. `elevation_range()` fornece os extremos em metros. Linha horizontal central quando todas as alturas são iguais; ausência de altura produz `None` e uma lacuna, nunca altitude zero inventada. O fechamento de um circuito termina em x=1 com a altitude da origem; na chegada, o marcador volta a x=0.

## Orçamento do caderno em 480 × 480

Medidas propostas em pontos lógicos, a conferir com a moldura atual; não equivalem a pixels físicos em todo DPI. Reservar 40 para faixa, 36 para subabas, 24 para status, 48 para doca e 24 para margens: restam aproximadamente **308 pontos de conteúdo**.

| Área | Altura | Conteúdo |
| --- | --- | --- |
| Cabeçalho da rota | 28 | Nome curto, ida/volta e pausa; nome inteiro acessível |
| Esquema | 152 | Até cinco filas: 128 entre centros + margens, sete colunas |
| Perfil | 52 | Altitudes mínima/máxima, distância e posição; rótulos extremos |
| Estação selecionada | 52 | Nome completo, km, altitude conhecida, descoberta e próxima parada |
| Legenda/crédito | 24 | Descoberta/pendente + acesso à fonte da linha |

Não tentar escrever 29 nomes longos no esquema. Cada ponto exibe ordinal de 1 a N; foco/seleção abre nome e dados na ficha. Em largura útil aproximada de 424, centros das sete colunas ficam a 70,7 pontos. Área interativa mínima sugerida de 28 × 28, independente do sprite de 6–8 pixels. Não comunicar pendência só por cor: círculo cheio + marca para descoberto; círculo vazio + ordinal para pendente; seleção com aro; foco com contorno separado. Trem tem silhueta e seta de sentido. A lista acessível conserva nome, ordem, estado e ações de teclado de todas as paradas.

Para mais filas que couberem, usar **rolagem vertical apenas no esquema/lista**, mantendo a ficha/controles alcançáveis; `schematic_height()` recomenda ao menos 32 pontos entre filas. Não encolher 60 paradas para caber na mesma área. O perfil continua mostrando a linha inteira e destaca o intervalo visível; ação “Ver trem” traz sua fila à vista sem deslocar foco automaticamente a cada frame. Para circuitos muito grandes, a interface pode ampliar o anel e permitir rolagem, sem sobrepor alvos. Setas percorrem ordem ferroviária; Enter inspeciona, sem mover o trem.

## Contrato do módulo

```rust
let mapa = MapLayout::new(linha, &itinerario)?; // reconstruir na troca de linha/seed
let trem = mapa.train(distancia_total);       // consulta sem alocação
let conhecida = mapa.discovered(ordinal, distancia_total);
```

`MapStation.ordinal` é índice na lista **ativa** do itinerário. `catalog_index` aponta para o registro original de `LineDef.stations`, inclusive quando há aposentadas intercaladas. Os pontos são normalizados em 0..1: a UI aplica margem e escala; não deve reordenar as estações nem usar `catalog_index` como índice de segmento. `point` é esquema e `profile` é perfil. Distância do `Itinerary` pode ser diferente da base do catálogo: isso é necessário para a seed de Aurora.

`train()` retorna posição nos dois desenhos, origem/destino, sentido e estação atual. Interpola pela fração real do segmento, inclusive na volta; não cria um relógio de animação separado. Ao estar exatamente na plataforma, usa o ponto da estação sem erro de interpolação. Durante serviço/pausa, manter a distância do domínio: o trem permanece parado. Entrada negativa/NaN/infinita segue a política de `Itinerary::locate` e mostra a origem.

`discovered()` deriva visita distinta da distância acumulada desde a origem desta linha: origem conhecida no início, cada nova parada descoberta na primeira passagem, retorno nunca apaga descoberta. Essa inferência é válida para percurso contínuo completo, sem teleporte. Não representa histórico de outra linha, nem ramais, nem um futuro modo que comece no meio do trajeto. Se a seleção de linhas permitir reiniciar posição e conservar descobertas, o integrador deverá fornecer um conjunto persistido por ID; não reinterpretar distância de Aurora. Durante excursão, apresentar o ramal separado e recolher o circuito como “aguardando retorno”, sem um segundo trem viajando.

## Conferência e integração

Testes cobrem 14 paradas reais nos dois sentidos e extremos, 29 paradas sintéticas, estações aposentadas, altitude faltante/plana, circuito com seed, fechamento do perfil, entradas inválidas e recusa de linha/ordem incompatível. O exemplo `map_gallery` produz imagem local com pontos numerados e perfil; é uma conferência da geometria, **não uma tela egui nem uma ferrovia real de 29 paradas**. A fixture de 29 é fictícia (`kind: fictional`).

O adaptador do caderno ainda deve fornecer seleção, teclado, nomes, semântica acessível, escala e crédito `© OpenStreetMap contributors` na linha real; nada disso é provado pelo módulo matemático. Dados e metragens vêm do catálogo local existente. Paranaguá–Curitiba continua `draft`; a mudança não integra viagem, serviço genérico ou escolha de linha. Validar a interface em 480 × 480 e DPI 100/125/150/200% no Windows antes de aceitar o desenho como produto.
