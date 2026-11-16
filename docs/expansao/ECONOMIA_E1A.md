# Economia E1a — contratos para integração

2026-11-16. Implementação de domínio na branch `expansao`, sem tela de loja e sem distribuição nova. Os ajustes de `PLANO_EXPANSAO.md` prevalecem sobre as propostas anteriores.

## Usar a economia

- `World::economy()` consulta carteira, inventário, histórico e compra pendente, sem mutação.
- `ItemId::{Wagon, Locomotive, Livery}(String)` separa os três espaços de IDs do catálogo. `ItemId::wagon(ServiceWagon)` e `ItemId::livery(Livery)` evitam conversões manuais; os enums também expõem `id()`. Posse é do modelo: não há contagem de exemplares nem consumo ao equipar.
- `World::set_service_composition` exige posse e capacidade para o serviço a bordo. `ServiceState::set_composition` permanece como adaptador dos marcos legados para ferramentas antigas; o aplicativo e a oficina passam pelo `World`. Não usar esse adaptador para autorizar uma compra ou equipar no produto.
- Locomotivas e pinturas continuam equipadas nas preferências visuais. A abertura, a importação e os controles conferem a posse; pinturas também são espelhadas em `service.owned_rewards` para os consumidores existentes.
- `World::set_machine_bonus(f32)` aceita o **multiplicador**, não o percentual: `1.15`, por exemplo. Limita a `1.0..=1.3`, quantiza em milésimos e trata NaN/infinito como `1.0`. Não persiste leituras nem o multiplicador; o E2 precisa realimentá-lo com a amostra válida atual.

## Transações e revelação

O caminho do aplicativo é `progress::economy_action(path, &mut world, action)`. Retorna `Result<Option<String>, String>`: `Err` significa que a operação não foi confirmada; `Ok(Some(aviso))` significa **sucesso**, com aviso de que a cópia de recuperação não foi renovada. Não repetir o débito ao receber esse aviso.

As ações públicas são:

```rust
EconomyAction::BuyCrate { kind: CrateKind::GeneralYard, free: false }
EconomyAction::BuyCrate { kind: CrateKind::GeneralYard, free: true }
EconomyAction::BuyDirect(ItemId::Wagon("mail".into()))
EconomyAction::Reveal { transaction_id }
```

1. A compra cria um candidato de `World`; debita, sorteia, incorpora a peça, aplica eventual reembolso, atualiza a garantia e registra `pending_purchase` no candidato.
2. Grava o checkpoint inteiro por `save_checkpoint`, incluindo a cópia `.previous.json`, antes de publicar o candidato em memória.
3. Só depois do sucesso a interface pode mostrar `world.economy().pending_purchase`. O resultado já contém ID da transação, caixa, versão do conjunto, item, raridade, preço pago, indicação de repetida e reembolso.
4. `Reveal` significa **confirmar que a apresentação terminou**, e também grava imediatamente. Não sorteia nem concede novamente. Fechar durante a animação mantém a mesma compra pendente. Um ID antigo é recusado. Enquanto existe uma pendência, novas compras são recusadas.

`World::economy_action(action, persist)` é o contrato puro com escritor injetável. Usar `|_| Ok(())` somente em simuladores/testes sem perfil; isso não é persistência. Para pagamentos do E1b, usar `progress::transact_economy(path, world, closure)`: a closure recebe o `EconomyState` candidato. Verificar a missão no domínio, inserir seu ID em `paid_missions` e chamar `record` na mesma closure. `record(reason, credited, debited)` mantém totais, saldo e IDs crescentes. O texto `reason` é apresentação, não um identificador de missão. Erro da closure, da validação ou do escritor descarta o candidato inteiro.

O app usa `progress::advance_checkpoint` para chegadas: só clona o mundo em passos que podem alcançar uma estação. A gravação ocorre antes de consumir notificações. Uma falha conserva mundo e trem anteriores, pausa a viagem e mostra erro; retomar permite tentar novamente. Passos comuns continuam sem clone de `Config`/`World` e sem gravação por frame. O save periódico continua para movimento entre transações.

## Renda e sorteio

- Cada chegada regular: 1 moeda.
- Passageiros: quociente por quatro das entregas novas mais o resto anterior.
- Contrato entregue: quociente por vinte do prêmio mais o resto anterior, mínimo duas moedas. Nada por unidade, aceite, carregamento, recusa ou devolução. O mínimo não descarta o resto da divisão.
- O multiplicador afeta essas três fontes; frações ficam em `bonus_remainder`. Boas-vindas, reembolsos e compras não recebem bônus.
- A chegada do serviço é idempotente pelo contador durável já existente. `cargo_by_kind` incrementa apenas na entrega, com o ID exato de `CargoKind`; migrar nunca distribui o total histórico entre espécies.

`CrateKind::GeneralYard.definition()` publica preço 60, probabilidades `[70,22,7,1]`, garantia na décima abertura e os 29 IDs explícitos (`GENERAL_WAGONS` e `GENERAL_ENGINES`). `CrateDef::validate()` rejeita IDs inexistentes, repetidos, probabilidades que não somam 100 e faixas com peso sem conteúdo. `effective_weights(misses)` retorna `[0,0,7,1]` na garantia: a interface deve normalizar esses pesos para exibir 87,5%/12,5%.

