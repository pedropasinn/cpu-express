# Bandeja Windows — adaptador isolado do Agente 01

## O que foi implementado

`src/platform/tray.rs` fornece uma implementação nativa de `SystemTray`, usando a dependência `windows-sys` já existente. A bandeja tem uma thread própria, um ícone, uma janela auxiliar invisível e um loop bloqueante de mensagens. O desenho do trem e a janela do eframe continuam na thread principal.

O ícone é o recurso compartilhado `IDI_APPLICATION` do Windows. Isso evita introduzir arte pertencente à frente de rendering; uma identidade visual própria para a bandeja pode ser integrada depois. O menu oferece **Mostrar trem**, **Ocultar trem**, **Reposicionar na área útil**, **Alternar sempre no topo** e **Sair do CPU Express**. Um clique/seleção no ícone pede para mostrar a janela.

Essas ações chegam como eventos à UI. O módulo não guarda o `HWND` do eframe e não fecha, move, oculta ou ativa a janela do jogo por conta própria. A aplicação original continua sem criar a bandeja até o coordenador aplicar a integração descrita em `INTEGRACAO.md`.

## Contrato público

```rust
SystemTray::new(wake: impl Fn() + Send + Sync + 'static)
    -> Result<SystemTray, String>
SystemTray::poll(&self) -> Option<TrayEvent>
SystemTray::is_available(&self) -> bool
SystemTray::shutdown(&mut self) -> Result<(), String>
```

`TrayEvent` tem as variantes `Show`, `Hide`, `Dock`, `ToggleTopmost`, `Exit`, `DisplayChanged` e `Unavailable(String)`.

| Evento | Responsabilidade da thread da UI |
|---|---|
| `Show` | Mostrar/restaurar a janela, preservando as regras de foco do controlador |
| `Hide` | Ocultar somente se `is_available()` for verdadeiro e o controlador permitir |
| `Dock` | Fazer um reposicionamento solicitado pelo usuário |
| `ToggleTopmost` | Alterar a preferência e aplicar o estado da janela; a bandeja não salva configuração |
| `Exit` | Solicitar fechamento real pelo eframe e encerrar os workers |
| `DisplayChanged` | Conferir a área útil e recuperar uma janela fora da tela; não desfazer arraste válido |
| `Unavailable(error)` | Mostrar uma janela que estava oculta, informar o erro e impedir novas ocultações até haver bandeja disponível |

No momento de criar a bandeja, clone o contexto do egui e use `move || context.request_repaint()` como callback. A função deve ser curta, não bloquear e não acessar diretamente janelas Win32. Drene os eventos em `App::logic`, incluindo enquanto a janela está oculta. Na versão travada `eframe 0.37.0`, o caminho de atualização de lógica permanece disponível quando a pintura da janela oculta é suspensa; isso foi verificado no código-fonte local da dependência, não em execução Windows.

O construtor espera uma confirmação inicial do shell. Se a criação da thread, do helper ou do ícone falhar, devolve `Err` e libera os recursos que já adquiriu. A aplicação deve continuar visível e apresentar o erro, sem oferecer uma ação que deixe a janela irrecuperável.

`shutdown()` é idempotente e deve ser chamado pela thread da aplicação, nunca pelo callback de wake. Ele marca a bandeja indisponível, acorda o helper, remove o ícone, destrói a janela auxiliar e aguarda a thread. `Drop` executa o mesmo procedimento. Em `App::on_exit`, prefira chamar explicitamente `shutdown()` para que um eventual erro possa entrar no diagnóstico local. O fechamento do aplicativo não deve ser convertido implicitamente em ocultação.

