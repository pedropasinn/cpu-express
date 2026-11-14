# CPU Express — entrega do Agente 03: mundo e viagem

Este pacote contém a implementação da missão `MISSAO_AGENTE.md` sobre a linha A congelada. A versão do manifesto continua `0.1.0`; esta é uma entrega de código para integrar à v0.2, não um release Windows já validado. `README.md`, `BUILD-INFO.json`, `LEIA-ME.txt` e os relatórios anteriores preservam o histórico do baseline. Para esta entrega, comece por este arquivo, `INTEGRACAO.md` e `TESTES.md`.

## Resumo em 10 linhas

1. Núcleo de viagem contínua implementado em Rust, com quatro estações em um circuito.
2. Seed reproduzível varia os comprimentos, mantendo identidade, ordem e biomas das estações.
3. Distância acumulada, posição cíclica, tempo ativo e distância até a próxima estação são separados.
4. Chegadas e partidas emitem eventos; elas não param nem alteram a velocidade do trem.
5. A curva original de CPU e a integral analítica foram preservadas nos passos ativos.
6. Pausa é reversível; intervalos maiores que um segundo são descartados pelo núcleo.
7. A fila comporta 64 eventos, descarta o mais antigo e informa quantos foram descartados.
8. Descobertas e três conquistas ficam em memória, independentemente de consumir os eventos.
9. `World::update` já alimenta a viagem; um adaptador adicional coordena trem, mundo e pausa.
10. Demonstração, testes e instruções de integração acompanham a fonte; UI e arte permanecem a cargo das outras frentes.

## O que foi implementado

### Circuito Aurora

Os valores abaixo correspondem à seed padrão `0x4350_5520_4558_5052`. As unidades são fictícias. A convenção atual é uma unidade de viagem por pixel de deslocamento retornado pelo trem; não há equivalência com distância geográfica, frequência do processador ou trabalho computacional.

| Saída | Próxima estação | Comprimento do trecho | Dica de bioma |
| --- | --- | ---: | --- |
| Vila Aurora | Bosque dos Pinheiros | 1.900 | `Meadow` |
| Bosque dos Pinheiros | Serra Clara | 2.375 | `Forest` |
| Serra Clara | Porto do Sol | 2.025 | `Highland` |
| Porto do Sol | Vila Aurora | 2.725 | `Coast` |

Uma volta mede 9.025 unidades na seed padrão. Cada seed é aceita como `u64`, inclusive zero. O gerador é explícito e determinístico; as variações dos trechos são discretas e pequenas. Seeds diferentes podem produzir a mesma combinação, pois o conjunto de comprimentos é finito.

O estado começa na Vila Aurora, já descoberta, sem emitir uma chegada fictícia. O primeiro deslocamento positivo emite sua partida. Ao cruzar outra estação, a jornada registra chegada; se ainda houver deslocamento no passo, registra também partida. Se o passo terminar exatamente na estação, a partida aguarda o próximo deslocamento positivo. Não há tempo de espera obrigatório.

### Conquistas da sessão

- `FirstStation`: primeira chegada após sair da origem.
- `FirstCircuit`: primeira volta completa.
- `LongJourney`: 36.000 unidades acumuladas.

Cada conquista é emitida uma única vez por sessão. A fila de eventos pode ser consumida ou perder itens antigos sem apagar essas conquistas. Ociosidade não reduz progresso; com os padrões originais, 0% de CPU continua tendo alvo de 12 km/h fictícios.

### Tempo, robustez e memória

Passos ativos têm `0 < dt <= 1` segundo. A simulação descarta integralmente gaps maiores que um segundo, tempo inválido e deslocamento impossível. O programa fechado não acumula nada. Simulações aceleradas usam uma sequência de passos pequenos, sem esperar tempo real ou produzir carga sintética para mover o trem.

O limite de deslocamento é 275 unidades por segundo ativo, correspondente ao máximo já permitido pela configuração original: 500 km/h fictícios × 0,55 pixels/s. Esse limite valida entradas; não muda a curva normal. Uma tolerância relativa de 32 vezes o epsilon de `f64` admite apenas arredondamento na borda desse limite.

