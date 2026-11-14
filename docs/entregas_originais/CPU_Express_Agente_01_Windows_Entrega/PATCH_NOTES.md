# Agente 01 — alterações da frente Windows

## Resumo em 10 linhas

1. A entrega usa a linha A recebida e preserva o motor, as métricas, a arte e os arquivos de UX.
2. A geometria física foi isolada, mantendo as assinaturas e testes existentes.
3. O posicionamento aceita bordas explícitas, coordenadas negativas e escalas DPI distintas.
4. Áreas pequenas reduzem a janela; áreas úteis inválidas produzem erro recuperável sem movimento.
5. O adaptador identifica a barra em ocultação automática no monitor correspondente.
6. A recuperação por evento desloca apenas o necessário e preserva posições arrastadas válidas.
7. O controlador distingue janela visível, minimizada, oculta por usuário e encerrada.
8. A bandeja implementa menu, recuperação após reinício do Explorer e liberação de recursos próprios.
9. Uma demonstração e um patch de integração acompanham o código; o CI MSVC está preparado.
10. Compilação, testes executados e pendências de Windows real estão separados em TESTES.md.

## Alterações de comportamento

`dock_window(&eframe::Frame)` continua disponível para o jogo. Ele valida o HWND
emprestado, sua thread proprietária e os contextos de DPI antes de trabalhar em
pixels físicos. A origem global do monitor nunca é dividida pela escala. A área
útil retornada pelo sistema é intersectada com o monitor; uma área vazia ou
inconsistente não é substituída pelo monitor inteiro, pois isso poderia cobrir a
barra de tarefas.

O modo automático consulta as barras em ocultação automática do monitor atual.
A reserva de `rcWork` continua delimitando a área disponível. Quando não há barra
detectável, permanece o alinhamento previsível junto à maior reserva, com
canto inferior direito como fallback. A margem padrão é oito pontos convertidos
para pixels no DPI da janela. Em áreas excepcionalmente pequenas, a margem cede
e as dimensões encolhem; a adaptação dos controles é uma tarefa da UX.

O `DesktopController` enfileira ações e as aplica no início de `App::logic`,
inclusive enquanto a janela está oculta. A visibilidade é sincronizada com o
cache do winit. A preferência de topo é aplicada pelo eframe e fica pendente
quando a janela está minimizada, para não restaurá-la por acidente. A criação
deve continuar usando `.with_active(false)`.

A bandeja usa uma thread e uma janela auxiliar próprias, ambas independentes do
HWND do jogo. O menu oferece mostrar, ocultar, reposicionar, alternar topo e sair.
O ícone é o recurso compartilhado padrão do Windows; nenhuma arte de outra
frente foi modificada. Não há hooks globais, subclassing da janela do jogo,
appbar com reserva de espaço, injeção no Explorer ou sondagem periódica do shell.

## Arquivos alterados e adicionados

| Arquivo | Alteração |
| --- | --- |
| `src/platform/mod.rs` | Reexports, módulos isolados e manutenção do contrato original. |
| `src/platform/windows.rs` | HWND/DPI/área útil, auto-hide, posicionamento e visibilidade com cache coerente. |
| `Cargo.toml` | Somente três features adicionais da dependência windows-sys já existente. |
| `src/platform/geometry.rs` | Geometria pura extraída e expandida. |
| `src/platform/lifecycle.rs` | Estados e regras de recuperação testáveis sem Windows. |
| `src/platform/desktop.rs` | Controlador e integração ao ciclo do eframe. |
| `src/platform/tray.rs` | Bandeja nativa, eventos, menu e cleanup. |
| `tests/platform_geometry.rs` | Matriz de monitores/DPI/bordas e recuperação. |
| `tests/platform_lifecycle.rs` | Transições, proteção de ocultar e encerramento terminal. |
| `examples/windows_desktop.rs` | Demonstração executável independente. |
| `.github/workflows/windows.yml` | CI MSVC, testes, executáveis e hashes. |
| `integration/windows-ux.patch` | Proposta concreta para o app do Agente 05, não aplicada. |
| `integration/windows-features.patch` | Registro restrito do diff de Cargo.toml, já aplicado neste ZIP. |
| `docs/windows_*.md` | Arquitetura, bandeja, checklist e CI. |
| Documentos de entrega e `validation/windows-agent/` | Instruções, resultados, metadados e logs desta rodada. |

`Cargo.lock`, `rust-toolchain.toml`, `src/app.rs`, `src/config.rs`, `src/main.rs`,
`src/lib.rs`, métricas, simulação, renderização, assets, testes baseline e scripts
existentes permanecem idênticos ao ZIP recebido. O inventário
`validation/windows-agent/source-audit.json` registra a comparação.

### Exceção mínima no manifesto

Para que o pacote compile isoladamente no Windows, foram habilitadas
`Win32_UI_Shell`, `Win32_System_LibraryLoader` e `Win32_System_Threading` na mesma
versão `windows-sys = 0.61.2`. Não foi acrescentada crate ou alterada versão.
Essa é a única edição compartilhada: o diff foi isolado para o coordenador
mesclar as features, sem substituir mudanças de outros agentes. O lockfile foi
preservado.

## Três riscos reais

1. **Integração Windows ainda sem execução visual nesta entrega.** Foco, DPI
   misto, Explorer e taskbar precisam do checklist no equipamento de destino.
   Compilação cruzada e testes puros não substituem esse aceite.
2. **Disponibilidade da bandeja é o último reconhecimento do shell.** Se o
   Explorer morrer e permanecer parado, o módulo não detecta imediatamente a
   ausência. Ele tenta registrar novamente no evento `TaskbarCreated` e solicita
   restauração quando detecta falha. Não há promessa de recuperação instantânea
   durante a ausência permanente do shell.
3. **A integração precisa manter uma única autoridade sobre a janela.** Misturar
   comandos diretos de visibilidade/topo com o controlador pode desalinhar os
   estados. O patch demonstra o encaminhamento necessário. Atualizações de
   eframe/winit e o layout em áreas minúsculas exigem nova conferência.

## Rollback

Se o patch de UX foi aplicado ao baseline sem outras edições, reverta-o com
`git apply -R integration/windows-ux.patch`. Em uma árvore já mesclada, reverta
somente os trechos introduzidos por essa integração.

Restaure `src/platform/mod.rs`, `src/platform/windows.rs` e a linha de
`windows-sys` em `Cargo.toml` a partir do ZIP original. Remova apenas os módulos,
testes, demonstração, workflow e documentos novos desta frente. Não remova
features que outro agente passou a utilizar. Não há migração de configuração,
dados ou dependências a desfazer; a bandeja não escreve estado persistente.
