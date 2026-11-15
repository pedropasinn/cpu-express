# Notas C — "Playing with Balance: Game Economy Design" (Oscar Clark)

Capítulos lidos: 9 "Sinks and Sources", 10 "Balance", 11 "Validating Your Game", 13 "Sustainability and Ethical Design".

**Sobre as páginas.** Os números de página abaixo são os dos marcadores `=====PAGE N=====` do arquivo extraído (página do PDF). A página impressa no livro é N − 11 (ex.: PAGE 243 = p. 232 impressa).

**Sobre a extração.** O texto corrido está legível. As figuras não vieram: só as legendas (9.1 a 9.6, 10.1 a 10.5, 11.1, 11.2). Isso pesa em dois pontos: a Figura 9.5 (exemplo de mapa de fontes e sumidouros) e a Figura 9.6 ("Expected power level example", a planilha de exemplo que o autor anuncia na p. 249) não têm conteúdo recuperável. Portanto **não há, no trecho lido, nenhuma tabela numérica nem fórmula de curva de progressão**; o livro é conceitual. Os poucos números que o autor dá estão registrados abaixo. As páginas 251, 256, 257, 279, 284, 285, 291, 325, 331, 336, 337, 375 e 381 estão vazias na extração (provavelmente páginas em branco ou só com imagem).

**Convenção.** Tudo nas seções 1 a 4 é paráfrase do livro. As seções 5 e 6 são interpretação minha para o CPU Express; onde há números, são hipóteses de partida, não vêm do livro.

---

## 1. Capítulo 9 — Sinks and Sources (PAGE 236–255)

Citação curta: "There MUST be at least one sink and one source" (p. 243).

### 1.1 Definições (p. 237)

- **Fonte (Source, Tap, Faucet)**: o ponto do jogo onde o jogador obtém um bem de utilidade (moeda, recurso, item).
- **Sumidouro (Sink, Drain)**: o ponto onde esse bem é usado ou retirado.
- **Análise de fontes e sumidouros (Sink & Source Analysis)**: uma forma de pensamento sistêmico para entender a que ritmo o jogador adquire cada bem e como se regula a escassez e o gasto. É, nas palavras dele, um modelo de oferta, demanda e valor de cada troca.
- **Equilíbrio dinâmico (dynamic equilibrium)**: para o jogo ser "equilibrado", fontes e sumidouros precisam ser desenhados para manter um equilíbrio que se move, não um ponto fixo.

### 1.2 Mapear o jogo pelos três laços (p. 237–239)

O autor percorre um jogo genérico e pergunta, em cada etapa, o que entra e o que sai:

- **Laço central (Core loop)**: o jogador investe tempo e alguma "energia" (o preço de entrada da cena), escolhe atalhos (consumíveis, equipamento), perde saúde na resolução, ganha experiência (mais se vence do que se falha) e, na recompensa, recebe recursos e moeda.
- **Laço de contexto (Context loop)**: experiência vira nível; recursos e moeda viram poder por meio de fabricação; a etapa de otimização é onde o jogador decide como alocar o que tem; a etapa narrativa paga em status ou história.
- **Laço cultural (Cultural loop)**: utilidades mais abstratas (tempo dedicado a este jogo em vez de outra coisa, cooperação, posição em ranking, status, pertencimento, cosméticos). Ele admite que aqui o modelo fonte/sumidouro encaixa menos bem.

### 1.3 Mapa alternativo: fluxo linear (p. 240)

Os laços mostram como as camadas interagem, mas não como o jogador vive a experiência em linha. Os laços giram em ritmos diferentes, como os ponteiros de um relógio. A alternativa é desenhar o fluxo de um jogador típico (e de vários perfis de jogador), recurso por recurso, eventualmente nível a nível com números. Para cada recurso do jogo, anotar se age como fonte (oferta) ou sumidouro (gera demanda).

### 1.4 Quatro categorias de fontes e sumidouros (Categories of Sinks and Sources, p. 241–242)

1. **Jogador (Player)**: o que pertence diretamente ao jogador: poder, habilidades, saúde, inventário. É de onde ele traz coisas para a atividade (fonte) e para onde recompensas e progresso vão (sumidouro).
2. **Atividade (Activity)**: tudo o que o jogador faz; em geral coincide com o laço central. Ele investe itens para melhorar o desempenho (sumidouro) e recebe moedas, recursos e itens como saída (fonte). O que se investe pode ser consumido ou mantido, conforme o desenho.
3. **Progressão (Progression)**: o mecanismo que faz jogador e mundo evoluírem. Gasta-se experiência para subir de nível, ou recursos e moeda contra receitas (Blueprints) para fabricar equipamento melhor. Aqui ele também inclui sistemas que apenas **mantêm** o status em vez de avançá-lo: desgaste de itens (degradation of assets) e sistemas de prestígio (Prestige) são sumidouros eficazes porque renovam o benefício das fontes originais.
4. **Recompensa (Reward)**: sistemas que liberam recompensas, em geral com gratificação adiada. Exemplo dele: fichas, chaves ou baús obtidos no fim de uma sessão; o jogador "afunda" a chave em troca do conteúdo do baú. Ofertas são recompensas com alguma troca comercial, mesmo que seja só reciprocidade (item diário grátis). Cosméticos são o exemplo típico: nascem como recompensa e são absorvidos ao serem postos no inventário.

Ele sugere acrescentar categorias próprias quando fizer sentido (por exemplo, **sazonalidade** — Seasonality — para acompanhar passes de temporada e itens únicos), mas manter poucas categorias para não se perder.

### 1.5 Construir o mapa (Building a Sink/Source Map, p. 242–243)

- **Fatores de troca** que ele usa no exemplo: tempo (Time), repetição (Grind), recurso (Resource) e moeda (Currency).
- **Modificadores**: âncoras (Anchors) e catracas (Ratchets), vindos do capítulo 8. Âncoras separam o que vem só de jogar do que vem de ofertas; catracas são condições que restringem o acesso (ter desbloqueado algo, estar em certo nível).
- **Técnica prática**: uma tabela com uma coluna por categoria e um papel adesivo por ocorrência de cada item de utilidade. **Regra dura**: cada item precisa de pelo menos dois adesivos, um de fonte e um de sumidouro. Isso é o que evita inflação. Ele acrescenta que há um argumento para ter **mais sumidouros do que fontes** por item, para sustentar a demanda.

### 1.6 Regular o fluxo (Regulating Flow, p. 244–249)

Razões e instrumentos para regular:

