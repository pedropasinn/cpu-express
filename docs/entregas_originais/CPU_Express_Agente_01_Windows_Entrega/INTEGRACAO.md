# Integração da frente Windows com o Agente 05

## O que já funciona nesta fonte

O jogo chama a mesma função `platform::dock_window(frame)` e recebe o novo
posicionamento automaticamente. A bandeja e o controlador estão implementados
como APIs opt-in, com demonstração compilável em `examples/windows_desktop.rs`.
O arquivo `src/app.rs` entregue continua sendo o original da linha A.

## Patch concreto para o app

`integration/windows-ux.patch` modifica somente `src/app.rs` e foi preparado
contra o baseline recebido. Ele acrescenta o controlador, ativa a bandeja,
encaminha comandos de janela, cria o botão explícito de ocultar e mantém o
encerramento real. A ativação da bandeja pode falhar sem impedir o jogo; o erro
é apresentado como notificação, preservando avisos anteriores de configuração.

```powershell
git apply --check integration/windows-ux.patch
git apply integration/windows-ux.patch
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked
```

O coordenador deve aplicar o patch na linha A ou mesclar seus trechos com a
versão do Agente 05. Se `git apply --check` detectar conflito com alterações de
UX, use os pontos de ligação abaixo; não substitua o app inteiro.

`integration/windows-features.patch` registra o diff mínimo de `Cargo.toml`, que
**já está aplicado neste pacote**. Ao juntar frentes, faça união das features de
`windows-sys`; preserve o lockfile. Não aplique o mesmo patch duas vezes.

## Pontos de ligação obrigatórios

1. Crie `DesktopController::new(ctx, config.always_on_top)` na thread da UI.
   Preserve `.with_active(false)` e `.with_taskbar(true)` de `main.rs`.
2. Chame `enable_tray()` uma vez. Se houver erro, mantenha o jogo visível e o
   botão de ocultar indisponível. Não encerre o jogo por uma falha opcional da
   bandeja.
3. Chame `desktop.update(frame)` no **início de `App::logic`**, antes de retornos
   por janela oculta ou pausada. O callback da bandeja chama `request_repaint`,
   que permite ao eframe 0.37 executar a lógica mesmo sem desenhar a janela.
4. Encaminhe ocultar, minimizar, mostrar e topo ao controlador. Esses métodos
   **enfileiram pedidos**; `DesktopUpdate` informa estado e erros após a aplicação.
   Não envie comandos concorrentes `Visible`, `Minimized` ou `WindowLevel`.
5. Copie `DesktopUpdate.topmost_changed` para `Config.always_on_top` e marque a
   preferência para persistência. Isso também vale quando o comando veio da
   bandeja com a janela oculta. O modo minimizado guarda a preferência até a
   restauração.
6. `Close` permanece fechamento real. `on_exit` chama `desktop.shutdown()` e
   continua liberando o worker de métricas. `Drop` é uma proteção adicional,
   não uma mudança de fechar para ocultar.
7. A mudança do DPI já presente no `ViewportInfo` pode chamar
   `desktop.display_changed()`. A bandeja também encaminha `WM_DISPLAYCHANGE`
   e `WM_SETTINGCHANGE/SPI_SETWORKAREA`. A recuperação move apenas janelas que
   deixaram de caber; não prenda novamente ao canto a cada frame.

## API pública nova

| API | Semântica |
| --- | --- |
| `DockEdge::{Auto, Bottom, Top, Left, Right}` | Preferência explícita de borda, sempre limitada à área útil. |
| `dock_placement_at_edge(...)` | Cálculo puro em pixels físicos. |
| `intersect_work_area(...)` | Recorta um resultado anômalo do sistema; rejeita interseção vazia. |
| `fit_placement_to_work_area(...)` / `constrain_placement(...)` | Contenção com deslocamento mínimo, sem resnap. |
| `dock_window_at_edge(frame, edge)` | Posicionamento nativo imediato na thread proprietária. |
| `recover_window_placement(frame)` | Ajuste imediato, retorna se houve mudança; ignora minimizada. |
| `DesktopState`, `DesktopAction` | Política pura; fechamento terminal e ocultação com recuperação. |
| `DesktopController::new(ctx, topmost)` | Constrói o controlador sem criar thread ou ícone. |
| `enable_tray()` | Criação explícita, fallível; aguarda confirmação inicial do shell. |
| `show()`, `hide()`, `minimize()`, `dock()`, `dock_at(edge)` | Pedidos assíncronos aplicados no próximo `update`. |
| `set_topmost(bool)`, `display_changed()`, `disable_tray()`, `close()` | Preferência/eventos/encerramento pela mesma fila. |
| `update(frame) -> DesktopUpdate` | Processa ações e falhas na thread UI, inclusive oculta. |
| `state()`, `always_on_top()`, `tray_available()` | Última observação/preferência/saúde confirmada. |
| `shutdown()` | Libera bandeja durante a saída real; não destrói HWND do eframe. |
| `SystemTray`, `TrayEvent` | Adaptador de baixo nível para quem implementar outro controlador. |

As APIs nativas/controlador/bandeja só existem sob `cfg(windows)`. A geometria e
a política de estados são multiplataforma para permitir testes, sem oferecer
uma integração desktop Linux. Os contratos anteriores não mudaram.

## Decisões de UX que continuam com o Agente 05

O exemplo não define a aparência final do botão ou do menu de controles do jogo.
Em uma área útil menor que 640 × 140 pontos lógicos, o nativo pode reduzir a
janela física; os controles devem se adaptar ao espaço disponível. O módulo não
reexpande automaticamente uma janela reduzida quando o monitor aumenta.

Uma janela minimizada é recuperável pela barra de tarefas. Uma janela oculta é
recuperável pela bandeja enquanto o shell responde. Ações `Ocultar` e `Sair`
precisam de rótulos diferentes. Desativar a bandeja primeiro mostra uma janela
oculta; o encerramento do processo libera a bandeja sem reabrir a janela.

Não há persistência nova no formato JSON. O valor de topo continua usando o
campo existente; a ativação da bandeja no patch é apenas da sessão.

## Demonstração isolada

```powershell
cargo run --locked --example windows_desktop
cargo run --locked --example windows_desktop -- --smoke-seconds 15
```

Teste todos os botões, minimizar/restaurar pela taskbar e mostrar pelo ícone.
Depois oculte, reinicie o Explorer pelo Gerenciador de Tarefas e confira o ícone
novamente. O menu da bandeja também deve permitir sair enquanto a janela está
oculta. A matriz completa está em `docs/windows_checklist.md`.

Os resultados de compilação do patch e da demonstração constam em `TESTES.md`.
Nenhum desses comandos é apresentado como executado no Windows nesta entrega.
