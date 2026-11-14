# CPU Express — Agente 02: arte, paisagens e renderer

## Resumo em dez linhas

1. Implementados vale verde, litoral e interior árido em Rust, com geometria e cores próprias.
2. A viagem escolhe biomas pela distância do `World`, sem relógio de parede ou coleta direta de CPU.
3. Transições misturam duas paisagens alinhadas; o trem e a fumaça são desenhados uma única vez.
4. Adicionados ciclo de luz, nuvens, chuva opcional, pequenas construções e detalhes à beira da linha.
5. Incluídos cinco tipos de vagão, sem alterar posições das rodas, biela, trilho e chaminé.
6. Preservados canvas 320×70, apresentação 640×140 e ampliação nearest-neighbor da UI existente.
7. Criados dois buffers reutilizados; nenhuma alocação por frame no renderer após a construção.
8. Modo `Legacy` reproduz o frame original pixel a pixel; modo `Low` reduz passes decorativos.
9. Entregues testes, exportador de seis cenas, sequência animada e benchmark reproduzível.
10. Nenhuma mudança nos arquivos das outras quatro frentes, no manifesto, no lockfile ou na API baseline.

## Resultado e escopo

Esta entrega executa a missão de `MISSAO_AGENTE.md` sobre a linha A fornecida no ZIP. A versão do pacote Cargo permanece 0.1.0 para evitar conflito com a coordenação. Identificador desta entrega: **agente02-arte-1**.

`SceneRenderer::new()` ativa a viagem nova imediatamente com o `app.rs` original. `set_theme(SceneTheme::Legacy)` recupera a aparência antiga sem adicionar campos ao `World`. O fallback tem um teste de hash obtido da execução independente do ZIP original.

Não foram alterados `src/app.rs`, `src/main.rs`, `src/config.rs`, `src/lib.rs`, `src/diagnostics.rs`, `src/metrics/**`, `src/platform/**`, `src/simulation/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, scripts ou o README compartilhado. Os documentos herdados de validação, `BUILD-INFO.json` e `BASELINE.md` descrevem o pacote anterior; **os resultados desta rodada estão em `TESTES.md` e `assets/validation/`**.

## Arquivos alterados

Arquivos de entrada modificados:

- `src/rendering/mod.rs`: exports dos controles opcionais e organização dos módulos.
- `src/rendering/scene.rs`: composição nova preservando `new()` e `render()`.
- `src/rendering/sprites.rs`: apenas a visibilidade de `wheel`, agora compartilhada dentro do renderer.
- `assets/sprites/README.md`: documentação da arte e da reprodução atualizada.

Arquivos novos de implementação e verificação:

- `src/rendering/legacy.rs`
- `src/rendering/palette.rs`
- `src/rendering/rolling_stock.rs`
- `src/rendering/scenery.rs`
- `src/rendering/theme.rs`
- `tests/rendering_contract.rs`
- `tests/rendering_allocations.rs`
- `examples/scene_gallery.rs`
- `examples/scene_bench.rs`

Materiais de entrega: `PATCH_NOTES.md`, `INTEGRACAO.md`, `TESTES.md`, `PATCH_ARTE.diff`, amostras e manifesto em `assets/gallery/`, registros desta execução em `assets/validation/`.

## API pública adicionada

`Biome`, `SceneTheme`, `RenderQuality`, `SceneOptions`, `Weather`, `SceneContext`, `CoachStyle`, `Palette`; setters `set_theme`, `set_quality`, `set_options`, `set_scene_context`, `set_coaches`; consulta `scene_context`; função pura `route_at`; constantes `BIOME_LENGTH_PX`, `TRANSITION_LENGTH_PX`, `ROUTE_PERIOD_PX`, `DAY_CYCLE_SECONDS`, `FRAMEBUFFER_BYTES`.

Nenhuma assinatura baseline foi removida. Nenhuma dependência foi adicionada. Os métodos e tipos estão detalhados em `INTEGRACAO.md`.

## Três riscos reais

1. **Integração gráfica no Windows ainda depende de execução real.** Compilação, testes de lógica e exportações no host não demonstram comportamento de DPI, driver, taskbar ou consumo no computador de destino. A coordenação deve validar o executável final no Windows; esta frente não muda a plataforma.
2. **A transição desenha dois cenários.** Seu custo é maior que o de um bioma estável, e há sobreposição visual temporária de silhuetas. O benchmark inclui esse caso. `Low` reduz passes; `Legacy` reverte a cena. A textura enviada não aumenta, mas os dois arrays de pixels somam 179.200 bytes.
3. **Controles novos ainda são API, sem menu ou persistência novos.** A viagem automática funciona agora; escolha de qualidade, chuva, composição de vagões e integração com Journey devem ser expostas pelo agente de UX/domínio. A rota visual atual é cíclica e não constitui progresso salvo ou inventário de jogo.

## Rollback

- Rollback visual imediato: `renderer.set_theme(SceneTheme::Legacy)`.
- Redução de custo mantendo biomas: `renderer.set_quality(RenderQuality::Low)`.
- Rollback completo antes de outras alterações nesta frente: `git apply -R PATCH_ARTE.diff`, ou restaurar somente `src/rendering/**` e os arquivos específicos do patch a partir da linha A. Execute `git apply -R --check` antes; não reverta alterações de outros agentes à força.

O patch de código/documentação da arte é independente dos arquivos binários de demonstração e dos relatórios da entrega. Consulte `INTEGRACAO.md` para aplicar com segurança à base correta.