- **Complexidade (Complexity)**. Prever a interação de vários recursos com relações ortogonais é comparado ao problema dos três corpos: pode ser insolúvel analiticamente. Ferramentas que ele cita: Machinations, simulação de Monte Carlo, planilhas e protótipos em papel. E dois caminhos mais diretos: construir um chute e testar jogando (o modo normal de fazer jogos) e usar âncoras.
- **Moeda âncora / padrão-ouro (Anchor currency, Gold Standard)** (p. 245). Definir um número pequeno de moedas (talvez uma só) que **só se obtém jogando** e fixar o valor de todos os outros itens em relação a ela. Reduz a complexidade e mantém o foco no jogo em si.
- **Escassez e restrição (Scarcity and Constraint)** (p. 245–246). O ritmo de acesso define o valor. Alavancas: esforço versus taxa de queda (drop rate); aumentar o volume exigido conforme o jogador progride; especializar o recurso. Bens digitais não têm limite natural: um jogador dedicado chega perto do infinito e quebra a economia se nada o contiver. A repetição (grind) é necessária para uma economia sustentável, mas é também grande fonte de inflação. E mais poder tende a render mais recompensas, o que reforça a inflação. Remédios: limite de capacidade (quanto se pode carregar), fontes finitas (só tantos por dia) e recursos especializados. Aviso: um recurso especializado que perde o uso depois que o caminho que o exigia foi concluído fica sem valor. Exemplo dele: uma erva que só nasce ao lado de um vulcão, com **15 plantas acessíveis por dia**.
- **Otimização (Optimisation)** (p. 246–247). Na forma simples, uma taxa de câmbio entre ações e recompensas. O desenhista não precisa manter relação de um para um entre tempo e recompensa: missões, tarefas e desafios podem pagar em proporções diferentes para motivar modos diferentes de jogar (habilidade, repetição, social). Para isso não sair do controle, amarrar tudo à moeda âncora, e até exigir um pouco de cada modo para avançar.
- **Economia do cachorro-quente (Hotdog Economics)** (p. 247). Salsichas vêm em pacotes de 8 e pães em pacotes de 6 (nos EUA, 10 e 8). O descasamento entre dois bens consumidos juntos gera sempre uma sobra e uma vontade de comprar mais do outro. A ambiguidade entre fonte e sumidouro é, em si, prazerosa de otimizar e ajuda a sustentar a economia.
- **Laços de realimentação (Feedback Loops)** (p. 247–248). Fonte é realimentação positiva (aumenta a oferta); sumidouro é negativa (retira). "Positiva" aqui é termo matemático, não juízo. Desequilíbrio entre as duas produz inflação ou deflação. Efeitos emergentes: dar as melhores recompensas só a quem vence abre um abismo crescente entre jogadores; não dar nada a quem falha repetidamente impede que ele ganhe poder e o trava. Por outro lado, exigir esforço crescente por melhoria (uma piora na taxa de câmbio esforço/benefício) pode tornar a experiência mais significativa e reter por mais tempo.
- **Oferta, demanda e preço (Supply, Demand, and Price)** (p. 248–249). No jogo o desenhista controla quase tudo, então pode mapear e monitorar. Pacotes de moeda aumentam a oferta e afetam a escassez. Pagar para **não** jogar raramente é boa experiência; ele nota que jogos ociosos (Idle Games) e Monopoly GO! são exceções em que pular o jogo é visto como positivo, mas raras.

Fecho do capítulo (p. 249): a economia existe para dar estrutura à progressão; e a matemática não basta, porque não se separa o equilíbrio da sensação de jogo (Game Feel).

### 1.7 Entrevista com Huel Fuchsberger (p. 252–255)

Sobre desenho de fases em jogos de combinar três. O que serve aqui:

- Estrutura de começo, meio e fim em cada unidade de conteúdo; método de apresentar um conceito, deixar dominar e depois torcer.
- Evitar que o jogador se sinta burro; rotular com precisão o que vai ser mais difícil, porque isso ajusta a expectativa.
- O atrito faz parte da experiência: tirar todo o atrito (saldo ilimitado) não satisfaz.
- Conteúdo paralelo em excesso pode engolir a experiência central.
- **Coleções** podem trazer a empolgação de um gacha sem afetar a jogabilidade, criando momentos de recompensa e surpresa (p. 255).
- Desenhar com intenção clara; intenção confusa é sentida como confusa.

---

## 2. Capítulo 10 — Balance (PAGE 258–290)

Citação curta: "There is no such thing as perfect balance." (p. 261).

### 2.1 O que é equilíbrio (p. 259–260)

Equilíbrio é o processo de ajustar mecânicas, recompensas e desafios para uma experiência envolvente. O autor insiste que **equilíbrio não é igual a justiça**:

- **Justiça (Fairness)**: nenhuma opção deve dominar, mas o que importa é o jogador continuar acreditando que o sucesso é alcançável. Sentir que perdeu por causa da mecânica e não por escolha própria leva a abandonar com raiva.
- **Experiência pretendida (Intended Player Experience)**: equilíbrio é vivido como a combinação de desafio, recompensa e satisfação; a sensação de que habilidade, escolhas, paciência ou repetição acabam dando o resultado desejado. A ambiguidade (não saber o que vem) faz parte.
- **Processo contínuo (Ongoing Process)**: um jogo sempre "em equilíbrio" tende a ser sem graça; o equilíbrio é dinâmico e muda ao longo do ciclo de vida do jogador.

Como ele **mede** equilíbrio: não há métrica única neste capítulo. Ele o trata como percepção, e remete a quatro lentes: sensação de jogo, equilíbrio perfeito, ciclo de vida e curvas de poder. As medidas quantitativas aparecem nos capítulos 11 e 13 (retenção, frequência e duração de sessão, razão produção/consumo).

### 2.2 Sensação de jogo (Game Feel, p. 260–261)

Baseado em Steve Swink: a sensação nasce do ciclo entre a intenção do jogador, o dispositivo físico (com latência), o mundo do jogo, a resposta sensorial (imagem, som, toque, lógica, narrativa) e a cognição do jogador. A expectativa do jogador vem de outros jogos e do mundo real. Só se desenvolve o "músculo" com muito jogo e muito teste.

### 2.3 Equilíbrio perfeito (Perfect Balance, p. 261–263)

- Não existe fórmula simples que converta mecânica em poder e status; matemática simples costuma sair pela culatra.
- **Exemplo numérico dele**: um sorteio que paga de 1 a 50 ouros. Com aleatoriedade verdadeira, existe a chance de alguém tirar 1 sempre e outro tirar 50 sempre, e quanto mais jogadores, mais provável que aconteça com alguém. Conclusão: usar **números pseudoaleatórios (pseudorandom)**, que parecem sorte mas evitam os aglomerados ruins.
- Equilíbrio está no olho de quem vê: jogadores reclamam de mudanças mesmo cosméticas ou compensadas em outro lugar. Muitas queixas de desequilíbrio são, na verdade, falta de informação.
- Opções exatamente iguais são tediosas. É preciso um **dilema** com custo e benefício. O caminho mais difícil deve pagar mais.
- Jogadores gostam de achar uma vantagem contextual; ajustes intencionais para cima e para baixo (buffs e nerfs) podem dar sensação de autonomia e esperteza.

### 2.4 Ciclo de vida do jogador (Player Lifecycle, p. 263–270)

Distingue o ciclo de produto (pico inicial e queda, cauda longa) do ciclo de serviço, que tem cinco estágios:

