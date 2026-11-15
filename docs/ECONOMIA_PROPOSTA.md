# Economia proposta — moedas de jogo e remessas

15/11/2026, Agente 3. **Documento de decisão, sem saldo, caixas ou campos novos implementados.** Hipóteses para Pedro testar; não são números prescritos pelo livro nem resultados de teste com jogadores.

## Base e experiência pretendida

Base principal: Oscar Clark, *Playing with Balance: Game Economy Design*, por meio das três notas locais disponíveis no início desta rodada: [A](economia/notas-A.md), [B](economia/notas-B.md), [C](economia/notas-C.md). Foram consultados conceitos e aplicações ao CPU Express; não houve nova consulta externa nem alegação de releitura do livro original. A: autonomia, coleção, minuto ativo e ralos que não pareçam punição; B: utilidade própria de cada peça, coleção em vez de substituição e revelação compreensível; C: orçamento de fontes/sumidouros, teste de extremos e transparência do sorteio. As notas distinguem interpretação de trechos do livro: **percentuais, preços e garantias abaixo são autoria desta proposta**.

Complemento: [ECONOMIA_REFERENCIAS.md](ECONOMIA_REFERENCIAS.md), com notas locais de Lost Garden, *Value Chains*, e das páginas iniciais Machinations *Game Economy* e *Loot & Craft*. Não foram consultados os diagramas/lições ausentes das notas. O método usado é explicitar ações, reservas, fontes e destruição de recursos até a motivação final.

**Cadeia principal em uma linha:** `tempo ativo → viagens e missões → moedas → remessa escolhida → peça com papel próprio → montar e contemplar meu trem (âncora: coleção e expressão pessoal)`.

Caminho paralelo determinístico: `tempo ativo → missões → moedas → encomenda direta da peça → mesma âncora`. Descoberta de estações/postais permanece uma lembrança durável, sem preço nem consumo. Moedas não medem prestígio; pontos atuais não viram saldo.

Pessoa de referência: amigo com três horas ativas por sessão, deixando o jogo acompanhar o trabalho. Conferir também uma e oito horas; “ativa” exclui pausa, ocultação, minimização e fechamento. Sem dinheiro real, anúncios, saque, energia, sequência diária, ofertas que expiram ou caixa automática interrompendo o trabalho. Linhas e capacidade básica não dependem de sorte. Raridade não multiplica moedas nem velocidade lógica.

## Fontes, transformações, reservas e sumidouros

| Nó | Classe | Regra proposta |
| --- | --- | --- |
| Turno tranquilo | Fonte recorrente limitada por tempo ativo | 15 moedas a cada 15 min ativos; independente de CPU/composição. É uma missão simples, não pagamento por frame. |
| Serviço acompanhado | Fonte recorrente por esforço, com teto temporal | 10 moedas por ciclo de pelo menos 30 min + 20 passageiros ou oito unidades entregues desde a última concessão. Bônus máximo de 20/h; sem prêmio paralelo por unidade. |
| Contrato especial | Alternativa ao requisito da fonte anterior | Um contrato especial concluído substitui a entrega regular no ciclo. Não paga novamente fora do teto. |
| Primeira chegada | Fonte limitada, única | 45 moedas, preservando a pintura já concedida. |
| Marcos de missões | Fonte limitada, por esforço/descoberta | 265 moedas nos 11 objetivos + 140 nas sete metas por tipo futuras; ver MISSOES. |
| Carteira | Reserva | Inteiro de moedas, sem expiração ou juros. Limites numéricos protegidos, não teto baixo para forçar gasto. |
| Comprar remessa | Sumidouro repetível | Consome preço integral antes da revelação. Compra e resultado são uma transação persistida. |
| Abrir remessa | Transformação + fonte aleatória de peça | Uma peça do conjunto publicado; probabilidades e regra de garantia visíveis. A embalagem é consumida, não há segundo sorteio. |
| Inventário | Reserva de duráveis | Peça própria não some ao equipar outra. Equipar é uso, **não** sumidouro. |
| Repetida | Transformação, destruição da cópia excedente e fonte parcial de moeda | 20% do preço efetivamente pago volta, uma vez, somente se toda a raridade sorteada já estiver possuída. |
| Encomenda direta | Sumidouro único por peça | Preço fixo por raridade; caminho para completar a coleção sem depender de sorte. |
| Pintura/placa nova | Sumidouro único por cosmético | A partir de 120 moedas, sem substituir recompensas existentes nem afetar renda. Precisa de conteúdo próprio. |
| Selo de apoio ao museu | Sumidouro repetível opcional | 120 moedas por selo numerado durável, sem bônus econômico e sem apagar peças; depende de Pedro aprovar o valor expressivo. |

