# CPU Express — entrega do Agente 04

**Telemetria, eficiência e diagnóstico · 13/11/2026 · linha A preservada**

## Resumo em 10 linhas

1. A fila que preservava a primeira amostra foi substituída por um slot em que a última publicação vence.
2. A UI retira o snapshot com `try_lock`, sem esperar consultas de CPU/RAM ou a liberação de um mutex ocupado.
3. `poll`, `spawn`, `shutdown`, `MetricsSample`, `MetricStatus` e `ManualProvider` continuam compatíveis com o aplicativo.
4. A API nova calcula idade e distingue amostra atual, amostra vencida, aquecimento e erro de coleta.
5. Uma amostra já stale quando consumida não fornece CPU/RAM nem recebe timestamp novo.
6. O provider valida falhas observáveis, conserva métricas parciais válidas e não inventa RAM ausente.
7. O encerramento acorda o timer; o limite de chamadas nativas travadas está documentado.
8. O JSONL opt-in passa a schema v2, com bytes de RAM opcionais e valores stale como `null`.
9. O pacote inclui benchmark Windows, análise de CPU/FPS/memória/P50/P95 e orientação de integração com a UI.
10. Passaram 61 testes Rust e 19 Python; o teste novo falha no baseline e passa na correção; Windows nativo permanece pendente.

## Comece aqui

Este é um **pacote de código-fonte atualizado**, sem executável novo. `README.md`, `LEIA-ME.txt`, `BUILD-INFO.json`, `BASELINE.md` e a imagem antiga foram preservados como documentos do pacote de origem. Os hashes de executável e relatos de execução nesses arquivos são históricos; **não descrevem um novo `.exe` gerado nesta rodada**. A evidência atual está em `TESTES.md` e `benchmarks/evidence/`.

Para integrar no projeto principal, usar `INTEGRACAO.md`. Para aplicar apenas código/ferramentas, há `benchmarks/agent04.patch`. O ZIP não contém `target/`, caches ou binários de release.

## Mudanças realizadas

### Última amostra e isolamento da coleta

Antes, publicar 10%, 60% e 90% sem ler deixava **10%** pendente, pois o canal cheio descartava os próximos valores. Agora a primeira leitura após essas publicações recebe **90%**, e a seguinte não encontra backlog.

O slot contém dados próprios, não o `System` do sysinfo. A consulta ao SO e a validação ocorrem fora do lock. O leitor tenta retirar uma amostra e retorna imediatamente quando o lock está ocupado. Não foram adicionadas dependências, rede, polling por frame nem uma arquitetura inteira de outro fork.

### Erro, aquecimento e idade

`SampleFreshness` e `MetricsSnapshot` acrescentam estado temporal sem mudar variantes existentes. O limite é `max(3 × intervalo efetivo, 3 s)`. O timestamp original é preservado; erros recentes continuam sendo erros, mesmo classificados como atuais. `Unavailable` pode preservar CPU ou RAM válida quando só a outra leitura falhou.

No provider real, a primeira amostra não publica CPU. Chamadas precoces reaproveitam o snapshot sem reestampá-lo. NaN, infinito, valores inválidos de RAM e falhas observáveis tornam-se ausência explícita. O comportamento de normalização do modo manual foi preservado.

### Parada e diagnóstico

A parada usa sinal de capacidade um e é idempotente. Um worker no timer não espera pelo fim de um intervalo longo. Um panic recuperável do provider publica falha e encerra o worker, sem reutilizar um objeto possivelmente comprometido.

O diagnóstico continua sendo habilitado explicitamente. O schema v2 acrescenta classificação de frescor e representa memória ausente como `null`. **Leitores que exigem bytes de RAM sempre numéricos precisam ser adaptados.** A API Rust do writer continua igual. O registro com flush é síncrono; o benchmark compara o custo com/sem logging.

### Ferramentas de medição

O wrapper Windows usa o script de processo existente, acrescenta hardware/driver/energia e identidade do build, e mantém os parâmetros declarados pelo operador identificados como declarações. O analisador Python usa só a biblioteca padrão, calcula CPU a partir de deltas de tempo e FPS a partir de deltas de frames. Reporta CPU equivalente a um processador lógico e normalização total separadamente, com P50/P95, N, durações e unidades.