1. **Descoberta (Discovery)**: decisão binária de jogar agora ou não. Reduzir atrito; não enterrar o jogador em telas iniciais e exposição. Exemplo negativo dele: um jogo de 2024 com cadastro forçado, download de 10 GB (cerca de 40 minutos) e mais 20 minutos de introdução sobre trilhos.
2. **Aprendizado (Learning)**: aprender os sistemas e também **onde o jogo cabe na rotina**. A primeira experiência (First Time User Experience, FTUE) decide entre ficar e sair. Dado dele: em jogos de serviço, sobretudo no celular, a maioria sai nas primeiras 24 horas. Mostrar por que vai valer a pena voltar; antecipar o que vem (foreshadowing).
3. **Engajamento (Engaging)**: começa quando há padrão habitual de jogo, não quando o tutorial termina. Medidas de sucesso: retorno consistente (retenção de D7 a D30), frequência e duração de sessão. Cuidar para o jogador não sentir que perde tempo em laços que já entendeu por completo. Ele propõe pensar numa "segunda experiência" (Second Time User Experience, STUE): em vez de ensinar, ajudar o jogador a afinar e otimizar.
4. **Superengajamento (Super‑Engaging)**: fãs que adotam o jogo como parte da identidade. Ele rejeita o termo "baleias" (Whales) e defende que fãs comprometidos valem mais e de forma mais previsível, gastem muito ou pouco. Valorizam status e visibilidade social.
5. **Reengajamento (Reengaging)**: todo mundo sai um dia. Quem volta não lembra como jogar e se frustra se tiver de refazer o tutorial. Propõe uma experiência para quem ficou afastado (Lapsed Time User Experience, LTUE).

**Funil de lado (Sideways Funnel / Lifecycle Pipeline)**: entram mais do que chegam ao fim; pensar em ativar, converter e reengajar ao longo do cano.

### 2.5 Curvas de poder (Power Curves, p. 270–275)

Poder é a multiplicação do efeito das ações básicas por fatores de progressão, situação e equipamento. Elementos:

- **Curva natural de aprendizado (Natural Learning Curve)**: demora a engrenar, sobe rápido, depois estabiliza. Cada jogador tem seu teto.
- **Progressão em escala (Scaling progression)**: cada degrau de poder exige **esforço e troca crescentes**, o que desacelera o avanço, reduz a distância entre jogadores e o peso do poder sobre a habilidade. Risco central: escalada excessiva de poder tira o desafio e, com ele, o sentido de jogar. **O autor não dá fórmula nem expoente para a curva.**
- **Recompensas de repetição (Grind Rewards)**: ligar recompensa a esforço e tempo, não só a sucesso, para que quem é menos habilidoso também avance. Quem perde para um chefe deixa de ganhar o prêmio grande mas leva o prêmio menor.
- **Reforços e reduções temporários (Short Term Buffs and Nerfs)**: modificadores de uma sessão ou por tempo limitado. A escassez de consumíveis pode ser regulada por raridade, intensidade, custo, duração, tempo de recarga, capacidade, empilhamento. Exercício de entrevista que ele cita: inventar poções de vida usando só as variáveis tempo e pontos de vida (exemplos: 50 de uma vez; 10 a cada 3 s por 15 s; 200 agora e −100 depois de 30 s). O mínimo aceitável é dez variações.
- **Aleatoriedade (Randomness)**: nivela e cria ambiguidade, atenuando o efeito inexorável da progressão; mas a faixa de variação precisa ser acertada para as ações não parecerem inúteis. Repetir: preferir modelos pseudoaleatórios. Comparação: D&D (d20 com +1 por incremento) faz sentir poder rápido; Call of Cthulhu (percentual, ganhos de poucos pontos) faz sentir vulnerável. A regra de vantagem/desvantagem (rolar mais de um dado e ficar com o melhor ou o pior) reduz extremos sem eliminá-los.
- **Desenho ortogonal (Orthogonal Design)**: sistemas independentes que dão benefícios separados. Graus de relação: exclusividade mútua (Mutual Exclusivity), contrajogo circular (Circular Counterplay, pedra/papel/tesoura), poder equivalente com sensação diferente (Equal Power Dynamics), diversidade de escolhas (Diversity of Choices), sistemas emergentes (Emergent Systems).
- **Sistemas de prestígio (Prestige Systems)** (p. 275): o jogador escolhe zerar um item ou progresso em troca de mais jogo, recebendo sinalização visual, alguma melhoria e a chance de reviver a subida. Quando o benefício é ortogonal ao sistema central, sustenta o jogo no longo prazo sem escalada de poder.

### 2.6 A economia como história (Game Economy as a Story, p. 275–278)

- **Desbloqueios narrativos (Narrative Unlocks)**: comunicar mudanças de poder e dificuldade por meio de história, revelações e momentos de "abrir a caixa" (unboxing), para que um dragão não pareça só um rato maior.
- **Missões e desafios (Missions and Challenges)**: a forma de apresentar e enfileirar missões enquadra a experiência; missões encadeadas formam um arco, com dificuldade e recompensa escalando até um clímax; depois as apostas podem ser zeradas ou travadas em relação ao progresso geral.
- **Textura emocional (Emotional Texture)**, a partir de Florian Steinhoff (GDC 2014): a experiência precisa de picos de dificuldade perceptivelmente mais duros, seguidos de vitória intensa e de um trecho de **alívio pós-pico** (post‑peak‑relief), mais fácil, bom para apresentar novidades. Cada tentativa falha deve dar alguma recompensa reduzida. **Única "fórmula" do capítulo**: o fator de frustração é definido como a razão entre tentativas de jogar e "quase ganhei" (Attempts to Play versus Nearly Won). Falhar por completo não interessa emocionalmente, nem vencer fácil; interessa acreditar que dava para ter vencido. A legenda da Figura 10.5 ("Spikes in reward and difficulty") indica picos coincidentes de recompensa e dificuldade; o gráfico não foi extraído.

### 2.7 Entrevistas

**Luna Javier** (p. 280–283): distingue quem desenha pela sensação (feels designer) de quem desenha pela matemática. Pontos úteis:

- No protótipo do laço central, não se preocupar com os números: basta garantir que o jogador ganha moedas e as gasta em alguma coisa.
- O ajuste fino começa na primeira experiência: nos primeiros minutos é preciso ver e sentir a progressão, suave e recompensadora.
- Um desenhista de sistemas imaginou uma sequência de progresso que o jogo real não reproduzia: a experiência vem primeiro, os números se ajustam a ela.
- Lançar em pedaços bem pequenos, sem precisar ter a planilha inteira pronta.
- Fazer um rascunho dos números, pôr na versão jogável e testar na hora.
- A recompensa tem de ser algo que o jogador realmente quer.

**Valentina Tamer** (p. 286–289): narrativa e economia devem andar coladas; o que se faz para ganhar o quê já é história. A coerência (nomes e tipos de recompensa combinando com o mundo) importa. Uma ação repetitiva ganha sentido se enquadrada (ajudar um personagem de quem se gosta). Depois de um inimigo duro, um trecho com inimigos comuns deixa sentir o poder conquistado. Estrutura de boneca russa: arcos pequenos dentro de capítulos dentro do arco total. Cumprir o que se promete. Sem enquadramento narrativo, a economia vira obrigação.

---

## 3. Capítulo 11 — Validating Your Game (PAGE 292–335), leitura dirigida

