# Verificação da entrega — Agente 04

**Data:** 13/11/2026. **Host:** Linux x86_64. **Rust/Cargo:** 1.100.0, conforme a toolchain congelada. A MSRV 1.95 declarada no manifesto foi preservada, mas não foi executada separadamente.

## Resultado

| Verificação | Resultado observado | Evidência |
|---|---|---|
| Testes originais, antes das alterações | 27 passaram | `benchmarks/evidence/baseline-tests.log` |
| Regressão nova contra o código original | Falhou como esperado: recebeu `Some(10.0)`, exigia `Some(90.0)` | `benchmarks/evidence/baseline-regression.log` |
| `cargo fmt --check` | Passou | `benchmarks/evidence/fmt.log` — vazio porque não houve diferenças |
| Clippy em todos os targets do projeto no host, avisos como erro | Passou | `benchmarks/evidence/clippy.log` |
| Testes Rust após as alterações | **61 passaram; 0 falharam** | `benchmarks/evidence/tests.log` |
| Build `release` no host Linux | Passou | `benchmarks/evidence/release-linux.log` |
| `cargo check --all-targets --target x86_64-pc-windows-gnu` | Passou; checagem de código para Windows, sem link de `.exe` | `benchmarks/evidence/windows-check.log` |
| Probe real de métricas, release, sem interface | Aquecimento seguido de 3 leituras `Ready` | `benchmarks/evidence/metrics-probe.log` |
| Testes Python do analisador | **19 passaram; 0 falharam** | `benchmarks/evidence/python-tests.log` |
| Comparação com a árvore original | Apenas 3 arquivos preexistentes alterados | `benchmarks/evidence/modified-existing-files.json` |
| Aplicação/reversão do patch em cópia isolada | Resultado em `patch-validation.json` | `benchmarks/evidence/patch-validation.json` |

Total automatizado da versão final: **80 testes (61 Rust + 19 Python)**. Os 61 Rust incluem os 27 originais, 20 novos testes unitários e 14 regressões de integração do worker. O teste deliberadamente falho do baseline é evidência histórica separada e não faz parte da suíte final.

## Comandos executados

A cache local continha as dependências congeladas; usamos `--offline` para a verificação não acessar rede. O diretório de build ficou fora do ZIP. Na raiz de `cpu-express`:

```bash
cargo fmt --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --locked --offline
cargo build --release --locked --offline
cargo check --locked --offline --all-targets --target x86_64-pc-windows-gnu
cargo run --release --locked --offline --example metrics_probe
python -m unittest discover -s benchmarks -p 'test_*.py' -v
```

Se for reproduzir em uma máquina sem cache, instalar a toolchain/dependências de desenvolvimento primeiro e omitir `--offline` quando for necessário baixá-las. Isso é preparo de build; o aplicativo e o benchmark não fazem coleta remota nem exigem rede durante uso.

Os comandos e códigos de saída principais estão em `benchmarks/evidence/checks.json`. `environment.json` identifica o ambiente e registra explicitamente ausência de execução gráfica/Windows.

## Prova de que a regressão encontra o bug original

O início de `tests/metrics_worker.rs`, até `// END BASELINE-COMPATIBLE REGRESSION.`, usa apenas a API original. Foi copiado para `tests/metrics_latest_regression.rs` em **uma cópia isolada** da linha A, sem alterar a árvore baseline preservada. Executou-se:

```bash
cargo test --locked --offline --test metrics_latest_regression \
  unread_slot_yields_newest_completed_measurement_and_no_backlog
```

O teste obteve:

```text
assertion `left == right` failed
  left: Some(10.0)
 right: Some(90.0)
test result: FAILED. 0 passed; 1 failed
```

O código de saída foi `101`, esperado para essa demonstração. O mesmo teste está na suíte corrigida e passou. A quarta chamada do provider fake só começa depois de publicar a terceira amostra, evitando que o teste escolha valores com base em `sleep` ou velocidade do processador. O prefixo isolado contém um import de thread não utilizado; o aviso desse experimento não é aviso do projeto final, cujo Clippy passou com `-D warnings`.

## Cobertura relevante

