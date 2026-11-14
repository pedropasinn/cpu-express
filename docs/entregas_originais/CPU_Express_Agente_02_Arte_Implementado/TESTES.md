# Testes e medições — Agente 02 / Arte

**Data:** 13 de novembro de 2026. **Entrega:** agente02-arte-1, baseada na linha A fornecida. Fonte e lockfile originais preservados fora da frente de arte.

## Resultado desta rodada

| Verificação | Resultado |
|---|---|
| `cargo fmt --check` | Passou |
| `cargo clippy --offline --locked --all-targets -j 1 -- -D warnings` | Passou, sem avisos |
| `cargo test --offline --locked -j 1` | **45 testes passaram**, zero falhas |
| Release Linux com `host-validation` e dois exemplos novos | Compilado com sucesso |
| Seis cenas exportadas pelo renderer em debug e release | Pixels idênticos entre os perfis |
| Modo Legacy versus execução independente da fonte original | **Igualdade pixel a pixel** |
| Aplicação efetiva de `PATCH_ARTE.diff` sobre a base fornecida | Passou: 13 arquivos esperados idênticos; 41 arquivos não selecionados preservados |
| Janela real de desenvolvimento Linux / Mesa software | Abriu, exibiu a cena, renderizou 144 frames e fechou automaticamente com código 0 |
| Compilação cruzada Windows GNU x64 | **Compilou com sucesso**, features padrão, sem `host-validation` |
| Execução em Windows real | **Não executada neste ambiente** |

Os 45 testes são os 27 da base mais 18 da frente de arte: quatro testes dos sprites/vagões, 13 testes de contrato visual e um teste com alocador instrumentado. Nenhum teste de física, CPU, configuração ou posicionamento existente foi removido.

## Ambiente e comandos executados

- Host: Linux x86_64, kernel 6.18.57; CPU virtual identificada como AMD EPYC 9V74 80-Core Processor.
- Rust: `rustc 1.100.0 (0ef1bc386 2026-11-09)`; Cargo 1.100.0.
- `rust-toolchain.toml` preservado em 1.100.0; MSRV declarada no manifest continua 1.95. A MSRV 1.95 **não foi retestada**.
- Dependências do `Cargo.lock` já disponíveis; todos os comandos Cargo desta rodada usaram `--offline --locked` quando aplicável.
- Release com o perfil original: `opt-level = "s"`, LTO thin, uma unidade de codegen e símbolos removidos.
- Builds pesados executados com `-j 1`. Um primeiro build paralelo do baseline sofreu encerramento por pressão de recursos; a repetição serial passou. Isso não exigiu mudança no projeto.

Comandos de reprodução sem depender dos caminhos deste host:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

No Linux, a janela de investigação usa a feature existente:

```bash
cargo build --release --locked --features host-validation --bins   --example scene_gallery --example scene_bench -j 1
```

Esse modo não representa um port Linux de produto. A build do alvo Windows usa a fonte normal, sem `host-validation`.

## O que os testes de arte provam

- **Determinismo:** mesmo `Train`, `World`, opções e contexto resultam no mesmo frame; renderizações anteriores não alteram o resultado.
- **Memória:** shape 320×70, endereço de saída estável e zero chamadas de alocação/realloc durante `render()` e setters depois de `new()`. A alocação da `ColorImage` na UI é separada e continua existindo.
- **Opacidade:** todos os pixels devolvidos têm alpha 255 em todos os modos e transições testados.
- **Entradas extremas:** CPU/delta inválidos passam pelo contrato da simulação; NaN/±infinito em blend, valores fora de faixa e tempo finito extremo não corrompem a renderização.
- **Transições:** extremos iguais aos respectivos ambientes no mesmo deslocamento; interior opaco do trem não sofre mistura; rota compatível com o wrap do World; diferença no limite de wrap limitada à rasterização de um pixel.
- **Atmosfera:** nuvens e chuva alteram o Full, podem ser desligadas e são suprimidas no Low.
- **Geometria:** variantes diferentes ocupam o envelope dos vagões, preservam chaminé/biela e mantêm rodas em contato com o trilho y=57, sem desenhar abaixo dele.
- **Fallback:** Legacy ignora opções novas e volta ao original mesmo após troca de tema.

