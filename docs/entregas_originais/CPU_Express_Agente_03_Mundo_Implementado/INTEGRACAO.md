# Integração da frente Mundo

## Estado da entrega

O núcleo de viagem funciona e `World::update(dt, distance, speed)` já o alimenta junto do cenário. O código do app, do renderer, da configuração e das métricas não foi editado. A interface original continua desenhando seu cenário de referência; os trechos e as conquistas novas ainda não estão apresentados visualmente.

Todos os tipos abaixo estão reexportados em `cpu_express::simulation`.

## API pública nova

| Tipo/API | Uso |
| --- | --- |
| `Route::default()`, `Route::from_seed(u64)` | Construir o circuito imutável. |
| `Route::{stations, segments, station, segment, seed, name, length_units}` | Ler a rota; `segment` retorna `Option` para índice inválido. |
| `StationId::{Aurora, Pines, Ridge, Harbor}` | Identificadores estáveis, com `index()` e `name()`. |
| `Station`, `RouteSegment` | Dados das estações e dos quatro trechos de saída. |
| `Biome::{Meadow, Forest, Highland, Coast}` | Dica de conteúdo, sem paleta ou desenho. |
| `Journey::default()`, `from_seed(u64)`, `new(Route)` | Estado de domínio independente da UI e do trem. |
| `Journey::advance(dt, distance_units) -> AdvanceOutcome` | Aplicar tempo ativo e deslocamento integrado. |
| `Journey::snapshot() -> JourneySnapshot`, `route() -> &Route` | Consultar o estado sem obter acesso mutável ao mundo. |
| `Journey::{set_paused, is_paused}` | Pausa reversível da jornada. |
| `Journey::{pop_event, drain_events, pending_event_count, dropped_event_count}` | Consumir e acompanhar a fila limitada. |
| `World::with_seed(u64)` | Construir cenário e jornada reproduzíveis. |
| `World::{journey, journey_snapshot, set_paused, is_paused}` | Consultar a jornada e controlar sua pausa. |
| `World::{pop_travel_event, drain_travel_events}` | Consumir notificações da jornada pertencente ao mundo. |
| `World::advance_train(&mut Train, cpu, dt, &Config)` | Coordenar um passo de trem, cenário e jornada com as mesmas regras. |
| `Achievement`, `Achievements` | Três conquistas locais e flags que não dependem da fila. |

Constantes: `STATION_COUNT = 4`, `DEFAULT_JOURNEY_SEED`, `MAX_TRAVEL_EVENTS = 64`, `LONG_JOURNEY_DISTANCE_UNITS = 36_000`, `MAX_ACTIVE_DELTA_SECONDS = 1`, `MAX_TRAVEL_UNITS_PER_SECOND = 275` e `TRAVEL_UNITS_PER_SCENE_PIXEL = 1`.

As APIs antigas de `Train`/`World` e as constantes do cenário permanecem disponíveis.

## 1. Patch mínimo proposto para o coordenador de UX

O baseline em `step_frame` limita `dt` a 0,25 segundo antes de chamar `Train::update` e `World::update`. Esse limite evita uma grande recuperação de tempo, mas elimina a informação de que o sistema ficou suspenso. A proposta abaixo **não foi aplicada a `app.rs`**.

Substituir o cálculo de `dt` e as duas chamadas de atualização por:

```rust
// Dentro de step_frame, usando o `now` já obtido nessa função.
let raw_dt = now.duration_since(self.last_step).as_secs_f64();
self.last_step = now;
let cpu = self.cpu_percent();
let _outcome = self.world.advance_train(
    &mut self.train,
    cpu,
    raw_dt,
    &self.config,
);
```

Não chamar `train.update` ou `world.update` novamente para esse mesmo passo. O adaptador já faz ambos. Ele usa o deslocamento analítico do trem e só confirma o estado do trem quando a jornada aceita o passo. Não gera carga de CPU, não espera tempo e não coleta métricas.

Se houver controle de pausa, o callback deve atualizar também o relógio do app nas duas transições:

```rust
// `paused` vem de uma ação explícita do usuário.
self.world.set_paused(paused);
self.last_step = std::time::Instant::now();
```

Continuar renovando `last_step` nos frames pausados. Isso impede que uma pausa curta seja creditada como movimento ao retomar. Pausa não altera conquistas, fila, seed ou velocidade guardada. Não usar somente `World::set_paused` enquanto se continua chamando `Train::update` separadamente: essa combinação pausa o mundo, mas o trem continua atualizando sua velocidade/fase. O adaptador evita essa divergência.

### Resultado de um passo

| `AdvanceOutcome` | Significado |
| --- | --- |
| `Advanced { seconds, distance_units }` | Passo aceito; zero deslocamento ainda conta tempo ativo. |
| `Paused` | Tudo congelado pelo controle de pausa. |
| `NoElapsedTime` | `dt == 0`, sem avanço. |
| `Suspended` | `dt > 1` segundo, descartado integralmente. |
| `InvalidDelta` | Tempo negativo, NaN ou infinito, ignorado. |
| `InvalidDistance` | Distância negativa, não finita ou incompatível com o limite de velocidade, ignorada. |

Esses resultados são sinais para o código chamador; não precisam virar avisos na tela. A jornada não fabrica eventos de erro/suspensão e não cria uma fila de problemas para mostrar depois.

`Train::update(cpu, 0, config)` continua atualizando o alvo e normalizando a velocidade, como no baseline. A garantia de congelamento conjunto vale para `World::advance_train`; não muda o contrato histórico dessa chamada direta do trem.

