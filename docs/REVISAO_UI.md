# Revisão de UI/UX de jogo — CPU Express

**Autor:** Codex, para Pedro e para a implementação pelo Claude Code. **Data:** 14/11/2026. **Base:** cópia de trabalho `ui-redesign`, posterior à base integrada 0.4.0-rc.2. Esta é uma avaliação e especificação de melhorias; não é uma alteração de `src/` nem evidência de execução do aplicativo.

## Parecer

O caminho visual está certo. Madeira, pergaminho, faixa verde, fonte própria e doca já dão ao CPU Express uma identidade de jogo. A próxima passada deve investir em **composição da informação e resposta às ações**, não em outra troca de tema.

O que ainda parece site/formulário é a sequência **título → explicação → caixa → explicação → botão largo**. As seções têm pesos parecidos, até quando uma contém a decisão do jogador e outra apenas explica uma regra. Serviços começa pela configuração de despacho; Diário começa por importação/exportação; Vagões repete instruções e totais antes de mostrar as peças. Acolhida começa pedindo coordenadas. Há conteúdo de jogo, mas ele entra depois da administração.

A comparação com as referências sugere uma direção: objetos reconhecíveis, números próximos desses objetos, seleção inequívoca e uma ação principal por situação. Aproveitar essa linguagem, sem copiar sprites, personagens, layouts exatos ou sistemas de monetização. Densidade não é automaticamente ruim: **uma grade de vagões legível pode mostrar mais coisas e exigir menos esforço que três parágrafos**.

## Escopo e evidência

Foram examinadas as 26 imagens em `pesquisa/redesenho-menus-2026-11-14/` e as 17 referências locais em `entrada/ui-references/1/`. As imagens atuais são desenhos headless das telas reais, vários em DEMO/pausa. Não provam comportamento de foco, compositor do Windows, leitor de tela, áudio ou conectividade. Não interpretar RAM indisponível, importação desabilitada ou pinturas liberadas na demonstração como falha do produto normal.

Leitura: `README.md`, `docs/GUIA_04.md`, `AGENTS.md`, `src/ui_game.rs`, `src/ui_services.rs`, `src/ui_stock.rs`; em `src/app.rs`, `floating_panel`, `overlay`, `drive_controls`, `compact_drive_controls`, `welcome_controls`, `behavior_controls`, `help_contents`, `shortcuts`, prévia, notificações, salvamento e adaptador de cartões; todos os cinco arquivos de `src/app/`: `journey.rs`, `branches.rs`, `services.rs`, `storage.rs`, `workshop.rs`. Usar nomes de funções como localização: o outro agente está editando essa árvore e as linhas podem mudar.

| Tela/estado | Capturas examinadas |
| --- | --- |
| Trem | `floating_trem.png` |
| Mapa | `floating_viagem.png` |
| Ramal | `floating_branch.png`, `floating_branch_minimum.png`, `floating_branch_minimum_scrolled.png` |
| Objetivos | `floating_objetivos.png` |
| Diário vazio/ativo | `floating_diario.png`, `floating_diario_active.png` |
| Serviços vazio/ativo | `floating_servicos.png`, `floating_servicos_active.png` |
| Oferta, bloqueio, devolução | `floating_contract_offer.png`, `floating_contract_blocked.png`, `floating_contract_return.png` |
| Oferta estreita/mínima | `floating_contract_small.png`, `floating_contract_minimum.png`, `floating_contract_minimum_scrolled.png` |
| Coleção | `floating_colecao.png` |
| Vagões | `floating_visual.png`, `floating_visual_small.png`, `floating_visual_minimum.png`, `floating_visual_minimum_scrolled.png` |
| Pinturas, clima, fluidez, motores | `floating_pinturas.png`, `floating_clima.png`, `floating_fluidez.png`, `floating_motores.png` |
| Janela | `floating_janela.png` |

**Ajuda e acolhida:** avaliação pelo código; não há captura dessas duas telas entre as 26. As sugestões de interação abaixo são propostas, não resultados de teste nativo.

## Prioridades e regras comuns

- **Essencial:** a pessoa entende o estado, encontra a ação principal em 480 × 480 e consegue agir sem depender de mouse, cor ou tooltip. Corrigir também mensagens que prometem um estado diferente do domínio.
- **Importante:** tornar objetos e decisões mais visuais, reduzir repetições e conectar recompensa à coleção/oficina.
- **Desejável:** detalhes de ambientação e pequenas animações depois de resolver os pontos anteriores.

Ordem de implementação: (1) área útil mínima e estados/ações de Serviços e Vagões; (2) dicas, foco e semântica; (3) acolhida e Diário; (4) mapa, objetivos e coleção; (5) ajustes e acabamento. Não é necessário refazer todas as páginas para começar a melhorar o jogo.

### Hierarquia compartilhada

1. Um objeto ou situação principal: trem atual, rota ativa, encomenda, vagão selecionado ou recompensa.
2. Até três fatos necessários à decisão, próximos do objeto. Ex.: carga `11 un.`, destino e `110 pontos`.
3. Um verbo principal: `Aceitar`, `Equipar`, `Retomar`, `Começar viagem`. A alternativa tem menos peso visual.
4. Regras e detalhes secundários em expansão acessível. Não esconder o motivo de bloqueio, a ausência de capacidade ou o efeito de uma devolução.

Usar ícone **com** nome curto na navegação. Somente ações universais ou redundantes podem ficar sem texto visual, sempre com nome acessível: fechar, setas, som. Os ícones do pacote estão em [assets/icons](../assets/icons/README.md). `pontos` é uma condecoração, nunca moeda ou preço; `encomenda` é pacote etiquetado, `caixote` representa suprimentos/carga, `tanque` identifica líquidos. Não usar o mesmo caixote para todas as categorias.