Hash FNV-1a64 dos bytes RGB nativos da fixture Legacy: `0xb1bc11a7571f5fc3`. Referência obtida pela execução do ZIP original: 150 atualizações de 1/30 s, CPU manual 50%. Além do hash, a comparação dos PNGs confirmou igualdade dos pixels.

A inspeção visual complementa os testes: seis imagens em tamanho real e frames inicial/intermediário/final da animação foram revisados. As três paisagens, os cinco vagões e o trem ao anoitecer ficaram legíveis; objetos entrando/saindo pelas bordas são recortes naturais do parallax. Isso não é uma prova automatizada de qualidade estética.

## Amostras e reprodução exata

As imagens em `assets/gallery/` são saídas reais de `SceneRenderer`, o mesmo do app. O manifesto registra CPU manual, seek inicial, aquecimento, tempo e deslocamento finais, contexto visual e hashes dos PPMs. Todos os PNGs foram comparados com seus PPMs e com a redução/ampliação nearest-neighbor 2×: **não há pixels suavizados**.

| Amostra | Configuração principal |
|---|---|
| `01_legacy` | Modo original, 5 segundos, 50% manual |
| `02_green_valley_morning` | Vale de manhã, vagões panorâmico/leito/carga |
| `03_coast_day` | Litoral, tempo total 180 segundos |
| `04_high_desert_sunset` | Interior árido, 350 segundos, carga/tanque/panorâmico |
| `05_valley_dusk_rain` | Vale, 450 segundos, chuva e iluminação |
| `06_valley_to_coast` | Mistura explícita de 50%, 180 segundos |

```bash
cargo run --release --locked --example scene_gallery -- gallery minha-galeria
cargo run --release --locked --example scene_gallery -- sequence meus-frames 240 20
```

A sequência contém 240 frames a 20 FPS, total de 12 segundos, com rota automática e CPU **manual simulada de 100%**. Começa após seek de 1.216 unidades mais cinco segundos de aquecimento e percorre a primeira transição. O avanço da simulação é calculado imediatamente, sem aguardar o tempo real e sem gerar carga artificial para manipular o sensor do jogo.

O GIF entregue é essa sequência convertida com FFmpeg:

```bash
ffmpeg -framerate 20 -i meus-frames/frame_%04d.ppm   -filter_complex "[0:v]split[a][b];[a]palettegen=stats_mode=full:reserve_transparent=0[p];[b][p]paletteuse=dither=none"   -loop 0 CPU_Express_Arte_Transicao.gif
```

O GIF usa paleta indexada e pode quantizar cores. Os PPMs/PNGs são a referência exata. O GIF repete o recorte de 12 segundos e, ao terminar, volta ao primeiro frame; esse reinício da prévia não corresponde a um salto no aplicativo.

## Microbenchmark do renderer e preparo de textura

```bash
cargo run --release --locked --example scene_bench > scene-benchmark.json
```

Metodologia: fixture fixa de cinco segundos a 50% manual, 1.000 renderizações de aquecimento por caso, sete rodadas de 10.000 iterações, `black_box` para preservar o trabalho, relógio monotônico. Tabela de **medianas em microssegundos por frame**; 1.000 µs equivalem a 1 ms.

| Caso | Desenhar frame (µs) | Copiar para ColorImage (µs) | Cópia + `texture.set` + retirada da fila na CPU (µs) |
|---|---:|---:|---:|
| Legado exato | 68,15 | 1,70 | 2,05 |
| Vale verde / Full | 55,74 | 1,74 | 1,82 |
| Litoral / Full | 51,06 | 1,85 | 2,12 |
| Interior árido / Full | 54,24 | 1,85 | 1,91 |
| Transição 50% / Full | 181,41 | 1,96 | 2,30 |
| Transição + chuva / Full | 167,35 | 1,93 | 2,03 |
| Transição / Low | 165,10 | 1,97 | 2,28 |
| Vale verde / Low | 45,08 | 1,76 | 1,84 |

Fonte completa: `assets/validation/scene-benchmark.json`, com cada rodada e mínimo/máximo. A última coluna já inclui a cópia; não deve ser somada à coluna de cópia como se fossem custos disjuntos.

A execução independente da fonte original mediu 59,58 µs para o mesmo fixture, enquanto o Legacy idêntico desta rodada mediu 68,15 µs. Essa diferença explicita a variação de agendamento/carga do host. A chuva ter mediana menor que a transição limpa **não significa que adicionar chuva melhora desempenho**. As medições são uma aproximação do custo, não uma análise estatística de superioridade.