O sinal de parada usa primeiro `PostMessageW`. Se a postagem falhar, inclusive por fila cheia durante um menu modal, usa `SendNotifyMessageW`: o envio entre threads é assíncrono e não depende da fila de mensagens postadas. O helper recebe o sinal dentro do processamento de mensagens do menu e chama `EndMenu` na própria thread. Não há envio síncrono segurando o mutex do endpoint. Se ambos os envios falharem, `shutdown()` devolve erro e mantém o `JoinHandle` para outra tentativa; não espera cegamente uma thread cujo sinal de parada não foi aceito. A distinção entre envio e postagem está em [Messages and Message Queues](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-messages-and-message-queues#nonqueued-messages); a semântica entre threads está na [referência de SendNotifyMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendnotifymessagew).

## Protocolo do Windows e recuperação

A janela auxiliar é top-level, com `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`, sem `WS_VISIBLE`. Ela não reserva espaço nem aparece como appbar. Uma janela message-only não receberia os broadcasts usados para recuperar o ícone. Não há subclassing do eframe, injeção no Explorer, hooks globais, atalho global ou timer de polling.

O registro do ícone executa `NIM_ADD` e `NIM_SETVERSION` com `NOTIFYICON_VERSION_4`. Nesse protocolo, o evento e a identidade do ícone são extraídos separadamente de `lParam`; as coordenadas do menu em `wParam` preservam o sinal, incluindo monitores à esquerda ou acima do principal. Essa disposição e o requisito de selecionar a versão depois de cada inclusão são descritos na [referência de Shell_NotifyIconW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw) e na [estrutura NOTIFYICONDATAW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw).

`TaskbarCreated` provoca uma tentativa de recriar o ícone, também abrangendo broadcasts relacionados a DPI. Se o shell rejeitar a inclusão ou o protocolo, a bandeja fica indisponível e envia `Unavailable`. O controlador deve restaurar imediatamente a janela oculta. Não existe retry infinito; uma nova tentativa ocorre com outro broadcast ou quando o coordenador recria o adaptador. O uso desse broadcast por janelas top-level está documentado em [The Taskbar — Taskbar Creation Notification](https://learn.microsoft.com/en-us/windows/win32/shell/taskbar#taskbar-creation-notification).

Uma mensagem `WM_CLOSE` externa que chegue ao helper, fora de um encerramento já solicitado, também envia `Unavailable` e acorda a UI **antes** de marcar a thread em encerramento. Assim, fechar o helper inesperadamente não elimina silenciosamente o caminho de recuperação de uma janela principal oculta. O sinal privado de `shutdown()` não produz esse aviso de falha.

`WM_DISPLAYCHANGE` e `WM_SETTINGCHANGE` com `SPI_SETWORKAREA` são repassados como `DisplayChanged`. Outros avisos de configuração não provocam reposicionamento. O callback é apenas uma notificação; o controlador da UI decide o movimento necessário.

Abrir o menu por uma ação explícita do usuário segue o protocolo Win32 `SetForegroundWindow` → `TrackPopupMenu` → `PostMessage(WM_NULL)`. Essa é a única solicitação de foreground neste módulo. Ela serve ao menu pedido pelo usuário e não é executada ao iniciar o jogo, encaixar a janela ou ligar sempre no topo. O procedimento de fechamento correto do menu é descrito na [referência de TrackPopupMenu](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-trackpopupmenu).

## Propriedade, `unsafe` e encerramento

- O `Box<TrayState>` mantém endereço estável. `GWLP_USERDATA` o referencia apenas enquanto o helper existe, e `WM_NCDESTROY` limpa o ponteiro antes de liberar o estado Rust.
- Todo acesso ao estado nativo ocorre na thread do helper. Campos mutáveis usam `Cell`, evitando referências exclusivas atravessando chamadas Win32 que podem executar callbacks reentrantes.
- O canal entrega intenções e um callback acorda a UI; nenhum ponteiro emprestado é passado entre as threads.
- Um mutex protege o destino da mensagem de encerramento. O worker remove o `HWND` desse destino antes da destruição, impedindo uma mensagem enviada a um identificador reutilizado.
- Um menu popup possui um guard que chama `DestroyMenu`. O ícone é um recurso compartilhado do sistema e não recebe `DestroyIcon`.
- A janela auxiliar é destruída na própria thread, seguida de `UnregisterClassW` apenas para a classe única dessa instância.
- `catch_unwind` contém um panic do callback ou do tratamento de mensagens antes que atravesse a fronteira `extern "system"`. Falha do loop sinaliza indisponibilidade à UI.
- Se o Windows excepcionalmente recusar destruir o helper, o cleanup tenta remover `GWLP_USERDATA`. Se até essa remoção falhar, o estado Rust é conservado em memória para impedir um ponteiro pendente. É um caminho defensivo de falha, não um comportamento de encerramento considerado aprovado.

## Limites da disponibilidade

`is_available()` representa a última confirmação recebida do shell. Ela não é uma prova de que o Explorer continua vivo neste exato instante. Um encerramento abrupto do Explorer pode retirar o ícone antes de qualquer broadcast; a recuperação automática espera o shell reiniciar. Se ele permanecer encerrado, a aplicação não pode prometer uma bandeja acessível. Não há polling periódico ou hook para detectar essa ausência mais cedo.

Também há políticas do Windows que colocam ícones na área de ícones ocultos. Isso não equivale a falha de registro; o usuário pode abrir essa área ou configurar a visibilidade do ícone. A ocultação da taskbar por auto-hide não é alterada pelo programa.

Chamadas síncronas ao shell e menus nativos continuam dependentes de um Windows responsivo. O worker retira essas operações da thread que pinta o trem, mas criação e encerramento aguardam a conclusão cooperativa. Não se usa término forçado de thread, que poderia deixar recursos ou locks inconsistentes. Se o Windows rejeitar todas as tentativas de sinalização e o objeto for destruído, o worker preserva a propriedade do estado nativo; o retorno bem-sucedido do cleanup não é prometido nesse caminho excepcional. Trate o erro de `shutdown()` e registre o diagnóstico antes de descartar o controlador.

## Testes e validação pendente

O módulo inclui três testes de lógica para o protocolo de callbacks e as coordenadas negativas/sentinela de menu. Eles não chamam o shell. Como o módulo é compilado apenas no Windows, devem ser executados pelo CI Windows junto com `cargo test --locked`.

Os resultados reais de compilação e testes desta entrega estão em `TESTES.md`. **Esta documentação não declara execução do ícone, do menu ou do cleanup no Windows real.**

Checklist específico para a execução de integração no Windows:

1. Criar a bandeja mantendo outro aplicativo com foco; confirmar que a janela principal não o toma.
2. Mostrar, ocultar, mostrar pelo ícone e restaurar a janela que estava minimizada. Confirmar que são operações diferentes.
3. Abrir e cancelar o menu com Escape; abri-lo novamente e fechar clicando fora. Testar teclado e mouse.
4. Abrir o menu num monitor com origem negativa. Verificar a posição do menu.
5. Ocultar o trem e reiniciar o Explorer pelo Gerenciador de Tarefas; confirmar reaparecimento do ícone e recuperação da janela quando houver erro.
6. Trocar resolução, conectar/desconectar o segundo monitor e alterar área útil; verificar `DisplayChanged` na UI e preservação de uma posição ainda válida.
7. Sair pelo menu, pelo botão de fechar e com o menu aberto; confirmar remoção do ícone e término do processo sem worker restante.
8. Repetir criação/encerramento em sessão interativa, verificando que não se acumulam ícones, helpers ou classes.
9. Em teste instrumentado, enviar `WM_CLOSE` diretamente ao helper enquanto o trem está oculto: esperar `Unavailable`, repaint e restauração da janela principal.
10. Em teste instrumentado, simular falha de `PostMessageW` por quota durante o menu aberto: verificar o fallback assíncrono, o retorno de `TrackPopupMenu` e o término da thread. Rejeitar ambos os envios deve produzir erro de `shutdown()`, sem `join` bloqueante.

As únicas features adicionais de `windows-sys` necessárias ao adaptador são `Win32_UI_Shell` e `Win32_System_LibraryLoader`. Não há nova crate e não é necessário mudar `Cargo.lock`.