**Essencial — coerência da navegação.** A faixa diz “Diário de viagem” inclusive em Mapa, Objetivos e Serviços, enquanto existe uma subaba “Diário”. Trocar a faixa por **“Viagem”** e deixar o título específico no conteúdo. “Cabine do maquinista” e “Oficina ferroviária” podem permanecer. Manter os nomes da doca nesta rodada; se “Visual” virar “Oficina” no futuro, mudar a documentação e todos os atalhos juntos.

**Importante — ritmo tipográfico.** A fonte pixelada é uma assinatura. Mantê-la em faixas, números destacados e rótulos curtos. Hoje `game::button` decide a fonte por cabimento; nas telas estreitas, “Mapa” aparece pixelado enquanto “Objetivos” fica proporcional. Escolher uma regra para o grupo inteiro: no modo compacto, todos os cinco rótulos têm a mesma escala/família. Não encolher uma palavra isolada até ficar ilegível. Texto de explicação e campos continuam em fonte de leitura.

**Importante — madeira com função.** Manter a moldura e a faixa principal; reduzir as placas de madeira internas de 30 pontos quando só repetem um título. Cartões de lista podem ter nome + objeto + estado no próprio pergaminho. Não transformar todos os blocos em pequenas janelas.

## Melhorias tela por tela

### Trem — `drive_controls`

**Essencial.** Reunir “Próxima parada”, destino, barra e pausa em uma cabine compacta. Substituir “Trecho · 0%” por **“Bosque dos Pinheiros · faltam 80 s ativos”**, usando o tempo real do domínio. Durante serviço: **“Embarque · partida em 8 s ativos”**; pausado: **“Pausado · faltam 80 s ativos”**, sem contagem regressiva. Em excursão, destino e fase do ramal substituem o circuito. Não dar a entender que km/h calculados da CPU determinam a chegada.

**Importante.** Colocar a prévia do trem junto do destino, com a ação `Pausar`/`Retomar` ao lado ou logo abaixo. “Espaço também pausa e retoma. Seu progresso fica preservado.” vira dica de atalho junto ao botão e explicação na Ajuda. “0 visitas · 0 circuitos” vira resumo discreto no mapa/diário. Os três cartões “Velocidade”, “Processador”, “Memória” viram uma linha compacta: locomotiva `56 km/h visuais`, CPU `42%`, memória `61%`; nome acessível completo e estado `— / indisponível` por métrica. Manter a origem simulada identificada.

**Importante.** “Passageiros e encomendas” vira um atalho com ocupação real: passageiro `7/28` e caixote `11/20`, com rótulo `Serviços`. Última chegada vira bilhete resumido de uma ou duas linhas; `Ver bilhete` abre o Diário. No início, mostrar **“Primeira parada: Bosque dos Pinheiros”**, sem outro cartão vazio explicando que o bilhete virá depois.

**Desejável.** Uma breve marca na estação e atualização do bilhete ao chegar. Não chacoalhar a janela nem deslocar o botão de pausa. “Simulação e diagnóstico” continua recolhido e secundário; não trazê-lo para a cabine principal.

### Viagem / Mapa e ramais — `journey_controls`, `branch_controls`, `branch_map`

**Essencial.** A rota em andamento domina a tela. Durante excursão, recolher “Circuito Aurora” em **“Circuito principal · aguardando retorno”**. Hoje o ramal tem mapa, três barras, percentual total, tempo total e depois um segundo mapa; em 480 × 480 nem a primeira barra aparece inteira. Manter o mapa do ramal + uma barra da fase atual + três marcadores **Ida / Visita / Volta**, com marca de concluído, atual e futura. Ex.: **“Ida · faltam 15 s ativos”**; linha secundária **“Retorno ao circuito em 113 s ativos”**. Na pausa, mostrar “Relógio parado” junto do tempo.

**Essencial.** No circuito pausado, o texto “Próxima parada em 80 s” precisa qualificar o tempo ativo e a pausa. Não manter aparência de prazo correndo. Estações desconhecidas precisam de diferença de forma: marcador vazado/cadeado + “Não visitada”; hoje a diferenciação dos ícones passa sobretudo pela cor. A estação selecionada e o foco devem ter indicadores distintos.

**Importante.** Tirar “Explorar ramais” da frente do mapa principal quando não há excursão. Depois do mapa, mostrar dois destinos com `montanha` e `porto`, nomes e estado: **“Libera ao completar 1 circuito”**, **“Agendar visita”** ou **“Agendada · parte de Vila Aurora”**. Mostrar “3 min 08 s ativos” e “Retorna ao circuito” uma vez. Não prometer entregas no ramal. Ao agendar, resposta local **“Mirante da Serra agendado”** e marca no destino, sem modal nem novo botão de confirmar.

**Importante.** “Ver objetivos e recompensas” repete uma aba que já está visível. Substituir por um objetivo contextual curto, por exemplo **“Próxima pintura · 3/4 chegadas”**, clicável para o objetivo. Na ficha da estação, “7 passageiros a bordo com destino a esta estação” vira passageiro **“7 desembarcam aqui”**; descrição do lugar em expansão. A seleção da estação precisa trazer essa ficha à vista sem esconder o mapa inteiro.

**Desejável.** Dar ao mapa pequenas referências próprias de terreno: ponte, túnel, porto e serra. Ícones marcam lugares; não inventam ramais, distâncias nem mecânicas. Não redesenhar a rota aurora-v1 para adequá-la à decoração.

