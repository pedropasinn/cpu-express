# Base atual — CPU Express 0.3.0-rc.5

Use esta árvore integrada; não reaplique patches de entregas anteriores. O funcionamento atual está em [VERSAO_03](docs/VERSAO_03.md), e os resultados e limites em [VALIDACAO_03](docs/VALIDACAO_03.md).

Contratos para contribuições:

- `World::advance_expedition` avança o itinerário por tempo ativo, independentemente da CPU, com paradas de serviço. `advance_train` conserva o adaptador matemático legado.
- Pausa, ocultação e minimização não acumulam progresso offline. A rota determina o ambiente; `SceneContext::from_journey` adapta o domínio ao renderer.
- A coleta pertence a `metrics`; a UI revalida a amostra em cache e aceita métricas parciais válidas.
- `DesktopController` é o proprietário das ações nativas de visibilidade/topo. A faixa se integra como filha da barra horizontal primária; a central abre em janela nativa independente. O modo flutuante continua disponível.
- Preferências e checkpoint de jornada são separados. `progress.rs` mantém checkpoints versionados e bloqueio exclusivo do perfil no Windows. Jornada e desbloqueios persistem; o diário é da sessão. Arquivos incompatíveis são preservados.
- DEMO e encerramento automático não carregam nem gravam a jornada normal.

A pesquisa consolidou 96 achados de A1, A2, A3, A4 e A6, com fontes e índices em `pesquisa/` na raiz do projeto. Trata-se de análise estática; não é código de gameplay recuperado. A arte implementada é procedural e autoral.

As instruções originais permanecem em `docs/entregas_originais/`. O texto anterior desta entrada foi preservado em [INTEGRACAO_RC1](docs/historico/INTEGRACAO_RC1.md). Relatórios da consolidação anterior e `docs/evidencias_consolidacao/` são históricos, não certificação da 0.3.


RC2: a faixa usa uma janela independente no retangulo da barra, na mesma altura e alinhada a esquerda. O encaixe como filha do Explorer foi removido porque uma janela nativa visivel e um framebuffer valido nao garantiam exibicao pelo compositor. A superficie ocupa essa area sem reservar espaco para widgets. A confirmacao visual no desktop e separada dos testes de geometria.
