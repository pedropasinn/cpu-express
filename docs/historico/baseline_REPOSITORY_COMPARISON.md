# Escolha da base do CPU Express 0.1

**Decisão: aplicação nativa em Rust com eframe/egui e sysinfo.** A investigação dos cinco repositórios aconteceu antes da implementação da cena. Todos foram clonados e tiveram código, dependências, licenças e mecanismos de janela examinados. Nenhum entrega o fluxo CPU → velocidade suavizada → trem com parallax. Os forks acrescentariam uma webview e código periférico, enquanto o fluxo principal ainda precisaria ser escrito.

Isso não significa evitar reaproveitamento. O produto usa as bibliotecas eframe, egui, winit e sysinfo; os repositórios ajudam a identificar padrões úteis e problemas concretos. Nesta versão, nenhum trecho substancial nem recurso visual dos cinco projetos foi incorporado. Os desenhos do trem e da paisagem são originais e procedurais.

## Escopo da evidência

Auditoria em **13/11/2026**, em Linux x86-64. “Existe no código”, “compilou” e “executou no Windows” são resultados distintos. Os frontends de WindowPet e Desktop-pet compilaram, mas seus backends/janelas não foram executados nesta auditoria. Não foram medidos CPU, RAM ou execução prolongada dos cinco aplicativos; portanto, não há ranking numérico de desempenho entre eles.