Citação curta: "Test early, test often, and focus your designs using data insights." (p. 293).

A maior parte do capítulo trata de estúdio comercial. O que é aproveitável num projeto de uma pessoa:

### 3.1 Visão antes de medir (p. 293–297)

- Antes de validar, definir "como é o bom" (What Good Looks Like): quem são os jogadores, o que eles valorizam e por que o jogo atende a isso. É uma pergunta de psicologia, não de tecnologia.
- Dimensões que ele lista para definir sucesso: dinheiro (TAM/SAM/SOM), valores, expressão e mercado. Em **expressão (Expression)** entram comunicar uma ideia, acrescentar algo ao gênero e reconhecimento dos pares; ele admite que pode ser algo tão simples quanto gostar de gatos.
- **Pilares versus vetores (Design Pillars vs. Vectors)**: pilares são frases imóveis e podem virar vacas sagradas; vetores são direções com medida associada.
- "Triângulo da estranheza" (Triangle of Weirdness, Scott Rogers): mudar uma coisa por vez.

### 3.2 Método científico (Science and Design, p. 297–300)

- Tratar o desenho como hipótese: declarar a expectativa, testar, comparar.
- **Indicador-chave (KPI)**: a métrica que valida a previsão. Definir o alvo **antes** do teste, para não maquiar o resultado.
- **Pesquisa qualitativa (Qualitative Research)**: entrevistas, grupos, observação, questionários. Escala Likert de 5 pontos (simétrica) ou de 4 (obriga a tomar partido).
- **Dados quantitativos (Quantitative Data)**: atribuição, jogabilidade, conversão, sentimento. Coletar com responsabilidade quanto à privacidade.
- Um bom experimento é repetível, com variáveis significativas, independentes e, tanto quanto possível, cego dos dois lados. Na prática sempre há concessões.
- "Falhar é sempre uma opção": planejar ao menos um segundo teste antes de desistir de algo, mas definir de antemão a melhora mínima exigida e não ser indulgente com os dados.
- Lista de indicadores típicos (p. 299–300): ranking, receita, ROAS, LTV, retenção, engajamento (duração e sessões por dia), engajamento social, NPS, Metacritic, prêmios, imprensa, sentimento, custo por instalação, visualizações, jogadores ativos.

### 3.3 Estágios de desenvolvimento (p. 300–307)

- Quatro origens de um conceito: guiado pelo mercado (Market Driven), experimentação (Experimentation), instintivo (Instinctive), reconstrutivo (Reconstructive).
- Estágios tradicionais: protótipo, fatia vertical, alfa, beta, acesso antecipado, lançamento.
- **Desenvolvimento guiado por teste** (inspirado em Lean Startup): protótipo em papel; primeiro trailer (medido por custo por clique); **teste D1** (laço central com 20 minutos a uma hora de jogo, laço sem fim com recompensas e ainda sem progressão; teste de usabilidade com gravação; NPS); **teste D7** (acrescenta o laço de contexto: propósito, progressão, otimização, narrativa; acompanha a primeira experiência inteira); teste D14–D30; teste de ROAS; lançamento.
- Aproveitável: validar **em camadas** (primeiro o laço central, depois a progressão), acrescentar pouco entre um teste e outro, e não conduzir pessoalmente o teste de usabilidade porque isso introduz viés.

### 3.4 Depois do lançamento (p. 307–313)

- **Percepção do jogador (Player Insights)**: além de retenção, medir duração de sessão, conclusão de desafios, razão vitória/derrota, uso de cada sistema. O que os jogadores fazem difere do que deveria acontecer; partes inteiras podem passar despercebidas e desbalancear a curva de poder.
- **Análise de lacunas (Gap Analysis)**: diferença entre comportamento esperado e real; teste A/B mudando um elemento por vez.
- **Definir uma atividade (Defining an Activity)**: para quem é, por que vai importar, como é a troca, como é comunicada, como é a abertura da recompensa. Rever fontes e sumidouros a cada atividade nova.
- **Consequências inesperadas (Unexpected Consequences)**: toda mudança em pagamentos, conquistas ou dificuldade repercute. Perguntar: quanto isso acrescenta à oferta de cada item? E os sumidouros que usam esse item? Problema comum: recompensas sazonais ou ofertas injetam oferta demais e causam inflação. Remédios citados: reenquadrar (o caso do World of Warcraft, em que a penalidade de cansaço foi reapresentada como bônus de descanso, com a mesma proporção); isolar as recompensas de uma atividade dando-lhes "sabor", como uma **moeda sazonal** que só vale na loja daquela temporada e é convertida no fim; criar um sumidouro específico para o evento.
- **Teste (Testing)**: cada atualização é um minilançamento; nenhum plano sobrevive ao contato com o jogador.
- **Pós-morte (Postmortem)**: o que foi o impacto, o que foi bem, o que podia ser melhor, o que se aplica adiante, o que se aprendeu sobre o que os jogadores valorizam.
- **Pré-morte (PreMortem)**: fazer o pós-morte antes de lançar, partindo da premissa de que falhou; listar as causas possíveis e achar os pontos de controle onde mitigá-las (lógica do HACCP da indústria de alimentos).

### 3.5 Saber parar e pivotar (p. 313–324)

- Sinais: queda de engajamento; novos sistemas que não mudam nada (os jogadores chegam até eles? há barreira ou sinalização ruim?); **redução do volume de retorno** (crítica negativa mostra que as pessoas se importam; o silêncio é pior); mudança no padrão de retenção (jogo que só agrada veteranos e ficou confuso para quem chega).
- Vetores de ajuste dentro do desenho (p. 317–321):
  - **Fluxo de progresso (Progress Flow)**: análise de funil; muitas vezes é só sinalização.
  - **Taxas de queda (Drop Rates)**: recompensa só no sucesso prende o jogador num laço sem saída; recompensa desproporcional tira o desafio. Mexer no pagamento irrita quem já joga; fazer com cuidado.
  - **Dificuldade/poder (Difficulty/Power)**: se a curva estiver plana demais, variar a dificuldade para dar textura.
  - **Enquadramento narrativo (Narrative Framing)**: o mesmo número apresentado como reforço positivo ou negativo tem efeitos diferentes.
  - **Sumidouros de recurso (Resource Sinks)**: acrescentar sumidouros é dos ajustes de maior impacto. Formas: mudar a taxa de câmbio; dar um novo uso a um recurso existente (cria dilema de escolha); exigir um recurso diferente em níveis altos (torna o antigo redundante); gerir capacidade. Se a fonte for restritiva demais, prejudica quem está começando.
  - **Modos de jogo (Game Modes)**: introduzir como sazonal e só efetivar se der certo; sistemas demais complicam.
  - Pivôs de monetização, público, plataforma e conteúdo (pouco aplicáveis).
- Processo iterativo de retorno (p. 321–322): coletar, achar dores comuns, prototipar e testar a mudança com um grupo pequeno, implementar, monitorar. Jogadores são bons em apontar problemas e ruins em propor soluções; os que reclamam são minoria vocal, e os insatisfeitos de verdade já foram embora.
- Mudar sem alienar (p. 323–324): transparência sobre o porquê; mudanças graduais mas inequívocas; convidar a testar; recompensar a lealdade respeitando o que o jogador já investiu, de preferência com algo que dinheiro não compra (reconhecimento, créditos); manter a identidade central.

