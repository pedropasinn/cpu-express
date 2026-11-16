# Revisão de UI/UX das telas: CPU Express (E5 + E6)

Base: as imagens em `docs/expansao/telas/` (704×676, headless). Abri todas as páginas distintas da E5 (acolhida, ajuda, arquivo, clima 0/1, colecao 0/1, diario 0/1, favoritos, fluidez 0/1/2, janela 0/1, janela-livre, loja, mapa, missoes 0/3/vazio, motores 0/1, pinturas 0, ramais, ramais-ativo, servicos, servicos-oferta/bloqueado/devolucao, sessao, trem, vagoes 0/4) e da E6 (loja, loja-inicial, abertura 0/1, revelacao 0..3, repetida 0/4, chances 0, garantia 0, direta 0/10, colecao 0/10/16, moedas, museu, maquina, compra-concluida).

Áreas de referência na janela (medidas nas imagens): a faixa de título ocupa y 27–68, a 1ª linha de abas y 76–106, a 2ª linha de abas y 113–143, a linha de status y ≈580 e a doca y 598–648. A área útil de conteúdo vai de x 27 a 677 (650 px) e de y ≈150 a 570 (≈420 px com duas linhas de abas, ≈460 com uma).

**Atenção:** as exportações `floating_e5_trem_0` e `floating_e6_moedas_0` ainda mostram a cabine antiga (botão RETOMAR solto, linha "CPU DEMO · CPU 50.0% · Memória sem leitura…", caixa "Serviços · 33/48…"). A cabine que o Pedro elogiou (cartão "Próxima parada", botão PAUSAR grande, medidores com sprite) **não aparece nas imagens**. Antes de tudo, reexportar as telas a partir do código atual. As observações sobre a Cabine abaixo valem para o que está exportado.

---

## 1. Os 10 problemas que mais tiram a cara de jogo

1. **Botões egui brancos de formulário em quase todas as telas novas.** São retângulos creme com borda fina e texto sans 14 px: "Equipar/Continuar", "Abrir", "i", "Recolher/Trocar", "Comprar · 120 réis", "Ver no trem", "Ver compra direta", "Ver peças do trem", "< Remessas", "Equipar/Esquecer", "Guardar composição atual", "Exportar/Importar viagem…", "Salvar agora", "Começar viagem", "Usar São Paulo, SP", "Aplicar local", "Minimizar/Ocultar na bandeja", "Seguir (Espaço)/Reposicionar (R)", "Rever boas-vindas", "Ouvir exemplo", "Restaurar posição e tamanho", "Comprar selo nº 2", além do paginador "<" e ">". Nesta pasta, esse é o principal motivo de "site".
   **Regra:** toda ação clicável que não seja campo de ajuste usa `game::button` (mínimo 150×40; ação principal 190×48 em teal/selected; secundária em latão), com ícone, centralizada no bloco a que pertence.

2. **Metade inferior vazia e conteúdo grudado no topo à esquerda.** Isso acontece em Arquivo, Clima, Fluidez, Janela, Máquina, Loja E5, Abertura, Revelação, Compra concluída, Museu e Favoritos: de 150 a 380 px vazios entre o conteúdo e a doca.
   **Regra:** cada tela distribui o conteúdo na altura útil inteira. Peças de recompensa ficam centralizadas na vertical, grades crescem até preencher, e telas de ajuste usam cartões `section_frame` que ocupam a largura toda.

3. **Linhas de texto com "·" fazendo papel de interface.** Exemplos: "Comum 70.0% · Incomum 22.0% · Raro 7.0% · Épico 1.0%", "+20 embarcaram · −0 desembarcaram · +0 pontos", "3 / 19 modelos possuídos · posse permite repetir no trem", "CPU DEMO · CPU 50.0% · Memória sem leitura · 56 km/h visuais".
   **Regra:** número com ícone ou barra; nunca uma frase com separadores. No máximo uma linha de texto corrido por cartão.

4. **Texto de aviso, legal ou técnico dentro do jogo.** Exemplos: "Demonstração: compras exigem um perfil salvo", "Sem dinheiro real, anúncios ou ofertas que expiram", "As leituras da máquina não saem do computador", "O clima consulta o serviço de meteorologia", "Dados: Open-Meteo · CC BY 4.0", "Resultado salvo · fechar e reabrir mantém esta peça", "Peças elegíveis da mesma raridade têm chances iguais. Raridade não dá bônus", "Sem ler teclas…", e a linha fixa "Viagem pausada · progresso preservado" acima da doca.
   **Regra:** créditos e privacidade vão para um único cartão "Sobre" em Ajustes; o resto sai ou vira `game::hint` (o (i) com tooltip).