| Repositório | Commit realmente clonado | Licença observada |
|---|---|---|
| [SeakMengs/WindowPet](https://github.com/SeakMengs/WindowPet) | `aba8d3ab94d30f4f5abf92b3f3ec7dff50d3326b` | MIT no código; não herdamos os assets |
| [Mukund2074/Desktop-pet](https://github.com/Mukund2074/Desktop-pet) | `2ee887fb754a42e58677d62e1fa6fbae54129264` | MIT |
| [Sids15/desktop-sprite](https://github.com/Sids15/desktop-sprite) | `ffd31731df81ab860411ed0546a9e7f23e0fa903` | Nenhum arquivo/campo de licença localizado; não copiar |
| [quinnjr/clippy-desktop](https://github.com/quinnjr/clippy-desktop) | `036c8acf5bc7682f8b691be30f43af27b077411f` | MIT no código; arte e dados Clippy explicitamente excluídos |
| [iamdhakrey/RustyIsland](https://github.com/iamdhakrey/RustyIsland) | `2538533a3d180f0a433cf31ac06ff4b472cebcf6` | MIT |

## Comparação dos cinco projetos

| Projeto | Janela e rendering efetivos | Animação e métricas | Bandeja | Evidência Windows e Linux |
|---|---|---|---|---|
| **WindowPet** | Tauri 1 + React + Phaser; overlay transparente fullscreen. Render WebGL/Canvas, física Arcade, sprite sheets. | 30 FPS configurados no Phaser; sprites a 9 FPS; polling de cursor por IPC para click-through. Sem CPU → trem. | Show, Pause, Settings, Restart, Quit; pausa fecha a webview e Show a reconstrói. | Releases EXE/MSI e matriz Windows/macOS/Ubuntu em `.github/workflows/release.yml`; distribuição Linux existente. Nenhuma execução nativa nesta auditoria. |
| **Desktop-pet** | Tauri 2; HTML/SVG/JS. A janela é ampliada ao retângulo que contém todos os monitores, apesar das referências a 160×160 no README. | Deslocamento fixo por callback de `requestAnimationFrame`; depende da frequência. Distância zero gera NaN em teste. Sem métricas de CPU. | Show/Hide/Settings/Quit implementados. | Configuração WebView2; nenhuma release pública resolvida e nenhum workflow rastreado. Linux sugerido pelas abstrações, não validado. |
| **desktop-sprite** | Tauri 2 + React; pet 128×128 e painel separado. Sprite por CSS `background-position`. | FSM idle/walk/react. Callback de animação limitado por FPS, mas rAF permanece ativo; click-through consulta cursor ~60 Hz. | Mostrar/esconder/abrir painel/sair, realmente implementados. | Workflow Windows/macOS em `.github/workflows/release.yml`. Não testado em Windows; Linux sem matriz de build. |
| **clippy-desktop** | Rust + winit/glutin/glow; egui para painéis. Quad OpenGL com textura nearest e alfa pré-multiplicado. | Deadlines explícitos, tempo injetável, estado separado. Event loop dorme até o próximo frame e textura só muda quando necessário. Sem métricas. | Não tem tray; tem menu de contexto `muda` em Windows/macOS, teclado no Linux. | Código multiplataforma real. Wayland tem ressalva explícita de posicionamento/empilhamento. Não compilado/executado nesta auditoria. |
| **RustyIsland** | Tauri 2 + React; widget transparente 320×40 no topo. Transições CSS, sem sprites. | sysinfo real a cada 2 s, mas `refresh_all()` inclui processos; primeira CPU não é marcada como aquecimento. A UI pede foco a cada 5 s. | Não encontrada. | APIs Tauri e tentativas específicas de Wayland; sem testes/workflow rastreados. Nenhuma execução nativa nesta auditoria. |

### Dependências e consequências

WindowPet fixa Tauri 1.6.7 no lock e reúne React 18, Phaser 3, Mantine, Zustand, React Query e plugins vindos de branches Git. Isso oferece bastante funcionalidade para pets que passeiam sobre a tela, mas exige manter uma pilha maior que a necessária para o CPU Express.

Desktop-pet contém duas árvores Rust, na raiz e em `src-tauri/`, com manifests divergentes e nenhum Cargo.lock versionado. O movimento efetivo está em `index.html`; o módulo Rust de comportamento não comanda a animação em execução. Um fork teria primeiro de resolver essa duplicidade e corrigir os defeitos de movimento.

desktop-sprite fixa Tauri 2.11.2. Sua bandeja é uma referência clara, mas o código não tem licença de reutilização localizada. clippy-desktop fixa egui 0.33.3, winit 0.30.13 e glutin 0.32.3, e organiza bem renderer, simulação, janela e configuração. Sua arte Microsoft não pode ser tratada como MIT. RustyIsland fixa sysinfo 0.30.13; o CPU Express usa a API atual verificada, com coleta restrita a CPU/RAM.

As versões atuais escolhidas para o CPU Express estão em `Cargo.toml` e `Cargo.lock`; a investigação de compatibilidade está em [dependency-research.md](dependency-research.md). Quantidade de dependências, tamanho de instalador e tamanho de assets não substituem medições de memória residente ou utilização da CPU.

## Defeitos e limitações que influenciaram a decisão

**A janela dos forks não corresponde ao produto pedido.** WindowPet é fullscreen; Desktop-pet cobre a união dos monitores. O CPU Express precisa de uma janela opaca e compacta. Retirar overlay, click-through e controles de pet não fornece o trem, o cenário ou o coletor que faltam.

**O movimento de Desktop-pet depende de frames.** Em um harness Node da classe original, 30 atualizações moveram 36 unidades e 60 moveram 72. Com destino igual à posição, o cálculo dividiu por distância zero e produziu coordenadas não finitas. Esses resultados são da lógica executada em isolamento; não são medições de FPS do aplicativo.

**RustyIsland toma foco periodicamente.** `src-tauri/src/lib.rs:134–139` chama `set_focus()` em `ensure_always_on_top`, invocado a cada cinco segundos em `src/DynamicIsland.tsx:50–66`. Isso pode interromper digitação em outro programa. O CPU Express não precisa forçar foco para permanecer visível.

**Posicionamento exige implementação própria.** WindowPet subtrai uma altura presumida de taskbar, Desktop-pet usa a união dos monitores, clippy-desktop usa margens fixas e RustyIsland fixa o topo em y=0. Nenhuma dessas implementações comprova os requisitos combinados de área útil, barra em qualquer borda, origem negativa e DPI misto.

**A licença limita o reuso de recursos.** desktop-sprite não tem licença localizada. Clippy exclui expressamente os assets Microsoft da MIT. Os assets WindowPet não têm uma cadeia de origem por imagem demonstrada na auditoria. O trem original elimina essa dependência e atende à identidade desejada para o produto.

## Três estratégias

| Critério | Fork WindowPet | Fork Desktop-pet | CPU Express em eframe/egui |
|---|---|---|---|
| O que já vem pronto | Loader, animações/estados, física, interação de pets, bandeja e distribuição | Bandeja, configurações e overlay com pets SVG | Janela, eventos, UI, backend gráfico e integração DPI do framework |
| O que falta para o MVP | CPU/RAM, suavização, trem, cenário, parallax e refatoração da janela | Mesmos itens, mais correções do movimento e escolha da árvore Rust | Núcleo pequeno de simulação, coleta, desenho original e work area Windows |
| Rendering integralmente Rust | Não | Não | Sim |
| Janela 640×140 | Mudar arquitetura fullscreen | Mudar união de monitores e mínimos de tamanho | Configuração direta do viewport |
| Aprendizado de Rust | Parte pequena do rendering/movimento | Movimento principal em JavaScript | Ownership, traits, estados, threads e borrowing no fluxo principal |
| Risco concreto encontrado | Testes parcialmente falhos; stack ampla; lógica de overlay desnecessária | Movimento por frame/NaN; duplicidade; docs divergem do código | Integração Windows deve ser implementada, compilada e validada no SO |
| Ganho provável de um fork para este escopo | Moderado nas funções secundárias; baixo no fluxo principal | Baixo, com correções obrigatórias | Mais alinhamento com o produto e menos camadas próprias |
| Linux futuro | Pacotes existentes; comportamento do compositor ainda a validar | Possível via Tauri; promessa sem validação local | Núcleo portável; módulo de desktop e empacotamento ficam para v0.5 |

**Recomendação fundamentada:** eframe/egui. WindowPet seria a escolha mais forte entre os dois forks para um pet interativo de tela inteira, graças à implementação e aos artefatos distribuídos. Para um trem compacto ligado a métricas, o código reutilizável dos forks está principalmente nas funções adiadas pelo próprio escopo. Não há necessidade concreta que justifique introduzir Tauri/React/Phaser no CPU Express 0.1.

O padrão mais útil dos cinco é o de clippy-desktop: estado testável, agendamento pela próxima atualização e reutilização da textura. Ele orienta decisões sem exigir herdar a aplicação, sua arte ou o OpenGL manual. A forma do snapshot CPU/memória do RustyIsland é útil como comparação, mas a nova coleta deve ter aquecimento, trabalho limitado, canal seguro e encerramento.

## Testes e verificações efetivamente feitos nos repositórios

| Projeto | Executado | Resultado e alcance |
|---|---|---|
| WindowPet | Clone; instalação JS com scripts desativados; `npm run build`; Vitest com um worker | Frontend passou; teste de mock da janela passou; suíte de componentes falhou com `Cannot find module 'phaser3spectorjs'`. Não comprova Tauri/Windows. |
| Desktop-pet | Clone; instalação JS com scripts desativados; `npm run build`; harness Node de movimento | Frontend passou. Harness reproduziu dependência de frames e NaN. Sem execução desktop. |
| desktop-sprite | Clone; parse dos manifests/JSONs/lock; seis asserções Node do FSM original | 6/6 passaram: transições válidas e eventos desconhecidos. Não houve build Tauri. |
| clippy-desktop | Clone; leitura de fontes/testes; parse de manifest/lock | Inspeção concluída; testes Rust não executados. |
| RustyIsland | Clone; leitura de fontes; parse de manifests/JSONs/lock | Inspeção concluída; sem suíte localizada e sem build/runtime. |

Os relatos completos, com arquivos, linhas e links fixados nos commits, estão em [repository-audit-forks.md](repository-audit-forks.md) e [repository-audit-others.md](repository-audit-others.md). Os dois documentos descrevem observações do mesmo checkout, não revisões independentes em sistemas operacionais diferentes.

## O que o build Windows já demonstrou

A janela mínima do CPU Express foi compilada com Rust 1.100.0, eframe 0.37.0 e alvo `x86_64-pc-windows-gnu`, usando GCC MinGW 14 e binutils 2.44. O arquivo resultante é PE32+ x86-64, subsistema Windows GUI, com imports do Windows/OpenGL e sem DLL externa de libgcc/libwinpthread na tabela de imports. Isso demonstra compilação/link da etapa 1.

Wine 10 foi extraído localmente, mas abortou ainda na inicialização do programa de teste; não foi usado como comprovação de runtime. A janela gráfica foi validada no host Linux para desenvolvimento. Esses fatos não substituem teste real em Windows 10/11. O relatório final de validação registra a versão entregue e os testes posteriores.

## Checklist de validação real em Windows 10/11

Este checklist é uma lista de verificações pendentes, não uma afirmação de aprovação. Preencher versão do Windows, CPU, GPU, driver e escala em cada execução.

| Caso | Critério de aprovação | Status de runtime Windows nesta auditoria |
|---|---|---|
| Iniciar e fechar | Janela aparece sem console; X, Escape e Alt+F4 encerram sem processo residual | Pendente |
| Telemetria | Aquecimento explícito; CPU/RAM reais renovados ~1 Hz; UI responde durante coleta | Pendente |
| Movimento | CPU manual 0/50/100% converge suavemente para 12/56/100 km/h; paisagem/rodas/fumaça acompanham | Pendente |
| Área útil | Janela inteira dentro da área útil, perto da barra embaixo/em cima/à esquerda/à direita quando suportado pelo Windows | Pendente |
| DPI e monitores | 100/125/150/200%; monitores com escalas diferentes e origem negativa; arraste e reposicionamento utilizáveis | Pendente |
| Foco e topo | Always-on-top liga/desliga; digitar em outro app não é interrompido | Pendente |
| Minimizar/restaurar | Trabalho gráfico reduzido; sem salto grande ou amostra falsamente atual na volta | Pendente |
| Repouso prolongado | Medir CPU e memória do release, documentar duração e observar tendência; sem teste artificial de stress | Pendente |

Os testes de CPU manual alteram somente o valor de entrada da simulação. Não criam carga artificial no processador.
