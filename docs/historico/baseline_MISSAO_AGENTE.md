# MISSÃO AGENTE 05 — experiência do usuário, controles e configuração

Você é o responsável pela experiência do CPU Express como *companheiro de desktop* e pequeno jogo. O foco é **tornar recursos existentes acessíveis e preparar pontos de integração**, sem reescrever o renderer ou o domínio. Recebe a linha A e deve ler `CONTRATO_COMUM.md`.

## Experiência desejada

A janela deve poder ficar próxima à barra de tarefas sem ocupar a atenção da pessoa. Controles aparecem quando necessários; o cenário continua visível. O usuário pode abrir o painel, ocultar HUD, pausar, fixar no topo, testar CPU simulada 0/50/100, mudar preferências e encontrar facilmente a opção de sair. A linguagem deve ser compreensível para alguém que não programou em Rust.

## Propriedade exclusiva

`src/app.rs`, `src/config.rs`, `src/main.rs`; novos testes de configuração/UX em `tests/ux_*.rs`, documentação `docs/ux_*.md`. Não tocar em `src/platform/**`, `src/metrics/**`, `src/rendering/**`, `src/simulation/**` nem no código de diagnósticos. Para acionar futuros recursos, escrever adaptadores a APIs **já existentes** e documentar integração opcional; não assumir que outros agentes já terminaram.

## Entregas funcionais

**U1 — organização e acessibilidade.**

1. Auditar o layout baseline 640×140 e o painel de configurações. Garantir botões legíveis, mouse/teclado, tooltips, atalhos consistentes e foco sem conflito com o ato de arrastar.
2. Fazer **modo compacto** de HUD e modo limpo com o trem, preservando sinalização de métricas ausentes/manual; respeitar baixo contraste em paisagens claras/escuras.
3. Estabilizar painel de ajuda com documentação curta das teclas. `Esc` fecha painel se aberto; não mata inadvertidamente a aplicação em uma interação de entrada.

**U2 — modos, pausa e comandos.**

4. Implementar pausa/retomada visível (`Espaço`, se disponível) sem acumular delta de tempo que faça o trem saltar ao voltar. A coleta de CPU pode continuar, mas a simulação/rolagem ficam congeladas.
5. Implementar `--demo` **explícito** que permita iniciar sem tocar contadores reais (aproveitar `MetricsWorker::spawn_with_provider`/`ManualProvider` já presentes). Cabeçalho precisa mostrar `DEMO` ou `SIMULADO`. Não alegar RAM real quando o provider não a fornece.
6. Preservar `--cpu`, `--config`, `--smoke-seconds`, `--diagnostics` e demais opções CLI originais; harmonizar nomenclatura dos atalhos e informar no `--help`.
7. Proporcionar presets 0%, 50%, 100% da CPU manual e slider de precisão, distinguindo testes de uso real. Nunca gerar carga sintética para aumentar a velocidade.
8. Preparar ações de reposicionar e mostrar/esconder a janela via chamadas existentes de plataforma. Se tray novo do agente 1 não estiver disponível, **não inventar uma API**: anotar a conexão mínima em `INTEGRACAO.md` e deixar controle relacionado desligado.

**U3 — configuração e robustez.**

9. Expandir `Config` com valores default retrocompatíveis; manter tratamento de JSON parcial/corrompido e escrita atômica. Não destruir configuração antiga sem migração explícita.
10. Oferecer preferências de FPS, HUD, pinned, velocidade de transição e comportamento de pausa quando fizer sentido, protegidas por limites seguros.
11. Exibir alertas recuperáveis de telemetria e não travar a UI quando métricas ausentes. Mensagens em português, curtas, sem pop-ups repetidos.
12. Criar testes de config antiga e nova, teste de `--demo`, pausa/resume sem salto (extraia lógica pura se necessário), atalhos e persistência, além de checklist manual Windows com screenshots **apenas se executado**.

## APIs e integração

Usar contratos vigentes `MetricsWorker`, `Train`, `World`, `SceneRenderer` e `platform::dock_window`. Não tentar integrar biomas/viagens do agente 3 ou bandeja do agente 1 sem suas interfaces reais. O resultado próprio deve compilar e produzir melhora percebida **independentemente das outras quatro frentes**. Ao final, `INTEGRACAO.md` deve mostrar onde ficariam os futuros painéis de estação/bioma e os botões do tray, sem alterar estes módulos.

## Critérios de conclusão

- A janela continua utilizável em 640×140; modo limpo, ajuda, pausa e simulação realmente funcionam.
- Config antiga é aceita, nenhuma configuração inválida causa panic/travamento.
- `--demo` abre sem o provider Windows e identifica claramente dados manuais.
- Atalhos documentados, retorno de pausa sem teleport, foco/arraste consistentes.
- Entregar projeto completo com `PATCH_NOTES.md`, `INTEGRACAO.md`, `TESTES.md` e evidências reais.
