# Economia E1b — integração das telas

2026-11-16. Complementa `ECONOMIA_E1A.md`; os números abaixo substituem o balanceamento provisório do E1a. Domínio sem telas novas, checkpoint v5 mantido, nenhuma dependência nova.

## API para E5 e E6

Todas as ações do jogador usam `progress::economy_action(path, &mut world, action)`. O domínio valida requisitos, prepara o candidato e grava antes de publicar. `Err` conserva o estado anterior; `Ok(Some(aviso))` é sucesso com aviso de recuperação, não motivo para repetir a operação. Não chamar `record` na interface para conceder prêmio.

```rust
let economy = world.economy();
let journey = world.journey_snapshot();
let service = world.service_state();
let missions = economy.missions(journey, service);
// MissionStatus: id, title, current, goal, reward, paid, visible; claimable().
// Listar visible; claimable() dirige o aviso na doca e o botão Recolher.

EconomyAction::ClaimMission(id.to_owned())
EconomyAction::ClaimOrder { order_id }
EconomyAction::SwapOrder { order_id }
EconomyAction::BuyMuseumStamp
```

As 18 missões mantêm IDs e pagamentos de `MISSOES.md`: 265 moedas dos onze marcos e 140 das sete cargas, total 405. Consultar nunca paga. `ce.liveries-4` exige as quatro pinturas **originais**, não quatro pinturas quaisquer. As metas por espécie usam exclusivamente `cargo_by_kind`; `visible` exige posse de vagão compatível ou progresso já registrado. O título dessas sete entradas é o nome da carga; a tela compõe “Entregue 24 · [nome]” com `goal`. Novos perfis recolhem manualmente. Direitos de peças/postais continuam no fluxo de chegada do E1a.

Ordens em `economy.daily_orders`: `order.template()` informa título e tipo; `order.progress(economy, journey, service)` informa o contador limitado à meta; `order.claimable(...)` habilita Recolher. Identificar pelo **ID da instância**, não pelo modelo. Não inserir a substituta na UI: ela entra em chegada posterior. Não há calendário, vencimento, punição, sequência de dias ou reset no fechamento.

Uma ordem entra após os primeiros dez minutos ativos, depois no máximo uma a cada dez minutos, somente em chegada. Até três comuns e um telegrama. Todas guardam o baseline na entrada: serviços anteriores não cumprem uma ordem nova. O conjunto fixo fica em `economy_goals::ORDER_TEMPLATES`: 24 chegadas, 120 passageiros ou uma hora ativa pagam 10; oito contratos ou 48 unidades de uma das sete cargas pagam 15. Valores maiores que os exemplos iniciais evitam renda recorrente excessiva.

A troca é gratuita, exige estar na estação, dez minutos desde a última entrada **e** desde a última troca. Consome o mesmo intervalo de entrada, recebe baseline novo e exclui o modelo anterior. O botão pode apresentar o erro curto do domínio; a UI não deve tentar trocar por frame. Ordens mantêm o progresso se o jogador mudar a composição depois. Para gerar, exige capacidade equipada ou uma substituição de vagão possuído que preserve tudo a bordo; `ServiceState::can_carry_with` é consulta pura de capacidade, **não autoriza posse**. Essa elegibilidade conservadora pode deixar de oferecer um modelo que exigiria duas trocas simultâneas; nunca oferece um requisito impossível pela troca sugerida.

Para E2, `World::request_telegram()` pede uma oportunidade. Quando os três espaços comuns estão ocupados, uma chegada com intervalo disponível pode criar a quarta ordem. Um pedido pendente não acumula vários telegramas e não persiste leitura da rede. A criação confirmada consome o pedido; falha de gravação da chegada conserva também o pedido. O simulador de referência não pede telegramas; não há geração automática por CPU/rede nesta entrega.

## Caixas, loja e coleção

`CrateKind::ALL`, `kind.name()` e `kind.definition()?` publicam nome, preço, conjunto, pesos, versão e garantia. `definition.validate()` verifica catálogo e faixas. A UI normaliza `effective_weights(misses)` pela soma e mostra `guarantee - misses` como aberturas restantes. Conteúdo por ID está nas constantes `GENERAL_WAGONS`, `GENERAL_ENGINES`, `CARGO_WAGONS`, `CARGO_ENGINES`, `PLATFORM_WAGONS`, `PAINTS`.

