# Agente 04 — coleta, idade e custo das métricas

Data: 13/11/2026. Esta nota descreve a implementação entregue; resultados de execução estão em `TESTES.md` e `benchmarks/evidence/`. Windows 10/11 x64 continua sendo o alvo do produto. O ambiente de verificação desta rodada é Linux.

## 1. O defeito e a correção

A linha A publicava amostras em `sync_channel(1)`. Quando o slot estava ocupado, `TrySendError::Full` descartava **a nova** amostra. Uma interface que deixasse de consumir mantinha pendente o primeiro valor, mesmo depois de várias coletas.

Agora `LatestSample` contém `Mutex<Option<MetricsSample>>`, compartilhado por `Arc`. A thread de coleta substitui a opção inteira e solta o valor anterior depois de liberar o mutex. O consumidor usa `try_lock()` e `take()`: recebe a amostra mais recente já publicada, ou `None` se o registro está vazio/temporariamente ocupado. Uma publicação concorrente posterior será recebida na próxima chamada. A garantia não é prever coletas futuras.

Não há chamada a `sysinfo`, validação, serialização, alocação de mensagem ou destruição da amostra anterior dentro da região crítica normal. O leitor nunca aguarda a liberação do mutex. O produtor pode aguardar a operação curta de retirada, que não executa trabalho do sistema operacional. Não se trata de uma implementação lock-free nem de uma garantia de prazo de tempo real.

O armazenamento tem tamanho constante: um slot pendente, a amostra própria de cada etapa e o cache fixo do provider. Nenhuma lista de histórico cresce enquanto a janela está sem consumir. O canal de parada também tem capacidade um.

**Prova de regressão:** `unread_slot_yields_newest_completed_measurement_and_no_backlog` publica 10%, 60% e 90% sem ler. A chegada à quarta chamada do fake prova que a terceira amostra já passou pela publicação. O baseline devolve 10%; a versão corrigida devolve 90% e depois `None`. Os números vêm de um provider de teste; o teste não gera carga sintética de CPU para animar o jogo.

## 2. Estado de coleta e idade são informações distintas

`MetricStatus` mantém exatamente as variantes originais. A idade usa `Instant`, e `SampleFreshness` é uma classificação separada.

| Estado | Idade | Significado | Leitura utilizável |
|---|---|---|---|
| `WarmingUp` | `Current` | A baseline da CPU está sendo formada | CPU ausente; RAM pode ser válida |
| `Ready` | `Current` | Coleta válida dentro da janela de idade | CPU presente; RAM é opcional no contrato genérico |
| `Unavailable` | `Current` | Erro recém-observado | Pode haver uma métrica parcial válida |
| Qualquer estado original | `Stale` | Amostra vencida ou timestamp futuro | O snapshot consumido não entrega CPU/RAM |

O intervalo efetivo é `max(intervalo solicitado, mínimo do provider, 10 ms)`. O limite de idade é `max(3 × intervalo efetivo, 3 s)`, com multiplicação saturante. Idade exatamente igual ao limite ainda é atual. Um timestamp futuro é inválido para frescor; `age_at` satura a duração em zero, mas `freshness_at` classifica como `Stale`.

`poll_snapshot()` calcula idade no consumo, preserva `measured_at` e remove números de uma amostra velha. `poll()` é o wrapper compatível e faz a mesma proteção. Os dois métodos consomem o **mesmo** slot: escolher um por atualização.

Depois de receber uma amostra, a UI passa a possuir sua cópia. Se a thread do SO travar posteriormente, essa cópia envelhece sem nova publicação. A UI deve continuar consultando idade. A UI consolidada aplica essa regra a CPU, RAM e tooltip, preservando também as leituras parciais válidas.

## 3. CPU/RAM e falhas observáveis

O `SysinfoProvider` é construído dentro da thread de coleta. Erros recuperáveis de construção publicam `Unavailable` e repetem a tentativa com intervalo mínimo de um segundo. Isso não bloqueia a abertura da interface.

A primeira coleta não fornece CPU. Uma coleta posterior válida, após o intervalo mínimo do `sysinfo`, permite publicar utilização. Chamadas diretas precoces reutilizam o snapshot inteiro, com timestamp original: não inventam uma nova medição nem fazem um provider pronto voltar ao aquecimento. Uma falha observável da CPU exige nova baseline antes de voltar a publicar percentual. Falha só de RAM preserva a baseline e a CPU válida.

A CPU real não finita ou fora de 0–100 no backend é indisponível. O worker também protege a fronteira de providers injetados contra NaN/infinito e CPU ausente com status `Ready`. Valores finitos fora da faixa de providers genéricos mantêm o clamp de compatibilidade. O `ManualProvider` preserva sua API simulada: NaN/inf de entrada manual normalizam para 0%; isso não é usado para encobrir erro do SO.

