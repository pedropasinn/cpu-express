# Plano de expansão — economia, máquina, estações, cenário e menus

Pedido do Pedro em 2026-11-16, com autorização para executar durante a noite na branch `expansao` (parte de `integracao`, commit `3fb65a7` + junções). Coordenação: Claude Code. Execução: agentes Codex em fila, um por vez nesta pasta (o computador tem 16 GB; duas compilações ao mesmo tempo derrubam a fila). Revisão das telas: um agente separado, no fim.

Este documento é o briefing. Cada agente lê **este arquivo inteiro**, `AGENTS.md`, e os documentos citados na sua seção.

## O que o Pedro quer ver de manhã

1. Bem mais estações.
2. Menus melhores, em janela de tamanho travado, com as coisas cabendo nas páginas.
3. Economia funcionando: moedas, recompensas melhores, vários tipos de caixa (gacha).
4. Monitoramento melhor: o que acontece no computador ajuda o trem de formas diferentes.
5. Missões mais claras.
6. Cenário mais rico: cidades, prédios, casas e outros elementos.
7. O conteúdo que os sete agentes criaram realmente aparecendo no jogo.

## Regras para todos

1. **Arte original, desenhada em código.** Nada copiado de outro jogo; sem logotipos; sem fichas históricas inventadas (conteúdo fictício é marcado `kind: fictional`).
2. **Sem dinheiro real**, anúncios, energia, sequência diária, oferta que expira ou perda por ausência. Probabilidades sempre visíveis.
3. **Nunca tocar no perfil real** em `%LOCALAPPDATA%\CPU Express`. Testes usam `CPU_EXPRESS_CONFIG` apontando para pasta temporária.
4. **Saves antigos continuam abrindo.** Toda mudança de formato tem migração, backup exato do arquivo anterior e fixture de teste.
5. **Privacidade do monitoramento:** só números agregados da máquina. Nunca ler teclas, títulos de janela, nomes de processo ou arquivos. Nada sai do computador.
6. Território: nesta rodada **pode** editar `src/app.rs`, `src/app/*.rs` e `src/ui_*.rs` quando a seção pedir. Não desfazer o trabalho de agentes anteriores; ler os relatos em `docs/RODADA_CONTEUDO.md` e a seção "Relatos" deste arquivo.
7. Compilação: no máximo 2 jobs (`-j 2`), uma compilação por vez. Toolchain em `..\..\.tools` (ver `scripts/dev.ps1`).
8. Antes de terminar: `cargo fmt --all`, `cargo clippy --locked --all-targets -j 2 -- -D warnings`, `cargo test --locked --all-targets -j 2` passando. Imagens de conferência em `docs/expansao/`. Relato no fim deste arquivo: o que foi feito, o que foi verificado, o que **não** foi verificado, decisões tomadas que o Pedro pode querer rever.
9. Sem commit (a fila registra), sem apagar ou mover arquivos existentes, sem enviar nada para fora do computador, sem baixar nada.
10. Se uma parte da seção não couber, entregar a parte menor **funcionando e testada** e dizer o que ficou de fora. Árvore que não compila não é entrega.
11. Textos do jogo em português, curtos. Comentários de código em inglês, como o resto do projeto.

## Decisões de desenho (tomadas pelo coordenador para destravar a noite; o Pedro revê depois)

### Economia

Base: `docs/ECONOMIA_PROPOSTA.md` (Agente 3) e as notas do livro de Oscar Clark em `docs/economia/`. Mudanças em relação à proposta:

- **Uma moeda:** "moedas" (nome de exibição: **Réis de Ferro**; símbolo: moeda com roda de trem). Os "pontos de serviço" antigos continuam sendo registro histórico e não viram saldo.
- **Ritmo mais rápido no começo.** A proposta dava a primeira caixa perto de uma hora; isso é lento para quem está conhecendo o jogo. Fica assim:
  - Perfil novo ou migrado recebe **uma caixa Pátio geral grátis** ("remessa de boas-vindas") e 30 moedas.
  - Cada **chegada a uma estação** paga 2 moedas (fonte por tempo ativo; no circuito Aurora isso dá cerca de 40–60/h).
  - Cada **passageiro entregue** paga 1 moeda a cada 4; cada **unidade de carga entregue** paga 1 moeda a cada 2; contrato concluído paga o prêmio do contrato dividido por 10, mínimo 3.
  - **Bônus da máquina** (ver abaixo) multiplica só a renda de chegadas e entregas, entre ×1,00 e ×1,30.
  - Alvo: **90–130 moedas por hora ativa** para quem mantém serviços andando; primeira caixa comprada em 20–30 min, além da grátis.
  - Missões únicas pagam os valores de `docs/MISSOES.md` (os 11 marcos existentes + os 7 por tipo de carga, agora possíveis porque o save passa a contar carga por espécie). Perfis antigos recebem os marcos já cumpridos **uma vez**, na migração, com aviso.
- **Missões recorrentes ("Ordens do dia"):** três ordens visíveis ao mesmo tempo, sorteadas de um conjunto fixo (ex.: "entregue 30 passageiros", "conclua 2 encomendas", "visite 6 estações", "entregue 12 de minério"). Sem prazo, sem expirar; ao concluir, o jogador recolhe o prêmio (10–25 moedas) e outra ordem entra no lugar. Nunca pedem vagão que o jogador não possui.
- **Posse:** o save passa a ter inventário por ID de catálogo. Tudo o que o perfil antigo podia equipar vira possuído. Perfil novo começa com Vapor Aurora, um vagão de passageiros, um de carga e a pintura inicial. Os desbloqueios antigos por marco continuam valendo como **outra forma de ganhar** a peça (direito adquirido).
- **Caixas (remessas), cinco tipos:**

  | Remessa | Preço | Conteúdo | Comum / Incomum / Raro / Épico | Garantia de raro+ |
  | --- | --- | --- | --- | --- |
  | Pátio geral | 60 | todos os vagões e locomotivas | 70 / 22 / 7 / 1 | na 10ª |
  | Cargas e oficina | 90 | vagões de carga e locomotivas de carga | 60 / 29 / 10 / 1 | na 8ª |
  | Plataforma e conforto | 90 | vagões de passageiros e de serviço | 60 / 29 / 10 / 1 | na 8ª |
  | Garagem de locomotivas | 180 | locomotivas | 55 / 30 / 13 / 2 | na 6ª |
  | Ateliê de pinturas | 75 | pinturas | 65 / 25 / 9 / 1 | na 8ª |

  Regras: uma peça por caixa; sorteia a raridade e depois, dentro dela, uma peça **ainda não possuída** se houver; repetida devolve 20% do preço; contador de garantia por tipo de caixa, gravado no save; nunca vender caixa cujo conjunto tenha faixa vazia (validar no catálogo). **O resultado é sorteado e gravado no save junto com o débito, antes de qualquer animação.** O gerador é determinístico a partir de uma semente gravada no save.
