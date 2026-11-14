# Agente 05 — integração da frente de UX

## Base e ordem de integração

Esta entrega parte da linha A congelada recebida em `CPU_Express_Agente_05_UX(1).zip`. O digest da base está em `BASELINE.md`. As alterações funcionais ficam em `src/app.rs`, `src/config.rs` e `src/main.rs`; os testes originais permanecem intactos. Não há mudança em `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, scripts, renderer, simulação, métricas ou plataforma.

A ordem proposta para o coordenador continua 4 → 3 → 2 → 1 → 5. A frente 05 já funciona independentemente das demais. Não copiar sua árvore inteira por cima de uma árvore que já recebeu outras frentes: aplicar apenas os três arquivos alterados e os arquivos novos indicados em `PATCH_NOTES.md`, ou o patch fornecido. Conflitos nas assinaturas das APIs devem ser resolvidos pelo coordenador.

`AGENTE05_UX.patch` contém somente os três arquivos de produção e os quatro testes novos. Sua aplicação foi checada contra a base recebida. Na pasta de destino ainda sem esta frente, use `git apply --check CAMINHO/AGENTE05_UX.patch` e depois `git apply CAMINHO/AGENTE05_UX.patch`. Se o check indicar conflito, revisar o trecho com as outras frentes; não forçar nem substituir a árvore inteira. Copiar separadamente a documentação e as evidências desta entrega.

## Contratos preservados

| Interface existente | Uso nesta entrega |
| --- | --- |
| `LaunchOptions` e `CpuExpressApp::new` | Mesmos campos e assinatura; os exemplos antigos continuam compilando. |
| `MetricsWorker::spawn` | Continua no modo real e em `--cpu` sem `--demo`. |
| `MetricsWorker::spawn_with_provider` + `ManualProvider` | Usados exclusivamente na inicialização explícita do demo; nenhum contador real é criado nessa ramificação. |
| `Train::new` / `Train::update` | Sem mudança; o relógio da UI fornece apenas o tempo efetivamente percorrido. |
| `World::update` | Não recebe avanço durante pausa; fumaça, rodas e paisagem ficam congeladas. |
| `SceneRenderer::render` | Mesmo renderer, dimensões 320×70 e filtro de vizinho mais próximo. |
| `platform::dock_window` | Reposicionar continua usando esta chamada, executada em `App::logic`. |
| `config::load` / `config::save` | Mesmas assinaturas; leitura retrocompatível e gravação protegida. |

## APIs públicas novas

```rust
CpuExpressApp::new_demo(&egui::Context, LaunchOptions) -> CpuExpressApp
CpuExpressApp::dispatch(&mut self, AppAction, &egui::Context)
CpuExpressApp::snapshot(&self) -> SessionSnapshot
SimulationClock::new(Duration) -> SimulationClock
SimulationClock::set_suspended(&mut self, bool, Duration)
SimulationClock::take_step(&mut self, Duration) -> f64
shortcut_action(egui::Key, egui::Modifiers, bool, bool) -> Option<AppAction>
Config::effective_hud_mode(&self) -> HudMode
Config::set_hud_mode(&mut self, HudMode)
HudMode::next(self) -> HudMode
```

`AppAction` contém `TogglePause`, `CycleHud`, `TogglePin`, `ToggleControls`, `ToggleHelp`, `Reposition`, `ClosePanel` e `Quit`. O snapshot é somente leitura. Não fornece acesso a handles de plataforma ou ao provider.

## Conexão futura com o Agente 1: bandeja e janela

Não existe contrato de tray neste pacote. O botão **Ocultar na bandeja** fica desativado, com explicação; **Minimizar** usa `ViewportCommand::Minimized(true)` e a restauração é feita pela barra de tarefas. Não se emite `Visible(false)` sem uma forma garantida de restaurar a janela.

Quando a frente 1 entregar seus eventos reais, o coordenador pode traduzir as ações de menu em `AppAction` e chamar `dispatch` na thread de UI. A ação de reposicionar apenas agenda `platform::dock_window`; não há nova API de plataforma presumida. O evento de restaurar deverá chamar a interface efetivamente entregue pelo Agente 1. Não criar foco automático nem hooks globais como efeito colateral dessa conexão.

## Conexão futura com o Agente 3: estações, biomas e viagens

O ponto visual de entrada é `overlay`, que organiza as abas **Viagem**, **Visual** e **Janela**. `journey_controls` pode ganhar um seletor de destino ou um resumo da estação quando existir uma API concreta do domínio. A UI deve enviar um comando de domínio e mostrar uma leitura do estado; não deve calcular rota, recompensas ou progressão dentro de `app.rs`.

O relógio central já entrega `dt = 0` quando suspenso. Ao integrar eventos/viagens, usar o mesmo estado efetivo de pausa. Retomar não deve reconstruir o tempo de parede decorrido enquanto o usuário estava ausente.

## Conexão futura com o Agente 4: telemetria

O worker continua independente da UI. Amostras vencidas, não finitas ou indisponíveis não aparecem como percentuais atuais. Os avisos são incorporados à interface, sem janelas modais repetidas. O demo mantém `ram_percent = None` e usa a CPU manual efetiva nos diagnósticos.

Os registros JSONL preservam campos antigos e acrescentam `paused`, `effective_paused`, `world_elapsed_seconds` e `scroll_px`. O campo `mode` também admite `"demo"`; consumidores que validem um enum fechado precisam aceitar esse valor. O módulo `src/diagnostics.rs` não mudou.

## Configuração e migração

Os novos campos têm defaults: `hud_mode = "compact"`, `pause_when_unfocused = false`, `start_paused = false`, `auto_hide_controls = true`. `show_metrics` permanece para compatibilidade. Quando for `false`, o modo efetivo é limpo, sem reescrever silenciosamente o arquivo antigo. Use `set_hud_mode` ao alterar o modo pela interface para manter os dois campos coerentes.

Salvar uma configuração válida preserva campos JSON desconhecidos. Arquivo inválido, ilegível ou de schema incompatível não é sobrescrito; a alteração vale apenas na sessão e aparece um aviso. O usuário pode corrigir o arquivo ou selecionar outro `--config`. Alterações sucessivas de sliders são agrupadas por uma espera de 400 ms; a saída normal força a gravação pendente. A pausa atual e o modo demo/manual são estados da sessão, não são gravados como se fossem preferências.

## Arquivos históricos e distribuição

`README.md`, `LEIA-ME.txt`, `BUILD-INFO.json` e a pasta `validation` pertencem à base recebida e foram preservados para evitar conflitos entre os agentes. Suas alegações de versão, executável, contagem de testes e captura visual NÃO descrevem esta frente modificada. Para esta entrega, leia `PATCH_NOTES.md`, `TESTES.md` e `docs/ux_guia.md`; as evidências novas ficam sob `docs/ux_evidencias`.

O ZIP desta frente contém código-fonte, não um novo executável Windows. Antes de distribuir binários, restaurar os avisos e as licenças de dependências do pacote original completo, conforme o contrato comum. O script de build original não deve ser interpretado como prova de que esse material já está presente.

## Riscos concretos e rollback

1. **Integração nativa ainda não verificada:** foco, taskbar, múltiplos monitores e DPI exigem execução em Windows real. Usar o checklist em `TESTES.md`.
2. **Conflitos ao integrar outras frentes:** alterações de API ou novos campos do domínio devem ser conciliados pelo coordenador; preservar as implementações dos outros agentes.
3. **Persistência incompatível:** configurações inválidas ou escritas por uma versão futura podem deixar alterações somente na sessão. O arquivo é preservado e a UI informa o motivo; corrigir o arquivo ou fornecer um caminho novo.

Para rollback, restaurar somente `src/app.rs`, `src/config.rs` e `src/main.rs` da base congelada, e remover os testes/documentos exclusivos desta frente se necessário. A base ignora os novos campos JSON; `show_metrics` continua a registrar se o HUD fica visível. Não apagar os módulos já integrados dos outros agentes. Guarde uma cópia local da configuração antes de qualquer migração manual.
