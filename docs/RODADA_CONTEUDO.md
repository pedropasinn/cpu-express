# CPU Express — rodada de conteúdo (três agentes Codex em fila)

Criado em 2026-11-15 pelo Claude Code, a pedido do Pedro: "colocar outros agentes do Codex para criar vários outros tipos de vagões e também locomotivas; outros cuidando de criar mais elementos de cenário; e alguns para pensar em novas missões, melhorar os mapas e construir outras estações".

Esta cópia de trabalho (`cpu-express-conteudo`, branch `conteudo`) é só dos agentes de conteúdo. O Claude Code segue refazendo os menus em outra cópia (`cpu-express-ui`, branch `ui-redesign`) e compila em paralelo. Os três agentes rodam **um de cada vez**, nesta ordem, porque a máquina tem pouca memória; um script registra um commit ao fim de cada um.

## Regras para os três

1. Leia `AGENTS.md`, `docs/LINHAS.md`, `docs/REVISAO_UI.md` e este arquivo antes de editar. Documente em português; comentários de código em inglês, como no restante da base.
2. **Arte original.** Tudo desenhado em código, no estilo e na paleta existentes. Nada copiado nem extraído de outro jogo (as capturas em `entrada/ui-references` são de outro jogo e servem apenas como linguagem de gênero). Sem logotipos ou marcas de ferrovias reais. Conteúdo fictício declara `kind: fictional`; um item real exige fontes verificáveis, que ainda não temos para material rodante: **não invente fichas históricas**.
3. **Contratos do renderer:** grade interna, filtragem nearest, determinismo, nenhuma alocação por frame, rodas/engates/baseline e sockets preservados, períodos de wrap do parallax. O Circuito Aurora existente não pode mudar de aparência sem querer: há testes de hash; se um hash mudar, explique por quê ou desfaça.
4. **Contratos do domínio:** CPU não rende progresso; sem moeda de compra, loja ou monetização; sem prazo punitivo nem progresso offline. O save é v4: valores já gravados (nomes de vagões, cargas, estações, recompensas) continuam válidos; só acrescente, nunca renomeie nem remova. Se precisar de campo novo no save, não implemente: registre em "Pedidos".
5. **Território.** Não edite `src/app.rs`, `src/app/*.rs` nem `src/ui_game.rs`, `src/ui_services.rs`: os menus estão sendo refeitos em outra branch. Se uma lista da interface precisar conhecer um item novo, prefira que ela derive de uma constante `ALL` do seu módulo; o que não der, registre em "Pedidos". `src/ui_stock.rs` pode ser editado apenas para desenhar miniaturas dos itens novos.
6. **Verificação:** `cargo fmt`, `cargo clippy --locked --all-targets -j 2 -- -D warnings` e `cargo test --locked --all-targets -j 2` passando ao terminar. Use `scripts/dev.ps1` (a toolchain fica em `..\..\.tools`). Nunca mais de uma compilação ao mesmo tempo e no máximo 2 jobs. Testes novos para o que criar. Gere imagens de conferência com os exemplos de galeria existentes (`examples/*gallery*`, `examples/art_review.rs`) e salve em `docs/conteudo/` os PNGs que mostrarem o resultado.
7. Sem commit (o script faz), sem apagar ou mover arquivos existentes, sem pacote ou release, sem enviar nada para fora do computador, sem tocar em `%LOCALAPPDATA%\CPU Express`.
8. Ao terminar, acrescente sua seção no fim deste arquivo: o que entregou, arquivos, o que foi verificado, o que não foi, decisões de balanceamento que tomou (para o Pedro confirmar) e pedidos para a integração.

## Agente 1 — material rodante: vagões e locomotivas

Hoje há cinco vagões (`CoachStyle`: Passenger, Sleeper, Freight, Tank, Observation) e três locomotivas (`EngineStyle`: Steam, Diesel, Electric), todos fictícios.