- **Compra direta:** comum 120, incomum 240, raro 480, épico 900. Caminho sem sorte para qualquer peça.
- **Depois da coleção completa:** as moedas compram "selos do museu" numerados (120 cada), puramente de coleção. Nenhuma cobrança de manutenção.
- Raridade é tema e estética; **não** multiplica renda. Capacidades e papéis continuam os definidos pelos agentes 1 e 5.

### A máquina ajuda o trem ("Sala de máquinas")

Princípio: cada sinal do computador alimenta **um sistema do trem**, com efeito pequeno, visível e **que satura cedo**. Usar o computador normalmente já enche os medidores; forçar a máquina não rende nada a mais. Nenhum sinal penaliza: o mínimo é o jogo base.

| Sinal (agregado) | Sistema do trem | Efeito no jogo | Satura em |
| --- | --- | --- | --- |
| Processador (%) | Pressão da caldeira | ritmo visual do trem e da fumaça (já existe) + até +10% de moedas | 35% de uso |
| Atividade de teclado e mouse (segundos com atividade por minuto; **não** quais teclas) | Foguista | "fôlego": até +10% de moedas; sem atividade por 5 min o trem entra em marcha tranquila (só visual) | 20 s ativos por minuto |
| Memória (%) | Lotação da plataforma | passageiros esperando embarcam um pouco mais depressa (visual + até +1 passageiro por parada) | 60% |
| Disco (bytes lidos+escritos/s) | Guindaste do pátio | carga e descarga mais rápidas na parada (reduz espera de carga em até 20%) | 5 MB/s |
| Rede (bytes/s) | Telégrafo | chance de **telegrama**: uma ordem do dia extra, com prêmio pequeno | 200 kB/s |
| Bateria / tomada | Carvão e água | na tomada: tênder cheio (ícone); na bateria abaixo de 20%: modo econômico automático (menos animação) | — |
| Tempo ligado | Hodômetro da máquina | só registro e missões ("viaje 3 h ativas") | — |

- Soma dos bônus de moedas limitada a **+30%**. Cada medidor é uma média móvel de ~60 s, para não piscar.
- A atividade de entrada vem de `GetLastInputInfo` consultado uma vez por segundo: informa só "houve entrada neste segundo". Sem gancho de teclado, sem contagem por tecla. Desligável em Ajustes ("Usar atividade do teclado e mouse"), ligado por padrão, com explicação em uma linha.
- Disco, rede e bateria vêm do `sysinfo` (já é dependência) ou de chamada simples do Windows; se um sinal não existir na máquina, o sistema fica "sem leitura" e o bônus é zero, sem erro.
- Modo demonstração simula os sinais.

### Estações e linhas

- O jogo passa a ter **seletor de linha**: Circuito Aurora (4 estações, fictício) e **Curitiba–Paranaguá** (14 paradas reais, dados em `data/lines/curitiba-paranagua.json`, vai-e-volta). A viagem, os serviços, o mapa e as estações passam a usar o `Itinerary` genérico (`src/simulation/itinerary.rs`), não mais as quatro estações fixas.
- Trocar de linha é possível só parado em estação terminal, sem carga a bordo; o progresso de cada linha fica guardado.
- Crédito "© OpenStreetMap contributors" visível na tela do mapa e na Ajuda.
- Arquitetura de estação por parada (`StationDef.architecture`, opcional) usando as seis silhuetas do Agente 3 e as sugestões de `docs/conteudo/ESTACOES.md`.
- Vitória–Minas continua rascunho, fora do seletor.

### Menus

- Janela da central com **tamanho travado em 704 × 676** pontos lógicos (o tamanho que o Pedro mostrou), sem redimensionar; continua arrastável pela faixa do título. Em tela pequena, reduz para caber na área útil.
- Toda página cabe sem rolagem vertical nesse tamanho, ou usa páginas internas (setas) em vez de rolagem.
- Doca passa a ter: Trem · Viagem · **Loja** · Coleção · Ajustes · Fechar (Ajuda vira botão "?" na faixa do título).
- Saldo de moedas sempre visível no canto da faixa do título.
- Estilo: o que já existe em `src/ui_game.rs` (madeira, faixa, fonte pixelada, sprites 16×16 de `assets/icons/` gerados por `assets/icons/build_sprites.py`). Menos texto, mais objeto e número.

## Ajustes após a avaliação do Codex (valem sobre o texto acima quando houver conflito)

Avaliação completa em `docs/expansao/AVALIACAO_PLANO_CODEX.md`. O que mudou:

**Fatos corrigidos.** Hoje existem 19 vagões, 15 cargas, 10 locomotivas e 10 pinturas; as miniaturas já existem (`src/ui_stock.rs`, `StockGallery`) e a oficina já as usa. `ServiceState::arrive` embarca e desembarca na hora e a parada geral dura 8 s: **não existe espera de carga para encurtar**. `SysinfoProvider` só coleta CPU e RAM e o `Cargo.toml` só habilita `system` no sysinfo. `SegmentScenery(u16)` já usa 12 bits e `Port` já existe. `app.rs::step_frame` ainda não chama `set_scenery_context`, `set_station_styles` nem as opções de movimento. `StationId` tem quatro variantes, `ServiceState::arrive` usa `% 4`, e `progress.rs` recusa outras linhas.

**Ordem nova da fila** (o que o jogador vê vem antes do que é estrutural e arriscado):
`E1a → E1b → E5 → E6 → E2 → E4 → E3 → revisão de telas → E7`.

**Persistência.** O E1a é o dono do formato v5 e já reserva nele: progresso **por linha** (mapa `id da linha → progresso`, hoje só Aurora), inventário, carteira, transações com ID crescente, compra pendente, contadores por espécie de carga, garantia por caixa, semente do sorteio, missões pagas. O E3 usa esses campos; só cria v6 se for impossível evitar, com migração. Compra, recolhimento de prêmio e troca de linha **gravam na hora** (não esperam os 30 s de `persist_progress`); se a gravação falhar, o estado em memória volta ao anterior e o jogador é avisado.

