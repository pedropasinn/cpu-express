# Missões — primeira rodada de conteúdo

Proposta e implementação do Agente 3, 15/11/2026. **Moedas e pagamentos não estão implementados.** O código acrescenta 11 objetivos consultáveis em `simulation::objectives::MISSIONS`, calculados a partir de estado durável v4, sem botão de resgate, efeito colateral ou evento que conceda novamente. Os três `OBJECTIVES` antigos e suas pinturas permanecem intactos. As telas ainda precisam consumir a nova lista.

Os nomes de missões e contratos são conteúdo `kind: fictional`. Não descrevem serviços históricos nem comprovam embarque atual nas estações da linha real. Valores abaixo são hipóteses para Pedro confirmar, apoiadas nas notas locais de Oscar Clark e detalhadas em [ECONOMIA_PROPOSTA.md](ECONOMIA_PROPOSTA.md).

## Objetivos implementados

“Moedas” nesta tabela significa **pagamento futuro, uma vez por identificador**, nunca conversão dos pontos atuais. Os itens mencionados já são concedidos pelo domínio antigo; o objetivo apenas apresenta o mesmo marco.

| ID | Missão e requisito | Fonte durável v4 | Recompensa proposta | Ritmo e motivo |
| --- | --- | --- | --- | --- |
| `ce.passengers-20` | Companhia de viagem: entregar 20 pessoas | `passengers_delivered` | 15 moedas + postal existente; Elétrica da Serra e Automotriz dos Campos já liberadas pelo postal | Primeiro contato com serviço; não contar embarques como entrega. |
| `ce.passengers-100` | Plataformas acolhedoras: entregar 100 | Mesmo contador | 25 moedas | Incentiva composição de passageiros sem exigir modelo raro. |
| `ce.passengers-300` | Encontros sobre trilhos: entregar 300 | Mesmo contador | 40 moedas | Marco de várias sessões; sem sequência diária. |
| `ce.cargo-20` | Correio sobre trilhos: entregar 20 unidades de qualquer carga | `cargo_delivered` | 15 moedas + postal existente; Diesel do Porto, Manobreira do Pátio e Vapor das Pedreiras já liberadas pelo postal | Mantém nome legado: o contador inclui todos os tipos, não somente encomendas. |
| `ce.cargo-100` | Abastecendo o circuito: entregar 100 unidades | Mesmo contador | 30 moedas | Cerca de 9–25 contratos com lotes atuais de 4–12; composição importa. |
| `ce.contracts-10` | Dez entregas bem cuidadas: concluir 10 contratos | `contracts_completed` | 25 moedas | Contratos, não unidades; recusar, cancelar ou devolver não conta. |
| `ce.stations-4` | Quatro lugares no caderno: conhecer as quatro estações Aurora | `discovered_stations` reconstituído da jornada | 20 moedas | Conta lugares distintos, inclui Vila Aurora inicial; conclui na terceira chegada. Não muda o objetivo antigo de quatro chegadas. |
| `ce.branches-1` | Além do circuito: descobrir um ramal | `branch.discoveries` | 20 moedas + postal de ramal existente | Após liberar ramais no primeiro circuito; conta na chegada ao destino, após 90 s de ida. |
| `ce.branches-2` | Entre a serra e o mar: descobrir os dois ramais | Mesmo conjunto | 30 moedas + segundo postal existente | Cada passeio completo leva 188 s ativos, além da espera até a partida em Vila Aurora. Repetir um destino não aumenta o conjunto. |
| `ce.postcards-2` | Álbum de serviços: obter os dois postais de serviço | `collection` | 20 moedas | Recompensa variedade: passageiros **e** carga; não contar postais de ramal novamente. |
| `ce.liveries-4` | Quatro cores de viagem: possuir as quatro pinturas | `owned_rewards` | 25 moedas | Inclui a pintura inicial; doze chegadas liberam a quarta. Equipar outra pintura não altera o progresso. |

Somatório destes 11 pagamentos únicos: **265 moedas**, distribuídas ao longo de marcos diferentes. Não entram no rendimento sustentável por hora. A coleção implementada é de postais e pinturas: o v4 não tem inventário de vagões/motores. Equipamento atual não é substituto para propriedade.

Os testes verificam limites antes/no/depois da meta, identificadores únicos, descoberta distinta, capacidade dos contratos e reconstrução dos mesmos resultados após serialização/deserialização de um `WorldCheckpoint` v4 em memória e restauração de jornada/serviços. Consultar o objetivo repetidamente não modifica saldo, serviços, itens ou histórico. O teste não simula falha de disco nem prova pagamento em moeda, que inexiste.

## Cargas por tipo — desenho aguardando contador durável

| ID proposto | Requisito | Vagão compatível | Moedas propostas |
| --- | --- | --- | --- |
| `ce.parcels-24` | Entregar 24 encomendas | Carga ou Correio e bagagem | 20 |
| `ce.supplies-24` | Entregar 24 suprimentos | Carga | 20 |
| `ce.liquids-24` | Entregar 24 líquidos | Tanque | 20 |
| `ce.ore-24` | Entregar 24 minério | Gôndola de minério | 20 |
| `ce.grain-24` | Entregar 24 grãos | Graneleiro | 20 |
| `ce.timber-24` | Entregar 24 madeira | Plataforma de toras | 20 |
| `ce.chilled-24` | Entregar 24 refrigerados | Frigorífico | 20 |