| Caixa | Moedas | Comum / incomum / raro / épico | Garantia |
| --- | ---: | --- | ---: |
| Pátio geral | 150 | 70 / 22 / 7 / 1 | 10 |
| Cargas e oficina | 225 | 60 / 29 / 10 / 1 | 8 |
| Plataforma e conforto | 225 | 60 / 29 / 11 / 0 | 8 |
| Garagem de locomotivas | 450 | 55 / 30 / 13 / 2 | 6 |
| Ateliê de pinturas | 188 | 65 / 25 / 10 / 0 | 8 |

Plataforma inclui Passageiros, Dormitório, Panorâmico, Restaurante, Correio, Bicicletas e Feira: esta última fornece a faixa rara pelo tema de serviço ao público. Não há motor nessa caixa. Cargas reúne os quinze vagões com carga e cinco motores de tema de carga (Diesel do Porto, Vapor das Pedreiras, Manobreira, Articulada, Dois Horizontes). Garagem contém os dez motores; Ateliê as dez pinturas. Plataforma e Ateliê não têm épicos: o ponto percentual foi somado a raro. Nenhuma raridade foi inventada ou alterada.

`BuyCrate`, `BuyDirect` e `Reveal` continuam como no E1a. A remessa grátis segue exclusiva de Pátio, sem reembolso. Compra direta permanece 120/240/480/900; pode custar menos que uma caixa temática e deve ficar visível como alternativa garantida. Repetida restitui `paid / 5`, arredondado para baixo (Ateliê: 37). Pátio usa versão de conjunto 2 para distinguir o novo preço; resultados pendentes da versão 1 (60 moedas ou grátis) continuam válidos e **não** são recalculados. Os outros quatro conjuntos estreiam na versão 1.

`EconomyState::collection_items()` lista os 39 modelos atuais de vagões, motores e pinturas; `collection_complete()` exige todos. Postais têm sua coleção separada e não bloqueiam selos. `museum_stamps` é a quantidade, com números de 1 a N; cada compra custa 120 e incrementa um número. Exige coleção completa, saldo e ausência de remessa pendente; não dá bônus econômico. Falha de gravação não consome saldo nem número.

## Renda, migração e limites

A unidade de cálculo herdada é chegada + passageiros/4 + prêmio do contrato/20 (mínimo 2 para contrato; restos preservados). A calibração multiplica essas unidades por **0,35 moeda**, antes do bônus de máquina de ×1,00–1,30. `bonus_remainder` mantém milésimos de moeda entre chegadas; não há novo campo. A conversão final para milésimos trunca frações menores que 0,001 moeda por chegada. Boas-vindas, missões, ordens e reembolsos não recebem bônus. Valores já pagos nunca são recalculados.

`load_world` permanece leitura pura. `load_profile`, usado pelo app, trata `legacy_mission_claim_pending`: paga apenas missões completas ainda não pagas, limpa a flag na mesma gravação e fornece aviso com o valor. Mantém backup exato v1–v4 e `.previous.json`. Se não puder gravar, devolve o mundo carregado **sem pagamento**, com flag pendente e aviso para nova tentativa na próxima abertura; não substitui a viagem por um perfil vazio. Um v5 intermediário do E1a com a flag ligada recebe o mesmo tratamento. Importações antigas que passam por `import_world` conservam a flag e terão a conferência na próxima abertura por `load_profile`; E5 pode antecipá-la com `SettleLegacyMissions` via o escritor transacional e mostrar o valor.

Os campos reservados v5 bastaram. As definições das ordens agora são validadas contra o conjunto fixo (ID, meta, prêmio e chave do baseline); fixtures históricas v1–v5 permanecem intactas. As telas ainda precisam ser conectadas por E5/E6. O balanceamento e os limites da amostra estão em [economia-simulacao.md](economia-simulacao.md).

**E3:** os contadores de chegadas e tempo das ordens usam hoje o snapshot Aurora. Ao integrar troca de linha, fornecer contadores globais monotônicos ou migrar os baselines de forma equivalente; não reiniciar os intervalos ou reinterpretar progresso acumulado. **E2:** o telegrama usa os mesmos modelos/prêmios fixos e o mesmo intervalo das ordens; não editar `reward` no chamador, pois a validação do checkpoint confere a definição. Aferir separadamente o efeito da frequência de telegramas no balanceamento. Mudanças futuras nas metas/prêmios exigem preservar instâncias já salvas ou migrar suas definições.