**Posse e progressão** (resolve "os marcos esvaziam a loja"):
- Posse é do **modelo**: quem possui pode equipar em quantas vagas quiser.
- **Perfil antigo:** tudo o que podia equipar na migração vira possuído (direito adquirido).
- **Perfil novo:** começa com Vapor Aurora, Passageiros, Carga, Tanque e a pintura inicial; composição inicial Passageiros / Passageiros / Carga. Os marcos antigos de chegadas e postais passam a pagar **moedas e postais**, e concedem só: Dormitório (primeiro circuito completo), Diesel do Porto (postal de carga), e as quatro pinturas originais pelos marcos atuais. Todo o resto vem de caixa ou compra direta.

**Renda** (corrige a sobreposição e a conta errada):
- Chegada: 1 moeda (≈ 70/h no Aurora já seria demais com 2).
- Passageiros: 1 moeda a cada 4 entregues, **guardando o resto** para a próxima.
- Carga: só o contrato concluído paga, `prêmio ÷ 20` com resto guardado, mínimo 2. Nada por unidade.
- Ordens do dia: no máximo 3 ativas; uma nova só entra **em uma chegada**, e no máximo uma por 10 min ativos; cada ordem guarda o valor dos contadores no momento em que entrou; pode ser trocada sem punição uma vez a cada 10 min; só pede o que a composição **equipada ou possuída e equipável agora** consegue fazer.
- Alvo conferido pelo simulador: 90–130 moedas por hora ativa **já com** ordens e bônus; primeira caixa comprada entre 15 e 30 min, além da grátis (a grátis não gera reembolso de repetida).
- O simulador tem estratégia fixa e declarada: compra a caixa mais barata que ainda pode dar peça nova; quando nenhuma pode, compra direta da peça mais barata que falta; depois selos. Informa quantas execuções não completam a coleção em 30 dias.
- Conjuntos de cada caixa são listados por ID no código e validados em teste; se "Plataforma e conforto" ou "Ateliê" não tiverem peça rara ou épica suficiente, o agente ajusta as faixas daquela caixa (ex.: sem épico, a chance vai para raro) e registra no relato — não inventa raridade só para preencher tabela.

**Máquina** (mantém o pedido do Pedro de a máquina ajudar o trem, sem premiar desperdício):
- **Todo** efeito que mexe em moedas fica dentro do teto único de +30%. Não há passageiro extra, parada mais curta nem carga mais rápida: Memória, Disco e Rede têm efeito **só visual** (movimento na plataforma, guindaste do pátio trabalhando, fios do telégrafo piscando) e contam para os medidores da Sala de máquinas.
- Moedas: Foguista (atividade de teclado e mouse) até +15%; Caldeira (processador) até +10%, saturando em **15%** de uso; "Linha viva" (rede com qualquer tráfego) +5%. Usar o computador normalmente já dá o teto; nada cresce além disso.
- Telegrama = uma quarta ordem do dia ocasional, respeitando o limite de uma ordem nova a cada 10 min.
- Atividade de entrada: ligada por padrão, **avisada na tela de acolhida** com o botão para desligar ali mesmo; nenhum histórico é gravado, só a média do último minuto em memória.
- Tempo ativo continua vindo do `SimulationClock`. Amostras parciais e vencidas seguem as regras de `metrics/mod.rs`.
- Redação de privacidade: "as leituras da máquina não saem do computador". O clima local, quando ligado pelo jogador, continua consultando o serviço de meteorologia como hoje.

**E3 — linha real.** Jogável significa: a viagem percorre as 14 paradas reais, com serviços **do jogo** (passageiros e encomendas fictícios, marcados `kind: fictional`). As `open_questions` da linha não são apagadas: passam para um campo de notas editoriais que não bloqueia `playable`, com a frase "os serviços mostrados são do jogo e não representam a operação real". Aurora é circuito: a troca de linha acontece parado em qualquer estação, sem passageiros nem carga a bordo e sem contrato aceito, fora de excursão. Se a generalização não fechar, entregar o que estiver verde (domínio genérico testado, mapa da linha consultável) e manter a linha fora do seletor.

**E4.** Primeiro confere que o E5 ligou `set_scenery_context`/`set_station_styles`/movimento (se não ligou, liga). Para etiquetas novas, alargar `SegmentScenery` para `u32`.

**E5.** As miniaturas já existem: o trabalho é paginar e mostrar raridade/posse. Em tela menor que 704 × 676 a janela encolhe proporcionalmente por escala (`pixels_per_point`), sem layout alternativo. Também liga ao app o conteúdo pendente: contexto de cenário por trecho, estilos de estação e opções de movimento de passageiros e paisagem.

**E7 / verificação final.** Além das correções das telas: um teste de ponta a ponta "ganhar moedas → comprar caixa → equipar a peça → fechar e reabrir → tudo igual", e um relato do que só pode ser conferido na tela (foco, teclado, DPI, compositor).

## Fila de agentes

### E1 — Economia no domínio e save v5

Ler: `docs/ECONOMIA_PROPOSTA.md`, `docs/MISSOES.md`, `src/progress.rs`, `src/simulation/{objectives,service,world,catalog}.rs`, `docs/conteudo/{VAGOES_SEGUNDA_LEVA,LOCOMOTIVAS_PINTURAS}.md`.

Entregar, **sem interface**:
- `src/simulation/economy.rs` (e módulos vizinhos se precisar): carteira, registro de transações limitado, inventário por ID, renda por chegada/entrega/contrato, missões únicas pagas uma vez, ordens do dia, cinco tipos de caixa com tabela validada contra o catálogo, garantia, repetidas, compra direta, selos, gerador determinístico com semente no save, compra em duas fases (débito + resultado gravados; revelação depois).
- Raridade de cada vagão, locomotiva e pintura no catálogo (usar as propostas dos agentes; completar as que faltam com bom senso e listar no relato).
- Contadores de carga entregue por espécie.
- Checkpoint **v5** com migração v1–v4 → v5, backup exato, fixtures novas em `tests/fixtures/`, e os testes de recuperação existentes continuando a passar.
- Um ponto de entrada para o multiplicador da máquina (`set_machine_bonus(f32)` limitado a 1,0–1,3), que o E2 vai alimentar.
- Equipar exige possuir; a regra antiga de desbloqueio por marco passa a **conceder** a peça.
- `examples/economy_sim.rs`: simula 1 h, 3 h e 8 h por dia durante 30 dias, com e sem serviços, e imprime saldo, caixas abertas, tempo até a primeira peça rara e até a coleção completa (mediana e percentis de 1000 execuções). Colar a saída em `docs/expansao/economia-simulacao.md` e **ajustar preços ou renda** se o alvo de 90–130 moedas/h ou "primeira caixa comprada em 20–30 min" não for atingido; registrar o ajuste.
- Testes: soma das probabilidades, garantia, nenhuma duplicação de moeda por falha de gravação ou reabertura no meio da revelação, migração, missão paga uma vez, propriedade preservada.

