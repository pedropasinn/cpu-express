# Fronteira Windows: unidades, foco e propriedade

## Separação das responsabilidades

`geometry.rs` recebe números e retorna posições. `windows.rs` empresta o HWND
atual, consulta Win32 e aplica uma operação breve na thread da UI.
`lifecycle.rs` define estados e restrições sem sistema operacional.
`desktop.rs` converte intenções em ações no ciclo eframe. `tray.rs` possui apenas
seu ícone, sua janela auxiliar e sua thread; ele envia intenções à UI por canal.

O controlador não é Send/Sync e nunca guarda um HWND do jogo. Sua marca de
afinidade com a UI impede o deslocamento acidental entre threads. A fronteira
Win32 também verifica o identificador da thread proprietária antes de operar.

## Pixels físicos

Os retângulos usam limites semiabertos. A largura é `right - left`, calculada
em `i64` antes de qualquer conversão para `i32`. Isso evita overflow com
monitores negativos e extremos do tipo inteiro. O tamanho atual da janela é
medido em pixels; apenas a margem de oito pontos é convertida usando DPI.

`GetWindowRect` pode ser virtualizado conforme o contexto de DPI. Por isso o
adaptador verifica awareness por monitor na janela e na thread. Ele não muda
o contexto global nem tenta corrigir um ambiente desconhecido dividindo as
coordenadas do desktop. Se o processo for iniciado com DPI incompatível, a
operação falha com mensagem e preserva a posição. Referências:
[GetWindowRect](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowrect),
[GetDpiForWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow).

`rcWork` define a área utilizável. Uma interseção vazia é rejeitada; não se usa
`rcMonitor` como uma área inventada. `ABM_GETAUTOHIDEBAREX` identifica a borda de
uma appbar em ocultação automática no monitor informado. A consulta não cria
nem altera appbars. Quando o shell não responde com uma barra, o cálculo usa
as reservas já conhecidas da área útil. Referência:
[ABM_GETAUTOHIDEBAREX](https://learn.microsoft.com/en-us/windows/win32/shell/abm-getautohidebarex).

Com auto-hide, a margem deixa espaço junto à borda para acionar a barra. O
módulo não reserva toda a espessura que a barra poderá ocupar ao aparecer:
essa espessura não é garantida por `rcWork`. A barra expandida pode sobrepor o
aplicativo temporariamente; não se bloqueia nem se modifica sua política.

## Foco e cache do gerenciador de janelas

Reposicionar usa `SetWindowPos` com `SWP_NOACTIVATE | SWP_NOZORDER`. Quando o
tamanho não precisa mudar, acrescenta `SWP_NOSIZE`.
[Referência SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos).

Restaurar usa `ShowWindow(SW_SHOWNOACTIVATE)`, confere visibilidade e ausência
de minimização e só depois sincroniza o estado com `winit_window()`. Minimizar
usa `SW_SHOWMINNOACTIVE` e sincroniza `MINIMIZED` após confirmar o resultado.
O valor retornado por ShowWindow descreve a visibilidade anterior, não sucesso.
[Referência ShowWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow).

O caminho foi revisado nas fontes das versões fixadas no lockfile:
`winit 0.30.13`, `platform_impl/windows/window.rs` e `window_state.rs`, e
`egui-winit 0.37.0`, `src/lib.rs`. Chamadas diretas só ao Win32 não atualizam
todos os flags mantidos pelo winit. Portanto ocultar usa `set_visible(false)`,
e topo segue o comando eframe para manter os caches coerentes.

Há uma precondição: manter `.with_active(false)` da criação original. Nesta
versão do winit, isso controla a variante de ShowWindow usada ao alterar flags.
Mudar topo enquanto a janela está minimizada pode restaurá-la; o controlador
guarda essa preferência até uma restauração. As ações são enfileiradas e o
comando de topo é emitido somente após as ações de visibilidade daquela passada.

O menu contextual da bandeja chama `SetForegroundWindow` **somente depois do
pedido explícito de abrir o menu**, seguindo o protocolo de fechamento dos
menus Win32. Isso não dá foco ao trem na inicialização, no docking ou no topo.

## Eventos e encerramento

Não há timer que confira continuamente os monitores. A recuperação é explícita
ou provocada por mensagens de alteração da área de trabalho/DPI. Uma posição
arrastada válida permanece; uma janela fora da área útil recebe o menor ajuste
necessário. Uma janela minimizada aguarda restauração.

A bandeja recebe `TaskbarCreated` por uma janela auxiliar top-level invisível;
message-only windows não recebem esse broadcast. O módulo remove qualquer
registro sobrevivente e tenta ADD + SETVERSION uma vez. Falha gera
`Unavailable`, acorda a UI e permite restaurar a janela oculta. Consulte
`windows_tray.md` para as chamadas e os casos de erro detalhados.

`shutdown` sinaliza a thread própria, remove o ícone, destrói a janela auxiliar
e aguarda seu término. Ele não chama DestroyWindow no HWND do eframe. A saída
do aplicativo continua usando `ViewportCommand::Close` e a liberação normal
do worker de métricas. Criação/encerramento são cooperativos: não se promete
um tempo máximo quando uma chamada do sistema está travada.

## Falhas recuperáveis

| Falha | Comportamento |
| --- | --- |
| HWND ausente, inválido ou thread errada | Erro de operação; nenhum handle é inventado ou retido. |
| DPI incompatível ou área útil vazia | Posição preservada; UI recebe mensagem. |
| Área útil pequena | Janela reduzida fisicamente; UX adapta os controles. |
| Registro inicial da bandeja recusado | Jogo continua visível, ocultar permanece desabilitado. |
| Bandeja falha após ocultação | Controlador tenta mostrar a janela; mensagem registra falha nativa se houver. |
| Desativar bandeja com janela oculta | Mostra primeiro; se isso falhar, conserva a recuperação existente. |
| Explorer ausente sem novo evento | A última confirmação pode ficar desatualizada; limitação declarada. |

Os testes puros provam os invariantes numéricos e a política de estados. Foco,
ordem visual, callbacks do shell e liberação real de handles ainda exigem
execução do checklist em Windows.
