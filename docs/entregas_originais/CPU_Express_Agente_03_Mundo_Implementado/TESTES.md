# Validação da entrega — Agente 03 / Mundo

Entrega de 13 de novembro de 2026. Este relatório descreve as verificações desta frente. Os documentos originais em `docs/` e o `BUILD-INFO.json` continuam sendo registros históricos do baseline.

## Resultado

**48 testes do projeto aprovados: 27 originais e 21 novos.** Além disso, **5 testes do núcleo independente** foram compilados e executados diretamente com `rustc`, sem as dependências do aplicativo. Formatação, Clippy, build release no host e checagem cruzada do alvo Windows também passaram.

Nenhum teste original foi editado. A comparação de conteúdo do pacote confirmou apenas três arquivos originais alterados, todos em `src/simulation/`.

## Ambiente e comandos executados

- Host: Linux x86_64.
- Rust: `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- Cargo: `cargo 1.100.0 (3acc79c0c 2026-11-05)`.
- Toolchain: `1.100.0`, exatamente a fixada em `rust-toolchain.toml`.
- Dependências: cache existente, modo `--locked --offline`; sem atualizar o lockfile.
- A MSRV declarada `1.95` não foi testada separadamente.

Executar os comandos a partir da pasta que contém `Cargo.toml`:

| Comando executado | Resultado | Log em `validation/journey/` |
| --- | --- | --- |
| `cargo +1.100.0 fmt --check` | Aprovado | `final-fmt.log` |
| `cargo +1.100.0 clippy --all-targets --locked --offline -- -D warnings` | Aprovado, sem warnings | `final-clippy.log` |
| `cargo +1.100.0 test --locked --offline` | 48 aprovados, 0 falhas | `final-test.log` |
| `cargo +1.100.0 build --release --locked --offline` | Compilação/link no host Linux aprovados | `final-release.log` |
| `cargo +1.100.0 check --all-targets --target x86_64-pc-windows-gnu --locked --offline` | Verificação cruzada aprovada | `final-windows-check.log` |

No seu computador, `cargo` já respeita `rust-toolchain.toml`; o seletor `+1.100.0` é opcional. Se as dependências ainda não estiverem em cache, retirar somente `--offline` para permitir o download. Manter `--locked`.

O build release acima usa o comportamento Linux previsto no baseline, não a abertura de uma janela de produto. `cargo check` para Windows verifica o código específico desse alvo, mas não faz link de um `.exe` nem executa o aplicativo. **Não houve execução em Windows real, teste de taskbar, DPI, monitores ou suspensão física do sistema.** Não há executável novo neste ZIP de fontes.

## Cobertura dos testes de Cargo

| Grupo | Quantidade | Situação |
| --- | ---: | --- |
| Configuração original | 8 | Preservado, aprovado |
| Métricas originais | 5 | Preservado, aprovado |
| Simulação original | 7 | Preservado, aprovado |
| Posicionamento de janela original | 7 | Preservado, aprovado |
| Rota — `journey_route.rs` | 5 | Novo, aprovado |
| Jornada — `journey_invariants.rs` | 16 | Novo, aprovado |
| **Total** | **48** | **0 falhas** |

Os testes novos verificam comportamento de domínio e integração, incluindo:

- circuito conectado de quatro estações, comprimentos positivos/distintos, biomas e acesso seguro a segmentos;
- reprodução de seeds, casos zero/`u64::MAX`, 256 seeds na verificação de propriedades e fixtures estáveis de rota;
- 720 segundos ativos simulados com a mesma sequência de CPU a 15, 30, 60 e 144 FPS;
- distância monotônica, posição cíclica válida, mesmas estações/voltas/conquistas e mesma sequência de eventos;
- preservação da integral analítica, da fase das rodas e do movimento com CPU ociosa;
- chegada na fronteira, partida posterior somente com movimento e passagem contínua quando sobra deslocamento;
- distinção entre tempo ativo e distância, com zero deslocamento sem transição de estação;
- pausa reversível, gaps grandes, NaN, infinito, negativos e deslocamento impossível sem avanço ou eventos extras;
- congelamento conjunto pelo adaptador de trem/mundo, incluindo velocidade-alvo em passos rejeitados;
- fila de 64 eventos, descarte do mais antigo, contador de perdas e conquistas preservadas após consumo/descarte;
- regressão do erro de arredondamento que podia colocar a posição lógica de uma partida antes da chegada anterior.

No teste integrado de FPS, as tolerâncias são `1e-6` unidade para distância/posição/eventos, `1e-8` segundo para tempo ativo, `1e-10` para velocidade e `1e-8` radiano para a fase das rodas. Os testes analíticos de um passo usam `1e-12`. Essas são tolerâncias dos testes, não uma promessa de tempo exato para viagens reais.

## Núcleo independente de interface e métricas

Os exemplos importam os próprios módulos de produção por `#[path]`. Não há mock de `Journey`, reimplementação do domínio, coleta de CPU, GPU, eframe ou sysinfo nessa compilação direta.