Não adotar manutenção obrigatória sugerida como opção nas notas C: no modo de companhia ela introduziria cobrança. Mais sumidouros não significam obrigar gasto. Se os cosméticos acabarem e ninguém quiser selos, haverá sobra; aceitar esse estado até ampliar usos, sem destruir saldo ou cobrar ausência. Conteúdo finito não sustenta consumo infinito por si só.

## Conta por hora ativa

Regime regular, excluindo marcos únicos, sem reembolsos: `60/15 × 15 = 60 moedas/h`. Com serviços: `60 + (60/30 × 10) = 80 moedas/h` no máximo. É bônus de **33,3%**, dentro da hipótese de 25–50% sugerida nas aplicações das notas C. Um contrato lento pode levar mais de 30 min: o ciclo aguarda as duas condições, sem dívida, prazo ou perda de progresso. Reinicia os baselines ao concluir; não pode guardar entregas de um ciclo para reapresentá-las nos seguintes. Não existem duas contagens de tempo.

| Tempo ativo por sessão | Só turno | Turno + serviço no teto | Remessas gerais de 60, sem bônus único/reembolso |
| --- | --- | --- | --- |
| 1 h | 60 | 80 | 1 a 1,33 equivalentes |
| 3 h | 180 | 240 | 3 a 4 |
| 8 h | 480 | 640 | 8 a 10,67 |

Frações são poder de compra acumulável, não pedaços de caixa. A primeira chegada + primeiro turno = `45+15=60` até 15 min se já houve uma chegada. Outros marcos podem antecipar a compra; não garantir um minuto exato. A linha real, sem novo motor genérico integrado, ainda não é cenário executável para medir esta promessa.

No limite de coleção completa, remessa geral repetida devolve 12: gasto líquido 48. Então `60/48=1,25` ou `80/48≈1,67` aberturas/h, em vez de 1–1,33. **Não** multiplicar a renda por 1,25 e contar o reembolso novamente. Sem nova renda, a cadeia de compras sempre reduz moedas em pelo menos 80% do preço; não existe arbitragem positiva. Preferir encomenda/cosméticos quando a coleção estiver completa, mostrando que uma remessa já não pode dar item novo.

## Remessas e chances

Uma peça por remessa; raridade primeiro, item uniforme entre os ainda não possuídos daquela faixa. Quando a faixa estiver completa, sorteio uniforme entre todos dela e reembolso de repetida. Isso protege novidade sem transferir a chance de comum para épico. Nunca escolher faixa vazia: a versão do conjunto deve ser validada antes de vender. Novos agentes podem ampliar itens, mas não alterar automaticamente estas probabilidades.

| Remessa | Preço | Comum | Incomum | Raro | Épico | Garantia de raro ou épico |
| --- | --- | --- | --- | --- | --- | --- |
| Pátio geral | 60 | 70% | 22% | 7% | 1% | Na 10ª após nove sem raro+ |
| Cargas e oficina | 90 | 60% | 29% | 10% | 1% | Na 8ª após sete sem raro+ |
| Garagem de locomotivas | 180 | 55% | 30% | 13% | 2% | Na 6ª após cinco sem raro+ |

Todas as linhas somam 100%. Preços regulares: geral 45–60 min, cargas 67,5–90 min, garagem 135–180 min ao intervalo de 80–60 moedas/h. A remessa temática compra foco, não mais potência.

Conjuntos propostos com os itens presentes: geral = os 11 vagões e seis motores. Cargas = Carga, Tanque, Correio, Gôndola, Graneleiro, Toras, Frigorífico, Manobreira e Vapor das Pedreiras. Garagem = seis motores. Confirmar raridades antigas para tornar os conjuntos completos: Passageiros/Carga/Tanque/Vapor Aurora comuns; Dormitório/Panorâmico/Diesel do Porto incomuns; Elétrica da Serra rara. Preservar propostas do Agente 1: Correio/Gôndola/Manobreira comuns, Restaurante/Graneleiro/Toras incomuns, Frigorífico/Automotriz raros e Vapor das Pedreiras épico. Raridade estética/temática não significa aumento universal de capacidade. O vagão comum de passageiros conserva 24 lugares; o restaurante incomum tem oito e abre um convite específico.

