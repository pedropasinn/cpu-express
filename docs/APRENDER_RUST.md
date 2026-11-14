# Aprender Rust com o CPU Express

O CPU Express tem duas perguntas independentes: **quanto da CPU está em uso?** e **como o trem deve reagir a esse número?** A primeira exige conversar com o sistema operacional. A segunda é uma pequena simulação matemática, que podemos testar com valores inventados sem criar carga no computador.

Essa separação permite aprender Rust por partes. Você não precisa compreender a API do Windows antes de experimentar a aceleração do trem.

## 1. Por onde começar a leitura

| Ordem | Arquivo | Pergunta que ele responde |
|---|---|---|
| 1 | `src/config.rs` | Quais parâmetros definem o comportamento? |
| 2 | `src/simulation/train.rs` | Como CPU, velocidade e tempo se relacionam? |
| 3 | `tests/simulation.rs` | Como sabemos que essas relações estão corretas? |
| 4 | `src/simulation/world.rs` | Como avançam cenário e fumaça? |
| 5 | `src/rendering/scene.rs` e `sprites.rs` | Como transformar estado em pixels? |
| 6 | `src/metrics/` | Como obter uma leitura real sem bloquear a janela? |
| 7 | `src/app.rs` | Quem coordena essas partes e controla o relógio? |

`main.rs` é o ponto de entrada, mas a parte mais instrutiva da simulação está em `train.rs`.

## 2. `struct`: um estado com campos bem definidos

O trem guarda três números: velocidade atual, velocidade desejada e ângulo das rodas. Eles pertencem a uma `struct`, chamada `Train`.

Os campos são privados. Código externo consulta a velocidade por `speed_kmh()` e altera o trem por `update(...)`. Assim, outro módulo não consegue atribuir acidentalmente `NaN` à velocidade ou um ângulo ilimitado às rodas.

`Config` também é uma `struct`, mas seus campos são públicos porque representam parâmetros que queremos editar. `normalized()` transforma valores inválidos ou fora dos limites em uma configuração utilizável. Esse método consome uma configuração e devolve a versão normalizada:

```rust
let config = Config::default().normalized();
```

O código interno às vezes usa `config.clone().normalized()`. `clone()` cria outra configuração; não altera a original. Na configuração atual, os campos são números e booleanos, portanto essa cópia é pequena. Clonar uma estrutura não é sempre barato: uma estrutura com um grande `Vec` ou `String` poderia copiar muito mais memória.

## 3. A matemática fica dentro de um método

Com os parâmetros iniciais, a velocidade desejada é:

\[
v^* = 12 + 0{,}88u,
\]

em que \(u\) é o uso de CPU entre 0 e 100. São unidades fictícias do jogo.

A velocidade atual segue:

\[
\frac{dv}{dt}=k(v^*-v),\qquad k=1{,}8\ \mathrm{s}^{-1}.
\]

O método `Train::update` usa a solução dessa equação quando a CPU permanece constante durante um passo:

\[
v(t+\Delta t)=v^*+(v(t)-v^*)e^{-k\Delta t}.
\]

No código, `exp_m1()` calcula a expressão equivalente de forma mais precisa quando o intervalo é muito pequeno. O método também integra a velocidade durante o passo, retornando a distância percorrida pelo cenário. Usar a integral evita fazer a distância depender da quantidade de frames.

Para um trem que começa a 12 km/h, temos aproximadamente:

| CPU constante | Alvo | Velocidade após 1 segundo |
|---|---:|---:|
| 0% | 12 km/h | 12,00 km/h |
| 50% | 56 km/h | 48,73 km/h |
| 100% | 100 km/h | 85,45 km/h |

O trem se aproxima progressivamente do alvo. `56 km/h` é o alvo de 50% de CPU; não é a velocidade instantânea logo após mudar o controle.

Este trecho usa a mesma API do aplicativo:

```rust
use cpu_express::{config::Config, simulation::Train};

let config = Config::default();
let mut train = Train::new(&config);

let distance = train.update(50.0, 1.0, &config);
println!("Alvo: {} km/h", train.target_speed_kmh());
println!("Atual: {:.2} km/h", train.speed_kmh());
println!("Cenário avançou: {distance:.2} pixels");
```

