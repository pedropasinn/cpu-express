# Base única — CPU Express 0.2.0-rc.1

As cinco entregas já estão integradas nesta árvore. Não é necessário aplicar nenhum patch original.

O relato da consolidação está em `docs/CONSOLIDACAO.md`. A validação corrente está em `docs/VALIDATION.md` e `BUILD-INFO.json`. O guia de uso está em `docs/ux_guia.md`.

Para novas contribuições, use esta versão como base. Contratos centrais:

- `World::advance_train` coordena movimento, cenário e jornada com o mesmo tempo ativo.
- `SceneContext::from_journey` traduz o estado de domínio para os ambientes visuais.
- `DesktopController::update` roda antes dos retornos por janela oculta; `animation_suspended` não bloqueia o primeiro paint do eframe.
- A interface revalida a idade da cópia das métricas e respeita leituras parciais.
- `Config.visual` persiste somente escolhas de apresentação. O progresso da jornada ainda é da sessão.

As instruções recebidas dos agentes estão preservadas em `docs/entregas_originais/`. Patches e validações anteriores em `docs/historico/` são documentos históricos, não ações pendentes.
