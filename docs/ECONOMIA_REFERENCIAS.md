# Economia do jogo — referências iniciais

Indicadas pelo Pedro em 2026-11-15 como ponto de partida para a economia de moedas e caixas. Lidas pelo Claude Code na mesma data; o que segue são notas próprias, não cópia dos textos.

| Referência | O que tem de útil | Limite da leitura |
| --- | --- | --- |
| Lost Garden, "Value Chains" — https://lostgarden.com/2021/12/12/value-chains/ | Método completo para desenhar e depurar uma economia de fontes e sumidouros. | Lido por inteiro. |
| Machinations, modelos de economia — https://machinations.io/selected-models/game-economy | Sete diagramas de exemplo (ganhar-gastar-reinvestir, coletar-fabricar-vender, reciclagem por desmonte) e conselhos de balanceamento. | Só a página de índice; os diagramas em si não foram abertos. |
| Machinations, tutorial "Loot & Craft" — https://machinations.io/learn/loot-craft | A trilha tem lições sobre itens repetidos, reembolso de repetidos, caixas e pacotes e "modelar uma caixa de gacha". | Só a página de abertura foi lida; o conteúdo dessas lições **não** foi consultado. |

## O método das cadeias de valor, em resumo

- Uma **cadeia de valor** é uma sequência de ações em que cada passo só importa porque habilita o seguinte, até chegar a uma **âncora**: a necessidade real do jogador que dá sentido a tudo antes dela (expressão pessoal, domínio, coleção, companhia).
- Cada passo tem ação, entradas e saídas. As peças são **fontes** (criam recurso), **transformações**, **reservas** (acumulam) e **sumidouros** (destroem).
- **Como construir:** escrever a cadeia em uma linha, do começo à âncora; detalhar cada nó; manter a cadeia linear; conferir se todo nó serve à âncora.
- **Como diagnosticar:** "isso parece inútil" indica elo quebrado; "não sei por que estou jogando" indica âncora fraca ou invisível. Falhas econômicas típicas: recurso sobrando sem ter onde gastar, recurso morto porque duas fontes alimentam a mesma entrada e só a melhor é usada, escassez que vira espera dolorosa, repetição que cansa.
- **Como balancear:** classificar cada fonte (limitada, por tempo, por esforço, de investimento, aleatória) e cada sumidouro (único, repetível, exponencial); casar fonte com sumidouro de força igual ou maior; nunca deixar fonte exponencial sem teto; manter os sumidouros um pouco maiores que as fontes; em jogo contínuo, equilibrar por janela de tempo.
- **Fontes aleatórias** (é o caso das caixas): calcular pela média e depois conter a variação com garantia após N tentativas ou sorteio sem reposição.
- **Várias cadeias:** cadeias paralelas são o padrão seguro; evitar que duas fontes diferentes caiam na mesma entrada sem distinção.
- **Promessa ao jogador:** mostrar cedo aonde a cadeia leva, para ninguém sair antes de ver o porquê.

Dos modelos da Machinations: começar com uma moeda só e fazer o ciclo dela funcionar antes de acrescentar outra; acompanhar o saldo ao longo do tempo para ver se estabiliza ou dispara; desmontar itens repetidos devolve recurso e cria ao mesmo tempo um sumidouro e uma fonte.

## Primeira aplicação ao CPU Express (rascunho para discussão)

Âncoras candidatas: **coleção** (ter e ver na barra um trem que é seu) e **expressão** (montar a composição com a sua cara). A cadeia principal, em uma linha:

`viajar com tempo ativo → chegar às estações → cumprir serviços e missões → +moedas → abrir caixas → vagões e locomotivas → montar e exibir o trem (âncora)`

- **Fonte por esforço:** moedas de missões e serviços, limitadas pelo tempo ativo; a carga do processador não rende nada.
- **Fonte aleatória:** o conteúdo das caixas, contida por garantia e por não repetir enquanto houver item novo na faixa.
- **Sumidouro repetível:** preço das caixas. Precisa de um sumidouro que continue existindo depois de a coleção completar, senão as moedas sobram: melhorias, pinturas ou caixas temáticas novas.
- **Itens repetidos:** não podem ser prêmio perdido; viram moedas ou peças de melhoria.
- **Risco a evitar:** se um vagão melhor render mais moedas por viagem, a fonte vira investimento exponencial; precisa de teto.
- **Uma moeda só no começo.**

Regras que não mudam: nenhuma compra com dinheiro real, probabilidades visíveis, e sorteio gravado no save antes de a caixa abrir.