## 2. Leitura para HUD ou painel

```rust
let snapshot = self.world.journey_snapshot();
let next_station = snapshot.next_station.name;
let remaining_units = snapshot.distance_to_next_station_units;
let current_leg_progress = snapshot.segment_progress;
let circuit_progress = snapshot.route_progress;
let cumulative_units = snapshot.total_distance_units;
let laps = snapshot.completed_laps;
let achievements = snapshot.achievements;
```

Os nomes acima ilustram dados disponíveis; o renderer/UX decide onde exibi-los. Rotular as distâncias como **unidades de viagem**, não quilômetros reais. `segment_progress` e `route_progress` são proporções em `[0, 1]`; o progresso de circuito volta a zero ao completar a volta. `total_distance_units` nunca é zerado por uma estação ou volta.

`JourneySnapshot` também expõe `seed`, `route_name`, `route_length_units`, `route_position_units`, `active_seconds`, `route_segment`, `at_station`, `stations_visited`, `discovered_stations`, `pending_events`, `dropped_events` e `paused`.

## 3. Eventos visuais

```rust
use cpu_express::simulation::TravelEvent;

for event in self.world.drain_travel_events() {
    match event {
        TravelEvent::StationArrived { station, .. } => {
            // Entregar `station.name()` a uma pequena indicação visual.
            // Não abrir diálogo, roubar foco ou alterar a velocidade.
        }
        TravelEvent::StationDeparted { station, .. } => {
            // Animação opcional de passagem/saída.
        }
        TravelEvent::CircuitCompleted { completed_laps, .. } => {
            // Atualizar uma indicação discreta de voltas.
        }
        TravelEvent::AchievementUnlocked { achievement, .. } => {
            // Indicação opcional de conquista; o registro vive no snapshot.
        }
    }
}
```

O exemplo acima é uma proposta de consumo para a outra frente, não código já conectado à tela. A chegada é emitida na fronteira da estação. A partida vem imediatamente se o mesmo passo ainda tem deslocamento; se não, no primeiro passo posterior que anda. Isso não impõe parada ou penalidade por CPU baixa.

Os eventos têm posição lógica (`distance_units`) e a chegada também tem número cumulativo da visita (`visit`). Não se inventa um horário de chegada dentro do frame: o recebimento ocorre no frame que atravessa a fronteira. Um resíduo de até `1e-9` unidade pode ser ajustado na posição local; a distância lógica da partida coincide com a da chegada.

A fila guarda no máximo 64 eventos. Ao lotar, remove o mais antigo antes de inserir o novo e incrementa um contador saturante. O estado de progresso/conquistas não é reconstruído a partir dessa fila. Para mostrar apenas uma notificação por frame, usar `pop_travel_event()` e manter uma política visual curta; **não** usar `drain_travel_events().take(1)`, pois descartar o iterador `Drain` remove também os itens restantes.

## 4. Integração com o Agente 02: biomas

O renderer já recebe `&World`; portanto pode ler os dados sem mudar sua assinatura:

```rust
let snapshot = world.journey_snapshot();
let biome_hint = snapshot.route_segment.biome_hint;
let blend_progress = snapshot.segment_progress;
```

`biome_hint` é do trecho de saída da estação: prado, floresta, serra e litoral. A arte decide cores, cenário, sinalização, duração de transição e desenho. Esta frente não inclui uma regra que obrigue a trocar a imagem bruscamente na estação. O renderer original continua sendo a opção de referência.

## 5. Persistência e versão futura

Esta entrega não escreve nem lê progresso. Ao reiniciar, a jornada começa na origem. A seed por si só reconstrói a rota; ela não restaura distância, descobertas ou conquistas.

Para persistência futura, o coordenador deve definir um DTO versionado, validar números finitos, IDs e limites, e criar uma API de restauração validada. Não atribuir campos do snapshot a estado interno nem inferir tempo offline pelo relógio. Mudança no algoritmo de geração de rota exige versão da rota: os fixtures de seed nos testes servem como contrato de reprodução. Nada disso exige uma alteração imediata no `Config` global nesta frente.

## 6. Demonstração e validação isolada

```powershell
cargo run --locked --example journey_demo
cargo run --locked --example journey_demo -- 42 720
```

Para compilar o núcleo sem eframe, sysinfo ou qualquer crate externa:

```powershell
rustc --edition 2024 -D warnings examples/journey_demo.rs -o journey_demo.exe
.\journey_demo.exe 42 720
rustc --edition 2024 -D warnings --test examples/journey_core_tests.rs -o journey_core_tests.exe
.\journey_core_tests.exe
```

Em Linux, trocar o nome de saída por `journey_demo`/`journey_core_tests` e executar com `./`. Esse uso é uma verificação do núcleo portátil, não um port nem uma validação do produto Windows. Os exemplos importam os arquivos reais por `#[path]`; não reimplementam o domínio.

## Limites da garantia numérica

A invariância por FPS vale para passos aceitos e a mesma entrada de CPU em cada intervalo de tempo. Um intervalo de 1,1 segundo é suspensão e será descartado; dois passos ativos de 0,55 segundo representam outra situação e avançam. Para acelerar testes de muitas horas, iterar passos de até um segundo. Não enviar uma hora em um único `dt`.

O envelope de 275 unidades/s corresponde aos limites da configuração atual. Se uma frente mudar o teto de 500 km/h fictícios ou a escala de 0,55 pixels/s, os limites compartilhados e seus testes precisarão ser revisados conjuntamente. Não aumentar o limite só para fazer entradas inválidas passarem.
