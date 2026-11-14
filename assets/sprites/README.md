# Arte original do CPU Express — frente 02

A arte é desenhada programaticamente pelo renderer Rust real do aplicativo. Não há carregamento de sprites externos, downloads ou novas dependências. As imagens em `assets/gallery/` são saídas de execução desse renderer, não mockups.

## Organização

| Arquivo | Responsabilidade |
|---|---|
| `src/rendering/scene.rs` | Interface preservada, buffers reutilizados, composição e fumaça |
| `src/rendering/legacy.rs` | Cena pastoral original preservada pixel a pixel |
| `src/rendering/theme.rs` | `Biome`, `SceneTheme`, opções, qualidade e rota determinística |
| `src/rendering/palette.rs` | Papéis de cor, três paletas originais e ciclo de luz |
| `src/rendering/scenery.rs` | Céu, relevo, água, construções, vegetação, trilhos e chuva |
| `src/rendering/rolling_stock.rs` | Cinco tipos de vagão e detalhes adicionais da locomotiva |
| `src/rendering/sprites.rs` | Primitivas com recorte e sprites originais; rodas compartilhadas |

A versão anterior já tinha céu, nuvens, montanhas distantes/próximas, colinas, árvores, casa, cerca, trilhos, primeiro plano, três vagões, locomotiva e fumaça. Essas funções continuam no modo `Legacy`. A frente 02 acrescenta ambientes e composição, preservando seus pontos de contato mecânicos.

## Resolução e memória

A textura permanece **320×70 RGBA**, ampliada pelo aplicativo com **nearest-neighbor** para 640×140 pontos lógicos. O DPI pode alterar o tamanho físico da janela. Cada exportação repete os pixels em blocos exatos de 2×2.

- Buffer enviado: 89.600 bytes por frame.
- Segundo buffer reutilizado para transições: 89.600 bytes.
- Total reservado pelos dois arrays de pixels do renderer: 179.200 bytes, sem contar metadados e recursos da UI/driver.
- Zero alocações no caminho de `render()` após `new()`, verificado com alocador instrumentado.
- O app original ainda cria sua `ColorImage` com uma cópia via `to_vec()`. Esse custo não foi ocultado nem alterado nesta frente.

O modo `Low` evita nuvens, chuva, a camada intermediária de relevo e detalhes decorativos. Ele mantém os dois buffers reservados e a rota; não é uma redução da resolução. `Legacy` reconstitui exatamente o desenho original.

## Ambientes

- **Vale verde:** serras com neve pontual, pinheiros, campos, casa, caixa-d'água ferroviária, sinal e cerca.
- **Litoral:** ilhas, mar, reflexos discretos, barcos à vela, dunas, capim e farol.
- **Interior árido:** mesas rochosas, escarpas, cactos, depósito de estação e telégrafo.

O trem usa contrastes de verde, cobre, creme e azul e é desenhado uma vez depois da paisagem. A frente de arte não muda velocidade, emissão de fumaça nem dinâmica das rodas. Estilos: passageiro original, leito, carga, tanque e panorâmico.

## Reprodução

```bash
cargo run --release --locked --example scene_gallery -- gallery minha-galeria
cargo run --release --locked --example scene_gallery -- sequence meus-frames 240 20
```

A primeira linha escreve seis PPMs e um manifesto. A segunda exporta 12 segundos de viagem automática a 20 FPS, começando perto de uma transição. A CPU de 100% nesse exemplo é **entrada manual simulada**; não é leitura do computador nem geração de carga para movimentar o trem.

A geração computacional de imagens e o microbenchmark consomem recursos durante sua execução, naturalmente; não há workload sintético acoplado ao jogo para forçar leituras de CPU.

PPMs são RGB sem compressão. Os PNGs entregues são conversões sem perdas desses PPMs. O GIF é uma prévia indexada da sequência e pode quantizar cores; os PPMs/PNGs são as referências de fidelidade.

Código, cores e geometria novos foram criados nesta frente e distribuídos sob a licença MIT do projeto. Não foram incorporados sprites, paletas exatas ou layouts de outros jogos. Consulte `../../LICENSE` e `../../TESTES.md`.