5. **Navegação de três níveis com abas de site.** Doca, depois abas creme 124×30, depois sub-abas creme 210×30. A aba selecionada fica verde-acinzentada clara (#c4d3b8) e parece *desabilitada*, não escolhida. Em Ajustes > Fluidez chega a haver um 4º nível ("Compacto / Completo / Limpo").
   **Regra:** as abas viram `game::button` com ícone, em latão quando não selecionadas e teal quando selecionadas, com 36 px de altura. Sub-abas só onde não há outra saída; em Ajustes, trocar as sub-abas por seções empilhadas na mesma página.

6. **Moeda sem identidade.** O mesmo recurso aparece como "réis", "moedas" ("Repetida · +30 moedas", "+84 moedas") e "pontos" ("+92 pontos", "1604 pontos de serviço"). Na E5 o cabeçalho mostra "89 réis" sem ícone.
   **Regra:** um único nome ("réis") e sempre o sprite da moeda + número em `pixel_label` (ex.: ●225). "Pontos" só existe se for outro recurso, e então ganha ícone próprio e explicação.

7. **Raridade comunicada só pela cor de uma palavra e por uma borda de 2 px.** As cores de Comum (cinza-marrom) e Incomum (teal) se confundem, e Raro (azul petróleo) também é próximo do teal. Em Motores, Vagões e Pinturas o rótulo de raridade fica *fora e abaixo* do cartão, flutuando entre as linhas da grade.
   **Regra:** cada raridade tem cor de moldura de 4 px + faixa no topo do cartão com o nome em pixel (COMUM cinza #8a7a66, INCOMUM verde #3f8f4f, RARO azul #3a6fb8, ÉPICO roxo #7b46b0 com brilho dourado), sempre dentro do cartão.

8. **Paginação de site e grades pequenas demais.** O paginador "< Página 1 / 5 >" aparece em Vagões (5 páginas de 4), Motores, Pinturas, Coleção, Compra direta, Chances, Diário (11 páginas de 3 entradas) e Missões > Marcos. Cartões de 322×115 com o sprite em 120×60 desperdiçam área.
   **Regra:** grade de 3 colunas × 2 linhas (206×190 cada, sprite 160×80) e paginador com setas `game::button` de 40×40 nas laterais da grade mais pontos (●○○) no lugar de "Página 1 / 5".

9. **Títulos de seção em dois sistemas.** Alguns são pixelados e bons ("ENCOMENDA", "PASSAGEIROS", "EXCURSÃO EM ANDAMENTO"); outros são sans 20 px de documento ("Pátio geral", "Postais de viagem", "Remessas da ferrovia", "Sala de máquinas", "Selos do museu", "Uma pequena ferrovia…", "Compra direta concluída", "Vila Aurora").
   **Regra:** todo título de seção é `pixel_label` ou `section_frame` (cell 2, caixa alta); não existe heading sans.

10. **Ícones (i) órfãos, glifos quebrados e números sem tratamento.**
    - (i) soltos sozinhos numa linha: Arquivo y 245, Janela y 322 e 379, Sessão y 298, Ramais-ativo y 400.
    - Glifo tofu "Porto do Sol ▯ Vila Aurora" na barra do Mapa.
    - "−0 desembarcaram", "+0 pontos", "0.000" de volume, "1.8" de resposta, "70.0%", "0.0%", "em até 1 aberturas".

    **Regra:** (i) sempre colado ao título que explica, números formatados para jogador (inteiros, sem "−0"/"+0", sem casas decimais inúteis) e só glifos que a fonte tem (seta desenhada como `Icon` ou "→" verificado).

---

## 2. Por tela

### E5 — Seu Trem

**floating_e5_trem_0.png (Cabine, versão exportada)**
- Bom: prévia do trem com paisagem dentro da moldura escura dá contexto imediato.
1. Reexportar: a imagem não mostra o cartão "Próxima parada" nem o PAUSAR que o Pedro aprovou.
2. A prévia ocupa só 320×70 dentro de uma moldura de 650×84. Escalar o sprite para preencher a largura (≈640×140) ou centralizar com fundo de céu contínuo, sem faixas marrons laterais de 160 px.
3. "RETOMAR" (170×34, colado à esquerda) passa a ter 260×52, centralizado, dentro do cartão da próxima parada.
4. Remover a linha "CPU DEMO · CPU 50.0% · Memória sem leitura · 56 km/h visuais" e substituir por 3 medidores com sprite (CPU, Memória, Velocidade), cada um 200×56 com `progress` e número em pixel.
5. A caixa "Serviços · 33/48 passageiros · 11/20 carga" (botão branco) vira dois contadores com ícone: [passageiro] 33/48 e [carga] 11/20, cada um como mini barra de 140×14.
6. A barra "Relógio parado · 65 s ativos" tem o rótulo centralizado cortado pelo limite do preenchimento, e o texto "Relógio parado" fica sobre duas cores. Usar texto fora da barra (à direita) ou barra de 20 px com contorno no texto.
7. "Chegada 35" e "Passageiros +20 embarcaram / -0 desembarcaram" viram: ícone de estação + "Porto do Sol" em pixel_label, e abaixo [passageiro]+20 e [passageiro saindo]0 (ocultar quando 0).
8. Sobram ≈90 px vazios entre o cartão e a linha de status; o cartão da próxima parada deve crescer até ≈170 px de altura.

**floating_e5_vagoes_0.png / _4**
- Bom: sprites dos vagões e o cadeado de não possuído funcionam como objeto.
1. Trocar "Equipar na posição: Vagão 1 Vagão 2 Vagão 3" (texto selecionável) por uma tira com o trem atual: 3 slots de 120×60 com o sprite do vagão equipado; o slot ativo fica com moldura dourada.
2. Remover "Trem: 48 lugares · 20 espaços de carga" e mostrar ícones [passageiro] 48 [carga] 20 à direita da tira de slots.
3. Mover "Comum 👤24 lugares" de baixo do cartão para *dentro* do cartão (faixa de raridade no topo, capacidade com ícone embaixo do sprite). Hoje esses rótulos ficam entre as linhas e desalinhados com o cartão.
4. "Equipado · possuído", "Não possuído · obtenha na loja" e "Possuído · equipar" viram estados visuais: equipado = selo ✓ verde no canto; não possuído = sprite em silhueta escura 50% + cadeado + preço com moeda; possuído = cartão clicável com hover.
5. Grade 3×2 (6 por página): 19 vagões em 4 páginas em vez de 5. Em vagoes_4 sobra uma célula vazia de 322×115 e 80 px abaixo.
6. Ícone de canto (trenzinho/cadeado em 12 px) aumentar para 20×20.

**floating_e5_motores_0.png / _1**
- Bom: locomotivas com paisagem própria, coleção atraente.
1. Remover a frase "Locomotivas · aparência, sem bônus de velocidade" (vai para hint no título).
2. Mesmas correções 3–6 de Vagões. "Comum/Incomum/Raro/Épico" sai de baixo dos cartões (y 303/458), onde parece título da linha seguinte.
3. Sprite 120×60 → 180×90; o cartão fica 206×190 em grade 3×2.

**floating_e5_pinturas_0.png**
- Bom: a mesma locomotiva com cores diferentes deixa clara a função.
1. Correções idênticas a Motores. Acrescentar uma amostra de cor (quadrado 16×16) ao lado do nome para a diferença ser visível mesmo em sprite pequeno.
2. Remover "Pinturas · esmalte da locomotiva".

**floating_e5_favoritos_0.png**
- Bom: a ideia de salvar composições é boa.
1. A tela é uma lista de texto com quebra irregular ("Gôndola de / minério", "Guindaste de / socorro"); os botões Equipar/Esquecer ficam desalinhados em x 440/422/445/433.
2. Refazer como 4 cartões de 650×80, cada um com a mini-composição em sprites (3 vagões de 80×40 lado a lado), `game::button` "EQUIPAR" 130×40 à direita e um botão de lixeira 40×40.
3. "Guardar composição atual" vira `game::button` 260×44 com ícone Save no topo, ao lado de "1/4" (slots usados).
4. Remover "Até quatro composições; a ocupação atual precisa caber." e mostrar os 4 slots (cheios ou vazios tracejados), o que já comunica o limite.
5. Sobram ≈260 px vazios abaixo; os 4 cartões de 80 px resolvem.

**floating_e6_maquina_0.png (Sala de máquinas)**
- Bom: nada; é um placeholder.
1. Remover a aba "Máquina" até haver medidores. Hoje são 3 frases e 380 px vazios, e a aba força 6 abas apertadas (106 px cada).
2. Se ficar: um cartão central 400×200 com sprite de engrenagem/caldeira 96×96 e "EM OBRAS" em pixel; nada de "As leituras da máquina não saem do computador".

### E5 — Viagem

**floating_e5_mapa_0.png**
- Bom: o circuito com estações e o marcador do trem é exatamente o tipo de objeto que o jogo precisa.
1. Corrigir o glifo tofu na barra: "Porto do Sol ▯ Vila Aurora" deve usar seta desenhada ou "→".
2. Os nomes das estações se sobrepõem às linhas: "Vila Aurora" é cortado pela linha vertical em x 157 e "Bosque dos Pinheiros" pela linha em x 501. Posicionar os rótulos para fora do retângulo (à esquerda dos nós da esquerda, à direita dos da direita).
3. O marcador do trem (círculo 16 px) vira o sprite do trem 32×20 sobre a linha.
4. O mapa pode crescer de 650×200 para 650×300; abaixo dele sobram 170 px vazios.
5. "35 chegadas · 8 circuitos · relógio parado" vira três contadores com ícone em pixel. Remover "Ramais: visitas de exploração; os serviços do circuito aguardam o retorno." e "Circuito Aurora · linha fictícia".

**floating_e5_missoes_0.png / _3 / -vazio_0**
- Bom: barras de progresso e o +10 réis dão um alvo claro.
1. "Recolher" e "Trocar" (botões brancos 72×26) viram `game::button`: "RECOLHER" teal 150×40 com ícone de moeda, pulsando quando pronto; "Trocar" vira ícone ↻ 40×40.
2. O "+10 réis" vira sprite da moeda + "10" em pixel cell 2, à direita do título da ordem.
3. Remover o paginador "Página 1 / 1" das ordens (só há 3 e nunca pagina).
4. "Sem prazo · uma nova a cada 10 min ativos, na chegada" sai e vira hint no título "ORDENS DO DIA" (pixel_label).
5. Marcos: as barras têm larguras diferentes (94, 78, 94, 120, 156 px) e os botões Recolher ficam em x diferentes em cada página. Fixar as colunas: nome 230 px, barra 200 px, recompensa 70 px, botão 130 px.
6. "Próxima recompensa · Suprimentos · +20 réis" (caixa branca solta) vira um cartão com sprite da recompensa 40×40.
7. Em missoes-vazio, os 3 cartões repetem um parágrafo justificado de 4 linhas ("Continue a viagem. A próxima chegada…") com espaçamento ruim. Mostrar um relógio/apito 48×48 + "EM BREVE" e apenas uma linha curta no cartão.
8. O espaço entre "Ordens do dia" e os cartões é usado por um paginador inútil; os cartões podem subir 30 px e ganhar altura.

**floating_e5_diario_0.png / _1**
- Bom: o histórico por estação faz sentido.
1. As entradas são caixas creme de largura variável (330–380 px) encostadas à esquerda, e sobra metade direita vazia. Usar linhas de 650×56 com largura fixa.
2. Cada entrada: ícone de estação + nome em pixel, depois [passageiro↑]+20 [passageiro↓]13 [moeda]+26 [carga]11 em colunas fixas. Remover "−0" e "+0 pontos" (ocultar zeros).
3. 11 páginas de 3 entradas é demais: mostrar 7 por página.
4. "pontos" ≠ "réis"? Definir (ver problema 6).

**floating_e5_arquivo_0.png**
- Bom: —
1. O (i) solto em x 35, y 245 deve ficar ao lado do título.
2. Tela de backup não é tela de jogo: mover para Ajustes > Sessão e remover a sub-aba "Arquivo" do Diário.
3. "Salvar agora", "Exportar viagem…" e "Importar viagem…" viram `game::button` 200×44 com ícone Save, numa coluna centralizada.
4. Há 320 px vazios.

**floating_e5_servicos_0.png / -oferta / -bloqueado / -devolucao**
- Bom: **a melhor tela do pacote junto com a Cabine**: cabeçalhos pixelados, número grande "11 UN.", botões ACEITAR/RECUSAR com peso, passageiros com sprite e número em pixel.
1. "ENCOMENDA" encosta na moldura (x 27) enquanto os cartões têm margem; alinhar com o mesmo recuo do cartão ou colocar dentro de `section_frame`.
2. Os botões ACEITAR (190×38) e RECUSAR (130×34) têm alturas diferentes. Igualar a 190×44 e 150×44.
3. "DEVOLVER À ORIGEM" (240×32) mais a frase "O espaço é liberado ao chegar à origem. Sem prêmio nem penalidade." Remover a frase (vira hint) e deixar o botão com 44 px de altura.
4. A barra "Carga a bordo 11/20": o rótulo fica sobre a junção das cores ("1|1/20"). Mover o número para fora da barra ou adicionar contorno.
5. A linha de rodapé "372 entregues · 10 encomendas · 1604 pontos de serviço" em 11 px fica ilegível. Fazer 3 contadores em pixel cell 2 com ícones 20 px.
6. Em -bloqueado, "Faltam 11 espaços de carga" (vermelho) + "Equipe um vagão de carga (20 un.) ou correio (16 un.)" vira um único aviso com ícone de vagão de carga e "+11 [carga]". "ABRIR VAGÕES" está ótimo.
7. Em -devolucao, o ícone do texto "Voltando à origem…" fica deslocado 60 px à direita do bloco (x 103 vs 101 do texto acima); alinhar à esquerda do texto de rota.
8. O seletor "Despacho AUTO / MANUAL" está bom; remover a palavra "Despacho" ou usar ícone.

**floating_e5_ramais_0.png**
- Bom: os botões MIRANTE DA SERRA / CAIS DAS CONCHAS em latão com letra pixelada.
1. O collapsing header "▾ Explorar ramais" é widget de site; remover e mostrar os ramais direto.
2. Os botões ocupam 632×34, largos e baixos. Transformar em 2 cartões lado a lado de 320×220 com mini-mapa/sprite da paisagem do destino, duração com ícone de relógio "3:08" e `game::button` "PARTIR" 180×44.
3. Remover "Agende uma visita de 3 min 08 s de tempo ativo. Partida de Vila Aurora, retorno ao mesmo ponto."
4. Há 330 px vazios.

**floating_e5_ramais-ativo_0.png**
- Bom: cartão "EXCURSÃO EM ANDAMENTO" com cabeçalho de madeira e o trem no trilho.
1. O diagrama mostra "Ida > < Volta" como texto solto no meio e os rótulos "Aurora/Destino" em fonte 11 px; usar setas desenhadas nas próprias linhas e nomes reais ("Mirante da Serra").
2. Três barras empilhadas (Ida/Visita/Volta) mais a linha "Pausada · Retorno em 113 s ativos · total 40%" repetem a mesma informação. Deixar uma barra segmentada em 3 cores com o ícone da etapa atual e um relógio grande "1:53" em pixel.
3. Remover a breadcrumb "Vila Aurora > Mirante da Serra > Vila Aurora" (o diagrama já mostra) e o (i) solto em y 400.

### E5 — Coleção (postais)

**floating_e5_colecao_0.png / _1 · floating_e6_colecao_16.png**
- Bom: cadeado para não descoberto.
1. Todos os postais usam o mesmo ícone de casinha 24×24 num cartão de 322×125: não parecem postais. Usar arte de paisagem da estação (a mesma da prévia da cabine, recortada) em 180×100 com moldura branca de postal e carimbo.
2. "Conquistado" em texto vira carimbo/selo verde no canto. "Ainda por descobrir" vira silhueta escura do postal + cadeado.
3. "6 / 8 postais conquistados" vira contador grande em pixel com ícone de postal; 8 postais cabem numa página só em grade 4×2 (155×190), sem paginador.
4. "Ver peças do trem" (botão branco) sai, porque na E6 já existem as abas Vagões/Motores/Pinturas.

### E5 — Ajustes

**floating_e5_janela_0.png / _1 / janela-livre_0 / janela-flutuante_0**
- Bom: controles funcionais.
1. Checkboxes e sliders egui crus. Agrupar em 2 cartões `section_frame` ("POSIÇÃO", "DURANTE O TRABALHO") em vez de sub-abas, ocupando 650 px de largura.
2. Os (i) soltos em y 322 e y 379 vão para o lado dos títulos.
3. "Restaurar posição e tamanho" e "Minimizar" viram `game::button` 220×40.
4. Remover a frase "A coleta continua durante a pausa; a paisagem fica congelada." (fonte 11 px, ilegível).
5. "Ocultar na bandeja" desabilitado sem explicação: remover se não funciona.

**floating_e5_clima_0.png / _1**
- Bom: —
1. clima_0 é só texto ("Atmosfera ilustrativa…", "Demonstração: sem consulta à internet."). Mostrar um ícone grande de clima 64×64 (sol/chuva) + temperatura em pixel cell 3 + cidade.
2. clima_1: os campos Latitude/Longitude ficam expostos para quem joga. Esconder atrás de "Avançado" e deixar só a cidade e o `game::button` "APLICAR" 160×40.
3. "Usar São Paulo, SP" e "Aplicar local" são dois botões brancos empilhados de larguras diferentes. Unificar.
4. "Dados: Open-Meteo · CC BY 4.0" vai para o cartão Sobre.
5. Juntar Condições + Alterar cidade numa página (sem sub-abas).

**floating_e5_fluidez_0 / _1 / _2**
- Bom: —
1. "Limite de FPS" e "Resposta visual à CPU 1.8" são parâmetros técnicos. Trocar por presets com `game::button`: "SUAVE / EQUILIBRADO / ECONÔMICO".
2. "Volume dos eventos 0.000": mostrar 0–100 inteiro com ícone de alto-falante; "Ouvir exemplo" vira botão ▶ 40×40 ao lado.
3. fluidez_2: a página tem um seletor e um checkbox, com 370 px vazios. Fundir as 3 sub-abas numa só página com 3 cartões.
4. O checkbox "Permitir atividade agregada do teclado e mouse" aparece aqui **e** na Acolhida, com texto de privacidade em 11 px. Deixar só aqui, com hint.

**floating_e5_sessao_0.png**
- Bom: SALVAR E SAIR em latão pixelado.
1. "DEMO · CPU simulada; RAM indisponível" em 11 px, o seletor "CPU: 0% 50% 100%" + slider + campo "50.0 %" (três controles para um valor) e a frase "0 / 50 / 100%: alvos 12 / 56 / 100 km/h nos padrões." são ferramentas de dev. Esconder sob modo DEMO ou trocar por 3 `game::button` 80×40.
2. Seguir/Reposicionar/Exportar/Importar/Rever boas-vindas: 5 botões brancos de larguras diferentes. Fazer uma grade 2×3 de `game::button` 200×44 com ícone.
3. SALVAR E SAIR (200×34, à esquerda) vai para baixo, centralizado, com 260×48.
4. O (i) solto em y 298.

**floating_e5_ajuda_0.png**
- Bom: a tabela de atalhos é a parte mais legível.
1. A tela é um manual: 4 frases de topo, 7 atalhos, 3 parágrafos de rodapé em 11–14 px. Mantê-la só com os atalhos, com as teclas desenhadas como "tecla" (retângulo 3D 48×28 com letra em pixel) e o verbo ao lado.
2. Os passos 1–3 do topo viram 3 cartões ilustrados (ver seção 4).
3. Remover "Simular CPU não aquece o processador de propósito. DEMO não lê CPU nem RAM reais…", "As leituras da máquina…", "Arraste uma área livre da paisagem…" (este último vira um atalho "Arrastar" com ícone de mão).
4. Na Ajuda, a doca não destaca nenhum botão; manter o ? do cabeçalho em estado ativo.

**floating_e5_acolhida_0.png**
- Bom: a ilustração do trem chegando à estação é um ótimo abre-alas.
1. É um formulário: radio buttons, checkbox de privacidade, 8 linhas de texto e "Começar viagem" como botão branco 125×28. Trocar por: ilustração maior (640×180), título "BEM-VINDO A BORDO" em display_text, dois cartões grandes clicáveis com sprite para escolher o modo (Barra de tarefas | Janela flutuante, 300×120 cada) e `game::button` "COMEÇAR VIAGEM" 280×56 teal centralizado.
2. Remover: "Clima: São Paulo, SP, Brasil. Altere em Ajustes > Clima.", "As leituras da máquina não saem do computador.", checkbox de atividade + "Sem ler teclas…", "O clima consulta o serviço de meteorologia. Não há progresso offline."
3. Os passos 2 e 3 viram 2 ícones com legenda curta ("Viaje e ganhe réis", "Clique no trem para abrir").
4. Na primeira abertura, esconder a doca ou desabilitá-la; o único caminho deve ser COMEÇAR.

### E5 — Loja antiga

**floating_e5_loja_0.png**
- Substituída pela E6; garantir que não é mais alcançável. Lista de preços em texto puro e "Novas remessas chegam em breve".

### E6 — Loja

**floating_e6_loja_0.png / loja-inicial_0.png (Remessas)**
- Bom: cada remessa tem sprite próprio (caixote, maleta, baú).
1. Os cartões têm larguras diferentes (Pátio 422 px, os outros 430 px) e deixam 220 px vazios à direita. Ver a proposta da seção 3: grade 3+2 de cartões 206×200.
2. O botão "Abrir" (branco, 50×26) é o elemento mais importante da tela e o menor. Usar `game::button` 180×44 com moeda e preço: "● 225". Pátio geral: "GRÁTIS" em teal.
3. Remover a linha "Comum 70.0% · Incomum 22.0% · Raro 7.0% · Épico 1.0%" do cartão e trocar por uma barra empilhada de 180×8 nas 4 cores de raridade; os detalhes ficam no (i).
4. "Faltam 10 para raro ou épico" vira medidor de garantia: 10 pips 12×12 (cheios/vazios) com mini estrela.
5. O botão "i" (24×26 branco) vira `game::hint` ou um botão de lupa 32×32 no canto superior direito do cartão.
6. Remover "Uma peça por remessa · moedas do jogo · sem prazo" e "Demonstração: compras exigem um perfil salvo." (em modo demo, mostrar só o botão desabilitado com cadeado).
7. Em loja-inicial (30 réis), os botões de remessas que o jogador não pode pagar devem mostrar o preço em vermelho e o cartão levemente apagado; hoje todos parecem iguais (todos desabilitados por demo).

**floating_e6_chances_0.png / garantia_0.png**
- Bom: a lista de peças com miniatura e cor de raridade.
1. "< Remessas" (botão branco) vira `game::button` ← 120×36 com ícone.
2. As 5 linhas de texto do topo ("Próxima abertura…", "Garantia…", "Sorteia a raridade…", "Repetida: devolve 20%…", "Peças elegíveis…") viram: 4 colunas de raridade com ícone de estrela e % inteiro grande (70% / 22% / 7% / 1%), uma barra de garantia "7/10" e um ícone de moeda "devolve 30".
3. Erro de plural: "Garantia: raro ou épico em até 1 aberturas." Usar "na próxima abertura!" quando for 1, com destaque dourado.
4. "Comum 0.0% · Incomum 0.0%" em garantia: esconder raridades com 0%.
5. Peças em lista de 6 por página: usar grade 4×3 de miniaturas 140×80 com moldura de raridade e selo ✓ de possuído; 1 página só.
6. "Possuído" em 11 px preto ao lado da raridade vira o selo ✓.

**floating_e6_direta_0.png / _10 (Compra direta)**
- Bom: miniaturas com moldura de raridade.
1. "Comprar · 120 réis" é um botão branco desabilitado de 306×26, que parece campo de texto. Trocar por `game::button` 180×40 "● 120" centralizado no cartão.
2. Para "Comum · Possuído", trocar o botão por um selo "TEM" (se comprar de novo não tiver efeito) ou explicar "posse permite repetir no trem" em hint.
3. Sprite 84×56 colado à esquerda num cartão de 322×147, com 200 px vazios à direita do sprite. Centralizar o sprite em 180×90.
4. Remover as duas linhas de topo ("3 / 19 modelos possuídos · posse permite repetir no trem", "Comum 3/8 · Incomum 0/8 · Raro 0/2 · Épico 0/1") e trocar por um contador "3/19" com ícone de coleção.
5. Remover "Demonstração: compras exigem um perfil salvo. Compra direta não usa sorteio."
6. Este layout é idêntico ao da Coleção E6, então o jogador não distingue as duas telas. A loja deve destacar preço; a coleção, posse.

**floating_e6_museu_0.png**
- Bom: os sprites dos selos.
1. Três selos 48×48 soltos em x 40–225 e um texto de 5 linhas. Fazer uma vitrine: prateleira de madeira 650×140 com os selos em 64×64 e etiquetas "nº 1", e os selos não possuídos em silhueta.
2. "Coleção de modelos: 39 / 39" vira barra cheia dourada com "COMPLETA!" em pixel.
3. "Comprar selo nº 2 · 120 réis" vira `game::button` 220×48 "● 120".
4. Remover "Cada selo custa 120 réis. Só coleção, sem bônus ou manutenção." e "Demonstração…".
5. O que é o Museu não fica claro (ver seção 4).

**floating_e6_abertura_0.png / _1**
- Ver seção 3. Concretamente: a caixa tem só 84×64 numa área de 650×450 vazia; "Abrindo remessa…" está à esquerda, em sans; "Resultado salvo · fechar e reabrir mantém esta peça" é texto técnico. A tampa "levantada" de abertura_1 é um deslocamento de 6 px quase imperceptível.

**floating_e6_revelacao_0..3 / repetida_0..4 / compra-concluida_0**
- Bom: a moldura na cor da raridade e o sprite grande.
1. Ver seção 3 para o desenho completo.
2. Botões "Equipar/Continuar" (brancos, 66×28 e 82×28, colados à esquerda abaixo do cartão) viram `game::button` "EQUIPAR" teal 190×48 + "GUARDAR" latão 190×48, centralizados sob o cartão com 16 px entre eles.
3. "Equipar em: Vagão 1 Vagão 2 Vagão 3" vira 3 slots com sprite do vagão atual (100×50 cada) acima dos botões.
4. Remover "Equipar respeita os passageiros e a carga a bordo." e "Resultado salvo · fechar e reabrir mantém esta peça".
5. "Nova!" em sans 14 px preto não comemora nada; usar um selo "NOVA!" dourado em pixel no canto superior direito do cartão.
6. "Repetida · +30 moedas" vira o selo cinza "REPETIDA" + moeda grande com "+30" em pixel, contando de 0 a 30.
7. Em revelacao_3 (Articulada dos Vales) e repetida_4 o sprite é cortado na borda esquerda (cabine encostada em x 232); dar 8 px de respiro ou usar uma arte que caiba.
8. Em revelacao_2/3, que não têm seletor de vagão, o cartão termina em y 350 e os botões ficam em y 371. Mesmo assim sobram 170 px abaixo, então centralizar o bloco na vertical.
9. compra-concluida: o título "Compra direta concluída" em sans e "Modelo possuído · sem sorteio" viram "COMPRADO!" em pixel; o resto segue a revelação.

**floating_e6_colecao_0.png / _10 (peças)**
- Bom: abas Vagões/Motores/Pinturas/Postais organizam bem.
1. "Ver no trem" e "Ver compra direta" são botões brancos de 306×26 que parecem campos. Para possuído: `game::button` "EQUIPAR" 150×36; para não possuído: silhueta escura + cadeado + `game::button` "● 240" que leva à loja.
2. Não possuídos com sprite colorido igual ao possuído tiram o "desejo" da coleção. Escurecer (multiplicar por 0.35) ou mostrar silhueta.
3. Mesmas correções de grade e contador de Compra direta.

**floating_e6_moedas_0.png (cabine com ganho)**
- Bom: o balão "+4" com moeda sobre o trem é exatamente a linguagem certa.
1. O balão "+4" (48×30) fica sobre o próprio trem, cobrindo a locomotiva. Desenhá-lo acima do trem e subindo/desvanecendo em 1 s.
2. Rodapé "Desde a última visita nesta sessão: +84 moedas · 0 ordens prontas": diz "moedas" enquanto o topo diz "réis". Trocar por um toast centralizado "● +84" em pixel por 3 s e ocultar "0 ordens prontas".
3. "Seu diário começa na próxima estação: Bosque dos Pinheiros": remover.
4. 6 abas em 650 px: os rótulos ficam a 1–2 px das bordas ("Favoritos", "Máquina"). Removendo Máquina, voltam a 5.

### Elementos comuns (todas as telas)
1. Linha "Viagem pausada · progresso preservado" (11 px verde em y 580): remover; o estado pausado já aparece pelo botão PAUSAR/RETOMAR e pode virar um ícone ⏸ no cabeçalho ao lado dos réis.
2. Cabeçalho E5 "89 réis" sem ícone (E6 já tem a moeda): manter a moeda em todas, com o número em `pixel_label` cell 2 em vez de sans 20 px.
3. O botão "?" (24×24 branco) vira `game::button` com Icon::Help 32×32 em latão.
4. Doca: os botões não selecionados têm o rótulo em sans 11 px, enquanto o selecionado tem pixel grande. Usar pixel cell 1.5 em todos para coerência; "Fechar" deve ter cor diferente (vermelho-tijolo) e ficar separado por 12 px.

---

## 3. Loja e abertura de caixa

Hoje a sequência inteira (loja → abertura → revelação) é uma lista de texto, uma caixa de 84 px parada e um cartão com dois botões brancos. Nenhum momento parece recompensa. Proposta, toda feita com `rect_filled`, `convex_polygon`, `sprite`, `display_text` e animação por `animate_value_with_time` (respeitando `reduce_motion`):

### 3.1 Vitrine (Remessas)
- Grade 3+2 de cartões **206×200**, com 10 px de gap, centralizada; a 2ª linha tem 2 cartões centralizados.
- Cartão de cima para baixo: faixa de título 206×28 em madeira com o nome em pixel cell 1.5 ("PÁTIO GERAL"); sprite da caixa em **80×80** (cell 5), sobre um disco de luz suave (círculo PAPER mais claro, raio 48); barra de chances 180×8 nas 4 cores; medidor de garantia em pips; `game::button` 180×44 com moeda + preço.
- Pátio geral grátis: fita diagonal "GRÁTIS" vermelha no canto e o botão teal com "ABRIR". Quando o jogador não tem réis, o preço fica vermelho e o botão desabilitado com cadeado.
- Hover no cartão: a caixa sobe 3 px e balança ±1 px.

### 3.2 Abertura
- Ao clicar: a área de conteúdo escurece (overlay WOOD_DARK com alfa 200), as abas e a doca ficam desabilitadas, e a caixa aparece no centro geométrico da área útil (y ≈ 340) em **192×192** (cell 12).
- Título em `display_text` cell 3: "PÁTIO GERAL" acima da caixa. Abaixo: `game::button` "ABRIR!" 220×56 teal. Nada mais de texto ("Abrindo remessa…" e "Resultado salvo…" saem).
- Animação (≈1,2 s total, ou instantânea com reduce_motion):
  1. 0–400 ms: a caixa treme (±3 px em x, frequência crescente).
  2. 400–600 ms: a tampa sobe 40 px e gira levemente (desenhar a tampa como sprite separado deslocado; se não houver rotação, só subir); sai um feixe de luz da fresta (retângulo vertical 24×120 na cor da raridade sorteada, alfa 180). **Pista de raridade:** a cor da luz já antecipa o resultado, o que gera expectativa.
  3. 600–750 ms: flash: retângulo de toda a área útil em branco, alfa 255→0.
  4. Corte para a revelação.

### 3.3 Revelação
- Overlay escuro mantido. Raios atrás do cartão: 12 triângulos (`convex_polygon`) saindo do centro, alternando a cor da raridade com alfa 60/20, girando a 10°/s. Só para Raro e Épico; Comum e Incomum têm só um halo circular.
- Cartão central **360×320**:
  - Moldura de 6 px na cor da raridade + 2 px internos dourados (Raro/Épico).
  - Faixa superior 360×32 na cor da raridade com o nome em pixel cell 2, branco: "COMUM", "INCOMUM", "RARO", "ÉPICO". Épico com uma estrela de 16 px em cada ponta.
  - Sprite da peça em **288×144** (2× o atual), com 16 px de margem.
  - Nome em `display_text` cell 2.5, centralizado.
  - Selo no canto superior direito, por cima da moldura e saindo 10 px para fora: "NOVA!" (fundo dourado #f2c14e, texto marrom) ou "REPETIDA" (cinza) + logo abaixo moeda 32×32 e "+30" em pixel cell 3, contando de 0 ao valor em 600 ms.
- Entrada do cartão: escala 0.6→1.05→1.0 em 300 ms (desenhar com `rect` interpolado; o sprite é redesenhado no rect corrente).
- Épico: 30 confetes (quadrados 4×4 dourados/roxos) caindo por 1,5 s.
- Abaixo do cartão, centralizados: se for vagão, 3 slots 100×50 com o sprite do vagão atual de cada posição (o slot escolhido com contorno dourado); depois `game::button` "EQUIPAR" teal 190×48 + "GUARDAR" latão 190×48. Para repetida, o botão primário vira "OK" (EQUIPAR fica secundário, ou some se já estiver equipado).
- Se houver mais aberturas pagáveis: terceiro botão "ABRIR OUTRA ● 225" 220×44.
- Compra direta concluída usa o mesmo cartão, sem raios e sem animação de caixa, com o selo "COMPRADO!".

### 3.4 Som e texto
- Tocar um "ding" curto por raridade (o jogo já tem volume de eventos).
- Nenhuma frase explicativa durante a sequência. Os únicos textos: nome da remessa, raridade, nome da peça, selo e botões.

---

## 4. Clareza para quem abre o jogo pela primeira vez

Um amigo do Pedro, abrindo pela primeira vez, ficaria com estas dúvidas:

1. **O que é o jogo e o que eu faço?** A Acolhida fala em "acompanhar seu dia" e "tempo ativo", mas não diz que **o trem anda enquanto você usa o computador e ganha réis em cada estação**. Isso deve ser uma frase só, em pixel, sob a ilustração: "SEU TREM VIAJA ENQUANTO VOCÊ TRABALHA".
2. **O que fazer primeiro?** Não há uma primeira ação. A Cabine mostra RETOMAR; Missões mostra "Recolher" em vários lugares; a Loja tem um Pátio geral grátis que ninguém vai notar. Sugestão: na primeira sessão, um ponto de exclamação pulsante (o selo "!" que já existe na doca "Viagem") em **um** lugar por vez: 1) Cabine (ver o trem andar), 2) Missões (recolher a 1ª ordem), 3) Loja (abrir a remessa grátis), 4) Seu Trem (equipar a peça).
3. **Réis, moedas e pontos.** O jogador vê "30 réis" no topo, "+30 moedas" na repetida, "+84 moedas" no rodapé e "+92 pontos"/"1604 pontos de serviço" no Diário e em Serviços. Ele vai achar que são três recursos. Unificar em réis com a moeda, ou dar aos pontos ícone, nome e utilidade explícitos (hoje não servem para nada visível).
4. **Como ganho réis?** As fontes reais são ordens do dia, marcos, entregas e repetidas, mas isso só aparece espalhado. Um cartão "COMO GANHAR" na Ajuda com 4 ícones resolveria: [ordem] Ordens do dia, [estrela] Marcos, [caixa] Entregas, [moeda] Repetidas.
5. **Remessa x Compra direta x Museu.** Não está claro que Remessa é sorteio mais barato, Compra direta é escolher a peça mais caro e Museu é colecionável final. Colocar um subtítulo de 3 palavras com ícone em cada aba: "Sorteio", "Escolha", "Troféus".
6. **"Garantia", "faltam 10 para raro ou épico", "posse permite repetir no trem", "elegíveis".** É vocabulário de sistema. Trocar por pips + "RARO GARANTIDO EM 10".
7. **O que a Máquina faz?** A aba diz "Os medidores desta sala ainda não estão disponíveis". O jogador entende que algo está quebrado. Esconder até existir.
8. **CPU no jogo.** "CPU 50.0%", "Resposta visual à CPU 1.8" e "56 km/h visuais" não explicam a ligação: **CPU alta = trem mais rápido na tela, sem ganhar mais**. Se é isso, dizer uma vez no medidor de velocidade (hint) e esconder o resto.
9. **Pausa.** "Viagem pausada · progresso preservado" em todas as telas, com RETOMAR na Cabine: o jogador não sabe se o jogo começa pausado de propósito. Na primeira abertura o trem deve estar andando.
10. **Postais e Marcos.** Não fica claro como se conquista um postal (visitando a estação? fazendo ramal?). Mostrar, no cartão bloqueado, o ícone da condição ("Visite Mirante da Serra" + ícone de ramal).
11. **Favoritos.** A palavra não indica "composições salvas". Renomear para "Garagem" ou "Composições".