- Crie **pelo menos seis vagões novos** com silhueta e função distintas. Sugestões: restaurante, correio/bagagem, gôndola de minério, graneleiro (hopper), plataforma com toras ou contêiner, frigorífico, vagão de gado, caboose. Cada um legível na faixa de 60 px de altura.
- Crie **pelo menos três locomotivas novas** fictícias de tipos ferroviários genéricos e reconhecíveis (por exemplo: vapor de carga com mais eixos motores, manobreira a diesel, automotriz). Respeite sockets de chaminé, rodas, bielas e engates; quem não é a vapor não solta fumaça.
- Ligue cada vagão ao domínio (`ServiceWagon`, capacidades, que cargas leva) **acrescentando** variantes e entradas em `data/catalog/core.json`; os testes de paridade de `tests/catalog.rs` precisam continuar valendo e cobrir os itens novos. Se um vagão novo pedir um tipo de carga novo (minério, grãos, madeira), acrescente-o em `CargoKind` e no catálogo com o vagão compatível, mantendo os contratos existentes sorteáveis como hoje (um perfil antigo não pode mudar de comportamento).
- Defina como cada item é obtido reutilizando os mecanismos existentes (chegadas e postais); se achar que falta um mecanismo, proponha em "Pedidos" em vez de criar moeda.
- Miniaturas na oficina (`src/ui_stock.rs`) e galeria de conferência.

## Agente 2 — cenário

O renderer escolhe paisagem por bioma (`meadow`, `forest`, `highland`, `coast`). O catálogo de linhas já descreve cada trecho com etiquetas de cenário (`plain`, `farmland`, `forest`, `mountain`, `river`, `bridge`, `viaduct`, `tunnel`, `urban`, `port`, `coast`, `plateau`) que o renderer ainda ignora. A primeira linha real é a Paranaguá–Curitiba: porto e baixada, rios, Mata Atlântica, subida da Serra do Mar com 12 túneis e muitas pontes e viadutos, planalto com araucárias e chegada urbana.

- Crie elementos novos para essas etiquetas: guindastes e navios no porto, manguezal e baixada, pontes metálicas e viadutos em curva, bocas de túnel, encostas com mata, picos, araucárias e campos do planalto, fazendas, casario e prédios de chegada urbana, postes e sinais ferroviários.
- Dê ao renderer um caminho para receber as etiquetas de um trecho (uma estrutura pura, sem ler o catálogo no renderer) e uma galeria que mostre cada etiqueta em dia, entardecer, noite e chuva. Túnel e ponte precisam de solução que preserve a leitura do trem.
- Não altere a aparência do Circuito Aurora: os elementos novos entram quando as etiquetas pedem.

## Agente 3 — missões, estações e mapas

- **Missões:** proponha e implemente um primeiro conjunto de objetivos além dos três atuais (`src/simulation/objectives.rs`): por serviço (passageiros, cargas por tipo), por descoberta (estações, ramais) e por coleção. Recompensas são itens de coleção, pinturas, vagões ou locomotivas do Agente 1 — nunca moeda. Progresso derivado de contadores duráveis, para que reabrir o jogo não conceda duas vezes. Inclua o desenho de **contratos especiais de passageiros** (CON-02 do plano) e implemente o que couber sem mudar o formato do save; o que exigir campo novo fica em "Pedidos". Documente tudo em `docs/MISSOES.md` com tabela de missões, requisito, recompensa e justificativa de ritmo (a escala das linhas reais é 20 s por km; trechos de 1 a 7 minutos).
- **Estações:** hoje há quatro arquiteturas, uma por estação do Aurora (`src/rendering/stations.rs`). Crie arquiteturas novas escolhidas por um estilo, não por índice: terminal portuário, estação de serra em pedra, parada de madeira, estação de planalto, terminal urbano grande, posto de serviço simples. Proponha em "Pedidos" o campo de catálogo que ligará cada estação da linha real a um estilo (não edite `src/simulation/catalog.rs`), e sugira o estilo de cada uma das 14 paradas de `data/lines/curitiba-paranagua.json`.
- **Mapas:** não edite as telas. Entregue em `docs/MAPAS.md` o desenho do mapa do caderno para linhas de qualquer tamanho (14 e 29 paradas, ida e volta), com perfil de altitude, posição do trem, estações descobertas e pendentes, e como caber em 480 × 480; e entregue um módulo puro e testado (por exemplo `src/simulation/map_layout.rs`) que calcule as posições de estações e do trem a partir de uma linha do catálogo e do `Itinerary`, para a interface apenas desenhar.

## Pedidos para a integração

_(cada agente acrescenta aqui: data, agente, arquivo, o que precisa)_

## Relatos dos agentes

_(cada agente acrescenta sua seção abaixo)_