`50.0` é um número passado à simulação. Esse trecho não consulta nem força a CPU da máquina.

## 4. Ownership: cada estado tem um dono

Em `app.rs`, `CpuExpressApp` possui seu `Train`, seu `World`, seu `SceneRenderer`, a configuração e o controle do coletor. Eles são campos da aplicação.

| Valor | Dono | Motivo |
|---|---|---|
| Velocidade e fase das rodas | `Train` | Manter as regras de movimento juntas |
| Deslocamento e partículas | `World` | Manter a evolução do cenário junta |
| Buffer de pixels | `SceneRenderer` | Reutilizar a memória entre frames |
| Recursos do coletor | `MetricsWorker` | Controlar o período de coleta e seu encerramento |
| Os objetos acima | `CpuExpressApp` | Coordenar a vida do aplicativo |

Quando um dono sai de escopo, Rust descarta seus campos. `Vec` libera sua memória; o identificador da textura libera sua referência ao recurso gráfico; o coletor tem uma rotina de encerramento própria. Não há um `free()` espalhado por cada módulo.

Isso não elimina a necessidade de pensar no ciclo de vida. Uma thread, por exemplo, precisa receber o pedido para terminar; só liberar a variável que a representa não implica esperar automaticamente sua conclusão.

## 5. Borrowing: emprestar o estado para uma operação

Compare duas assinaturas reais:

```rust
pub fn update(
    &mut self,
    cpu_percent: f32,
    dt_seconds: f64,
    config: &Config,
) -> f64
```

```rust
pub fn render(&mut self, train: &Train, world: &World) -> &[Color32]
```

`&mut self` significa que a operação recebe acesso exclusivo temporário ao objeto. `Train::update` precisa disso porque altera velocidade e rodas. `SceneRenderer::render` também precisa porque escreve no seu buffer de pixels.

`&Train`, `&World` e `&Config` são empréstimos para leitura. O renderer consegue consultar o trem, mas não acelerar, frear ou criar partículas nele. A separação não depende apenas de um comentário: a assinatura restringe o que o código pode fazer.

É possível ter vários empréstimos imutáveis ao mesmo tempo. Um empréstimo mutável ativo exige exclusividade sobre o valor emprestado. O compilador verifica essas regras.

### Por que `render()` retorna um slice?

`&[Color32]` é uma vista emprestada dos pixels já pertencentes ao renderer. Não é um novo vetor. A aplicação pode ler essa vista enquanto o empréstimo for válido, mas não pode manter a vista em uso e simultaneamente pedir outra escrita no mesmo renderer.

No aplicativo, `.to_vec()` cria uma cópia destinada ao upload da textura do egui. O buffer original permanece no renderer. Essa escolha torna a fronteira de ownership simples e explícita; não significa que desenhar uma imagem sempre dispense cópias.

O buffer contém 320×70 cores RGBA, ou 89.600 bytes. A interface amplia a textura por vizinho mais próximo para a janela de 640×140 pontos lógicos. Assim, cada célula artística é armazenada uma vez, sem alterar a aparência de blocos de 2×2. A implementação anterior guardava quatro cópias de cada célula, em 358.400 bytes: a mudança reduz em 75% o buffer e os bytes enviados por frame. O exemplo determinístico a 50% de CPU gerou uma imagem final idêntica pixel a pixel antes e depois. A melhora do consumo total de CPU deve ser avaliada em medições do processo, porque a janela e o driver gráfico também têm custos.

O canvas tem uma vida útil explícita:

```rust
struct Canvas<'a> {
    pixels: &'a mut [Color32],
}
```

`'a` liga o tempo de uso do canvas ao empréstimo do buffer. O canvas não pode sobreviver aos pixels que está usando. Não é um relógio ou um tempo medido em segundos; é uma relação verificada pelo compilador.

## 6. `Option`, `Result` e estados de leitura

`Option<T>` representa um valor presente ou ausente. Na aplicação:

- `cpu_override: Option<f32>` distingue o modo automático do valor manual.
- `texture: Option<TextureHandle>` representa a textura antes e depois de sua criação.
- `poll() -> Option<MetricsSample>` retorna uma nova amostra quando ela existe, sem esperar por ela.
- A leitura de CPU também pode estar ausente enquanto o coletor ainda obtém as medições necessárias.

`None` não deve ser confundido com uma CPU medida em 0%. A interface informa quando aguarda uma leitura; a simulação pode adotar um valor de contingência para continuar funcionando.

`Result<T, E>` representa sucesso ou erro de uma operação. Criar uma thread ou gravar um arquivo pode falhar. Em `MetricsWorker::spawn(...)`, a aplicação trata o erro e continua sem o coletor, em vez de pressupor que a criação sempre funciona.

O `?` usado em `examples/export_frame.rs` significa: obtenha o valor se a operação deu certo; em caso de erro, retorne o erro para o chamador. Ele não ignora o erro e não encerra a aplicação por um `panic`.

### Um ciclo de estados simples

O coletor distingue três situações por um `enum`:

```rust
MetricStatus::WarmingUp
MetricStatus::Ready
MetricStatus::Unavailable(String)
```

No início, é necessário estabelecer uma referência para medir utilização de CPU. Uma leitura utilizável muda o estado para `Ready`. Indisponibilidade pode carregar uma explicação na variante `Unavailable`. Quando a situação melhora, uma amostra posterior pode voltar a estar pronta.

Um `match` enumera os casos. Ao acrescentar uma variante no futuro, o compilador ajuda a localizar os lugares que precisam tratar a nova situação. Isso é mais seguro do que espalhar textos livres como `"ok"`, `"loading"` ou `"erro"` pelo código.

## 7. Traits e threads: trocar a fonte sem trocar a simulação

`MetricsProvider` descreve o comportamento que um provedor de métricas deve oferecer. O provedor com `sysinfo` implementa essa interface com dados da máquina; `ManualProvider` implementa a mesma interface com dados controlados.

Seu contrato principal é `sample(&mut self) -> MetricsSample`. O empréstimo mutável permite que o provedor real guarde a leitura anterior, necessária para calcular a utilização de CPU. O método `minimum_interval()` informa quanto tempo precisa existir entre atualizações.

```rust
use cpu_express::metrics::{ManualProvider, MetricsProvider};

let mut provider = ManualProvider::new(50.0);
let sample = provider.sample();
assert_eq!(sample.cpu_percent, Some(50.0));
assert_eq!(sample.ram_percent, None);
```

Importamos o trait para usar seu método. O provedor manual não inventa dados de RAM: ele oferece apenas o valor de CPU definido pelo experimento.

Essa é uma utilização concreta de um `trait`: o restante da coleta pode trabalhar com o comportamento exigido, sem depender de uma implementação específica. O núcleo `simulation` nem sequer precisa importar `sysinfo`.

O modo manual da janela também pode substituir a CPU usada na simulação. Isso permite observar a aceleração e a frenagem sem criar carga artificial na máquina.

A aplicação mantém dois ritmos distintos:

| Trabalho | Ritmo inicial | Responsabilidade |
|---|---:|---|
| Ler métricas | Aproximadamente 1 vez por segundo | Thread coletora |
| Atualizar a viagem e sua imagem | Aproximadamente 30 vezes por segundo | Thread da janela |

O provedor real fica no worker. A janela recebe amostras pequenas; não fica esperando o período necessário para medir a CPU. Quando nenhuma amostra nova está pronta, segue usando o estado disponível e verifica se ele está desatualizado.

`mpsc::sync_channel(1)` cria uma fila com capacidade para uma amostra. `try_send()` publica sem bloquear; se a fila está cheia, a nova amostra é descartada. Essa política impede acumular um histórico quando a janela deixa de consumir dados. O horário `measured_at` permite reconhecer uma amostra antiga. Esse canal não promete conservar sempre a amostra mais recente: mantém a que já estava esperando até ela ser consumida.