Comandos equivalentes aos executados no host, com caminhos de saída simplificados:

```bash
rustc --edition 2024 -D warnings examples/journey_demo.rs -o journey-demo
./journey-demo

rustc --edition 2024 -D warnings --test examples/journey_core_tests.rs -o journey-core-tests
./journey-core-tests
```

Resultado do segundo comando de execução: **5 aprovados, 0 falhas**. Os testes cobrem reprodução por seed, chegada/partida, fila limitada, pausa/suspensão/entradas inválidas e viagem equivalente a 1, 15, 30, 60 e 144 FPS.

Evidência: `validation/journey/core-tests.log`. Esses cinco testes são apresentados separadamente dos 48 testes de Cargo; verificam também a possibilidade de usar o núcleo sem compilar as dependências do aplicativo.

## Demonstração executada

`journey_demo` usa uma entrada **explicitamente simulada** de 55 unidades/s e passos de 0,25 s. São laços de tempo virtual; não há espera de 18 minutos nem benchmark para aquecer a CPU.

| Resultado da demo padrão | Valor observado |
| --- | ---: |
| Seed | `4850470395768754258` |
| Tempo ativo virtual | 1.080 s |
| Distância acumulada fictícia | 59.400 unidades |
| Voltas completas | 6 |
| Chegadas registradas | 26 |
| Estações descobertas | 4 de 4 |
| Conquistas | 3 |
| Eventos pendentes / descartados | 0 / 0 |

No meio da demonstração, uma pausa explícita e um gap de 3.600 segundos são exercitados com comparação integral dos snapshots. Distância, tempo e eventos continuam idênticos; a viagem volta a avançar apenas nos passos ativos seguintes. O log completo está em `validation/journey/core-demo.log`.

Também foram exercitados nove casos de argumentos da CLI: ajuda, durações mínima e máxima, duração zero, duração excessiva, overflow de seed, texto inválido, duração fracionária e argumentos extras. Todos tiveram os códigos de saída esperados, registrados em `core-demo-cli.log`.

As execuções de domínio terminaram em uma fração de segundo no ambiente de desenvolvimento. Isso demonstra que o exemplo é rápido; **não é uma medição de uso de CPU/RAM do aplicativo durante horas de uso**.

## Evidência e integridade da entrega

`validation/journey/final-evidence.json` registra ambiente, comandos, resultados e durações. `tested-source-sha256.json` contém os hashes dos arquivos Rust, manifesto, lockfile e toolchain efetivamente testados. `scope-check.json` compara os arquivos originais com o ZIP fornecido, registra os novos e confirma que nenhum original foi removido ou alterado fora da frente.

Foram preservados byte a byte: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, app, configuração, renderer, plataforma, métricas, scripts, README e testes antigos. Os três originais alterados são `src/simulation/mod.rs`, `train.rs` e `world.rs`.

## Limites e trabalho do coordenador

1. **Integração visual:** o estado novo já acompanha `World::update`, mas HUD, arte de estações/biomas, notificações e controle de pausa ainda precisam ser conectados pelas respectivas frentes. Não foi produzida uma prova visual nova porque esta entrega não altera o desenho.
2. **Tempo bruto:** o app atual limita o delta a 0,25 s. Para a regra de descarte integral de suspensão, aplicar o pequeno adaptador em `INTEGRACAO.md` e renovar o relógio ao pausar/retomar. Na árvore recebida, uma retomada pode representar até esse pequeno passo de 0,25 s.
3. **Persistência:** não há save nem crédito offline. O snapshot é uma visão de leitura; restauração futura deve ter schema e validação próprios.
4. **Invariância:** vale para entradas equivalentes e passos ativos de até um segundo. Gaps descartados não são equivalentes a vários passos pequenos de tempo ativo.
5. **Produto Windows:** resta executar e conferir o aplicativo em Windows real depois da integração das cinco frentes; a checagem cruzada não substitui isso.

Não foram repetidos benchmarks, testes de instalação ou testes visuais do baseline por não serem necessários para validar esta alteração restrita de domínio.
