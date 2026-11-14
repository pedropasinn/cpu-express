# Validação 0.3.0-rc.7

A revisão integra condições meteorológicas de São Paulo e mantém somente a apresentação completa. Os resultados de formato, Clippy sem warnings e suíte completa ficam em `evidence/current/checks.json`, com logs correspondentes. Os testes incluem códigos meteorológicos, rejeição de respostas inválidas, migração de preferências e aparência distinta de seis condições durante dia/noite. A instrumentação de memória cobre todos os efeitos sem alocação no renderer.

A galeria `evidence/current/weather/weather-review.png` é produzida pelo renderer real com condições sintéticas controladas: céu limpo, nublado, neblina, chuva, tempestade e neve, de cima para baixo; dia à esquerda e noite à direita. Ela não é uma consulta meteorológica. Os painéis em `evidence/current/panels-rc7` são exportações headless do egui.

`scripts/validate-weather.ps1` executa uma consulta real no aplicativo Windows, com perfil isolado, duração limitada e diagnósticos em `evidence/current/weather/run-*`. O resultado consolidado registra o estado efetivamente observado; falha de rede não equivale a validação online bem-sucedida. O empacotamento exige que o resultado pertença ao executável final.

`scripts/validate-visual-refresh.ps1` verifica o painel independente, métricas reais e quatro alternâncias barra/janela. `evidence/current/transitions-rc7/result.json` confere a área de desenho contra os limites nativos. Capturas são framebuffers do aplicativo, não screenshots do compositor. A medição de cadência é curta e local, comparada ao baseline histórico da rc.4; não mede consumo nem desempenho universal.

BUILD-INFO registra hashes do binário e fontes, contagem de testes e evidências. DPI misto, autohide, reinício do Explorer e estabilidade prolongada continuam exigindo validação específica.

## Verificação da interface de jogo

A rc.7 acrescenta testes de interação com os widgets e com a oficina real: escolher uma posição, equipar apenas esse vagão, preservar pintura bloqueada e liberar a seleção no DEMO. Os testes de miniaturas conferem recorte completo, cores e reutilização das texturas. A exportação de painéis percorre as quatro abas principais, as páginas de pinturas/clima/fluidez e uma janela reduzida. Formato, Clippy e suíte completa permanecem gates de distribuição.
