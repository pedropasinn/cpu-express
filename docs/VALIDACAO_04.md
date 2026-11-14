# Validação CPU Express 0.4.0-rc.2

Executado em 2026-11-14 no Windows local. Executável Windows GNU sem assinatura, SHA-256 `f5766ad574431eb0d6e61e24ef4039cb409f10283dd3e9efaed381364465f985`. As evidências mantidas nesta árvore correspondem à versão atual.

## Código e jogabilidade

- Formatação e Clippy sem warnings; **288 testes Rust aprovados**, zero falhas e dois exportadores ignorados na suíte normal. O exportador de painéis foi executado separadamente.
- Doze testes novos de despacho: oferta sem expiração, recusa, capacidade, aceite na origem/em trânsito, cancelamento, devolução com ocupação preservada, entrega única, restart, migração v2, backup exato e proteção de saves futuros/incompletos.
- Três testes de apresentação das fases do ramal e uma regressão para os rótulos dos botões de 46 pontos.
- Revisão cruzada encontrou dois casos inválidos de save: defaults antigos reparando silenciosamente campos ausentes do v3 e carga devolvida que voltava a aparecer embarcada. Ambos foram corrigidos com regressões executadas.
- A conferência visual encontrou texto cortado na navegação mínima e setas ausentes na fonte. Layout compacto e textos foram corrigidos antes deste binário.

## Interface

26 cenários egui exportados, incluindo oferta, falta de capacidade, devolução e central de 480 × 480 antes/depois da rolagem. Inspeção visual concentrada nas telas alteradas: contratos, ramal e oficina em tamanhos normal/compacto/mínimo. Manifesto de fonte, meshes e PNGs em `evidence/current/game-ui-result.json`.

São exports headless da aplicação real. O conector de interação nativa retornou `Native pipe unavailable (Windows os error 2)`; nenhum fluxo de cliques no desktop foi certificado. Uma instância isolada foi aberta; fechar sua central manteve o trem, e fechar a janela principal encerrou a instância. Essa observação de ciclo de vida não substitui validação visual pelo compositor.

## Execução nativa

- Smoke encontrou faixa alinhada à barra e painel nativo independente, com captura do framebuffer da faixa. `panel_capture=false`: nenhum screenshot nativo da central foi obtido.
- Quatro alternâncias barra/flutuante passaram, em cinco estágios; `transitions-expansion/result.json` identifica este executável.
- Quatro amostras válidas de CPU/RAM reais. Ensaio curto com limite 60 FPS produziu aproximadamente **59,80 FPS** após aquecimento; mede frames produzidos, não apresentação do compositor ou energia.
- Clima São Paulo: 25 amostras prontas, sem erro, com vento, precipitação e horários solares. A consulta foi refeita para este executável.
- Cópia isolada do perfil v2: distância 154854,91 → 155189,07; v3 gravado, quatro pinturas mantidas, totais de serviço preservados e backup v2 byte a byte. A origem isolada permaneceu intocada. O perfil real não foi usado como destino desse teste.

## Execução estendida e pacote

Duas execuções de 120 segundos encerraram normalmente, com painel fechado e aberto. Working set final: 88,42 MiB e 105,46 MiB. Na janela final de 256 frames, p95 do trabalho da aplicação: 0,286 ms e 0,351 ms. Esses números não medem compositor, energia nem estabilidade de um dia inteiro. Logs completos em `evidence/current/soak/result.json`. O script `scripts/validate-portable.py` passou: CRC dos três ZIPs, extração para pasta com espaços e execução com perfil isolado e PATH apenas do Windows. Saída zero e encerramento normal confirmados, com o mesmo SHA-256 do binário validado.

## Limites

Não certificados: fluxo nativo de cliques, leitores de tela, áudio audível, Windows 10, DPI misto real, matriz completa de escalas, autohide, reinício real do Explorer, desconexão de monitor ou uso por um dia inteiro. Os testes geométricos e headless não substituem hardware/aceite humano. Instalador opcional não foi aplicado ao perfil real. Sem assinatura digital, nuvem, atualização automática ou certificação 1.0.