Os ambientes estáveis ficaram próximos ou abaixo da mediana do baseline neste host. Misturar dois cenários custa mais: cerca de 0,181 ms no Full. Low reduziu o vale a cerca de 0,045 ms; a mistura de dois buffers continua sendo o custo predominante durante transições.

### O que foi e o que não foi medido

- Framebuffer enviado: **89.600 bytes** (320×70×4), inalterado.
- Dois buffers do renderer: **179.200 bytes** reservados após `new()`, mais metadados. Eles também continuam reservados em Low/Legacy.
- Fluxo de bytes teórico da cena: 2.688.000 bytes/s a 30 FPS, ou 5.376.000 bytes/s a 60 FPS; isso é dimensão × taxa, **não medição de tráfego efetivo de GPU**.
- Cópia de pixels e preparação de atualização no gerenciador de texturas: medidas na CPU.
- Transferência e tempo de execução real na GPU: **não medidos**. `TexturesDelta` preparado no microbenchmark não executa backend gráfico.
- Consumo do processo no Windows, potência, temperatura e autonomia: **não medidos**.

## Janela de desenvolvimento executada

O binário release com `host-validation` foi aberto uma vez em Xorg dummy autenticado, usando Mesa em software, CPU manual 50% e arquivo de configuração temporário isolado. A janela exibiu a cena nova e o rótulo `CPU SIM 50%`. O app registrou `elapsed=5.03s frames=144 cpu=50.0% speed=55.994km/h`; encerrou automaticamente com código zero. A captura 640×140 está em `assets/validation/host-window.png`.

O harness durou 5,17 segundos. Seu registro adicional observou RSS final de aproximadamente 74,01 MiB e 12,63% de um núcleo para o processo do app, excluindo Xorg. Esses números incluem UI e renderização gráfica em software; o ensaio curto serve para comprovar integração, **não para estimar uso do Windows com driver de GPU**. Logs: `host-app.log`, `host-smoke.json` e `host-build-info.json`.

## Windows e limites restantes

Compilação cruzada concluída com Rust 1.100.0 para `x86_64-pc-windows-gnu`, `--release --offline --locked -j 1`, features padrão e linker MinGW já disponível. A etapa terminou com código zero em 4 min 09 s. O executável local produzido tem 5,312,000 bytes e formato PE x86-64 / Windows GUI.

SHA-256 do binário usado para essa verificação: `755be7dee4e091130403b4b62f870fe4e535ca08a47311920c6a56e4f725ba78`.

A inspeção da tabela PE não encontrou imports externos de `libgcc`, `libwinpthread` ou `libstdc++`; os imports incluem DLLs do Windows e OpenGL. A disponibilidade/resolução dessas DLLs no Windows real não foi testada. Metadados e lista completa em `windows-build-info.json` e `windows-imports.txt`.

```bash
cargo build --release --locked --target x86_64-pc-windows-gnu -j 1
```

Esse comando exige o target Rust e linker apropriados. No próprio Windows, o comando normal `cargo build --release --locked` utiliza o toolchain Windows instalado.

Não houve execução nativa Windows, Wine, inspeção real de DPI, taskbar, múltiplos monitores ou consumo no computador do Pedro. Não foi criado release de executável distribuível nesta frente: o ZIP contém código, patch, amostras e provas. Os arquivos completos de licenças das dependências do release original não estavam no pacote de desenvolvimento; a coordenação deve usar o pacote original ao montar um release binário.

## Integridade da entrega

O patch contém nove arquivos novos e quatro modificados no escopo de arte. Foi aplicado a uma cópia recém-extraída do ZIP, e os 13 arquivos resultantes coincidiram byte a byte com a árvore final. Os 41 arquivos originais não selecionados permaneceram inalterados. Registro: `assets/validation/patch-check.json`.

SHA-256 de `PATCH_ARTE.diff`: `4d9fba4b1e98bdc1f55db497fa500ab086b911b1176998a176e5ec33c42a8fca`.

Os documentos históricos herdados da base permanecem presentes por rastreabilidade. Este relatório descreve apenas a frente de arte desta rodada. Nenhum `target/`, cache, chave de autorização gráfica ou executável pesado entra no ZIP de fonte.