| Risco | Como foi verificado |
|---|---|
| Amostra antiga retida/fila crescente | Três publicações sem consumo; primeira leitura precisa ser a última, segunda deve ser `None`. |
| UI esperando consulta do SO | Provider bloqueado por canal; `poll` precisa retornar antes de liberar a consulta. |
| UI esperando mutex ocupado | Teste interno mantém o slot travado; o leitor precisa devolver `None` antes de liberar o guard. |
| Shutdown esperando intervalo de 60 s | Sinal de parada acorda o timer; chamadas repetidas são aceitas. Os timeouts de teste são watchdogs, não uma medida de latência nativa. |
| NaN/inf, erros e falsa RAM | Providers determinísticos verificam ausência explícita, métrica parcial válida e retorno a `Ready`. |
| Idade e retomada | Timestamps artificiais verificam limite exato, +1 ns, timestamp futuro e remoção de dados stale sem reestampar. |
| Erro de provider/panic | Publicação terminal `Unavailable`, sem CPU/RAM fictícias. |
| Aquecimento e consulta precoce | Testes puros do backend e teste original de primeira leitura real. |
| JSONL inconsistente | Schema v2, valores ausentes/stale como `null`, modo/input efetivo preservados e limite de registros. |
| Fórmulas de medição | Testes Python de CPU ponderada, normalização, P50/P95, aquecimento, intervalos, estados, CSV pt-BR/BOM e CLI completa. |

Os novos testes de comportamento usam fake providers ou funções puras. O teste original que inicializa o `SysinfoProvider` continua preservado. Nenhum teste depende de um contador Windows disponível no host Linux.

## Demonstração real do provider neste host

O exemplo `metrics_probe`, já existente e inalterado, observou:

```text
elapsed_s,cpu_percent,ram_percent,used_bytes,total_bytes,status
0.020,NA,41.966,4386066432,10451427328,WarmingUp
1.006,22.727,41.425,4329537536,10451427328,Ready
2.020,23.503,41.330,4319617024,10451427328,Ready
3.010,25.444,41.104,4295921664,10451427328,Ready
```

Esses números são **leituras reais do ambiente Linux de validação**, sem carga sintética deliberada. Não são uso de CPU do processo CPU Express, resultado de benchmark comparativo nem estimativa do computador do Pedro. Não há execução de janela associada a esse probe.

## O que não foi executado

- **Execução nativa Windows, minimização, retomada, DPI, taskbar, driver OpenGL e consumo de energia.** A verificação de código para o alvo não substitui esses ensaios.
- **Geração de um novo executável Windows.** O target Rust estava disponível, mas não havia linker/runtime GNU MinGW instalado. O ZIP é de fonte; os hashes de executável do `BUILD-INFO.json` original não pertencem a esta modificação.
- **Execução de PowerShell.** O wrapper foi revisado estaticamente; o ambiente não disponibilizava PowerShell nem Windows. O analisador Python, por sua vez, foi executado e testado.
- **Nova captura visual.** A imagem dentro da pasta original de validação pertence ao baseline. Não é prova visual desta rodada.
- **Ganho de CPU, FPS ou bateria.** Não há comparação controlada de desempenho A/B desta alteração. Os números antigos Linux/Mesa não devem ser usados como tal.

## Checklist de aceite no Windows

1. Compilar em release com a toolchain do projeto, registrar revisão, hash do `.exe` e hardware/driver/energia.
2. Rodar a suíte de testes e `cargo run --release --locked --example metrics_probe` no sistema de destino.
3. Aplicar pela frente 05 o tratamento de RAM envelhecida descrito em `INTEGRACAO.md`.
4. Executar cenários visível, minimizado e retomada do protocolo em `docs/performance_windows.md`, com/sem diagnóstico.
5. Conferir a transição aquecimento → leitura, labels de ausência/idade, ausência de travamento da UI e encerramento.
6. Analisar os arquivos brutos; reportar N, duração, CPU nas duas escalas, memória, FPS rasterizado e limites da evidência de ocultação.

O checklist contém a validação restante da plataforma; não é apresentado como trabalho já executado.