### 3.6 Entrevistas

**Tadhg Kelly** (p. 326–330): todos os jogos são jogos econômicos (tempo, esforço, atenção). Erros comuns: confundir complexidade com profundidade; montar o mesmo sistema três ou quatro vezes; moedas que são só "denominações" umas das outras. Conceitos: **espaço de possibilidades (possibility space)**, a gama de resultados significativos que o sistema permite, que deve ser "colapsado" em pontos de escolha crítica; **coceira (itchiness)**, a vontade de voltar a mexer num sistema. Se tudo é dado fácil, a coceira some; se a repetição é dura demais, vira frustração. Jogos digitais toleram **menos** relações obscuras do que jogos de tabuleiro, porque o sistema fica escondido atrás de menus: o efeito de uma compra tem de ser sentido na hora, e a função de uma moeda tem de ser óbvia.

**Elise Terranova** (p. 332–335): jogos casuais não têm necessariamente jogadores casuais. Olhar para o que o jogador já gosta e a que ele volta. Voltar depois de meses é o caso mais difícil. Jogadores gostam de mostrar e de ser reconhecidos; ter desbloqueado o último, o mais difícil ou o primeiro item é um marcador de progresso. Relançar itens antes "exclusivos" mina a confiança. O valor vem da expectativa de encanto, não só da função.

---

## 4. Capítulo 13 — Sustainability and Ethical Design (PAGE 364–380)

Citação curta: "What truly makes them Dark is in the intention of the designer." (p. 372).

### 4.1 Por que ética (p. 365)

Jogos são a maior forma de entretenimento digital; decisões de economia ficam sob holofote e já provocaram regulação (dados, caixas de recompensa, compras no aplicativo). Números que ele cita: mais de 40% da população mundial joga; mais de 85% da receita vem de jogos de serviço ou gratuitos; projeção de mais de 580 bilhões de dólares até 2030.

### 4.2 Armadilhas no desenho da economia (Pitfalls in Game Economy Design, p. 366–369)

1. **Não ligar preço ao valor percebido (Failing to Connect Price with Player Value)**. Se o preço não corresponde ao valor que o jogador atribui, ele não compra e se sente alienado. Remorso pós-compra é fator crítico. Quem gasta satisfeito tende a gastar de novo.
2. **Exemplos de preço errado (Examples of Mispricing)**:
   - cosméticos caros demais para o apelo que têm (exemplo: 20 dólares por uma aparência simples);
   - **barreiras pagas à progressão essencial (Paywalls for Essential Progression)**, muitas vezes criadas sem querer;
   - **baixar preço sem aviso (Dropping Price without Notice)**: quem acabou de pagar o preço alto sente-se roubado; e preço, em geral, só desce.
   Consequência dupla: menos compras e perda de confiança.
3. **Desconexão com a utilidade (Disconnect with Utility: Failing to Provide Meaningful Rewards)**. Recompensas ganhas ou compradas precisam ter utilidade real. Um item puramente cosmético ainda tem valor social. Um item que deveria melhorar a jogabilidade e quase não muda nada parece desperdício; um que muda demais é visto como quebrado. Quem se esforça num desafio difícil e recebe algo sem graça se frustra e sai cedo. As recompensas devem melhorar a experiência e ainda deixar vontade de mais. Sinal de alerta: queda de sessões ou de duração depois de um gasto.

### 4.3 Padrões obscuros (Overuse of Dark Patterns, p. 369–374)

Ele sustenta que a má-fé deliberada é menos comum do que parece; o mais frequente é descuido ou metas de curto prazo. Mas o que conta é **como o jogador percebe**, não a intenção. Os quatro padrões que ele nomeia:

1. **Pagar para vencer (Pay‑to‑Win Mechanics)**: gastar dinheiro para vantagem significativa sobre quem não paga.
2. **Ofertas por tempo limitado e medo de ficar de fora (Time‑Limited Offers and FOMO)**: urgência ou escassez artificial que empurra a compra por impulso, explorando a diferença entre o estado emocional quente e o frio (Hot/Cold Empathy gap). Gera ansiedade e arrependimento. Ele ressalva que dar um motivo para agir agora é legítimo e humano; o problema é a pressão desmedida, que além disso é proibida por normas de publicidade e de defesa do consumidor na Europa.
3. **Mecânicas de energia e bloqueios de progressão (Energy Mechanics and Progression Blockers)**: limitar quanto se joga numa sessão e cobrar para continuar. Imagem dele: o jogador tem **impulso** (momentum); cada parada tira energia, e paradas seguidas sem um respiro (recompensa positiva ou descanso) esgotam o impulso e ele abandona. Difícil de diagnosticar, porque exige olhar o caminho todo e não só a última atividade. Ressalva: regular o ritmo de progressão é parte essencial do desenho.
4. **Manipulação da aleatoriedade (Randomness Manipulation)**, o item que trata diretamente de caixas sorteadas (Loot Crate) (p. 371–372):
   - Bem usada, a sorte acrescenta empolgação e dá motivo para obter o item de novo.
   - Com aleatoriedade verdadeira, há o risco de o jogador **nunca** conseguir o que quer. Com pseudoaleatoriedade, há o risco de ser percebida como manipulação.
   - Se o item dá benefício material, o que distingue isso de loteria ou caça-níquel? O risco diminui quando não é possível converter o item em dinheiro.
   - É essencial (e exigido por algumas plataformas e mercados) **divulgar as taxas de sorteio** e garantir que o valor obtido seja **equivalente, ainda que não idêntico, a cada abertura**.
   - **Camadas compostas de sorteio** (precisar tirar algo para ter chance de tirar algo mais raro) foram proibidas no Japão em 2012 e a prática praticamente desapareceu.
   - **Ilusão de controle (Illusion of Control)**, de Suzanne C. Thompson: envolvimento pessoal, familiaridade e saber de antemão o prêmio desejado levam o jogador a crer que controla o resultado ("desta vez sai"), mesmo depois de falhar três vezes. Acontece até com quem conhece o efeito. Ele observa que esses mesmos fatores tornam a recompensa mais pessoal e imersiva quando usados com autenticidade.

Outras práticas condenadas no texto corrido (p. 373–374): caixas com **probabilidades ocultas** (hidden odds); **esconder o preço até o último momento** (o que, segundo ele, ainda reduz a conversão); custos abstraídos em vez de claros.

### 4.4 O que ele recomenda no lugar (p. 372–374)

- Vários caminhos de progressão; quem não paga deve progredir sem se sentir punido. Recompensar habilidade, tempo e decisão.
- Transparência: o jogador quer saber exatamente o que está obtendo e sentir que controla a decisão.
- Canais abertos de retorno; ajustar com base em dados e escuta. Não fazer exatamente o que os jogadores pedem: eles apontam problemas, não soluções.
- Economia "jogador primeiro": respeita o investimento do jogador, dá recompensas com sentido, constrói confiança.