O canal de parada é separado. `recv_timeout()` espera o próximo período de coleta, mas acorda antecipadamente quando recebe a ordem de parar. Por isso não precisamos de um loop consultando continuamente uma variável para descobrir se deve encerrar.

Ao encontrar `move` na criação da thread, leia como transferência dos valores capturados para a função que rodará nela. `Send` descreve valores que podem atravessar a fronteira de threads de forma segura. `'static` nesse contexto impede que a thread dependa de referências locais que podem desaparecer antes dela; isso não obriga o valor a existir até o fim do processo.

O encerramento do worker sinaliza a parada e espera a thread terminar. Essa regra fica junto do tipo que possui os recursos da coleta. A janela não precisa conhecer detalhes de encerramento do provedor real.

`MetricsWorker` implementa `Drop`, que chama `shutdown()`. O identificador da thread é um `Option<JoinHandle<()>>`; `take()` retira o identificador e deixa `None`. Assim, `join()` acontece no máximo uma vez, mesmo que alguém já tenha chamado `shutdown()` explicitamente. A parada é cooperativa: uma consulta ao sistema operacional já em execução precisa terminar antes do `join()` concluir.

## 8. Experimentos curtos, com resultado esperado

Faça uma alteração de cada vez e use o modo manual. Nenhum exercício abaixo exige estressar a CPU real.

### A. Aceleração: altere `smoothing_k`

Em `Config::default()`, experimente `0.5`, `1.8` e `4.0`. Execute com CPU manual em 100%:

```bash
cargo run -- --cpu 100
```

**Resultado esperado:** o alvo continua em 100 km/h. Com `k=0.5`, o trem demora mais para chegar perto dele; com `k=4.0`, reage mais rápido. Os FPS permanecem independentes dessa mudança.

Restaure `1.8` antes de executar os testes que usam o comportamento inicial como referência.

### B. Independência dos frames

Leia o teste `speed_distance_and_wheels_are_independent_of_frame_rate` em `tests/simulation.rs`:

```bash
cargo test speed_distance_and_wheels_are_independent_of_frame_rate
```

Ele calcula a mesma sequência de CPU com diferentes subdivisões de tempo.

**Resultado esperado:** velocidade final, distância e ângulo final praticamente iguais, com diferenças limitadas pela precisão numérica. Não é um teste de desempenho nem uma medição da CPU da máquina.

### C. Uma paleta própria

Em `sprites.rs`, altere as cores dos vagões no pequeno array da função `coach`. Gere uma imagem com o renderer real:

```bash
cargo run --example export_frame -- frame.ppm 50
```

**Resultado esperado:** o arquivo contém uma cena de 640×140 após cinco segundos de simulação a 50% de CPU manual. Mudam as cores; a simulação e a leitura da máquina continuam independentes. PPM é um formato simples de imagem; pode ser aberto em programas compatíveis ou convertido para PNG.

### D. Entenda o papel de cada dono

Localize estas três operações em `app.rs`: `train.update(...)`, `world.update(...)` e `renderer.render(...)`. Anote quais objetos cada chamada modifica e quais apenas consulta.

**Resultado esperado:** a primeira altera o trem e devolve distância; a segunda altera o mundo; a terceira altera apenas os pixels. Essa ordem explica como um número medido vira uma viagem animada.

### E. Diferencie ausência de zero

Leia os caminhos que criam `WarmingUp`, retornam `None` em `poll()` e mostram CPU aguardando. Depois compare com o modo manual em 0%:

```bash
cargo run -- --cpu 0
```

**Resultado esperado:** o modo manual possui um valor definido e alvo de 12 km/h. A ausência de primeira leitura é outro estado, mesmo quando a contingência da simulação também leva ao mínimo.

## 9. O que não precisamos complicar

O projeto não usa um grafo de entidades, um motor de física completo ou estado global mutável para representar um trem. Structs pequenas, métodos, um trait de coleta e empréstimos explícitos atendem ao problema.

Ao estudar uma função, procure sempre três informações: **quem possui os dados, quem pode alterá-los e por quanto tempo eles são emprestados**. Neste código, essas perguntas levam diretamente do modelo matemático ao comportamento da janela.