A ferramenta não confunde FPS configurado com FPS observado, não transforma ausência de amostra em zero e não chama os ensaios antigos Linux/Mesa de comparação A/B controlada. O cenário minimizado e o flag `hidden` não são prova independente do estado Win32.

## Arquivos alterados

A comparação de bytes com o ZIP original mostrou **somente três arquivos existentes modificados**:

| Arquivo existente | Alteração |
|---|---|
| `src/metrics/mod.rs` | Slot substituível, consumo sem bloqueio, validação, idade, parada e testes internos. |
| `src/metrics/sysinfo_provider.rs` | Baseline, cache temporal, saneamento CPU/RAM e testes puros. |
| `src/diagnostics.rs` | Schema v2, normalização de ausência/stale e testes JSONL. |

Arquivos novos de implementação e orientação:

| Arquivo novo | Finalidade |
|---|---|
| `tests/metrics_worker.rs` | 14 regressões por providers controlados, sem depender de contadores Windows. |
| `benchmarks/run-windows.ps1` | Contexto e execução explícita do benchmark Windows. |
| `benchmarks/analyze.py` | Analisador de CSV/JSONL e resumos com unidades. |
| `benchmarks/test_analyze.py` | 19 testes dos cálculos e da CLI. |
| `docs/performance_metrics.md` | Arquitetura, estados, custos e limites do backend. |
| `docs/performance_windows.md` | Protocolo reproduzível e interpretação dos resultados. |
| `PATCH_NOTES.md`, `INTEGRACAO.md`, `TESTES.md` | Handoff, API, propostas, comandos, resultados, riscos e rollback. |
| `benchmarks/agent04.patch` | Patch restrito de código, ferramentas e notas técnicas. |
| `benchmarks/evidence/` | Logs reais, ambiente, hashes, checks e comparação com o baseline. |

`src/app.rs`, `src/main.rs`, `src/config.rs`, renderização, simulação, plataforma, testes originais, exemplos originais, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, README e scripts compartilhados ficaram byte a byte iguais ao pacote recebido.

## API nova e demonstração

A API acrescentada é: `SampleFreshness`, `MetricsSnapshot`, `MetricsSample::{age, age_at, freshness, freshness_at, memory_bytes}` e `MetricsWorker::{stale_after, poll_snapshot}`. Assinaturas e regras estão detalhadas em `INTEGRACAO.md`.

Executar na pasta que contém `Cargo.toml`:

```powershell
cargo test --locked --test metrics_worker
cargo run --release --locked --example metrics_probe
python -m unittest discover -s benchmarks -p "test_*.py" -v
```

O primeiro exercita as regressões, o segundo lê a CPU/RAM real da máquina sem carga deliberada e o terceiro verifica a análise de benchmark. Para a coleta de processo/FPS no Windows, seguir `docs/performance_windows.md` após gerar o `.exe` em release.

## Três riscos reais e rollback

1. **Windows e PDH:** a verificação de compilação para o alvo passou, mas não houve execução nativa. A biblioteca pode ocultar alguns erros de contador com números plausíveis. Testar no Windows e conservar essa distinção nos relatórios.
2. **SO/driver travado:** o sinal interrompe a espera do worker, mas não uma consulta nativa em andamento. `shutdown()` pode aguardar essa consulta; não há promessa de timeout rígido.
3. **Integração com UI/consumidores:** uma amostra pode envelhecer depois de entrar no cache de RAM da UI; a política de minimização e os leitores JSONL também requerem atenção. `INTEGRACAO.md` traz o adaptador mínimo e as propostas para a frente responsável.

Rollback: restaurar os três arquivos Rust acima do ZIP original e retirar o novo teste `tests/metrics_worker.rs`; ou verificar/aplicar a reversão do patch em uma cópia sem modificações posteriores. Dependências e preferências do usuário não foram alteradas. Reverter o worker também restaura o defeito antigo de retenção da amostra.