### 4.5 Entrevista com Chris Kaczmarczyk‑Smith (p. 376–380)

É o trecho mais técnico do capítulo sobre fontes e sumidouros:

- Todo recurso acumulável precisa de um sumidouro natural. Não precisa ser troca de um para um: pode ser um **imposto embutido**, em que tudo o que se faz consome um pouco (combustível, no exemplo dele).
- **Assimetria de reação**: para reduzir um material na economia, pode-se encarecer o gasto ou fechar a torneira. O efeito é o mesmo, mas as pessoas **reagem menos a um aumento de custo do que a uma redução do ganho**.
- Modelar o comportamento agregado com poucas hipóteses simples: quanto material existe, a que velocidade pode ser extraído, com que velocidade circula.
- **Equilibrar para alguém**: quantas horas leva para fabricar tal coisa? Para responder é preciso dizer de que tipo de jogador se está falando.
- Métricas: **razão produção/troca** (production to trade; em alguns ativos dele, 2 para 1, isto é, cada unidade produzida mudou de mãos duas vezes) e **razão produção/consumo** (production to consumption), sobre o que é retirado da economia.
- **Material estrela-guia (North Star material)**: um recurso essencial usado como referência para equilibrar todos os outros.
- **Preço é uma razão entre quantidades**; a moeda só facilita enxergar essas razões. Vale mesmo em jogo de um jogador, sem comércio.
- Caso que ele relata: jogadores diziam que um item era raro, mas o preço não subia, e aumentar a oferta não mudou o preço. O problema não era escassez e sim **falta de usos**; acrescentar sumidouros resolveu.
- "Bom desequilíbrio": equilíbrio perfeito é impossível e, se atingido, dura cinco minutos. Disparidades (por exemplo, regionais) geram dinâmica.
- Três princípios finais: manter simples (complexidade gera brechas não previstas); usar preços e métricas para decidir com a economia rodando; pensar no longo prazo.

---

## 5. Aplicação ao CPU Express

Esta seção é interpretação minha. O livro não fala de jogos de barra de tarefas nem dá números de referência; onde aparecem valores, são **hipóteses de partida para testar**, na linha do que Luna Javier recomenda (rascunhar, pôr no jogo, testar na hora).

### 5.1 Decisões de estrutura

- **Uma moeda só, âncora** (p. 245). Ela só nasce de tempo ativo de operação (serviços e missões). Todo preço do jogo é expresso nela. Isso atende ao "manter simples" de Kaczmarczyk‑Smith (p. 380) e evita o erro que Kelly descreve, de criar moedas que são só denominações umas das outras (p. 327).
- **Estrela-guia: minutos ativos** (p. 379). Como o tempo ativo é a única coisa que o jogador de fato "paga", convém pensar todo preço como "quantos minutos ativos custa". A moeda é só a unidade visível dessa razão.
- **Sem energia, sem bloqueios** (p. 370–371). O jogo já tem um regulador natural, que é o tempo de viagem (20 s por km, trechos de 1 a 7 minutos). Não acrescentar espera artificial por cima.
- **Vagões e locomotivas como coleção, não como poder.** Fuchsberger nota que coleções dão a empolgação do gacha sem mexer na jogabilidade (p. 255). Se as peças raras rendessem muito mais moedas, haveria a realimentação positiva que o autor aponta como causa de inflação (mais poder gera mais recompensa, p. 245) e o sorteio passaria a decidir o progresso. Se quiser dar função às peças, que seja ortogonal (p. 274): um vagão-correio habilita cargas de correio, um vagão-restaurante habilita certo tipo de missão. Abre opções em vez de multiplicar o rendimento.

### 5.2 Mapa de fontes e sumidouros

Aplicando a regra dos dois adesivos (p. 243) aos itens do jogo:

| Item | Fontes | Sumidouros |
|---|---|---|
| Moeda | passageiros entregues; cargas entregues; missões (curtas e encadeadas); marcos de linha (primeira vez em cada estação ou linha); conversão de peças repetidas | caixas sorteadas; compra direta de peça específica (mais cara); cosméticos (pinturas, nomes, placas); manutenção leve; desbloqueio de linhas; sumidouros de fim de jogo (5.5) |
| Peça (vagão, locomotiva) | caixa sorteada; compra direta; recompensa fixa de marco | compor o trem (uso); peça repetida convertida em moeda ou em outro bem |
| Caixa | compra com moeda; recompensa de arco de missão | abertura |

Observações:

- **Mais sumidouros do que fontes** para a moeda, como o autor sugere (p. 243). Hoje o desenho declarado tem um sumidouro só (caixas); é pouco, e é exatamente a situação em que a moeda sobra quando a coleção fecha.
- **Missões pagam em proporção diferente dos serviços** (Otimização, p. 246): serviço regular paga pouco e sempre; missão paga mais, mas pede algo específico (levar tal carga a tal estação, completar a linha Paranaguá–Curitiba sem pular parada). Isso dá ao jogador algo a otimizar sem criar segunda moeda.
- **Imposto embutido** (p. 376–377): uma manutenção pequena e proporcional à distância rodada é um sumidouro que nunca se esgota. Precisa ser leve e bem enquadrada (carvão, diesel, revisão), porque num jogo ocioso de fundo um custo punitivo vira bloqueio. Apresentar como bônus de "trem revisado" em vez de penalidade de "trem desgastado" segue a lição do World of Warcraft (p. 311).
- **Enquadramento narrativo** (p. 275–276, 288–289): a moeda, as caixas e as raridades devem ter nomes do mundo ferroviário (remessa do fabricante, vagão recuperado do pátio, edição histórica). Para Tamer, uma lista de recompensas já conta que jogo é aquele.

### 5.3 Dimensionar ganhos e preços pelo tempo ativo

O livro não dá números. O método que se extrai dele:

1. **Fixar o ganho-base por tempo.** Como a viagem é proporcional à distância (20 s por km), pagar por quilômetro entregue torna o rendimento por minuto constante e previsível. Hipótese de partida: ganho-base tal que uma hora ativa de serviço regular renda um número redondo (por exemplo, 60 moedas por hora, uma por minuto, que é fácil de raciocinar).
2. **Equilibrar "para alguém"** (p. 378). Definir o jogador de referência: um amigo que deixa o jogo aberto num dia de trabalho, digamos 3 a 5 horas ativas. Todas as contas partem dele; depois conferir o que acontece com quem deixa 8 horas e com quem deixa 1 hora.
3. **Escrever os preços como tempo** (p. 379). Decidir primeiro a cadência desejada e só depois converter em moeda. Hipóteses de partida:
   - primeira caixa nos primeiros 10 a 15 minutos, de preferência como recompensa do primeiro arco de missão (Javier, p. 282: nos primeiros minutos é preciso sentir progressão);
   - caixa comum a cada 45 a 60 minutos ativos de serviço regular, um pouco mais rápido para quem faz missões;
   - com isso o jogador de referência abre de 3 a 6 caixas por dia, o que dá cadência de recompensa sem tornar a abertura banal (Kelly, p. 328: se tudo vem fácil, a vontade some).