### Viagem / Objetivos — `journey_controls`

**Essencial.** O cadeado antes de “Primeiro serviço” comunica “não posso fazer isto”, quando o objetivo já está ativo. Usar `bandeira` no objetivo em andamento, `confirmar` + **“Concluído”** no terminado e cadeado apenas em conteúdo de fato indisponível. A barra `0/1 chegadas` é útil e deve ficar.

**Importante.** Remover o título redundante “Caderno de objetivos”. Cada objetivo vira **miniatura da recompensa + nome do objetivo + requisito + barra**. “Recompensa: Azul do Porto” vira miniatura da locomotiva pintada + **“Azul do Porto”**. Texto “Chegue à próxima estação e conheça a plataforma.” pode virar **“Chegue a 1 estação”**. “Cartão de explorador” deve explicar o critério realmente contado: a implementação compara `stations_visited` com `goal`. Não trocar uma contagem de chegadas por contagem de estações distintas só na UI; alinhar descrição e domínio com o integrador.

**Importante.** Primeiro mostrar o objetivo em curso; concluídos em “Conquistas”, sem desaparecer a recompensa. “Ver pintura liberada” pode ser **“Ver pintura”** e continuar indo ao cartão correto. Nunca criar “Resgatar”: a propriedade já foi concedida pelo `World`.

**Desejável.** Um selo breve ao completar e um marcador “Novo” no destino da recompensa, removido ao inspecioná-la. Esse marcador é estado de apresentação; não deve alterar propriedade nem exigir novo formato de checkpoint nesta rodada.

### Viagem / Diário — `journey_controls`, `arrival_ticket`, `storage_controls`

**Essencial.** Começar pelos bilhetes. “Exportar viagem…” e “Importar viagem…” vão para **“Arquivo da viagem”** recolhido, junto de `Salvar agora`. Hoje o jogador abre um diário e encontra gerenciamento de arquivos. Manter erro de gravação/recuperação visível, mesmo com essa seção fechada.

**Essencial.** O texto “Chegada 9 · registro salvo · últimas 32 chegadas” não deve afirmar gravação em disco apenas porque existe `ArrivalSummary`. `arrival_ticket` monta a frase sem verificar o último resultado de save. Trocar por **“Chegada 9”**; no cabeçalho do arquivo, mostrar separadamente o estado confirmado do salvamento, ou **“Não foi possível salvar”**. “Últimas 32 chegadas” aparece uma vez no título da lista.

**Importante.** Bilhete compacto: **estação · chegada #9**; passageiro **“+20 embarcaram / −8 desembarcaram”**; carga **“10 entregues”**; pontos **“+116”**. Setas e números sempre têm verbos ou nomes acessíveis. Omitir categorias zeradas do resumo, mantendo-as em `Detalhes`. Não exibir “0 carregadas · 0 entregues · 0 devolvidas” em todos os bilhetes sem carga. A devolução permanece distinta de entrega e sem prêmio.

**Importante.** Hoje há bilhetes de chegada e, abaixo, “Diário da viagem” com mensagens que repetem eventos; “Estações e conquistas” ainda repete mapa, progresso em “un.” e coleção. Fazer uma cronologia visual única, com chegadas como bilhetes e conquistas como registros próprios; enquanto não houver união segura de identificadores, manter “Outros acontecimentos” recolhido, sem duplicar cada chegada. Remover a ficha repetida de rota daqui; usar atalhos `Mapa` e `Coleção`.

**Essencial, vazio.** Unir “Seu primeiro bilhete de chegada aparecerá após a próxima parada” e “As próximas chegadas e conquistas aparecerão aqui” em um único bilhete ilustrado: **“Seu diário começa na próxima estação”** + destino + `Ver mapa`. No perfil temporário, dizer **“Viagem temporária · não salva neste perfil”**; importação indisponível precisa de motivo visível ao foco/inspeção.

**Importante.** Na importação, explicar antes da substituição: **“Substituir a viagem atual? Uma cópia local será preservada.”** Exibir arquivo escolhido e duas ações `Importar e substituir` / `Voltar`. Isso é uma proteção dessa ação específica; não adicionar confirmações a equipar, pausar ou salvar. Preservar todas as validações e backups existentes.

### Viagem / Serviços — `services_controls`, `ui_services`

**Essencial.** Ordem proposta: **encomenda atual/oferta → capacidade necessária → decisão → passageiros → histórico resumido**. O seletor de despacho fica compacto no cabeçalho, **“Despacho: Automático | Manual”**. Tirar o parágrafo permanente “Analise a oferta…”; deixar **“Sem prazo”** junto da oferta e explicar a automação em detalhe acessível. Mudar o modo deve responder **“Manual ativado · encomenda atual mantida”** ou **“Automático ativado · novas ofertas nas paradas”**.

**Essencial.** Na oferta de referência, transformar “11 unidades · Bosque dos Pinheiros até Vila Aurora · 110 pontos na entrega” em três linhas curtas:

```text
[encomenda] 11 un.     [pontos] 110 na entrega
Bosque dos Pinheiros → Vila Aurora
Carga: 0/20 a bordo · comporta as 11 un.
[Aceitar]   Recusar
```

`Aceitar` recebe a maior ênfase; `Recusar` é secundário, no mesmo grupo. Hoje os dois botões largos têm aparência quase equivalente. Não usar só um “X” para recusa, pois pode ser confundido com fechar o painel.