---

## 5. Lista priorizada para o agente que vai corrigir

Cada item pode ser verificado reexportando as telas e olhando a imagem indicada.

### P0 — bloqueiam a cara de jogo
1. **Reexportar todas as telas** a partir do código atual. `floating_e5_trem_0` e `floating_e6_moedas_0` devem mostrar o cartão "Próxima parada" com PAUSAR.
2. **Revelação/repetida/compra-concluida:** "Equipar/Continuar" viram `game::button` "EQUIPAR" (teal, 190×48) + "GUARDAR" (latão, 190×48), centralizados 16 px abaixo do cartão. Verificar em `e6_revelacao_0..3`, `e6_repetida_0..4` e `e6_compra-concluida_0`.
3. **Revelação:** cartão 360×320 centralizado na vertical da área útil, moldura 6 px na cor da raridade, faixa superior com raridade em pixel, sprite 288×144, nome em display_text, selo "NOVA!"/"REPETIDA" no canto e moeda "+30" em pixel. Verificar em `e6_revelacao_0..3` e `e6_repetida_0`.
4. **Abertura:** caixa 192×192 no centro, overlay escuro, título pixel "PÁTIO GERAL" e `game::button` "ABRIR!" 220×56; remover "Abrindo remessa…" e "Resultado salvo…". Verificar em `e6_abertura_0/1`.
5. **Remessas:** grade 3+2 de cartões 206×200 com sprite 80×80, barra de chances 180×8, pips de garantia e `game::button` 180×44 "● preço" / "GRÁTIS"; sem a linha de percentuais em texto. Verificar em `e6_loja_0` e `e6_loja-inicial_0`.
6. **Compra direta e Coleção E6:** "Comprar · X réis", "Ver no trem" e "Ver compra direta" viram `game::button` 150–180×36–40; sprite centralizado 180×90; peças não possuídas em silhueta escura. Verificar em `e6_direta_0/10` e `e6_colecao_0/10`.
7. **Moeda única:** trocar "moedas" por "réis" em repetida e no rodapé da cabine; mostrar o sprite da moeda + número pixel no cabeçalho de todas as telas E5. Verificar em `e6_repetida_0`, `e6_moedas_0` e `e5_trem_0`.
8. **Abas:** abas e sub-abas como `game::button` (latão/teal, 36 px de altura); a aba selecionada deixa de usar o verde-acinzentado desbotado. Verificar em qualquer tela com abas.
9. **Remover a aba "Máquina"** enquanto não houver medidores. Verificar em `e6_maquina_0` (some) e `e6_moedas_0` (5 abas).
10. **Acolhida:** ilustração 640×180, título display_text, 2 cartões de modo 300×120 com sprite, `game::button` "COMEÇAR VIAGEM" 280×56 centralizado; remover as 5 linhas de clima/privacidade/offline e o checkbox. Verificar em `e5_acolhida_0`.

