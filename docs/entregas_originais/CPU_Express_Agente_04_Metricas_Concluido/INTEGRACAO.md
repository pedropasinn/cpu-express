# Integração — Agente 04 / métricas

Esta entrega implementa a frente de métricas sobre a **linha A congelada**. Não altera `app.rs`, `main.rs`, `config.rs`, renderização, simulação, plataforma, manifestos ou o lockfile. Os trechos de UI abaixo são **propostas ao Agente 05/coordenador**, não alterações aplicadas.

## 1. O que já funciona sem modificar a UI

Substitua os três arquivos modificados, acrescente os arquivos novos e execute os checks de `TESTES.md`.

- `MetricsWorker::spawn`, `spawn_with_provider`, `poll` e `shutdown` mantêm as assinaturas.
- `MetricsSample` conserva todos os campos e seus tipos; `MetricStatus` conserva todas as variantes. O `match` exaustivo de `app.rs` continua compilando.
- `ManualProvider::new`/`set_cpu`, CPU simulada e ausência de RAM fictícia mantêm o contrato.
- O `poll()` já usado pela UI recebe a última amostra publicada. Ele também remove os valores de uma amostra já vencida quando consumida, preservando o timestamp.
- A coleta real e a construção do `sysinfo` ficam no worker. O leitor usa `try_lock`, sem aguardar consultas do SO.
- Diagnóstico continua opt-in e utiliza o mesmo `Diagnostics::open/record`.

Não são necessárias novas dependências nem mudanças de `Cargo.toml`/`Cargo.lock`. Não altere a versão compartilhada do produto apenas para integrar esta frente.

## 2. API pública acrescentada

| API | Resultado / uso |
|---|---|
| `SampleFreshness::{Current, Stale}` | Idade separada de erro e aquecimento; não é nova variante de `MetricStatus`. |
| `MetricsSample::age()` | Duração desde a observação até agora. |
| `MetricsSample::age_at(now)` | Mesma conta com relógio fornecido; útil nos testes. |
| `MetricsSample::freshness(max_age)` | Classificação no instante atual. |
| `MetricsSample::freshness_at(now, max_age)` | Limite inclusivo e timestamp futuro como stale. |
| `MetricsSample::memory_bytes()` | `Option<(used_bytes, total_bytes)>`; não apresenta sentinelas como leitura. |
| `MetricsWorker::stale_after()` | `max(3 × intervalo efetivo, 3 s)`. |
| `MetricsWorker::poll_snapshot()` | `Option<MetricsSnapshot>` com `sample`, `age` e `freshness`. |
| `MetricsSnapshot` | Campos públicos `sample: MetricsSample`, `age: Duration`, `freshness: SampleFreshness`. |

**Escolha `poll()` ou `poll_snapshot()` por atualização:** ambos retiram a mesma amostra do mesmo slot. Não chamar um para medir idade e o outro para obter dados; não são leitores independentes. `None` significa ausência de publicação pendente ou contenção momentânea, nunca CPU de 0%.

Exemplo de consumidor novo:

```rust
if let Some(snapshot) = worker.poll_snapshot() {
    // snapshot.age e snapshot.freshness foram calculados no consumo.
    // Valores stale já vieram removidos; measured_at não foi reestampado.
    current_sample = snapshot.sample;
}
// Uma cópia que permanece na UI continua envelhecendo entre publicações.
let fresh = current_sample.freshness(worker.stale_after());
```

## 3. Adaptação mínima necessária em `app.rs`: RAM e tooltip

### Por que ainda há uma integração

O worker só pode proteger o valor que entrega. Depois que `app.rs` guarda uma amostra inicialmente fresca em `self.metrics`, ela pode envelhecer se a próxima chamada do SO demorar. O baseline verifica esse envelhecimento para CPU, mas não para o texto nem o tooltip de RAM. Essa alteração pertence à frente 05.

### A. Centralizar o limite e a classificação de idade

Adicionar `SampleFreshness` aos imports de métricas e substituir o corpo de `metrics_stale` pelo seguinte:

```rust
fn metrics_stale(&self) -> bool {
    let max_age = self.worker.as_ref().map_or_else(
        || Duration::from_millis(
            self.config.metrics_interval_ms.saturating_mul(3).max(3_000),
        ),
        MetricsWorker::stale_after,
    );
    self.metrics.freshness(max_age) == SampleFreshness::Stale
}
```

O fallback cobre falha de criação do worker. Esse ajuste também trata timestamps futuros de forma coerente. Não é necessário adicionar estado ao struct da UI.

### B. Aplicar o frescor ao texto da RAM

Substituir apenas a construção do texto `ram` em `status_bar`:

```rust
let ram = if self.metrics_stale() {
    "RAM sem leitura recente".to_owned()
} else {
    self.metrics
        .ram_percent
        .map_or("RAM --".to_owned(), |ram| format!("RAM {ram:.0}%"))
};
```

### C. Não apresentar sentinelas `0 / 0 GiB` no tooltip

No bloco que constrói `details`:

```rust
let memory_details = if self.metrics_stale() {
    "RAM sem leitura recente".to_owned()
} else if let Some((used, total)) = self.metrics.memory_bytes() {
    format!(
        "RAM: {:.2} / {:.2} GiB",
        used as f64 / 1_073_741_824.0,
        total as f64 / 1_073_741_824.0,
    )
} else {
    "RAM indisponível".to_owned()
};
let mut details = format!(
    "{memory_details}\nVelocidade fictícia; o próprio aplicativo participa da CPU total."
);
// Manter abaixo o bloco existente que acrescenta o erro Unavailable.
```

Com CPU manual, o rótulo `CPU SIM` continua pertencendo à UI. Não usar `effective_cpu_percent` do diagnóstico como se fosse a CPU real medida: ele é a entrada da simulação, incluindo override e fallback.

## 4. Políticas de eficiência propostas ao Agente 05

| Política | Proposta de integração | Como aceitar |
|---|---|---|
| Janela visível | Conservar 30 FPS como ponto de partida e o controle de FPS já existente. | Comparar movimento, CPU do processo e FPS observado no mesmo hardware. |
| Janela minimizada/oculta | Verificar minimização/visibilidade com a frente de plataforma; impedir `step_frame` no caminho realmente oculto. | Contador de rasterizações por estado no ponto do render; medir no Windows. |
| Agendamento oculto | Suspender repaint frequente; manter uma verificação lenta apenas se necessária ao lifecycle. | O caminho `ui` não pode reagendar um frame rápido depois de `logic` pedir 1 s. |
| Retomada | Recalcular idade antes de mostrar dados, preservar reset de `last_step` e pedir um repaint imediato. | Sem salto acumulado da viagem e sem rótulo de CPU/RAM recente para valor vencido. |
| Amostragem adaptativa | Manter inicialmente 1 s; só tornar mais lenta quando oculto se a medição demonstrar ganho. | Atualizar também o limite de idade; não criar vários workers ao alternar estado. |
| Disco e outros contadores | Adicionar apenas depois de escolher a grandeza e medir o backend. | Sem coleta de processos/IO por frame; sem dependência ou permissão nova implícita. |

**Não foi implementado FPS menor em minimização nesta frente.** A baseline calcula `hidden` por `viewport().visible() == Some(false)` e pede repaint lento em `logic`, mas `ui()` continua chamando `step_frame` e reagendando. A existência desse flag não comprova o estado Win32, oclusão nem ausência de trabalho de renderização.

O protocolo de medição separa cenário declarado, `hidden` observado nos extremos e contagem de rasterizações. Para contagem exata de frames ocultos seria necessário acumular `rendered_frames_visible`/`rendered_frames_hidden` no local em que a cena é reconstruída, junto de evidência do estado nativo. Isso deve ser integrado pela frente de UI/plataforma, sem alterar a semântica dos contadores antigos.

## 5. Compatibilidade do diagnóstico: schema v2

A assinatura Rust permanece igual, mas o **formato JSONL evolui**. Leitores antigos que exigem bytes sempre numéricos devem ser adaptados.