**Essencial, bloqueio.** Substituir “São necessárias 11 unidades de capacidade compatível. Ajuste os vagões na oficina.” e “Compartimento compatível 0/0” por **“Faltam 11 espaços de carga”** + **“Equipe um vagão de carga (20 un.)”** + ação **“Abrir vagões”**. Se houver capacidade parcial, mostrar déficit real; se a carga for líquida, usar tanque e capacidade de líquidos. Não apresentar `0/0` como barra vazia normal. Manter `Aceitar` indisponível e o motivo legível sem hover. O atalho deve selecionar a categoria apropriada na oficina e preservar a oferta ao voltar.

**Essencial, estados.** Separar requisito, andamento e resultado:

| Estado do domínio | Substituto visual e texto |
| --- | --- |
| `Offered` | Bilhete “Oferta · sem prazo”; `Aceitar` e `Recusar` |
| `Accepted` | Relógio + “Aguardando origem: Bosque dos Pinheiros”; se faltar espaço, motivo adicional real |
| `Loaded` | Caixote/tanque + “A bordo · entrega em Vila Aurora”; capacidade ocupada |
| `Returning` | `devolver` + “Voltando à origem: Bosque dos Pinheiros”; “11 un. continuam a bordo · sem pontos” |
| `Delivered` | Confirmação + “Entregue · +110 pontos registrados”; não repetir “110 pontos na entrega” como promessa futura |
| `Cancelled` | “Cancelada · nova oferta na próxima chegada”; sem aparência de erro ou punição |

Não fabricar barra de “progresso da encomenda” baseada no tempo: `ServiceView.progress` recebe `None` atualmente e não há medição de serviço contínuo. Mostrar etapas discretas ou uma rota verificável. Se o aceite já carregou na plataforma, a resposta deve ser **“11 un. carregadas”**, não a mensagem fixa “Carregamento na origem com espaço compatível”. Ler o estado resultante da transação.

**Essencial, devolução.** Antes da ação, manter **“O espaço será liberado ao chegar à origem. Sem prêmio nem penalidade.”** junto de `Devolver à origem`. Depois, substituir o botão pelo estado de retorno. “Solicitação registrada. Consulte o estado da encomenda acima.” vira resultado concreto: **“Devolução solicitada · destino: Bosque dos Pinheiros”**. Não sugerir descarga imediata ou desfazer se o domínio não oferece essa ação.

**Importante.** Passageiros: barra curta **“25/28 lugares”**, seguida de linhas por destino: **“Serra Clara · 5 passageiros”**. Agrupar visualmente grupos do mesmo destino, mantendo origens nos detalhes, sem mudar os grupos do domínio. “A bordo · desembarque automático” aparece uma vez. Totais de vida inteira ficam no rodapé/Diário, com `pontos` e nome; nunca como saldo de loja.

**Essencial, vazio.** Se há lugares livres: **“Passageiros embarcam na próxima parada”**. Se não há assentos: **“Sem lugares para passageiros · Abrir vagões”**. Sem contrato: **“Nova encomenda na próxima chegada”**; não usar spinner, contagem inventada ou botão `Buscar oferta`. Ao pausar, o vazio continua explicando a próxima condição, sem parecer tarefa em execução.

### Viagem / Coleção — `collection_controls`, `branch_collection`

**Essencial.** Diferenciar três famílias: **Estações**, **Excursões**, **Serviços**. Os postais de serviço precisam mostrar progresso **“8/20 passageiros entregues”** ou **“10/20 un. de carga entregues”** e o motor liberado. Não contar contratos como unidades de carga nem progresso parcial como recompensa já obtida.

**Importante.** Trocar os cartões de largura total com faixa de madeira, retângulo de cor e “Descoberta registrada” por uma grade de duas colunas em 480 px: paisagem/símbolo próprio + nome + selo. A imagem deve ser o postal, não uma grande faixa vazia com a mesma estação em todas. Descoberto: carimbo `Descoberto`; bloqueado: ilustração simplificada + cadeado + **“Visite Serra Clara”**. O selo e o nome continuam legíveis sem distinguir cores.

**Importante.** Clique abre detalhes do postal, sem botão de “coletar”. Os postais dos ramais usam a mesma estrutura e nomes reais; os de serviço mostram `Ver locomotiva` quando conquistados. Coleção vazia deve apresentar os lugares que virão e um convite **“Sua primeira lembrança vem na próxima estação”**, sem painel adicional de tutorial.

**Desejável.** Pequenas vinhetas originais baseadas nos cenários do próprio CPU Express. Os SVGs `postal`, `porto`, `montanha`, `ponte` e `tunel` são marcadores; não substituem uma ilustração de postal quando ela couber.

### Visual / Vagões — `appearance_controls`, `composition_preview`, `ui_stock::composition`

**Essencial.** O seletor “Vagão 1 / Vagão 2 / Vagão 3” deve representar a composição com miniaturas reais e engates. Preservar a orientação atual: cauda à esquerda, locomotiva à direita. Numerar posições, marcar a selecionada com moldura + ponteiro e exibir uma única frase **“Trocar vagão 1 · Panorâmico”**. A locomotiva pode ser atalho para Motores, se receber aparência e semântica de botão; hoje é apenas rótulo.

**Essencial.** Em `floating_visual_minimum.png`, instruções, totais e favoritos empurram as alternativas quase totalmente para fora. Cortar “Trocar vagão 1 · passe o mouse ou foque para conferir ocupação e categoria.”; o cabeçalho já informa a posição. “28 lugares · 20 unidades de carga · cauda à esquerda” vira passageiro **28** / caixote **20** / tanque **0**, com nomes acessíveis. A orientação passa ao desenho. “Composições favoritas” vai depois da grade ou para um botão estrela com rótulo `Favoritos`.