### P1 — legibilidade e consistência
11. Remover a linha de status "Viagem pausada · progresso preservado" de todas as telas (substituir por ícone ⏸ no cabeçalho). Verificar em qualquer tela, y 580.
12. Todo título de seção em `pixel_label`/`section_frame`: "Pátio geral", "Postais de viagem", "Selos do museu", "Sala de máquinas", "Compra direta concluída", "Vila Aurora"/"Bosque dos Pinheiros" da cabine. Verificar que nenhuma tela tem heading sans 20 px.
13. Vagões/Motores/Pinturas: raridade e capacidade dentro do cartão (faixa no topo + ícone embaixo), nada solto entre as linhas da grade; grade 3×2. Verificar em `e5_vagoes_0`, `e5_motores_0` e `e5_pinturas_0`.
14. Vagões: "Equipar na posição: Vagão 1/2/3" vira tira de 3 slots 120×60 com sprite. Verificar em `e5_vagoes_0`.
15. Missões: "Recolher" vira `game::button` teal 150×40 com moeda; "Trocar" vira ícone 40×40; colunas de Marcos com larguras fixas (barra 200 px, botão em x fixo). Verificar em `e5_missoes_0` e `_3`.
16. Missões vazio: substituir o parágrafo justificado de 4 linhas por ícone 48×48 + "EM BREVE". Verificar em `e5_missoes-vazio_0`.
17. Mapa: corrigir o glifo tofu na barra "Porto do Sol ▯ Vila Aurora"; rótulos das estações fora das linhas; trem como sprite; mapa com 300 px de altura. Verificar em `e5_mapa_0`.
18. Chances/Garantia: as 5 linhas do topo viram 4 percentuais inteiros com estrela + barra de garantia; corrigir "em até 1 aberturas"; ocultar raridades com 0%; peças em grade de miniaturas sem paginar. Verificar em `e6_chances_0` e `e6_garantia_0`.
19. Diário: entradas em linhas de largura fixa 650×56 com ícones e números; sem "−0"/"+0"; 7 por página. Verificar em `e5_diario_0/1`.
20. Ícones (i) soltos movidos para o lado do título. Verificar em `e5_arquivo_0` y 245, `e5_janela_0` y 322/379, `e5_sessao_0` y 298 e `e5_ramais-ativo_0` y 400.
21. Remover os textos de aviso/legais de telas de jogo: "Demonstração: compras exigem…", "Sem dinheiro real…", "As leituras da máquina…", "Dados: Open-Meteo…", "Peças elegíveis…", "Equipar respeita…". Centralizar créditos num cartão "Sobre". Verificar em `e6_loja_0`, `e6_direta_0`, `e6_museu_0`, `e5_clima_0` e `e6_revelacao_0`.
22. Serviços: ACEITAR/RECUSAR/DEVOLVER com a mesma altura (44 px); rótulo da barra de carga fora da junção de cores; rodapé de estatísticas em pixel cell 2 com ícones 20 px. Verificar em `e5_servicos_0` e `-oferta_0`.