| Campo | v2 |
|---|---|
| `diagnostics_schema_version` | Novo, valor `2` nos objetos de métricas. |
| `metric_freshness` | Novo, `current`, `stale` ou `null` se o chamador não forneceu classificação. |
| `cpu_percent`, `ram_percent` | Número válido ou `null`; valores stale são removidos. |
| `used_memory_bytes`, `total_memory_bytes` | **Agora podem ser `null`** quando RAM não está disponível ou está stale. |
| `metric_status` | Estado de coleta original preservado. Consultar também frescor. |
| `effective_cpu_percent` | Preservado: entrada do modelo, incluindo simulação manual ou fallback. |
| Frames, tempo, modo e demais campos | Preservados. |

Exemplo de erro recente sem leitura:

```json
{
  "diagnostics_schema_version": 2,
  "metric_freshness": "current",
  "metric_status": "unavailable",
  "cpu_percent": null,
  "ram_percent": null,
  "used_memory_bytes": null,
  "total_memory_bytes": null,
  "effective_cpu_percent": 0.0
}
```

O último zero é a entrada de fallback da simulação, **não** uma medição de CPU. `benchmarks/analyze.py` aceita logs antigos e novos; os cálculos de FPS usam contadores e timestamps, sem depender de RAM numérica.

O logger continua com serialização e flush síncronos opt-in, aproximadamente uma vez por segundo. A garantia de não aguardar a coleta não equivale a I/O de diagnóstico sem bloqueio. O protocolo prevê comparação com e sem logging.

## 6. Integração dos benchmarks

`benchmarks/run-windows.ps1` chama o script original e acrescenta metadados. Não foi necessário alterar `scripts/measure-windows.ps1`; não há patch oculto de script compartilhado. A execução é explícita e não muda plano de energia nem cria carga artificial.

Use `docs/performance_windows.md`. O script PowerShell ainda deve ser executado em Windows real. O analisador tem testes Python independentes; isso não valida as chamadas nativas do coletor.

O coletor original não observa o intervalo entre o último poll de processo e seu encerramento. Por isso o analisador informa CPU acumulada observada e `lifetime_cpu_total_complete=false`, sem prometer tempo integral até a última instrução. Se a medição integral for indispensável, propor um patch separado do coletor que leia o contador final pelo handle antes do dispose e valide essa semântica no Windows.

## 7. Três riscos e mitigação

1. **Validação de plataforma e falhas silenciosas de PDH.** O check para Windows não é execução nativa, e `sysinfo` pode ocultar certos erros. Executar o protocolo em Windows e não prometer detecção que a API não fornece; detalhes em `docs/performance_metrics.md`.
2. **Consulta de SO/driver travada.** O sinal interrompe o timer, mas não cancela uma chamada nativa em andamento; o join pode demorar. Manter a limitação explícita e investigar com captura nativa se ocorrer. Não matar a thread à força.
3. **Integração visual e leitores de diagnóstico.** RAM em cache, minimização e parser JSON antigo precisam das adaptações acima. Aplicar pela frente responsável e repetir os cenários visível/minimizado/retomada. Não declarar economia de energia nem render oculto zero sem medição.

## 8. Rollback

Para reverter só o comportamento, restaurar do pacote original os três arquivos: `src/metrics/mod.rs`, `src/metrics/sysinfo_provider.rs`, `src/diagnostics.rs`. Remover `tests/metrics_worker.rs`, que exercita APIs novas. As ferramentas/docs novas podem permanecer como documentação ou ser removidas em conjunto.

`benchmarks/agent04.patch` contém as mudanças de código e ferramentas desta entrega. Em uma cópia sem edições posteriores, `git apply --check --reverse benchmarks/agent04.patch` verifica a reversão; então `git apply --reverse benchmarks/agent04.patch`. Fazer isso na raiz de `cpu-express`. Se outra frente já alterou os arquivos, resolver o diff em vez de sobrescrever trabalho alheio.

O lockfile e as preferências do usuário não precisam ser revertidos: não foram modificados. O bug de reter a amostra antiga retorna se o worker original for restaurado.