**Essencial.** Mostrar a capacidade individual em cada peça e comparação apenas na opção focada/inspecionada: passageiro **24 lugares**, dormitório **12**, panorâmico **16**, carga **20 un.**, tanque **12 un. líquidas**. Cortar “0 carga” em vagões de pessoas e “0 lugares” nos de carga. “Trem: 28 > 36 lugares · 20 > 20 carga” em todos os cartões vira uma ficha estável: **“Após trocar: 36 lugares (+8); carga 20 (igual)”**. Separar carga seca e líquidos; soma genérica pode esconder incompatibilidade.

**Essencial, bloqueio.** O cartão atual parece equipável e a recusa aparece no clique/tooltip. Na opção inspecionada, mostrar **“Não cabe: 25 passageiros / 24 lugares após trocar”** ou **“Líquidos a bordo: 11 / tanque após troca: 0”**. Exibir `Ver serviços` e a condição para liberar. O cartão bloqueado continua inspecionável por teclado; a ação `Equipar` fica indisponível. O domínio continua sendo a autoridade final.

**Importante.** “Conferir / equipar neste vagão” é ambíguo: um clique equipa ou apenas confere? Escolher um contrato único. Proposta: seleção/foco mostra prévia fixa; clique ou Enter equipa imediatamente quando permitido, e o cartão diz **“Equipar”**. Bloqueado diz **“Ver impedimento”**. Manter `EQUIPADO` + marca de confirmação e realçar o slot alterado. Não criar confirmação extra para uma troca reversível.

**Importante.** Favoritos em até quatro miniaturas de composição, não nomes unidos por “/”. Vazio: **“Guarde esta composição para usar depois”** + `Guardar atual`. Lotado: **“4/4 favoritos”** e remoção explícita. Composição já guardada: **“Já está nos favoritos”**. Aplicar favorito exige as mesmas verificações da troca individual.

**Desejável.** Um breve encaixe visual no slot e som opcional existente. Reduzir movimento remove deslocamento, preservando confirmação estática. Não dar bônus inventado de conforto/turismo para justificar os modelos.

### Visual / Pinturas — `appearance_controls`

**Essencial.** Preservar a grade de duas colunas e a marca `EQUIPADA`. Para bloqueadas: cadeado, nome e **“3/4 chegadas”**, com barra curta opcional. `Ver requisito` precisa funcionar por teclado e manter a ficha visível; não depender exclusivamente do aviso de seis segundos no rodapé.

**Importante.** Remover “Pinturas da locomotiva” e a dica isolada se repetirem o contexto. Dar maior área à amostra do trem. A galeria atual renderiza pinturas sobre Vapor Aurora; quando outro motor estiver equipado, essa prévia pode sugerir que trocar pintura muda a locomotiva. Preferir prévia com o motor atual, atualizada só quando a escolha relevante mudar, ou identificar explicitamente **“Amostra no Vapor Aurora”**. Não gerar texturas por frame.

**Desejável.** Foco na pintura recém-liberada, selo “Nova” e `Ver no trem`. Não substituir os nomes das pinturas apenas por bolinhas coloridas.

### Visual / Clima — `appearance_controls`

**Essencial.** Manter distinção entre **clima real**, **última condição conhecida** e **paisagem ilustrativa**. A captura é DEMO; não prova o estado conectado. Em uso normal, o cabeçalho deve mostrar cidade + `sol`/`nuvem`/`chuva` + **“22 °C · Nublado”**. Sem dados: **“Paisagem ilustrativa · aguardando clima”**. Em falha com cache: **“Último clima conhecido · recebido há 45 min”**, sem rejuvenecer horário da condição.

**Essencial.** Em “Alterar local do clima”, manter **Cidade**, **Latitude**, **Longitude** identificadas. O texto **“O nome é só um rótulo; informe as coordenadas”** deve estar visível com o formulário, não somente numa dica. Não desenhar campo de busca, ícone de GPS nem botão “Localizar” enquanto a funcionalidade não existe. A cidade atual continua visível durante edição; `Aplicar local` responde ao sucesso ou erro ao lado dos campos.

**Importante.** Vento, precipitação do modelo, horário da condição e recebimento ficam em `Detalhes do clima`. Manter o link de atribuição Open-Meteo. `Atualizar clima` indisponível mostra a razão/intervalo; erros técnicos completos vão aos detalhes, mas o estado de falha não. Usar uma pequena amostra da atmosfera já disponível, sem exigir outra renderização contínua.

**Desejável.** Ícones de condição simples e consistentes; não transformar clima num painel meteorológico ou seletor de ambientes. A rota continua decidindo o ambiente.

### Visual / Fluidez — `appearance_controls`

**Essencial.** “Volume dos eventos” em `0.000` é linguagem de parâmetro. Mostrar `mudo` **“Desligado”** em zero; nos demais valores, `som` **“35%”**. `Ouvir exemplo` precisa explicar **“Silenciado pelo modo tranquilo”** quando esse modo estiver ligado, mesmo com volume positivo. Nunca comunicar som ativo apenas pelo valor do slider.

**Importante.** Três grupos enxutos: **Animação** (`Fluido · 60 FPS` / `Econômico · 30 FPS`), **Atenção** (`Menos movimento nos menus`; `Modo tranquilo`) e **Som**. “Trabalho tranquilo: ocultar avisos sobre a paisagem” vira **“Modo tranquilo”** com descrição visível **“Sem avisos sobre a paisagem e sem sons”**. É mais fiel ao comportamento. “Apenas a animação muda; a viagem mantém o ritmo” aparece uma vez.

