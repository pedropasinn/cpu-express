# CPU Express 0.3.0-rc.5 — viagem persistente e integração nativa

- Integra a faixa à esquerda da barra horizontal primária do Windows por HWND e abre a central em janela independente; oferece retorno ao modo flutuante.
- Separa a velocidade visual ligada à CPU da progressão lógica a 24 unidades por segundo ativo.
- Percorre o Circuito Aurora, com quatro estações e oito segundos de serviço em cada chegada.
- Libera pinturas por objetivos de uma, quatro e doze chegadas; preserva os cinco modelos de vagão.
- Salva a jornada em checkpoint versionado separado das preferências, com recuperação dos desbloqueios e bloqueio exclusivo do perfil no Windows.
- Salva a cada 30 segundos ativos, na saída normal e pelo comando Salvar viagem agora. Preserva arquivos incompatíveis e mantém retry limitado das preferências.
- Congela a viagem durante pausa, ocultação e minimização, sem progresso offline. O diário permanece da sessão; jornada e recompensas persistem.
- Usa 96 achados consolidados das pesquisas como referência de decisões; plataforma, passageiros e pinturas são procedurais e autorais.

Veja [funcionamento e limites](docs/VERSAO_03.md) e [validação da 0.3](docs/VALIDACAO_03.md). Há evidência de execução nativa no Windows, além de testes automatizados; a aceitação completa da matriz Windows permanece pendente. A exportação headless das abas não equivale a captura do compositor do painel. A revisão final dos glifos requer evidência correspondente ao build final.

Notas anteriores: [PATCH_NOTES_RC1](docs/historico/PATCH_NOTES_RC1.md).


RC2: a faixa usa uma janela independente no retangulo da barra, na mesma altura e alinhada a esquerda. O encaixe como filha do Explorer foi removido porque uma janela nativa visivel e um framebuffer valido nao garantiam exibicao pelo compositor. A superficie ocupa essa area sem reservar espaco para widgets. A confirmacao visual no desktop e separada dos testes de geometria.
