# Testes — CPU Express 0.3.0-rc.5

O relatório atual é [docs/VALIDACAO_03.md](docs/VALIDACAO_03.md). Ele separa testes automatizados, execução nativa, exportações headless e limites da aceitação. O comportamento esperado está em [docs/VERSAO_03.md](docs/VERSAO_03.md).

- `evidence/current/checks.json`: formatação, Clippy com warnings negados e testes com saída zero.
- `evidence/current/tests.log`: 184 testes Rust aprovados, zero falhas e dois exportadores ignorados na suíte padrão, em execução Windows.
- Python: 19 + 10 + 23 testes aprovados (52 no total), conforme resultado informado pelo integrador; não constam dos três checks Cargo acima.
- `evidence/current/native-smoke-result.json`: execução nativa encerrada normalmente, faixa filha da barra e painel independente verificados; captura da faixa pelo framebuffer nativo.
- `evidence/current/real-metrics-result.json`: nove amostras reais válidas e saída zero.
- `evidence/current/panels/`: exportação headless das quatro abas Trem, Viagem, Visual e Janela. O painel nativo foi validado por HWND/retângulo; não houve captura do compositor do painel.

Para repetir os gates automatizados, use `scripts/verify-current.ps1 -Jobs 2`. Os comandos e o alcance dos resultados estão no relatório atual. O empacotamento exige testes nativos e de metricas reais com hash correspondente ao executavel final.

O roteiro `docs/windows_checklist.md` serve como referência histórica para aceitação complementar, lido à luz da 0.3. DPI misto, autohide, reinício do Explorer, múltiplos monitores e conflitos com widgets ainda precisam de validação específica. Logs de `docs/evidencias_consolidacao/` não são os logs atuais.

Resumo anterior preservado em [TESTES_RC1](docs/historico/TESTES_RC1.md).


RC2: a faixa usa uma janela independente no retangulo da barra, na mesma altura e alinhada a esquerda. O encaixe como filha do Explorer foi removido porque uma janela nativa visivel e um framebuffer valido nao garantiam exibicao pelo compositor. A superficie ocupa essa area sem reservar espaco para widgets. A confirmacao visual no desktop e separada dos testes de geometria.