**Importante.** Manter `Ajuste fino` recolhido. “Resposta à CPU” recebe extremos **Suave / Rápida**, preservando valor exato nos detalhes. Não sugerir que FPS ou suavização aceleram recompensas. “Informações na janela flutuante” pode ficar aqui nesta rodada, mas com prévias pequenas de Compacto / Completo / Limpo e o nome do modo, sem adicionar mais uma aba.

**Desejável.** Uniformizar aparência de seletores com os botões do jogo. Checkbox e slider podem continuar sendo controles familiares; removê-los só por parecerem formulário prejudicaria a clareza dos ajustes.

### Visual / Motores — `appearance_controls`

**Essencial.** Manter Vapor Aurora, Diesel do Porto e Elétrica da Serra reconhecíveis. Bloqueados exibem cadeado e progresso real: **“Carga entregue 10/20 un.”** ou **“Passageiros entregues 8/20”**. O texto “Entregue 20 unidades de carga” explica a meta, mas não diz quanto falta. No desbloqueado não equipado, usar **“Equipar”**; não continuar mostrando só a condição de conquista.

**Importante.** Os cartões atuais ocupam toda a largura com um sprite pequeno centralizado. Usar linhas de garagem: sprite à esquerda, nome/estado no centro, ação à direita; em 480 px, duas linhas internas. Cabe mostrar as três locomotivas com menos rolagem. Se “Motores” ficar, manter o título interno “Locomotivas”; eventual renomeação da aba deve caber sem reduzir a fonte abaixo das demais.

**Desejável.** Pequena prévia da composição com o motor escolhido. Não adicionar números de potência, velocidade ou eficiência que não façam parte da mecânica.

### Janela — `behavior_controls`, `floating_panel`

**Essencial.** Separar **Local do trem**, **Durante o trabalho** e **Sessão**. Hoje “Posição e tamanho” também contém foco, início pausado, minimizar e bandeja. “Na área da barra de tarefas” vira escolha explícita **“Barra de tarefas | Janela flutuante”**, com dois desenhos simples do resultado. Assim, desmarcar não exige deduzir qual modo será usado.

**Essencial.** “Ocultar na bandeja” indisponível deve explicar **“Bandeja indisponível · use Minimizar”** ao lado ou ao foco, não só no hover. Preservar a diferença entre **Fechar painel**, **Minimizar**, **Ocultar** e **Salvar e sair**. Uma ação `X` na doca fecha só o caderno; não reutilizá-la para encerrar.

**Importante.** Sliders e números de posição/tamanho vão em **“Ajustar medidas”**; antes deles, instrução curta **“Arraste a faixa para mover; use as bordas para ajustar”** e `Restaurar posição e tamanho`. Manter alternativa por teclado para arrastar. Os defaults de largura automática e encaixe de altura continuam destacados; altura livre informa que cobre a área acima da barra e não reserva espaço. “Cantos arredondados” explica indisponibilidade no Windows sem suporte.

**Importante.** Agrupar **Sempre no topo**, **Pausar ao trabalhar em outra janela** e **Iniciar pausado**. Não abreviar o segundo para “Pausar fora de foco” se isso piorar compreensão. Ao marcar, mostrar uma amostra de estado: **“A viagem pausa quando você usa outra janela”**. Retorno do Windows pode falhar: confirmar posição aplicada apenas após resultado real e manter aviso do fallback para janela flutuante.

**Desejável.** Miniatura da posição na barra, sem tentar substituir os controles nativos de `DesktopController`. Configuração não precisa virar um minijogo.

### Ajuda — `help_contents` (leitura de código)

**Essencial.** “Manual do maquinista” hoje abre essencialmente uma lista de atalhos. Antes dos atalhos, três respostas: **“Como a viagem avança?”**, **“Como transportar?”**, **“Como voltar ao trem?”**. Textos curtos: tempo ativo independente da CPU; embarque/entrega automáticos; clique na faixa abre o caderno; Esc fecha apenas o painel. Manter `Ctrl+Q` identificado como saída.

**Importante.** Exibir teclas como teclas desenhadas, mas preservar seus textos e ação para leitores. Separar `O` (opções) e `F1` (ajuda); “O / F1” não é uma única tecla. Diagnóstico/DEMO ficam na parte final. “Arraste uma área livre da paisagem” precisa ser contextual ao modo atual, coerente com barra e janela flutuante.

**Desejável.** `Rever boas-vindas` como visita às instruções, sem resetar preferências nem progresso. Ajuda abre no mesmo lugar do caderno, preservando caminho de volta e foco.

### Acolhida — `welcome_controls` (leitura de código)

**Essencial.** O primeiro contato não deve começar por latitude/longitude. Proposta: prévia do trem; **“Uma pequena ferrovia para acompanhar seu dia”**; escolha **Barra de tarefas / Janela flutuante**; botão **“Começar viagem”**. Abaixo, **“Clima: São Paulo, SP · Alterar”**, recolhido por padrão. Não fingir localização detectada: São Paulo é o padrão declarado.

**Essencial.** No código, a acolhida retorna antes da área rolável comum de `floating_panel`. Com campos abertos, mensagens de validação ou escala maior, o botão final pode ficar inacessível. Garantir rolagem própria e ação alcançável em 480 × 480. Campo de cidade precisa de rótulo persistente, não só o conteúdo atual ou “Clima da sua cidade”.