Garantia por família de remessa, persistente entre sessões, sem expirar ao trocar de caixa. Um resultado raro ou épico zera o contador, mesmo repetido; comum/incomum incrementa. Na abertura garantida sortear apenas raro/épico com a proporção relativa publicada: geral **87,5/12,5%**, cargas **90,909/9,091%**, garagem **86,667/13,333%**. O painel mostra a chance efetiva desta abertura e “faltam N para raro ou épico”. Não usar ajuste secreto por tempo ausente, número de moedas, cliques ou CPU.

Conta analítica de conferência para intervalo até raro+: `E = [1 − (1−p)^N] / p`, com p = soma das duas faixas e N = limite. Resultado: geral **7,07** aberturas, cargas **5,51**, garagem **4,15**; são médias de ciclos até qualquer raro+, não até peça nova. A garantia corta a sequência sem raro+, mas **não promete um épico nem uma peça específica**. Encomenda direta elimina essa cauda para quem prefere guardar moedas. Sem testes de jogadores ou simulação de catálogo, não prometer prazo médio para “coleção completa”.

## Repetidos, compra direta e marcos

Reembolso de 20% do preço pago: 12 / 18 / 36 moedas nas três remessas. Igual em qualquer raridade; evita que uma conversão de épico supere o custo de abrir. Não criar segunda moeda de fragmentos. A cópia original fica intacta e equipável. Item já ganho por marco também entra como possuído na proteção contra repetidas.

Encomenda direta sugerida: comum 120, incomum 240, raro 480, épico 900 moedas. Custo em tempo regular de 60–80/h: 1,5–2 h; 3–4 h; 6–8 h; 11,25–15 h. A ordem não expira e não exige sacrificar um item. Motor raro é opção de coleção, não condição para linha real. Preservar marcos antigos como direitos adquiridos; decidir depois se novos perfis ainda recebem motores por postais ou se marcos concedem apenas o postal/moedas. Esta rodada não remove liberação alguma.

## Persistência e integração futura

Formato v4 permanece intacto. Pedir ao integrador nova versão com migração e backup, antes de ativar:

- carteira, totais ganhos/gastos e log limitado de transações com IDs monotônicos;
- inventário por ID de catálogo, preservando tudo que o perfil antigo podia equipar;
- IDs de missões pagas, versão da regra, baseline de contadores e ciclo recorrente;
- contadores de entrega por carga, sem atribuir espécie ao total legado;
- contador de garantia por família, versão do conjunto e estado determinístico do sorteio;
- compra pendente: débito, resultado já sorteado, reembolso e estado de revelação gravados juntos **antes** da animação; reiniciar apenas reapresenta o mesmo resultado;
- estado de contratos especiais/reservas e selos/cosméticos adquiridos, se aprovados.

Persistência falhou: não confirmar débito/resultado nem consumir nova aleatoriedade em tentativas de UI. Importação de backup restaura a economia inteira de um checkpoint, sem mesclar saldos. Em jogo local não prometer proteção antifraude contra edição manual, mas impedir duplicação por fluxo normal ou falha de gravação. Não reaproveitar `ServiceState::credits`: são pontos históricos verificáveis, não dinheiro gastável.

## Validação e decisões do Pedro

Confirmar: 60–80 moedas/h, impulso inicial de 45, metas/pagamentos únicos, três preços, raridades antigas, garantia por família, repetidas a 20%, encomenda direta, eventual selo de museu e tratamento de perfis já completos. Os 11 marcos implementados somam 265; sete por tipo adicionariam 140; impulso 45: **450 moedas únicas** no conjunto completo, além da renda regular. Reavaliar concentração desses bônus no começo sem cortar recompensas já pagas.

Próxima etapa autorizável pelo integrador: simulador **local** separado do produto com 1/3/8 h, com/sem serviço, migração completa e aquisição de cada item; registrar saldo e percentis de tempo até raro/épico/coleção. Depois, teste qualitativo com amigos: primeira aquisição compreensível, utilidade dos comuns, clareza da garantia e abertura que não interrompe o trabalho. Sem telemetria ou envio automático. Esta rodada conferiu aritmética e desenho; não implementou nem executou a economia, o sorteio ou uma simulação Monte Carlo.
