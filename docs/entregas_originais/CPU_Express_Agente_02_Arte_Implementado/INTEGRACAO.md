# Integração — Agente 02 / CPU Express

## O que já funciona sem editar outra frente

Copiar as mudanças de `src/rendering/**` para a linha A preserva exatamente:

```rust
SceneRenderer::new()
SceneRenderer::render(&mut self, train: &Train, world: &World) -> &[Color32]
```

A UI existente usa o mesmo buffer 320×70 e o mesmo filtro nearest-neighbor. A nova viagem é o tema padrão. Todas as mudanças no comportamento visual derivam dos getters existentes de `Train` e `World`; não há novo campo obrigatório na simulação, leitura de relógio do sistema ou chamada a `sysinfo`.

## Aplicar o patch na base de integração

O arquivo `PATCH_ARTE.diff` contém somente código do renderer, seus testes/exemplos e a documentação em `assets/sprites/README.md`. Foi conferido contra a linha A fornecida. Na raiz do projeto de destino:

```bash
git apply --check /caminho/PATCH_ARTE.diff
git apply /caminho/PATCH_ARTE.diff
cargo fmt --check
cargo test --locked
```

O ZIP também contém a árvore completa para inspeção e compilação isolada. Para juntar cinco agentes, aplique o patch desta frente em vez de sobrescrever a árvore inteira. Se outro agente já mudou arquivos do renderer, resolva esse conflito conscientemente. Os relatórios e a galeria podem ser copiados separadamente. Não é necessário alterar `Cargo.toml`, `Cargo.lock`, `src/lib.rs` ou o `README.md` compartilhado.

## Interface opcional para UX / Agente 05

```rust
use cpu_express::rendering::{
    CoachStyle, RenderQuality, SceneOptions, SceneRenderer, SceneTheme, Weather,
};

let mut renderer = SceneRenderer::new();
renderer.set_quality(RenderQuality::Low); // Full é o padrão
renderer.set_options(SceneOptions {
    day_cycle: true,
    clouds: true,
    weather: Weather::Rain, // Clear é o padrão
    details: true,
});
renderer.set_coaches([
    CoachStyle::Observation, // cauda, posição x = 58
    CoachStyle::Sleeper,     // centro, x = 100
    CoachStyle::Freight,     // junto da locomotiva, x = 142
]);

// Recuperar o visual antigo, ignorando todos os controles novos:
renderer.set_theme(SceneTheme::Legacy);
// Voltar à viagem:
renderer.set_theme(SceneTheme::Journey);
```

`Low` suprime nuvens, chuva, relevo intermediário, detalhes de cenário e a segunda pincelada de cada partícula de fumaça. A quantidade de partículas no `World`, sua emissão, as rodas e a física permanecem intactas. `clouds=false`, `day_cycle=false` e `details=false` também podem ser usados individualmente em `Full`.

A persistência dessas escolhas pertence à configuração/UX. Nenhum campo foi acrescentado por esta frente; o JSON antigo continua intocado. Se a UX criar campos, preserve `#[serde(default)]` e use defaults equivalentes aos de `SceneOptions`.

## Adaptador opcional para Journey / Agente 03

O adaptador só precisa traduzir o ambiente da jornada para os tipos visuais. Ele não pode reinicializar offsets de parallax:

```rust
use cpu_express::rendering::{Biome, SceneContext};

renderer.set_scene_context(Some(SceneContext {
    biome: Biome::GreenValley,
    next_biome: Biome::Coast,
    blend: 0.35,
}));

// Fixar apenas um ambiente:
renderer.set_scene_context(Some(SceneContext::new(Biome::HighDesert)));

// Devolver a escolha à rota automática baseada no World:
renderer.set_scene_context(None);
```

`blend=0` é o primeiro ambiente; `blend=1` é o segundo. Finito fora desse intervalo é limitado; NaN/±infinito vira zero. O setter é somente apresentação: não move o trem, não altera a distância e não cria eventos de domínio. `scene_context(&world)` consulta o contexto efetivo; `route_at(distance_px)` permite prever a rota sem construir um renderer.

A rota automática usa trechos de 2.048 unidades de deslocamento do baseline e mistura os últimos 512. A sequência é vale → litoral → interior árido → vale, com período 6.144. Exatamente 30 ciclos cabem no wrap de `World` de 184.320; a virada não reinicia as camadas. A mistura automática usa smoothstep e interpolação RGB quantizada em 257 níveis. Trem/fumaça não sofrem crossfade.

Se o agente 03 mudar `SCROLL_PERIOD_PX`, preserve a divisibilidade pelos períodos usados ou adapte os getters; o teste `route_cycles_compatibly_with_world_and_has_no_stateful_reset` detecta incompatibilidade da rota. Os períodos de cada camada estão na tabela abaixo.

| Camada | Fator sobre deslocamento do World | Período visual nativo | Período no World |
|---|---:|---:|---:|
| Relevo distante | 1/32 | 320 px | 10.240 |
| Relevo intermediário / barcos | 1/16 | 320 px | 5.120 |
| Colinas / dunas | 1/8 | 320 px | 2.560 |
| Objetos próximos | 1/4 | 320 px | 1.280 |
| Cerca | 1/4 | 24 px | 96 |
| Primeiro plano | 1/2 | 160 px | 320 |
| Dormentes | 1/2 | 10 px | 20 |

A luz usa `World::elapsed_seconds()` em ciclo artístico de 600 segundos. Não representa hora local. Nuvens usam ciclo de 400 segundos; a chuva repete sua trajetória a cada 20 segundos. Esses ciclos são independentes dos FPS, sem estado acumulado dentro do renderer.

## Regras de integração preservadas

A cena utiliza sempre os pixels atuais de `World`; o contador de distância real de Journey pode continuar independente. O renderer não decide carga de CPU, desempenho, velocidade ou física. Trilho de contato em y=57; rodas grandes com centro y=54/r=3; roda dianteira y=55/r=2; saída de fumaça em (220,32) na textura nativa. Esses pontos são cobertos por testes.

`Palette::for_biome` fornece os papéis de cor-base; `Palette::at_time` aplica o ciclo. Ambas são ferramentas visuais sem efeito colateral. Mudanças na arte devem continuar limitadas ao framebuffer nativo.

## Demonstração e custo

```bash
cargo run --release --locked --example scene_gallery -- gallery saida
cargo run --release --locked --example scene_gallery -- sequence frames 240 20
cargo run --release --locked --example scene_bench > scene-benchmark.json
```

O terceiro comando é um microbenchmark finito: mede o renderer e o preparo da textura no host. Não constitui leitura de CPU real do jogo nem benchmark de consumo no Windows. Resultados e limites desta execução: `TESTES.md`.