**Importante.** Após começar, orientar uma única ação: **“Clique no trem para abrir o caderno”**. Reforço curto **“A viagem pausa quando o trem está pausado, oculto ou minimizado; não há progresso offline”** na ajuda contextual. Não pedir nome de empresa, login, permissão nova ou tutorial longo. Se uma coordenada editada for inválida, mostrar o campo e a correção; não bloquear o início por uma consulta de clima ainda em andamento.

**Desejável.** Uma animação curta de partida, respeitando redução de movimento e início pausado. Nunca avançar viagem ou conceder uma chegada para encenar o tutorial.

## Chegada, entrega e recompensa: retorno visual

**Essencial.** Separar feedback persistente de aviso passageiro. Hoje `travel_notice` junta mensagens em uma string por sete segundos; `equipment_notice` tem precedência; o rodapé reserva altura fixa. Uma chegada com conquista pode quebrar linhas e disputar espaço com a doca. Reservar uma linha de resumo estável, por exemplo **“Bosque dos Pinheiros · +116 pontos”**, com `Ver bilhete`. Detalhes completos permanecem no Diário. Erros de gravação não devem sumir atrás de uma frase positiva.

**Importante.** Ao chegar: marca de estação + nome, atualização dos números e bilhete. Ao entregar: quantidade entregue + pontos **já registrados**. Ao desbloquear: miniatura da pintura/motor/postal + **“Conquistado”** + `Ver na coleção`/`Ver na oficina`. O sinal de recompensa não precisa abrir modal ou roubar foco. Reabrir o menu não repete a concessão.

**Essencial.** Estados precisam ser honestos: aceitar pode carregar imediatamente; devolver conserva ocupação; cancelado não é falha; “salvo” depende do resultado de escrita; pausa congela tempo ativo. O `World` aplica transações antes das notificações. Esta revisão não autoriza mover recompensas para widgets, duplicar relógios ou sintetizar progresso visual.

**Desejável.** Realce de 150–250 ms na área alterada, selo estático em movimento reduzido, som curto opcional já existente. Modo tranquilo mantém os registros, mas não produz aviso sobre a paisagem nem som. Nada de piscar continuamente para pedir atenção.

## Janela mínima: composição para 480 × 480

**Evidência:** em `floating_contract_minimum.png` o cartão de oferta só começa a aparecer no fim da rolagem; em `floating_visual_minimum.png` aparecem apenas os topos das peças; em `floating_branch_minimum.png` o mapa empurra a fase/tempo para baixo. As variantes `*_scrolled` demonstram que o conteúdo existe, mas contexto e ação ficam separados. A doca permanece acessível: preservar isso.

**Essencial.** Projetar a primeira área visível para uma decisão completa. Proposta de orçamento **em pontos lógicos de conteúdo**, a ajustar à moldura e ao DPI real: 36–42 de faixa, 36–40 de subabas quando houver, cerca de 260–290 para conteúdo, 20–24 de status e 44–48 de doca, mais margens. Não prometer equivalência entre pontos egui e pixels físicos em DPI fracionário. Não descontar a barra de título nativa duas vezes.

- Serviços: despacho em uma linha; oferta com origem/destino, quantidade, pontos, capacidade e `Aceitar` visíveis juntos. Passageiros ficam depois.
- Vagões: composição curta; capacidade resumida; primeira linha de duas opções inteiras, com requisito. Demais peças por rolagem vertical.
- Ramal: mapa compacto + fase atual + tempo; circuito recolhido. As três fases podem ser marcadores, não três barras completas.
- Trem: destino/estado + prévia pequena + pausa/retomada. Métricas e última chegada continuam acessíveis abaixo.
- Janela/Clima: um grupo expandido por vez, sem ocultar motivo de falha. Acolhida tem rolagem própria.

**Essencial.** Fixar a subnavegação ou mostrar um cabeçalho compacto com a página durante rolagem, sem gastar outra faixa larga. Preservar posição de rolagem por página e trazer controle focado à vista; isso já tem suporte no código. Não fixar todos os cabeçalhos ao mesmo tempo. Reservar espaço da barra de rolagem; nenhuma rolagem horizontal.

**Importante.** Reavaliar o degradê escuro no limite do `ScrollArea`: nas capturas ele encobre texto/botões que começam no corte. Usar indicação de continuação discreta e testar legibilidade na borda. Nunca colocar a ação principal parcialmente sob esse efeito.

**Critérios de aceite para a implementação, ainda não executados:** em 480 × 480 e tamanhos maiores, sem cortar rótulos; toda oferta pode ser entendida e aceita sem rolar duas regiões diferentes; todo bloqueio pode ser lido por teclado; doca e fechar permanecem visíveis; notificações longas não empurram a doca; ajuste de fonte/DPI não oculta `Começar viagem`. Verificar a mesma matriz a 100%, 125%, 150% e 200% em Windows real.

## Acessibilidade e riscos específicos