### E2 — Sala de máquinas: monitoramento ligado ao trem

Ler: `src/metrics/`, `src/app.rs` (uso de métricas), `docs/performance_metrics.md`, a tabela "A máquina ajuda o trem" acima.

Entregar:
- Coleta de disco, rede, bateria/tomada, atividade de entrada (`GetLastInputInfo`) e tempo ativo no worker de métricas existente, com custo desprezível (uma amostra por segundo, sem alocação por amostra) e degradação limpa quando o sinal não existe.
- `src/simulation/machine.rs`: os sistemas da tabela, médias móveis, saturação, bônus total limitado a +30%, efeitos de carga/embarque/telegrama; tudo testável com provedor manual.
- Ligação com a economia do E1 (`set_machine_bonus`) e com os serviços (espera de carga, passageiro extra).
- Preferência "Usar atividade do teclado e mouse" em `Config`, ligada por padrão.
- Simulação dos sinais no modo demonstração.
- Tela mínima "Máquina" (pode ser simples; o E5 refina): seis medidores com ícone, nome do sistema do trem e efeito atual em uma linha.
- Testes de saturação, limites, ausência de sinal e de que nenhum efeito reduz o jogo abaixo do base.

### E3 — Estações: linha real jogável e seletor de linha

Ler: `docs/LINHAS.md`, `src/simulation/{itinerary,route,journey,world,service,catalog,map_layout}.rs`, `data/lines/`, `docs/MAPAS.md`, `docs/conteudo/ESTACOES.md`, pedidos dos agentes 2 e 3 em `docs/RODADA_CONTEUDO.md`.

Entregar:
- Viagem, serviços (passageiros e encomendas com origem/destino), descobertas e mapa funcionando sobre `Itinerary` para qualquer linha jogável; Aurora continua idêntica para saves existentes.
- Curitiba–Paranaguá jogável (status passa de `draft` a `playable` no catálogo se as validações passarem): 14 paradas, vai-e-volta, tempos pela escala de 480 unidades/km já decidida, paradas pequenas com espera curta.
- Seletor de linha (domínio + tela simples na aba Viagem), progresso por linha no save (junto do v5 do E1; se precisar, v6 com migração).
- `StationDef.architecture` opcional e arquitetura por parada; contexto de cenário por trecho (`set_scenery_context`) e opções de passageiros/vida ligadas à preferência de reduzir movimento, como pedido pelos agentes 2, 4 e 6.
- Mapa da linha com as paradas, posição do trem, altitude quando houver, e crédito do OSM.
- Missões de descoberta para a linha nova ("conheça as 14 paradas", "suba a serra") com moedas pela economia do E1.
- Testes de itinerário, de serviços com 14 paradas, de troca de linha e de migração.

### E4 — Cenário: cidades, prédios, casas e mais

Ler: `docs/conteudo/{CENARIO,VIDA_PAISAGEM}.md`, `src/rendering/{scenery,segment_scenery,landmarks,landscape_life,scene}.rs`, `examples/scenery_gallery.rs`.