4. **Missões como bônus, não como base.** Elas devem acrescentar algo como um quarto a metade sobre o ganho regular, para que valha a pena prestar atenção sem que ignorá-las seja punição. Esse intervalo é hipótese minha.
5. **Tetos nas fontes** (p. 245–246). Como não há progresso offline e a carga do processador não rende, o tempo ativo já é o teto natural. Vale conferir se alguma fonte escapa dele: missão repetível sem limite, conversão de repetidas que devolve demais, marco que pode ser refeito.
6. **Errar para o lado do preço, não do corte de ganho.** Pela assimetria da p. 377 e pelo aviso da p. 318 (mexer em pagamento irrita quem já joga), começar com ganhos um pouco contidos e, se for preciso ajustar, preferir subir ganho, baixar preço **com aviso** ou acrescentar sumidouro. Evitar reduzir uma fonte que os amigos já conhecem. Atenção também à p. 367–368: baixar preço sem aviso frustra quem acabou de comprar.
7. **Simular antes** (p. 244, 249). Uma planilha ou um script de Monte Carlo com as taxas de sorteio e os preços responde, antes de qualquer amigo jogar: em quantas horas ativas o jogador de referência completa cada raridade? Qual é o saldo de moedas no dia 1, 7 e 30? Em que dia a coleção fecha?

### 5.4 Curva de progressão

- **Esforço crescente por degrau** (Scaling progression, p. 271). O livro não dá fórmula. Num sistema de coleção por sorteio, a própria matemática já produz isso: as últimas peças demoram muito mais que as primeiras (este é um raciocínio meu, o problema clássico do colecionador de cupons, não está no livro). O risco, portanto, é a curva natural ser **íngreme demais no fim**, não plana.
- **Suavizar a cauda** com os recursos que o autor defende: sorteio pseudoaleatório que evita sequências ruins (p. 262, 273); alguma forma de reduzir os extremos sem eliminar a sorte, na lógica da vantagem do D&D (p. 274) — por exemplo, sortear duas peças e entregar a que o jogador ainda não tem; e um caminho determinístico caro (compra direta) para a peça que não sai.
- **Textura, não rampa lisa** (p. 277–278, 318). Alternar trechos de acumulação com momentos de pico: um arco de missões que culmina numa caixa melhor, a conclusão de uma linha real, a primeira locomotiva rara. Depois do pico, um trecho de alívio em que o jogador simplesmente roda com a peça nova. A variedade de trechos (1 a 7 minutos) já dá um pouco disso no laço central.
- **Recompensa também na "falha"** (p. 272, 278, 318). No gacha, a falha é a peça repetida. Ela precisa render algo sempre, senão a abertura vira o "falhou por completo" que o autor diz não ter interesse emocional.
- **Ciclo de vida** (p. 264–270). Descoberta e aprendizado: o jogo tem de começar a rodar sozinho, sem telas de introdução, e mostrar em minutos por que vale deixar aberto. Engajamento: dar ao jogador algo a afinar (composição do trem, escolha de missão) para que o laço não pareça já todo entendido. Reengajamento: quem ficou semanas sem abrir deve encontrar tudo onde deixou, sem tutorial de novo e sem pilha de avisos.

### 5.5 Evitar que as moedas sobrem quando a coleção completar

Este é o risco estrutural: um recurso cujo único uso acabou fica sem valor (p. 246), e uma moeda com uma só utilidade repete o caso que Kaczmarczyk‑Smith conta, em que o problema era falta de usos e não de escassez (p. 379). Opções tiradas do livro, da mais barata à mais cara de implementar:

1. **Sumidouros que não se esgotam.** Cosméticos (pinturas, numeração, nomes de trem, placas de estação) e a manutenção leve. Terranova lembra que mostrar o que se desbloqueou é marcador de progresso (p. 333).
2. **Destino útil para a peça repetida.** Converter em moeda (devolvendo só uma fração, para não fechar um circuito inflacionário) ou em um segundo bem com uso próprio, por exemplo "peças de oficina" que restauram ou repintam o que já se tem. Um segundo recurso só se justifica se tiver função claramente diferente (Kelly, p. 327).
3. **Prestígio ortogonal** (p. 275). Exemplo: "aposentar" uma locomotiva para o museu em troca de um selo visual e uma variante histórica, ou recomeçar uma linha em modo "serviço de época". É voluntário, sinalizado visualmente e não aumenta o rendimento.
4. **Novo uso para a moeda em estágio avançado** (p. 319): encomendar uma peça específica à fábrica por preço alto; financiar melhorias de estação que só mudam o cenário.
5. **Moeda com sabor para eventos** (p. 311). Se um dia houver temporadas (uma linha nova, um mês temático), usar uma ficha própria do evento, gasta só no catálogo do evento, para não injetar oferta na moeda principal.
6. **Conteúdo novo como sumidouro.** Cada linha real acrescentada pode trazer suas peças próprias. É o mais eficaz e o mais caro; o autor lembra que conteúdo é o pivô de maior custo (p. 321) e Fuchsberger descreve a esteira de conteúdo (p. 254).

O que **não** fazer para resolver a sobra: cortar ganhos de quem completou, ou instituir teto baixo de carteira sem aviso. Um teto de capacidade é ferramenta legítima segundo o livro (p. 245, 319), mas precisa ser visível e ter enquadramento.

Indicador para acompanhar: a **razão produção/consumo** da moeda por jogador ao longo do tempo (p. 378). Se ela fica muito acima de 1 durante dias, a moeda está sobrando.

### 5.6 Validar com poucos jogadores (amigos)

Com meia dúzia de amigos não há significância estatística; o livro admite que testar com concessões ainda serve para pôr o projeto no rumo (p. 299). O que dá para fazer:

1. **Escrever antes "como é o bom"** (p. 293, 305). Três ou quatro expectativas com número, definidas antes de entregar a versão. Exemplos: "a primeira caixa sai em até 15 minutos ativos"; "depois de uma semana, ao menos metade ainda abre o jogo"; "ninguém fica com mais de N caixas de saldo parado"; "ninguém completa a coleção antes de X horas ativas".
2. **Validar em camadas** (p. 304–306). Primeiro só o laço: viajar, entregar, ganhar moeda, gastar em algo. Depois acrescentar caixas e raridades. Depois missões. Pouca novidade entre um teste e outro, e uma mudança por vez (p. 296, 309).
3. **Registro local de economia.** Um arquivo de log no próprio computador do jogador, que ele envia se quiser: moedas ganhas por fonte, gastas por sumidouro, saldo ao fim de cada sessão, minutos ativos, caixas abertas e o que saiu, percentual da coleção. É o equivalente doméstico das "percepções do jogador" (p. 308) e permite calcular a razão produção/consumo. Sem coleta automática e sem dados pessoais: o autor destaca a responsabilidade com privacidade (p. 298), e o jogo roda enquanto a pessoa trabalha.
4. **Qualitativo estruturado** (p. 297–298). Poucas perguntas fixas em escala de 4 pontos (que obriga a tomar partido) e a pergunta de recomendação: "você indicaria a um amigo?". Perguntas úteis: a caixa pareceu cara, justa ou barata? Em algum momento você teve moeda e nada que quisesse comprar? Em algum momento quis algo e achou que nunca ia sair?
5. **Observar em vez de explicar** (p. 304–305). O autor desaconselha conduzir o próprio teste por causa do viés. Entre amigos isso é inevitável, mas dá para reduzir: entregar o jogo sem explicar nada e pedir um relato ou uma gravação curta dos primeiros 15 minutos. O que eles entenderam errado é o dado.
6. **Atenção ao silêncio** (p. 315). Amigo que reclama está engajado. Amigo que parou de comentar e de abrir é o sinal que importa; perguntar a ele diretamente.
7. **Tratar o retorno como diagnóstico** (p. 322, 374). Anotar o problema que relatam; desconfiar da solução que sugerem.
8. **Pré-morte** (p. 313). Antes de distribuir: "daqui a um mês ninguém mais abre o jogo; por quê?". Candidatos óbvios: caixas demoram demais; só saem repetidas; coleção fecha em três dias; moeda sobra; atrapalha o trabalho.
9. **Segundo teste antes de descartar** (p. 299). Se uma ideia vai mal, um ajuste e mais uma rodada, com a melhora mínima definida de antemão. Se não melhorar, cortar.
10. **Mudar com cuidado** (p. 323–324). Avisar os amigos quando preços ou taxas mudarem, explicar o porquê e compensar quem gastou sob a regra antiga.