1. **Dicas inacessíveis pelo teclado.** `game::hint` aloca 18 × 18 com `Sense::hover()` e `on_hover_text`. Transformar em botão de ajuda focável com nome **“Sobre despacho automático”**, por exemplo, e abrir explicação no foco/Enter ou clique. Aumentar área interativa para pelo menos 28–32 pontos sem ampliar necessariamente o desenho. Regras indispensáveis ficam visíveis no conteúdo.
2. **Desenho de letra não equivale a texto semântico.** `button` e `dock_button` preservam `painter.text` (transparente quando o lettering pixelado o substitui) e `WidgetInfo`. Manter essa estratégia. `ribbon`, `section_frame` e `pixel_label` expõem `WidgetInfo`, mas desenham o título com malha; preservar também uma forma de texto com o conteúdo original quando isso for necessário aos consumidores/testes, e não apenas retângulos. Uma forma de texto transparente, sozinha, não garante leitura assistiva: testar a árvore de acessibilidade efetiva com o backend habilitado e Narrador/NVDA. Evitar anúncio duplicado do mesmo rótulo.
3. **Ícones não carregam rótulos.** Os SVGs entregues contêm apenas rect, por contrato. O chamador fornece nome, papel, valor e estado; não tentar inserir `<text>` ou `<title>` nos sprites. CPU `—` deve anunciar “Processador indisponível”; cadeado deve anunciar requisito; barra `7/28` deve anunciar “7 passageiros de 28 lugares”.
4. **Foco distinto de seleção.** `dock_button` desenha foco no ramo `else if` da seleção: o item já selecionado pode não ganhar uma marca de foco independente. Desenhar anel de foco também sobre o selecionado. Aplique a mesma distinção a estações, peças e subabas. O contorno dourado de seleção não basta como foco em todos os fundos.
5. **Navegação e atalhos.** Manter Tab/Shift+Tab e Enter/Espaço nos controles; não disparar pausa global ao ativar um botão com Espaço. Campos de cidade, coordenadas e valores numéricos precisam reter teclas de edição. Validar retorno de foco após tooltip expandida, importação, mudança de página e fechamento do caderno. Atalho nunca deve depender de capturar teclas de outro aplicativo.
6. **Cor e contraste.** Cor acompanha forma, palavra e posição. Bloqueado: cadeado + requisito; equipado: marca + “Equipado”; erro: símbolo + texto; pausa: ícone + palavra. Texto creme sobre barras azul/verde, texto secundário marrom sobre papel e foco dourado sobre botão manteiga exigem medição de contraste na implementação. Não assumir conformidade pela paleta. A legenda pode ficar fora da barra para não atravessar preenchimentos diferentes.
7. **Tamanho e escala.** Não resolver falta de espaço reduzindo texto informativo a 10–11 pontos. Reservar a fonte pixelada para funções em que ela é legível; manter acentos portugueses. Escala dos sprites em múltiplos inteiros físicos e nearest; área de clique pode ser maior que o sprite de 16 px. Verificar recorte e origem alinhada em DPI fracionário.
8. **Atualizações sem ruído.** Não anunciar ETA, CPU e RAM a cada frame. Anunciar transições relevantes, preservar foco e oferecer registro persistente. Avisos passageiros não são a única forma de saber que algo aconteceu. “Menos movimento” desliga deslocamentos decorativos, mantendo mudança de estado imediata.
9. **Bloqueio inspecionável.** `ui_stock` já diferencia `Inspect` e `Equip`; preservar esse contrato na integração. A pessoa precisa conhecer o requisito mesmo quando não pode executar a ação. Tooltip preto de `floating_visual_minimum_scrolled.png` cobre parte da grade: preferir ficha de inspeção em área estável e permitir fechá-la com teclado.

## O que não mudar

- A fonte pixelada autoral, os acentos, a madeira, o pergaminho e a faixa verde. Ajustar hierarquia e consistência antes de redesenhar identidade.
- A doca persistente com ícone e nome; o marcador de item ativo; Esc fechando só o painel. Manter saída explícita em Sessão.
- As miniaturas geradas pelo renderer do próprio jogo, com nearest, rodas e engates completos; a galeria em cache. Os novos ícones são símbolos de UI, não substitutos dessas miniaturas.
- A viagem tranquila: sem prazo de contrato, cobrança por ausência, lucro offline, moeda de loja ou obrigação de resgatar recompensa.
- Capacidade real, distinção entre carga e líquidos, motivos de troca recusada, despacho automático/manual e devolução com carga a bordo.
- `World` como dono de serviços e recompensas; tempo ativo único; independência entre CPU visual e progresso lógico; rota como origem do ambiente; métricas em cache e parciais válidas.
- Salvamento protegido, migração, recuperação e backups; configuração separada de checkpoint; compatibilidade aurora-v1 e excursões separadas. Mudança visual não é motivo para um novo formato persistido.
- Win32/bandeja em `platform` e visibilidade/topo sob `DesktopController`. Não retomar encaixe como filha do Explorer.
- Conteúdo técnico útil em detalhes/diagnóstico e controles convencionais nos ajustes. Não trocar precisão por metáforas que escondam o efeito de uma opção.

## Entrega e limites desta rodada

Entregues este relatório, 43 SVGs originais em `assets/icons/`, documentação, folha PNG de conferência e validador local. A folha mostra cada símbolo em 48 px sobre pergaminho e madeira, mais amostras em 16 e 32 px. As referências foram observadas para linguagem visual, sem extração de recursos. Validação executada nesta rodada: 43/43 SVGs aprovados, 2.298 retângulos inteiros sem sobreposição, máximo de seis cores por ícone; transparência e contorno externo conferidos. A folha final também foi inspecionada visualmente.

Não foram alterados `src/`, `tests/`, `data/`, Cargo ou arquivos de distribuição; não houve commit, remoção ou movimentação de arquivos. Cargo, Clippy, formatação Rust, testes do aplicativo e execução nativa não foram realizados, conforme a restrição de trabalho simultâneo. A validação feita aqui é dos SVGs e da folha visual. Integração no renderer, leitor de tela, teclado real, DPI e comportamento de janelas continuam sendo verificações da implementação posterior.