Metas de 24 representam 2–6 entregas, média nominal de três lotes de oito. Não há validade/perecimento punitivo. Especializadas aparecem como convite somente após possuir o vagão compatível; equipar é uma ação do jogador. As ofertas do Agente 1 já dependem de capacidade equipada e preservam o comportamento de composições antigas.

**Não implementadas:** `cargo_delivered` é agregado; o único contrato conserva seu tipo apenas até ser substituído; os 32 bilhetes não guardam tipo nem a história completa. Contar o contrato visível ou reconstruir a partir do diário faria o progresso regredir/esquecer entregas. Pedido: contador monotônico por `CargoKind` atualizado uma vez na mesma transação da entrega; migração começa os tipos em zero, mantém o total antigo e não inventa uma distribuição retroativa. Acrescentar objetivos por tipo depois da migração aprovada.

## CON-02 — contratos especiais de passageiros

Implementado apenas `PassengerCharter::ALL` e `preview(&ServiceState)`: calcula lugares livres, déficit e vagão temático ausente sem alterar passageiros, composição ou contrato. `fits()` significa **capacidade nesta prévia**, não aceite autorizado. Na interface futura, “Disponível após integrar contratos especiais”; não mostrar um botão funcional de aceite nesta fonte.

| Tipo fictício | Grupo | Condição | Percurso proposto | Pagamento futuro |
| --- | --- | --- | --- | --- |
| Visita de estudantes (`school-visit`) | 12 | Lugares livres em qualquer composição de passageiros | Próxima estação que aceite passageiros | 10 moedas dentro do bônus de serviço |
| Passeio de observação (`scenic-party`) | 8 | Panorâmico equipado e 8 lugares livres | Duas próximas paradas válidas, conforme sentido | 10 moedas dentro do bônus de serviço |
| Mesa entre estações (`dining-trip`) | 6 | Restaurante equipado e 6 lugares livres | Próxima parada válida | 10 moedas dentro do bônus de serviço |

O restaurante atual tem oito assentos, sem conforto/refeições simulados. A prévia soma assentos livres do trem; **não reserva um compartimento por grupo**, pois o estado atual não vincula passageiros a vagões. Se Pedro quiser essa exigência, será outro contrato de capacidade. Nenhum grupo é maior do que a capacidade do vagão temático que o convida.

Fluxo futuro: oferta persistida sem prazo → aceite validado pelo `World` → aguarda origem com lugares → grupo inteiro embarca, sem expulsar passageiros regulares → desembarca no destino → transação única de conclusão. Reservar lugares para o grupo antes do embarque regular nessa origem; a prioridade precisa ser mostrada. Redução de capacidade respeita todos os passageiros a bordo. Cancelar vazio não pune; cancelar embarcado oferece retorno à origem, conserva lugares ocupados e não paga bônus. Não reutilizar `CargoContract`, `credits`, `PassengerGroup.count` ou postais como bits ocultos de estado.

Campos futuros: ID e tipo, linha, origem/destino estáveis, quantidade, estado, vínculo com grupo/reserva, contador de concluídos e marca de transação. `PassengerGroup` hoje não distingue especial de regular; sem esses campos, não há como restaurar ou pagar honestamente. A linha real continua rascunho e exige confirmação de embarque nas paradas. Ramais Aurora continuam sem serviço de passageiros/carga.

## Ritmo e pagamento futuro

Escala vigente: 20 s/km de deslocamento ativo. Paranaguá–Curitiba: 109,3 km × 20 = **2.186 s = 36 min 26 s** sem paradas; com 13 serviços hipotéticos de oito segundos, **38 min 10 s**. Não é tempo real de operação da ferrovia. Trechos individuais ficam aproximadamente entre um e seis minutos nessa linha, dentro da meta de 1–7 minutos. Não aplicar a escala real aos comprimentos históricos de `aurora-v1`.

Uma missão recorrente simples, **Turno tranquilo**, pagaria 15 moedas por 15 minutos ativos, qualquer composição. Um bônus opcional **Serviço acompanhado** pagaria 10 por 30 minutos ativos mais 20 passageiros **ou** oito unidades entregues desde o ciclo anterior. Contrato especial pode substituir esse requisito, sem somar outro prêmio: teto de 20 moedas/h em bônus. Não há resgate manual, prazo, reinício diário ou acúmulo offline. Esta repetição exige baseline e índice de ciclo no save e **não foi implementada**.

Primeira chegada: propor 45 moedas únicas além da pintura Azul do Porto, para a primeira caixa de 60 caber em até 15 minutos ativos com Turno tranquilo. Este impulso não se repete. O ritmo dos totais de passageiros depende de lotação/destinos e o das cargas de equipamento/aceite; não prometer minutos exatos para os marcos de 100/300 antes de medir essas composições. Os 140 de cargas por tipo e os 265 acima são estoque finito de bônus, não salário permanente.

Na integração econômica, guardar IDs pagos e versão da regra; verificar requisito e atualizar contador, inventário e pagamento na transação do domínio, salvando antes de avisar. Um `complete: true` derivado não é autorização para pagar a cada leitura. Proposta de migração: preservar todas as propriedades já liberadas, marcar marcos anteriores como conhecidos sem moedas retroativas; Pedro precisa confirmar essa escolha. Não alterar preços/metas silenciosamente depois da validação com amigos.