### 5.7 Lista de verificação ética para as caixas sorteadas

Sem dinheiro real, o que o jogador gasta é tempo e atenção, e Kelly lembra que essas também são economias (p. 326). O autor diz que o critério é a percepção do jogador, não a intenção (p. 372). Itens derivados do capítulo 13:

- [ ] **Taxas de sorteio visíveis** em cada tipo de caixa, antes de comprar (p. 371, 373).
- [ ] **Preço visível desde o início**, na mesma moeda de tudo, sem custo revelado só na confirmação (p. 374).
- [ ] **Uma camada de sorteio só.** Nada de precisar tirar um item para ganhar o direito de sortear outro mais raro (p. 371).
- [ ] **Valor equivalente a cada abertura** (p. 371): toda caixa entrega algo aproveitável; a repetida tem destino útil.
- [ ] **Ninguém fica sem a peça para sempre** (p. 371). Garantia após N aberturas, proteção contra repetidas ou compra direta. Se houver garantia, ela é **declarada** na tela, para que a pseudoaleatoriedade não seja percebida como manipulação escondida.
- [ ] **Nada essencial atrás do sorteio** (p. 367, 369–370). Rodar o trem, acessar as linhas e progredir não dependem do que sai na caixa.
- [ ] **Sem pressa fabricada** (p. 370). Nada de caixa "só hoje", cronômetro regressivo ou peça que some. Se houver evento temporário, as peças voltam a ficar acessíveis depois, por outro caminho declarado. Cuidado ao prometer "exclusivo": Terranova observa que relançar o que era exclusivo mina a confiança (p. 334); o mais simples é não prometer exclusividade.
- [ ] **Sem alimentar a ilusão de controle** (p. 371–372). Evitar animação de "quase saiu", escolha falsa entre caixas idênticas ou qualquer sinal de que a habilidade ou o momento influenciam o resultado. A animação de abertura pode ser bonita; não pode mentir.
- [ ] **Sem bloqueio para vender alívio** (p. 370–371). Não criar espera ou energia cuja saída seja gastar moeda.
- [ ] **O jogo não cobra presença.** Nada de sequência diária que se perde, notificação insistente ou recompensa que expira. O jogo vive enquanto a pessoa trabalha; respeitar o tempo e o compromisso dela (p. 370) aqui significa não disputar a atenção.
- [ ] **Sem conversão para fora** (p. 371). Peças e moedas não têm valor de saque. Se algum dia houver troca entre amigos, reavaliar este item.
- [ ] **Mudanças anunciadas** (p. 367–368, 323). Alteração de preço ou de taxa é avisada com antecedência e, se piorar para o jogador, compensada.
- [ ] **Teste da percepção** (p. 372). Perguntar aos amigos, com estas palavras: "alguma coisa nas caixas pareceu injusta ou feita para te prender?". Se alguém disser que sim, tratar como defeito, independentemente da intenção.

---

## 6. O que não se aplica

Para não importar por engano:

- **Tudo sobre dinheiro real.** Compras no aplicativo, moeda paga (Hard Currency) versus moeda ganha, pacotes e ofertas de loja, preço comparado ao cafezinho, assinatura, passe de temporada pago, "Pioneer Pass", anúncios recompensados. No capítulo 13, as armadilhas de preço (cosmético de 20 dólares, barreira paga) só valem aqui por analogia, trocando dinheiro por tempo.
- **Pagar para vencer e pagar para não jogar.** Não há pagamento nem competição. A observação de que jogos ociosos toleram "pular o jogo" (p. 249) refere-se a pagar para pular; não é um aval para progresso offline, que o dono já excluiu.
- **Métricas comerciais.** LTV, ROAS, custo por instalação, custo por clique, taxa de conversão a pagante, recompra, TAM/SAM/SOM, ranking de loja, Metacritic, cobertura de imprensa, análise de sentimento em redes. A retenção D1/D7/D30 e o NPS servem como **ideia** (as pessoas voltam? recomendariam?), não como meta estatística com meia dúzia de amigos.
- **Aquisição paga e testes de trailer.** Campanhas de baixo custo, teste A/B por plataforma de anúncios, lançamento suave, "Launch & Scale".
- **Superfãs versus baleias, front-loading, funil de conversão.** É discussão de receita.
- **Operação ao vivo de estúdio.** Calendário de temporadas, gestão de comunidade, atribuição por evento, esteira de conteúdo. Só a ideia de moeda de evento isolada (p. 311) é aproveitável, e apenas se um dia houver eventos.
- **Multijogador e comércio entre jogadores.** Razão produção/troca, especialização e comércio (economia de Robinson Crusoé), disparidade regional, massa crítica de jogadores, bots, divisão do público por nível de poder, ranking competitivo.
- **Dificuldade por habilidade.** Curva de aprendizado de combate, chefes, o fator de frustração do "quase ganhei", reforços e reduções de personagens, dificuldade dinâmica. O CPU Express não tem falha por habilidade; do conceito de textura emocional aproveita-se só a alternância entre acumulação, pico e alívio.
- **Mecânica de energia como ferramenta.** O autor a considera legítima em certos contextos; aqui o tempo de viagem já regula o ritmo e uma segunda trava seria só atrito.
- **Regulação e conformidade.** Digital Markets Act, Online Safety Act, Digital Fairness Act, código britânico de design apropriado à idade, verificação de idade, normas de publicidade, lei japonesa de 2012. Não incidem num jogo gratuito sem compras distribuído entre amigos; servem apenas como indicação do que a sociedade já considerou abusivo.
- **Pivôs de negócio.** Monetização, público-alvo, plataforma; discussão sobre encerrar o projeto por métricas de investimento.
- **Entrevistas, na parte de carreira e de mercado.** Processo de pitch para publicadoras, mercado hipercasual, trabalho por encomenda, dados sobre o tamanho da indústria.
