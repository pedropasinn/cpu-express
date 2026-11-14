# Evidências visuais da UX

Estas imagens são **renderizações headless do egui; sem janela nativa**. O teste instancia a aplicação real em Rust, desenha seus controles e o renderer original, exporta as malhas trianguladas e as texturas do egui e as rasteriza por software. Não são mockups desenhados à mão nem screenshots do Windows.

Os sete quadros têm 640 × 140 pixels: HUD compacto, detalhado, limpo com controles visíveis, pausa, opções, ajuda e modo limpo sem foco. A animação de abertura do painel termina antes da captura.

## Reproduzir

Requer Rust da versão indicada pelo projeto. Para a conversão opcional de JSON em PNG, requer Python 3 com `numpy` e `Pillow`; isso não adiciona dependências ao aplicativo.

No PowerShell, a partir da raiz do projeto:

```powershell
$env:CPU_EXPRESS_VISUAL_DIR = Join-Path $PWD "target/ux-evidence"
cargo test --locked --test ux_visual -- --ignored
python docs/ux_rasterizar.py $env:CPU_EXPRESS_VISUAL_DIR
```

No Linux de desenvolvimento:

```bash
CPU_EXPRESS_VISUAL_DIR="$PWD/target/ux-evidence" cargo test --locked --features host-validation --test ux_visual -- --ignored
python docs/ux_rasterizar.py target/ux-evidence
```

O exportador escreve dados JSON intermediários. Eles não são incluídos no pacote-fonte, pois são grandes e podem ser regenerados. Nenhum código de captura ou rasterização é executado no aplicativo distribuído.

## Limites da evidência

Esta verificação mostra o layout, os textos, o contraste, os estados do HUD e a visibilidade do cenário usando a saída real do egui. A rasterização por software pode diferir do antialiasing/composição da GPU. Não verifica DPI ou escalonamento nativo, taskbar, múltiplos monitores, driver OpenGL, minimizar/restaurar no Windows nem comportamento de um executável Windows. Esses pontos continuam no checklist manual de `TESTES.md`.

A execução nativa com Xorg neste ambiente também não ocorreu: a criação de sockets Unix foi negada pelo sandbox.
