# Validação CPU Express 0.4.0-rc.1

Executado em 2026-11-14, no Windows local. SHA-256 do executável: `439ba4d50404d7f16c5038a70e48576660167b7f0a578d28bbdfd58ed8d4622a`.

## Gates executados

- `scripts/verify-current.ps1 -Jobs 4`: formatação, Clippy com warnings negados e 272 testes aprovados; zero falhas, dois exportadores ignorados na suíte normal.
- Release Windows GNU compilada com Cargo.lock, usando a toolchain local.
- 16 cenários de UI exportados do egui, rasterizados e inspecionados: cabine, mapa, objetivos, diário vazio/preenchido, serviços vazios/preenchidos, excursão, vagões, oficina pequena, pinturas, locomotivas, coleção, clima, fluidez e janela. Manifesto: `evidence/current/game-ui-result.json`. A lista tem 16 imagens; cabine e jornada compartilham cenários de estado. São imagens headless, não capturas do compositor.
- Smoke Windows: faixa alinhada à barra e painel nativo independente encontrados; framebuffer da faixa capturado. `panel_capture=false`: o painel foi verificado por geometria e pelos exports egui, não por screenshot nativo.
- Quatro mudanças barra/flutuante passaram, com cinco estágios e framebuffer limitado à área visível. Dados: `evidence/current/transitions-expansion/result.json`.
- Quatro amostras válidas de CPU/RAM reais. Cadência curta aquecida: aproximadamente 59,80 FPS produzidos, com limite 60 FPS. Isso não mede apresentação do compositor nem consumo de energia.
- Open-Meteo respondeu em São Paulo: 25 amostras prontas, sem erro; temperatura, vento, precipitação e horários solares presentes. Dados: `evidence/current/weather/result.json`.
- Uma cópia isolada do perfil v1 existente foi aberta e encerrada normalmente. Distância 123625.00 → 123836.82; formato v2, quatro pinturas preservadas, backup v1 verificado. Perfil original permaneceu intocado durante esse teste. Dados: `evidence/current/profile-migration-result.json`.

- Pacote portátil extraído para uma pasta com espaços e executado independentemente da toolchain: saída zero, binário com o mesmo SHA-256. Integridade CRC dos três ZIPs verificada. Dados: `evidence/current/portable-result.json`. Isso não é um teste em uma conta Windows nova.

## Cobertura de domínio

Testes exercitam capacidade, conservação de passageiros, categorias de carga, entrega única, pausa, diferenças de passo temporal, reinício nas fases de serviço/excursão, descarte de notificações, saves inválidos/futuros, migração e importação. A importação tem regressão específica para preservar a sessão ainda não salva quando o arquivo do disco está corrompido.

Renderer e atualização de movimento sem clonagem de Config têm testes de alocação. Locomotivas preservam rodas/engates; versões diesel/elétrica não desenham fumaça de vapor. As excursões têm parada e retorno, sem reinterpretar distância do circuito principal.

Áudio: sete testes de PCM, duração/volume, fila, descarte e encerramento; não foi feita aceitação audível do dispositivo. O produto começa mudo. O bloco de autostart do instalador foi ensaiado numa chave isolada, sem instalar nem alterar a chave Run real. Os scripts de instalação/desinstalação não foram executados contra uma instalação do usuário.

## Execução estendida curta

Duas execuções nativas de 120 segundos concluíram normalmente, ambas com embarque real (sete passageiros): painel fechado e aberto. Respectivamente, 7.197 e 7.168 frames produzidos. Na janela final de 256 frames, o p95 de trabalho da aplicação foi 0,597 ms e 0,463 ms. Working set final: aproximadamente 88,8 MiB e 105,3 MiB.

Após dez segundos de aquecimento, CPU média equivalente a um núcleo: 15,2% com painel fechado e 33,1% com painel aberto. Essa métrica não é o percentual normalizado de todos os processadores do Gerenciador de Tarefas. Variação de memória privada nesse intervalo: +2,08 MiB / −3,13 MiB. São observações curtas, não prova de ausência de vazamentos ou consumo de bateria. Dados completos: `evidence/current/soak/result.json`. Não equivale a um dia de uso.

## Limites

Não certificados: Windows 10, DPI misto real, matriz completa de escalas, autohide, reinício real do Explorer, desconexão de monitor, leitores de tela e sessão de um dia inteiro. Os testes geométricos cobrem casos sintéticos dessas áreas, sem substituir aceitação no hardware. Pacote sem assinatura digital. Sem alegação de consumo de bateria, ausência de vazamento em longa duração ou certificação 1.0.

## Revisão e correções

Seis frentes de implementação/revisão contribuíram. Corrigidos antes da entrega: autostart que poderia recriar a chave Run, importação sem preservar a sessão em memória, erro de importação escondido no histórico, miniaturas em escala insuficiente, informações de itens bloqueados e descrição incorreta de histórico ilimitado.