O gerador é SplitMix64, com rejeição para sorteio uniforme. Primeiro escolhe a raridade; depois escolhe uniformemente entre modelos não possuídos nessa raridade. Só quando todos já são possuídos ocorre repetida e reembolso de `preço efetivamente pago / 5`. A caixa grátis não reembolsa, mas conta na garantia e pode zerá-la ao sair raro/épico. Raro/épico repetido também zera a garantia. Nenhuma garantia de épico ou modelo específico.

Compra direta custa 120/240/480/900 para comum/incomum/raro/épico e aceita qualquer modelo atual do catálogo. Recomprar um modelo possuído é recusado. Raridades não alteram capacidade, renda ou velocidade.

## Checkpoint v5

Além dos campos anteriores, o documento contém `economy` e `lines`. Ambos são obrigatórios no v5. Os `Option`s em `WorldCheckpoint` servem exclusivamente para deserializar v2–v4; não são licença para omitir campos novos. Campos de economia ausentes, posse de pintura divergente, composição sem posse, restos inválidos, log inconsistente e contadores por espécie acima do total de carga são recusados.

| Grupo | Campos persistidos |
| --- | --- |
| Regras e saldo | `rule_version`, `balance`, `total_earned`, `total_spent` |
| Auditoria limitada | `next_transaction_id`, `transactions` (últimas 128, com crédito/débito/saldo) |
| Posse | `inventory`; espelho de pinturas em `service.owned_rewards` |
| Remessas | `pending_purchase`, `crate_misses` para as cinco famílias, `random_state`, `welcome_crate_available` |
| Restos | `passenger_remainder` (0–3), `contract_remainder` (0–19), `bonus_remainder` (0–999) |
| Carga | `cargo_by_kind`: os 15 IDs atuais, inclusive valores zero |
| Missões | `paid_missions`, `legacy_mission_claim_pending` |
| Ordens | `daily_orders`, `next_order_id`, `last_order_active_seconds`, `last_order_swap_active_seconds` |
| Museu | `museum_stamps` (quantidade; os selos serão numerados de 1 a N) |
| Linhas | `lines`: ID da linha → `LineProgress` |

`DailyOrder` já reserva ID da instância, ID do modelo, mapa de baselines, meta, prêmio, instante ativo de entrada e indicação de telegrama. Admite três ordens comuns e um telegrama, com IDs distintos. A geração, a elegibilidade, o intervalo de dez minutos, o progresso e o resgate pertencem ao E1b; ainda não ocorrem.

`LineProgress` contém `seed`, `distance`, `active_seconds`, `station_wait`, `stations_visited`, `completed_runs` e `discovered_stations` por ID de catálogo. O sentido de uma linha vai-e-volta é derivável da distância pelo `Itinerary`. `World::line_progress()` captura o mapa com Aurora atualizado. Nesta entrega, o loader exige somente Aurora e confere que sua entrada concorda com a jornada ativa. **E3:** adaptar o motor, a validação de outras linhas e a escolha da linha ativa usando este mapa; a reserva não torna a linha real jogável. Serviços e excursões continuam globais; só trocar de linha nas condições de esvaziamento definidas no plano. Persistir a troca inteira antes de confirmá-la.

## Perfis e migração

Novos perfis: 30 moedas, uma remessa grátis ainda não reclamada, Vapor Aurora/Passageiros/Carga/Tanque/Verde Aurora possuídos e Passageiros/Passageiros/Carga equipados. Ganham apenas Dormitório no primeiro circuito, Diesel do Porto pelo postal de carga e as quatro pinturas originais nos marcos existentes. Postais continuam sendo concedidos; o pagamento de missões fica para E1b.

Perfis v1–v4: a migração preserva jornada, serviços, pontos, composição, despacho e excursões. Concede os modelos equipáveis pelas regras antigas naquele instante, inclusive pinturas elegíveis que ainda não tinham sido incorporadas por uma chegada. Recebem 30 moedas e uma caixa grátis; pontos não são convertidos. `legacy_mission_claim_pending = true` deixa explícita a obrigação de E1b de pagar os marcos anteriores uma única vez, com aviso e gravação imediata. A partir daí seguem as concessões reduzidas do v5, sem continuar liberando toda a loja por chegadas.

Ler não regrava o arquivo. O primeiro save v5 cria backup exato `.v1.backup.json` até `.v4.backup.json`; um backup já existente com bytes diferentes bloqueia a migração. O fluxo `.previous.json` e a preservação de arquivos danificados/futuros continuam válidos. V5 não é legível pela distribuição rc.3 anterior.

## Próximas frentes

- **E1b:** implementar as outras quatro definições de caixa (hoje retornam indisponível), ordens, resgates de missões e selos usando os campos reservados; tratar a flag de migração; simular e ajustar o ritmo. Não inventar raros/épicos para preencher conjuntos de plataforma/pintura. Os preços atuais ainda não foram calibrados pelo simulador.
- **E2:** alimentar `set_machine_bonus`, sem usar CPU para distância, sem persistir leituras ou efeitos econômicos paralelos ao teto.
- **E5/E6:** consumir inventário e raridade; a oficina já autoriza por posse, mas não há loja nem animação nesta entrega. Mostrar a remessa grátis e qualquer resultado pendente; só enviar `Reveal` ao terminar a apresentação.
- **E3:** integrar o mapa `lines`, validação de itinerários e troca persistida; não resetar carteira, RNG, garantias nem IDs globais ao trocar de linha.

As fixtures estáticas e seu exportador estão em `tests/fixtures/economy/` e `examples/economy_fixtures.rs`. O relato no plano registra os gates executados e os limites da validação.