Há acumuladores compensados para reduzir deriva numérica. A posição local fica separada do total, e os contadores inteiros saturam. Nas fronteiras, resíduos de até `1e-9` unidade podem ser ajustados na posição local; o total integrado não é arredondado. A partida conserva a posição lógica da chegada.

Não há histórico crescente, rede, relógio de parede ou escrita em disco. A rota usa arrays fixos, as descobertas usam quatro flags, as conquistas usam três flags, e a fila tem no máximo 64 itens. O item mais antigo é removido **antes** da inclusão do novo; `dropped_event_count()` informa perdas.

## Arquivos alterados

| Arquivo | Alteração |
| --- | --- |
| `src/simulation/mod.rs` | Exporta a API de viagem e aplica a política compartilhada de tempo. |
| `src/simulation/train.rs` | Documenta o descarte de gaps; preserva fórmulas, assinaturas e getters. |
| `src/simulation/world.rs` | Mantém a jornada, expõe leitura/eventos e oferece o adaptador com pausa. |

## Arquivos acrescentados

| Arquivo | Finalidade |
| --- | --- |
| `src/simulation/route.rs` | Rota, estações, trechos, biomas e geração por seed. |
| `src/simulation/timing.rs` | Validação de tempo/deslocamento e resultado dos passos. |
| `src/simulation/journey.rs` | Progressão, conquistas, snapshots e fila de eventos. |
| `tests/journey_route.rs` | Propriedades e fixtures de reprodução da rota. |
| `tests/journey_invariants.rs` | Comportamento da viagem e integração numérica com trem/mundo. |
| `examples/journey_demo.rs` | Demonstração rápida, também compilável apenas com `rustc`. |
| `examples/journey_core_tests.rs` | Testes do núcleo real sem as dependências do aplicativo. |
| `PATCH_NOTES.md`, `INTEGRACAO.md`, `TESTES.md` | Entrega, integração, comandos e evidências. |
| `validation/journey/` | Logs desta entrega e conferência do escopo das alterações. |

Os outros arquivos do pacote original são preservados byte a byte. Não foram acrescentadas dependências. Os testes originais não foram alterados.

## Compatibilidade e integração

Todas as assinaturas exigidas no contrato e os getters antigos permanecem disponíveis. Em passos normais, a integração analítica da velocidade permanece a mesma. A mudança deliberada de comportamento é que chamadas diretas com `dt > 1` deixam de representar passagem ativa de tempo; consumidores de simulação acelerada devem subdividi-las.

O loop original já chama `World::update`, portanto a viagem agora progride internamente. A tela ainda não mostra estações, conquistas ou biomas novos, e não recebeu botão de pausa. A proposta para o coordenador está em `INTEGRACAO.md`.

## Três riscos reais

1. **O app esconde o delta bruto.** O baseline faz `.min(0.25)` antes de chamar a simulação; após suspensão pode avançar até 0,25 s. Para descarte integral e pausa conjunta, aplicar o adaptador documentado. A simulação isolada já obedece à política nova.
2. **Ainda falta apresentação visual e teste em Windows real.** O núcleo produz os dados e eventos, mas as outras frentes precisam decidir como mostrá-los. Compilar ou checar o alvo Windows em Linux não comprova o comportamento da janela no computador do usuário.
3. **O progresso é somente de sessão.** Reiniciar cria outra jornada com a seed escolhida. Uma futura persistência precisa de schema e versão da rota; `JourneySnapshot` é uma visão de leitura, não um formato de save validado. Eventos antigos podem ser descartados quando a UI não os consome, sem afetar o estado atual.

## Rollback

Antes da integração, conservar o ZIP original `CPU_Express_Agente_03_Mundo(1).zip`. Para voltar exatamente ao baseline, restaurar dele `src/simulation/mod.rs`, `train.rs` e `world.rs`, e remover os três módulos novos, os dois testes `journey_*` e os dois exemplos `journey_*`. Os relatórios novos podem ser arquivados. Como esta frente não salva progresso nem modifica a configuração, não existe migração de disco a desfazer.

Se o coordenador aplicar também o adaptador de `app.rs`, esse pequeno ajuste deve ser revertido junto. Não substituir arquivos das outras frentes por este pacote inteiro depois que elas já tiverem sido integradas: transportar apenas os arquivos listados acima.