RAM com total zero, uso maior que o total ou disponibilidade maior que o total é removida. O contrato Rust antigo usa `u64`, portanto `0,0` permanece como **sentinela de ausência**, com `ram_percent=None`. O novo `memory_bytes()` retorna `None` nesses casos. O JSONL transforma a ausência em `null`, inclusive nos bytes, e não a relata como um computador com zero memória.

O timestamp do provider marca o **início** da coleta. Uma chamada lenta não se apresenta como observação recém-feita ao terminar. CPU representa a média entre atualizações do sistema; não é uma leitura instantânea de hardware, e suspensão pode alongar essa janela.

## 4. Limite real do backend Windows

A fonte da dependência congelada, `sysinfo 0.40.3`, foi inspecionada. Seus métodos públicos de atualização de CPU e memória não retornam um `Result` de sucesso da coleta. No backend Windows, `Query::get()` pode transformar certas falhas de `PdhGetFormattedCounterValue` em um valor plausível; em outros caminhos um contador ausente pode manter valor anterior. Assim, não há informação pública suficiente para detectar **todos** os erros de PDH.

Esta entrega detecta contadores ausentes e valores inválidos que a API expõe, mas não promete detectar um número plausível substituído internamente pela dependência. Para eliminar essa lacuna, o coordenador teria de aprovar um backend com erro explícito ou uma alteração da dependência, acompanhada de testes Windows. Nenhuma API de erro fictícia foi inventada.

Referências técnicas: fonte local congelada `sysinfo-0.40.3/src/windows/cpu.rs`, `src/windows/system.rs` e `src/common/system.rs`; [documentação de sysinfo](https://docs.rs/sysinfo/0.40.3/sysinfo/struct.System.html).

## 5. Parada e panic

`shutdown()` toma o `JoinHandle` uma vez, envia um sinal bounded e aguarda a saída. O `recv_timeout` é interrompido pelo sinal mesmo quando o intervalo de coleta é de 60 segundos. Chamadas repetidas e `Drop` não acumulam sinais.

O worker verifica a parada antes e depois de uma chamada do provider. Uma consulta do sistema já em andamento precisa retornar para que o join termine. Não existe limite rígido de shutdown se o SO/driver travar; Rust não permite encerrar essa thread com segurança e liberar seus recursos à força.

Um panic com desenrolamento publica `Unavailable` e encerra a thread; o objeto potencialmente comprometido não é reutilizado. O hook padrão ainda pode imprimir o panic no stderr. Builds `panic=abort`, falha fatal do processo e travamento dentro do SO não são recuperáveis por essa proteção.

## 6. Diagnóstico e custo do observador

O jogo normal continua sem persistir métricas, sem rede e sem histórico remoto. O diagnóstico precisa ser habilitado explicitamente por `--diagnostics`, ou pelo benchmark com `-IncludeDiagnostics`.

O writer preserva `BufWriter`, limite de 86.402 registros e um `flush` por registro. O app solicita aproximadamente um registro por segundo, além do final. Portanto, não há escrita por frame. Ainda há serialização e I/O síncronos na UI **quando o diagnóstico está ligado**: a garantia não bloqueante desta entrega diz respeito à leitura do worker de métricas. Um disco lento pode afetar o ensaio instrumentado. Compare execuções com/sem diagnóstico no protocolo Windows; não extrapole um custo medido em outro computador.

O analisador roda após o processo observado e calcula FPS pelas diferenças do contador de rasterizações. Isso não mede latência de apresentação, GPU, tearing nem frames efetivamente vistos no monitor. O estado `hidden` da aplicação é observado apenas nos extremos dos intervalos e não comprova minimização nativa.

## 7. Métricas futuras: valor e custo

| Possibilidade | Disponibilidade nesta entrega | Decisão recomendada para o próximo passo |
|---|---|---|
| CPU global e RAM física | Implementadas; erros/idade explícitos | Usar como contrato estável da animação |
| CPU de cada processador lógico | O backend já atualiza a lista de CPUs, mas a UI não tem campo | Só expor se houver uso visual concreto; não aumentar frequência de coleta |
| CPU/memória do próprio aplicativo | Coletadas pelo benchmark externo | Continuar fora do renderer; comparar custo com e sem instrumentação |
| Disco | Não implementado | Escolher primeiro se interessa capacidade, bytes/s ou tempo ocupado; medir backend e custo antes de ampliar a coleta |
| Temperatura/GPU | Não implementadas | Exigem backend e validação específicos; não pressupor API universal, driver ou permissão disponível |

Na consolidação, a UI recebeu guarda de rasterização para ocultação/minimização e agendamento lento nesse estado. A coleta mantém seu intervalo; amostragem adaptativa continua futura. A validação Windows real dessas políticas permanece pendente em `docs/VALIDATION.md`.

## Fontes e evidência

- [Rust: `Mutex::try_lock`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.try_lock), [recuperação de poison](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.clear_poison).
- Código entregue em `src/metrics/` e `src/diagnostics.rs`.
- Regressões em `tests/metrics_worker.rs`; verificação e limitações em `TESTES.md`.
- Protocolo de medições, unidades e fontes Microsoft em `docs/performance_windows.md`.