Entregar, dentro do contrato do renderer (sem alocação por quadro, baseline e sockets intactos, legível na faixa de 60 px):
- **Cidades e vilas** em camadas de paralaxe: casario baixo (casas de madeira e alvenaria, sobrados, telhados variados, chaminés, varais, quintais), centro (prédios de 3–8 andares, igreja com torre, mercado, caixa-d'água, praça com coreto, fábrica com chaminé, armazéns, silos), porto (guindastes, galpões, contêineres genéricos sem marca, navio ao fundo, farol), subúrbio (postes, ruas, muros, campinho).
- Pelo menos 30 elementos novos, com variação por semente, janelas que acendem à noite, fumaça de chaminé e reação ao clima já existente.
- Etiquetas de cenário novas (`town`, `city`, `port`, `industrial`, `suburb`, `village`) aceitas pelo catálogo, e aplicadas aos trechos de Curitiba–Paranaguá (Curitiba e Pinhais urbanos; Paranaguá porto; Morretes vila histórica; serra continua floresta) e a pelo menos uma estação Aurora.
- Aproximação de estação: a densidade de construções cresce antes de uma parada urbana e some depois.
- Galeria (`examples/scenery_gallery.rs`) e imagens em `docs/expansao/cenario-*.png`, incluindo a faixa em 60 px.
- Testes de alocação e de determinismo por semente.

### E5 — Menus I: janela travada, páginas que cabem, missões claras, conteúdo aparecendo

Ler: `docs/REVISAO_UI.md`, `src/ui_game.rs`, `src/app.rs`, `src/app/*.rs`, `tests/ux_visual.rs`, `docs/ux_rasterizar.py`.

Entregar:
- Janela travada em 704 × 676, nova doca, saldo na faixa do título, botão "?".
- Todas as páginas existentes revistas para caber sem rolagem (Trem, Viagem/mapa, Serviços, Diário, Vagões, Locomotivas, Pinturas, Clima, Ajustes, Ajuda), com paginação por setas quando necessário.
- **Missões claras:** uma página "Missões" com três blocos — Ordens do dia (3 cartões: ícone, objetivo em uma linha, barra de progresso, prêmio em moedas, botão Recolher), Marcos (lista paginada com progresso e prêmio), e Próxima recompensa em destaque. Aviso discreto na doca quando há prêmio a recolher.
- Seletor de vagões e locomotivas com **miniaturas desenhadas pelo renderer** (pendência da revisão), mostrando raridade por cor de moldura, possuído/não possuído e papel em um ícone + número.
- Tela de acolhida refeita no estilo do jogo (três passos, uma frase cada).
- Exportar todas as telas com `tests/ux_visual.rs` no tamanho novo para `docs/expansao/telas/` (PNG via `docs/ux_rasterizar.py`).

### E6 — Menus II: loja, caixas, coleção, moedas e sala de máquinas

Ler: tudo do E5, mais `src/simulation/economy.rs` e `src/simulation/machine.rs`.

Entregar:
- **Desenho novo das moedas:** sprite 16×16 e versão grande (32×32 ou 48×48) da moeda com roda de trem, pilha de moedas, saco de moedas; ícones próprios para cada uma das cinco caixas (caixote de madeira, engradado de oficina, mala de viagem, galpão/garagem, lata de tinta), em SVG sem fundo em `assets/icons/` + regeneração de `src/ui_sprites.rs`.
- **Loja:** as cinco caixas como objetos grandes, preço, "faltam N para raro ou épico", botão Abrir; toque em "i" mostra as chances e o conteúdo. Compra direta em uma segunda página.
- **Abertura de caixa:** animação curta (≤ 2 s, respeita reduzir movimento): caixa treme, abre, peça aparece desenhada pelo renderer com moldura da raridade, nome, "Nova!" ou "Repetida · +N moedas", botões Equipar / Continuar. O resultado já está gravado antes da animação; fechar o jogo no meio reapresenta a mesma peça.
- **Coleção:** grade de vagões, locomotivas, pinturas e postais com contagem "possuídas/total" por categoria e por raridade.
- **Recompensas visíveis:** moedas ganhas aparecem como "+N" saindo do trem na chegada (na faixa e na prévia), e um resumo curto ao abrir a central ("Desde a última visita: +84 moedas, 3 ordens concluídas").
- **Sala de máquinas** refinada: seis medidores com sprite, barra, e o efeito atual ("Guindaste: carga 14% mais rápida"); total do bônus de moedas em destaque.
- Exportar as telas novas para `docs/expansao/telas/`.

### E7 — Correções da revisão de telas

Entrada: `docs/expansao/REVISAO_TELAS.md`, escrito por um agente revisor a partir das imagens exportadas. Aplicar as correções por prioridade, reexportar as telas, e listar o que não foi feito.

## Relatos

_(cada agente acrescenta aqui a sua seção, com data e hora)_

### E1a — economia no domínio e checkpoint v5 — 2026-11-16, 01h20 (Brasília)

**Entregue:** carteira com saldo/totais e 128 transações de ID crescente; inventário por categoria/ID e posse reutilizável do modelo; renda por chegada, passageiros e contrato com restos persistidos; contadores das 15 espécies de carga; entrada `World::set_machine_bonus(f32)` limitada a ×1,00–1,30; Pátio geral completo, com 29 IDs explícitos, probabilidades/garantia públicas, sorteio determinístico, proteção contra repetidas e reembolso; compra direta; compra e confirmação de revelação em duas gravações, com resultado persistido antes da apresentação. A remessa grátis e as 30 moedas iniciais existem tanto para perfis novos quanto migrados.

**Persistência e integração:** checkpoint v5 inclui todos os campos pedidos, inclusive mapa de progresso por linha, missões pagas, ordens com baselines, garantia das cinco caixas e selos do museu. Migra v1–v4 com backups exatos e preserva `.previous.json`, importação e proteção de versões futuras. Foram acrescentadas seis fixtures estáticas e um exportador que recusa sobrescrever arquivos. `progress::economy_action`/`transact_economy` gravam imediatamente e só publicam o candidato em memória após sucesso. `progress::advance_checkpoint`, ligado ao app, faz o mesmo na chegada; falha devolve mundo/trem ao estado anterior, pausa a viagem e avisa. Não clona Config; clona World somente em possível chegada ou transação explícita.

**Posse:** perfil novo equipa Passageiros/Passageiros/Carga e possui também Tanque, Vapor Aurora e Verde Aurora. Só ganha por marcos Dormitório no primeiro circuito, Diesel do Porto pelo postal de carga e as quatro pinturas originais. O migrado recebe tudo que podia equipar pelas regras anteriores naquele momento, inclusive pinturas elegíveis ainda ausentes do vetor antigo. Pontos históricos não viram moedas; espécies legadas começam em zero. A oficina, a abertura e a importação agora conferem posse de locomotivas, vagões e pinturas; não foi criada tela de loja nem redesenhado layout.

**Contratos públicos e pedidos para E1b:** documentação em [ECONOMIA_E1A.md](expansao/ECONOMIA_E1A.md). Usar `progress::economy_action(path, world, EconomyAction::{BuyCrate, BuyDirect, Reveal})`; `Reveal` é a confirmação de apresentação concluída, com o ID já persistido. `Ok(Some(aviso))` significa compra confirmada com aviso da cópia de recuperação — não repetir o débito. Para missões, usar `progress::transact_economy` e inserir o ID pago/chamar `EconomyState::record` no mesmo candidato, depois de validar o requisito no domínio. A flag `legacy_mission_claim_pending` está ligada nos migrados: E1b deve pagar seus marcos anteriores uma única vez, gravar e avisar. As outras quatro variantes de `CrateKind` e seus contadores já existem, mas `definition()` retorna indisponível até E1b publicar/validar os conjuntos. Ordens, selos, resgates e simulador não foram implementados, conforme a divisão solicitada. E2 deve alimentar o multiplicador com amostras válidas; ele não persiste leituras. E3 deve consumir `lines`, adaptar sua validação e persistir a troca sem reiniciar a economia global; nesta entrega só Aurora é aceita/jogável.

**Decisões para Pedro rever:** a caixa grátis conta na garantia, mas nunca devolve moedas; raro/épico repetido também zera a garantia. O mínimo de duas moedas de contrato preserva o resto da divisão por vinte. O bônus usa milésimos, com fração guardada entre chegadas; não incide sobre boas-vindas, compra ou reembolso. A posse antiga é fotografada na migração; chegadas futuras não continuam concedendo toda a loja ao migrado. Salvar uma chegada falha pausa a viagem até o jogador retomar, evitando tentativas de I/O a cada frame. As decisões estão documentadas sem afirmar que o balanceamento por hora já foi atingido.

**Raridades:** preservadas as propostas dos agentes 1, 5 e 7. Complementos antigos: Passageiros/Carga/Tanque/Vapor Aurora comuns; Dormitório/Panorâmico/Diesel do Porto incomuns; Elétrica da Serra rara. Para as quatro pinturas originais, defini Verde Aurora/Azul do Porto comuns, Cobre do Poente incomum e Expresso Noturno rara; nenhuma pintura épica foi inventada para preencher tabela. As seis pinturas adicionais mantêm Cereja/Lagoa/Ardósia comuns, Marfim/Girassol incomuns e Urze rara. Todas as 39 peças têm raridade explícita em `data/catalog/core.json`; capacidades e aparência não mudaram. A raridade ficou no catálogo, reexportada pela economia, para manter os exemplos que compilam o catálogo isoladamente.

**Verificação final executada:** `cargo fmt --all`, `cargo clippy --locked --all-targets -j 2 -- -D warnings` e `cargo test --locked --all-targets -j 2`, todos com código 0 via `scripts/dev.ps1`, toolchain Windows local e Cargo offline. **408 testes aprovados, zero falhas, dois exportadores opcionais ignorados, 61 alvos.** A compilação da suíte final levou 2 min 26 s. Log local: `docs/expansao/e1a-testes-final.log`, terminado em `EXIT_CODE=0` (logs seguem ignorados pelo Git). `git diff --check` passou. Cobertura nova: conjunto e probabilidades, garantia, novidade/repetidas/grátis, determinismo, renda/restos/mínimo, bônus, compra direta/equipamento, log limitado, campos reservados preenchidos, falha de compra/revelação/chegada, reabertura, migração v1–v4 com backups, corrupção e recuperação de pendência. Testes de renderer/alocações e os exemplos isolados também passaram; os programas dos exemplos de desktop não foram executados.

**Correções durante a verificação:** uma primeira chamada do wrapper PowerShell não repassou `-- -D warnings`; corrigida com `-CargoArgs` explícito. A primeira checagem encontrou o acoplamento do catálogo à economia nos exemplos isolados, corrigido. A primeira suíte completa parou na expectativa antiga de quatro pinturas em uma migração que agora concede também Cereja; a expectativa foi atualizada pelo direito adquirido, preservando a conferência do backup exato. Os limites de save passaram de 16 para 32 KiB no teste de um dia, para incluir as 128 transações novas; o limite de leitura do produto continua 1 MiB. Nenhum hash de renderer foi alterado. Depois dos gates finais, somente a documentação e o log foram completados.

**Não verificado:** compositor, teclado/foco/DPI, animação de abertura e loja no desktop, uso prolongado/energia, queda de energia física, release ou distribuição. A cobertura de falha de escrita usa diretórios temporários, arquivos bloqueados/inválidos e escritor injetado; não promete proteção contra edição manual ou restauração deliberada de backups antigos. Não executei o simulador nem aferi 90–130 moedas/h ou tempo até a primeira compra; isso pertence a E1b. Não houve pagamento efetivo de missões, ordens, outras quatro caixas, selos ou troca jogável de linha.

**Restrições:** trabalho sozinho, na branch `expansao`; sem commit, download, envio externo, alteração de Cargo/dependências, remoção/movimentação manual de arquivos existentes ou acesso ao perfil real. Cargo offline, uma invocação por vez, no máximo dois jobs, `CPU_EXPRESS_CONFIG` temporário e clima offline. Fixtures antigas preservadas. `BUILD-INFO.json`, manifests e binário distribuído rc.3 continuam intactos, pois esta entrega é fonte de desenvolvimento v5, não outra distribuição rc.3. Toda documentação nova está em português.

### E1b — caixas, missões, ordens, museu e simulação — 2026-11-16, 01h50 (Brasília)

**Entregue:** as outras quatro caixas, com conjuntos explícitos por ID e validação de catálogo/faixas; pagamento único dos onze marcos e das sete metas por espécie (405 moedas no total); ordens com baselines duráveis, criação em chegada, intervalo compartilhado de dez minutos, troca gratuita e limite de três comuns mais um telegrama; selos numerados após completar os 39 modelos; simulador de 1/3/8 h por dia por trinta dias, com e sem serviços, em bônus ×1,00 e ×1,30. A economia e a geração de ordens continuam no domínio; nenhuma tela foi criada nesta metade.

**Persistência:** usei os campos já reservados no v5, sem novo formato ou dependência. Resgates, trocas e selos passam pela mesma gravação imediata das compras. `load_profile` agora confere `legacy_mission_claim_pending`, paga os marcos completos ainda não pagos, grava e avisa o valor. Falha conserva o mundo carregado, o saldo e a flag pendente, com aviso para tentar na próxima abertura; não reinicia a viagem. `load_world` continua leitura pura. Backups exatos v1–v4, `.previous.json`, saves futuros e as fixtures estáticas anteriores permanecem preservados. Importações legadas conservam a flag e são conferidas na próxima abertura, ou antecipadamente por ação explícita transacional. A mudança de preço do Pátio usa versão de conjunto 2; resultado pendente da versão 1 mantém preço, item e reembolso originais.

**API para E5/E6:** referência completa em [ECONOMIA_E1B.md](expansao/ECONOMIA_E1B.md). Consultar `world.economy().missions(world.journey_snapshot(), world.service_state())`; cada `MissionStatus` tem `id/title/current/goal/reward/paid/visible` e `claimable()`. Consultar `daily_orders`, `order.template()`, `progress(economy, journey, service)` e `claimable(...)`. Enviar `EconomyAction::ClaimMission(id)`, `ClaimOrder { order_id }`, `SwapOrder { order_id }` e `BuyMuseumStamp` por `progress::economy_action`. As ações `BuyCrate`, `BuyDirect` e `Reveal` do E1a continuam; não pagar na UI. `Ok(Some(aviso))` é transação confirmada, não uma falha para repetir. `CrateKind::ALL`, `name()`, `definition()`, `effective_weights(misses)` publicam a loja; `EconomyState::collection_items()/collection_complete()` publicam os 39 modelos; `museum_stamps` representa números de 1 a N. A UI deve normalizar os pesos da garantia e continuar apresentando a pendência salva antes da animação.

**Balanceamento executado:** a primeira execução preliminar de dez sementes encontrou cerca de 220–280 moedas/h com serviços, e compra aos 6,68 min, acima do ritmo pedido. Apliquei fator 0,35 à renda recorrente de chegadas/entregas antes do bônus da máquina, preservando os restos e os valores já pagos. Preços Pátio/Cargas/Plataforma/Garagem/Ateliê: 150/225/225/450/188, antes 60/90/90/180/75. Compra direta 120/240/480/900, selos 120 e os 405 de missões ficaram intactos. A saída integral das 1.000 sementes por cenário está em [economia-simulacao.md](expansao/economia-simulacao.md), com estratégia, reprodução, percentis e censura das coleções incompletas. Para o perfil inicial com serviços, medianas de 101–124 moedas/h e primeira compra por volta de 20,3 minutos. Sem serviços, o começo é muito mais lento; não alego que esse extremo atinja o alvo do perfil padrão.

**Decisões para Pedro rever:** Plataforma contém Passageiros/Dormitório/Panorâmico/Restaurante/Correio/Bicicletas/Feira; Feira fornece a raridade rara pelo tema de serviço ao público. Plataforma usa 60/29/11/0 e Ateliê 65/25/10/0: a chance de épico foi somada a raro, sem inventar raridade. Cargas inclui os quinze vagões de carga e cinco motores temáticos; Garagem os dez motores; Ateliê as dez pinturas. Ordens pagam 10 por 24 chegadas, 120 passageiros ou uma hora ativa, e 15 por oito contratos ou 48 unidades de carga. Entram uma a uma após dez minutos; não aparecem três imediatamente. A elegibilidade aceita composição equipada ou uma substituição de vagão possuído que preserve a ocupação; é conservadora quando seriam necessárias duas trocas. Trocar exige estar na parada e respeitar tanto a última entrada quanto a última troca. `ce.liveries-4` exige as quatro pinturas originais. Selos exigem modelos, não todos os postais. A redução de renda também afeta quem escolhe não transportar ninguém: revisar essa diferença de ritmo e a longa cauda do último épico. A estratégia obrigatória do simulador insiste em caixas enquanto resta qualquer novidade; não representa quem usa compra direta para completar cedo.

**Pedidos seguintes:** E5 liga Missões e ordens, com aviso de prêmio disponível, resgate por ID e tratamento do erro de gravação; não gerar substitutas na tela. E6 liga as cinco caixas, selos, alternativas de compra direta e revelação pendente, mostrando as probabilidades reais sem épico nas duas caixas. E2 pode chamar `World::request_telegram()`; a próxima chegada elegível pode oferecer o quarto cartão, dentro do mesmo intervalo, sem acumular pedidos. Não editar o prêmio da instância: a validação confere a definição. Reavaliar a frequência dos telegramas com o simulador, que não os gera no cenário de referência. E3 deve manter contadores globais monotônicos de tempo/chegadas ou migrar baselines equivalentes ao trocar de linha; hoje o snapshot é Aurora. Mudanças futuras em metas/prêmios de ordens precisam preservar ou migrar instâncias já salvas.

**Verificação final executada:** `cargo fmt --all`, `cargo clippy --locked --all-targets -j 2 -- -D warnings` e `cargo test --locked --all-targets -j 2`, todos com código 0, Cargo offline e toolchain Windows local. **423 testes aprovados, zero falhas, dois exportadores opcionais ignorados, 63 alvos.** Logs locais: `docs/expansao/e1b-clippy-aprovacao-v2.log` e `e1b-testes-aprovacao-v3.log`, ambos terminados em `EXIT_CODE=0`. O teste final incremental compilou em 3,30 s; isso não é tempo de build limpo. `git diff --check` passou. Testes cobrem probabilidades/conjuntos e garantia das cinco caixas, 18 resgates únicos, limites por espécie, falha/reabertura de missão/ordem/selo, migração com aviso e backup, falha de migração sem perder a viagem, elegibilidade com ocupação, limites/cooldowns/baselines/telegrama, corrupção de definição e preservação da compra pendente antiga. O exemplo também compara carteira, inventário, ordens e RNG reproduzidos com o World real em quatro combinações de serviços/bônus.

**Correções encontradas pelos gates:** a primeira invocação do wrapper não repassou os argumentos; passei a usar `-CargoArgs` explícito. A inclusão dos redutores no simulador inicialmente referenciava uma API privada do serviço; foi substituída por `can_carry_with`, consulta pura de capacidade. O modo de teste do exemplo exigiu importação relativa. Os testes antigos de migração esperavam que `load_profile` não escrevesse nem avisasse: agora distinguem a inspeção pura de `load_world` da migração transacional, mantendo a conferência dos bytes de backup e acrescentando igualdade da economia em reaberturas. Não alterei fixtures antigas nem hashes de renderer. O teste de paridade do simulador detectou que o traço usava o fim do frame, até um segundo depois da chegada; corrigi o instante pela parcela já consumida dos oito segundos de parada e repeti as 1.000 sementes. Logs de tentativas anteriores permanecem históricos, não aprovações. O texto `NativeCommandError` nos logs PowerShell vem do stderr de status do Cargo; os gates citados encerram com código 0.

**Não verificado:** telas E5/E6, abertura animada, teclado/foco/DPI, compositor, uso prolongado/energia, queda física de energia, release e distribuição. A simulação varia sementes econômicas sobre a rota padrão e composições fixas, sem excursões, telegramas nem comportamento humano; não promete renda para qualquer composição/linha. A primeira rara reportada inclui a pintura de marco, não só resultados de caixas. Os testes de disco usam caminhos temporários e falhas controladas. A versão distribuída rc.3, BUILD-INFO e manifestos não foram substituídos: esta é uma entrega de fonte de desenvolvimento.

**Restrições respeitadas:** sozinho na branch `expansao`, sem commit, download, envio externo, remoção/movimentação manual de arquivos existentes ou acesso ao perfil real. Uma compilação por vez, até dois jobs, `CPU_EXPRESS_CONFIG` temporário e clima offline. Nenhuma alteração de Cargo, dependências, arte, métricas ou layout. Relatos anteriores preservados; a documentação histórica de missões e E1a recebeu apenas uma indicação para a API atual.
### E5 — menus I, missões e conteúdo no aplicativo — 2026-11-16, 02h36 (Brasília)

**Entregue:** central nativa travada em 704 × 676 pontos, sem alças de redimensionamento, arrastável pela faixa, com posição preservada e redução proporcional por zoom/pixels por ponto quando a área útil é menor. Doca Trem / Viagem / Loja / Coleção / Ajustes / Fechar; Ajuda em `?`; saldo de Réis de Ferro sempre na faixa. Trem reúne cabine, vagões, motores, pinturas e favoritos. Viagem reúne mapa, Missões, diário, serviços e ramais. Janela, clima, fluidez e arquivo foram divididos em páginas curtas. A doca e a linha de estado têm espaço reservado; não há ScrollArea na central nativa. O fallback de backends sem viewports nativos conserva o painel legado.

**Missões:** três cartões de ordens, espaços de espera honestos, progresso/prêmio, Recolher e troca gratuita sujeita às regras existentes; eventual telegrama fica na segunda página. Marcos visíveis paginados de três em três e próxima recompensa destacada. Indicador discreto na doca quando há prêmio. `src/app/missions.rs` consulta o domínio e chama `progress::economy_action` para resgatar/trocar; não gera ordens nem concede moedas na apresentação. Erro de escrita preserva carteira/progresso e permanece no status. Aviso de recuperação após sucesso não repete débito/crédito. Demonstração sem caminho de save explica por que não recolhe.

**Seletores e acolhida:** quatro miniaturas reais do StockGallery por página, raridade por moldura e palavra, posse/equipamento explícitos, posição do vagão e ícone/capacidade. A dica informa categorias de carga compatíveis. Locomotivas não receberam números fictícios de desempenho: permanecem escolhas de aparência. Favoritos continuam disponíveis; quatro composições com nomes longos cabem sem deslocar os botões. Acolhida com prévia e três passos, sem coordenadas na primeira abertura; cidade muda em Ajustes. Acrescentei `Config.use_input_activity`, default true e opt-out na acolhida/Fluidez, preparando a E2 sem implementar coleta. O texto informa a indisponibilidade atual; reabrir acolhida parte das preferências atuais. Sem mudança de checkpoint.

**Conteúdo pendente ligado:** `src/app/scenery.rs` envia as etiquetas do trecho atual/próximo do catálogo Aurora, estilos de partida/chegada e opções de movimento ao renderer em `step_frame`. Excursões limpam o contexto. Aurora/Bosque/Serra/Porto usam Planalto/Madeira/Pedra/Terminal portuário, escolhas fictícias explícitas no adaptador. A aparência do app muda intencionalmente para mostrar a arte pendente; o caminho vazio do renderer e seus hashes antigos permanecem. Reduzir movimento agora alcança menus, passageiros e vida da paisagem. Frio visual abaixo de 12 °C só usa amostra meteorológica finita e não vencida. Relógio, renda, serviço, capacidades e privacidade de métricas não mudaram.

**Evidências:** 45 PNGs em [telas/README.md](expansao/telas/README.md), exportados por `tests/ux_visual.rs` no novo tamanho. Incluem todos os seletores paginados, missões vazias/avançadas, diário, serviços com oferta/bloqueio/devolução, ramal ativo, ajustes e acolhida. Inspeção visual dos principais estados confirmou doca, saldo, botões e textos dentro das páginas. Guia e reprodução em [MENUS_E5.md](expansao/MENUS_E5.md). Os JSONs intermediários são regeneráveis e ficam ignorados pelo Git. O Python 3.13 local foi encontrado por `py`, mas retornou **Acesso negado** ao iniciar; sem instalação ou contorno de permissões, acrescentei `docs/ux_rasterizar.cjs`, alternativa local sem dependências às mesmas regras do rasterizador Python. Não comparei numericamente as duas implementações. As imagens são headless, não screenshots do desktop.

**Verificação final executada:** `cargo fmt --all`, `cargo clippy --locked --all-targets -j 2 -- -D warnings` e `cargo test --locked --all-targets -j 2`, todos com código 0, Cargo offline e toolchain Windows local. **429 testes aprovados, zero falhas, três exportadores opcionais ignorados, 63 alvos.** Exportação E5 foi executada separadamente e passou (1 teste). Logs locais: `e5-fmt-final.log`, `e5-clippy-aprovacao-v5.log` e `e5-testes-aprovacao-v4.log`, em `docs/expansao/`, terminados em EXIT_CODE=0. A compilação da suíte completa levou 1 min 45 s (incremental, não build limpo). `git diff --check` passou. Seis testes novos: limites de altura/largura e doca visível, estados excepcionais, escala proporcional, reaplicação dos limites nativos após zoom, botão Recolher com gravação/reabertura/repetição recusada/falha sem alterar carteira, e seleção/limpeza do contexto de cenário. Suíte existente de save, serviços, alocações e hashes também passou; nenhum hash foi trocado.

**Correções durante a verificação:** o teste de cabimento encontrou Fluidez longa, dividida em páginas; a primeira imagem revelou a doca abaixo da janela, corrigida a reserva de espaço conforme a API atual do egui e acrescentada asserção do botão Fechar visível. Ajustados APIs e avisos do Clippy, título/saldo e rótulos da sessão com contraste e subabas com fonte consistente. O zoom do egui só vale no quadro seguinte: acrescentei reaplicação das dimensões e limites nativos, com teste dos comandos de tamanho; a primeira asserção contava também um comando interno do egui e foi delimitada aos comandos pertinentes. A tentativa de redirecionar stderr dentro de `scripts/dev.ps1` disparou NativeCommandError; os gates finais com logs usam a mesma toolchain por `cmd /d /c`, preservando o código de saída. Logs anteriores de tentativa não são aprovação. `cargo fmt` normalizou as quebras de linha do exportador de testes.

**Decisões para Pedro rever:** quatro peças por página; três marcos e três bilhetes; telegrama paginado; geometria separada do comportamento; oficina dentro de Trem; quatro arquiteturas Aurora escolhidas por ID; limiar visual de frio em 12 °C; a preferência de atividade já existe, mas a leitura segue ausente até E2. Loja é uma apresentação honesta das cinco remessas/preços, sem botão de compra provisório; Coleção mostra postais nesta etapa, com acesso às peças pelo Trem. Mantive os fluxos existentes de importação/exportação e serviços, sem redesenhar suas transações.

**Pedidos aos próximos agentes:** E6 implementar compras, probabilidades/garantia, revelação pendente, coleção completa e retorno visual das moedas nas novas páginas, usando o domínio E1. E2 consumir `use_input_activity` e atualizar o texto de disponibilidade, mantendo o opt-out e sem gravar leituras. E3 substituir o ID Aurora fixo do adaptador pela linha ativa, selecionar próximo trecho pelo sentido do itinerário e levar arquitetura ao catálogo; E4 já encontra os três setters ligados. E7 conferir acessibilidade, foco, teclado, DPI e compositor, além do teste comprar → equipar → fechar/reabrir. A ação do domínio de resgate não deve ser substituída por pagamento na UI.

**Não verificado:** execução interativa/compositor Windows, redimensionamento real entre monitores, DPI misto, teclado/Narrador, animação contínua, consumo/energia, sessão longa, falha física de disco, loja/revelação ainda não implementadas, release ou distribuição. Os exemplos de desktop foram compilados como alvos de teste; seus programas não foram executados. Não afirmo que o cálculo de escala ou os PNGs provem apresentação nativa. O fallback de painel embutido não recebeu o novo layout. O fluxo econômico completo de E7 permanece pendente.

**Restrições respeitadas:** sozinho, branch expansao, sem commit, download, envio externo, remoção/movimentação manual de arquivos existentes ou acesso ao perfil real. Uma compilação por vez, até dois jobs, perfil temporário quando indicado e testes/exportação em demonstração/offline. Cargo/dependências, checkpoint v5, BUILD-INFO, manifestos e binário distribuído rc.3 não foram alterados. Relatos anteriores preservados; esta entrega é fonte de desenvolvimento, não nova distribuição.
