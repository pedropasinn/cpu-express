# Evidências da frente de arte

`../../TESTES.md` interpreta os resultados e descreve os limites. Os arquivos desta pasta registram apenas a execução desta rodada:

- `fmt.log`, `clippy.log`, `test.log`, `release.log`: checks do host.
- `scene-benchmark.json`: cada rodada dos oito microbenchmarks novos.
- `baseline-benchmark.json`: benchmark independente da fonte original.
- `environment.json`: toolchain/host; não descreve o computador do usuário.
- `visual-checks.json`: igualdade PPM/PNG, escala inteira e regressão do legado.
- `patch-check.json`: aplicação concreta do patch, hashes e arquivos preservados.
- `host-window.png`, `host-app.log`, `host-smoke.json`, `host-build-info.json`: execução de cinco segundos da janela de desenvolvimento no host Linux.
- `windows-release.log`, `windows-build-info.json`: compilação cruzada concluída; não prova execução em Windows.

Os benchmarks medem um host virtual, não consumo no Windows. A fotografia de estado exportada pelo renderer e a captura da janela são evidências diferentes: a primeira permite comparação determinística sem interface; a segunda confirma que a UI recebeu e mostrou a textura nova.

Nenhuma chave do servidor gráfico, configuração pessoal ou ferramenta temporária de autenticação acompanha a entrega.