### P2 — acabamento
23. Ramais: os 2 destinos viram cartões 320×220 com arte, "3:08" com relógio e `game::button` "PARTIR"; sem o collapsing header. Verificar em `e5_ramais_0`.
24. Excursão ativa: uma barra segmentada em 3 cores + relógio pixel "1:53"; remover "Ida > < Volta" solto e a breadcrumb. Verificar em `e5_ramais-ativo_0`.
25. Postais: arte de paisagem 180×100 com moldura de postal, carimbo "CONQUISTADO", 8 numa página (grade 4×2), sem "Ver peças do trem". Verificar em `e5_colecao_0` e `e6_colecao_16`.
26. Museu: prateleira 650×140 com selos 64×64 numerados + silhuetas, barra "39/39 COMPLETA!", `game::button` "● 120". Verificar em `e6_museu_0`.
27. Favoritos: 4 cartões 650×80 com mini-composição em sprites, "EQUIPAR" 130×40 e lixeira 40×40; slots vazios tracejados. Verificar em `e5_favoritos_0`.
28. Ajustes: as sub-abas viram cartões empilhados numa página; todos os botões brancos (Restaurar, Minimizar, Exportar, Importar, Rever boas-vindas, Seguir, Reposicionar, Aplicar local, Ouvir exemplo) viram `game::button` ≥200×40; lat/long atrás de "Avançado"; volume 0–100 inteiro. Verificar em `e5_janela_0`, `e5_clima_1`, `e5_fluidez_1` e `e5_sessao_0`.
29. Ajuda: só atalhos com teclas desenhadas (48×28) + cartão "COMO GANHAR RÉIS" com 4 ícones; remover os 3 parágrafos de rodapé. Verificar em `e5_ajuda_0`.
30. Cabine com ganho: balão "+4" acima do trem (não sobre ele), subindo e sumindo; rodapé "+84 moedas" vira toast "● +84" central; remover "Seu diário começa na próxima estação…". Verificar em `e6_moedas_0`.
