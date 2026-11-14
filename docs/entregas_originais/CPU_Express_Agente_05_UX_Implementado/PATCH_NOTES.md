# CPU Express — entrega Agente 05 / UX

Esta é uma implementação sobre a linha A. Comece por este documento: o `BUILD-INFO.json`, o `README.md` e a validação recebida na base são históricos e não certificam esta versão modificada.

## Resumo em 10 linhas

1. Implementados HUD compacto, completo e limpo, com sinalização persistente de simulação e pausa.
2. Barra de controles recolhe quando a janela não tem foco nem mouse sobre ela.
3. Painel de opções dividido em Viagem, Visual e Janela, com rolagem interna na janela 640×140.
4. Ajuda por F1 e pelo botão `?`; Escape fecha o painel e nunca encerra o aplicativo.
5. Pausa por Espaço/botão congela a simulação e descarta o tempo de pausa na retomada.
6. `--demo` usa `ManualProvider`, não consulta contadores reais e não inventa RAM.
7. Presets 0/50/100% e ajuste de CPU em passos de 0,1%, sem gerar carga artificial.
8. Preferências de FPS, resposta, HUD, pin e pausa com limites e defaults compatíveis.
9. Persistência atômica preserva campos desconhecidos e recusa destruir configuração corrompida.
10. Incluídos testes, guia de uso, evidências do egui e instruções de integração independente.

## Código alterado

| Arquivo | Responsabilidade |
| --- | --- |
| `src/app.rs` | Interface, pausa, relógio, mensagens, shortcuts, demo e adaptadores para comandos. |
| `src/config.rs` | Preferências, defaults, modo de HUD e proteção da persistência. |
| `src/main.rs` | CLI retrocompatível, `--demo`, ajuda e proteção de caminhos de saída. |

Testes novos: `tests/ux_app.rs`, `tests/ux_config.rs`, `tests/ux_cli.rs` e `tests/ux_visual.rs`. O último exporta evidências sob solicitação explícita e fica ignorado na execução normal. Há também testes unitários da fábrica de demo e do parser dentro dos arquivos de origem.

Documentação nova: este arquivo, `INTEGRACAO.md`, `TESTES.md` e arquivos `docs/ux_*`. O patch da frente e o manifesto de hashes são materiais de integração, não novas dependências do aplicativo.

Os módulos dos agentes 1–4, todos os testes originais, o manifesto, o lockfile, os exemplos e os scripts foram preservados. Não há dependência nova nem mudança de versão no manifesto compartilhado.

## Demonstração no Windows

```powershell
cargo run --locked -- --demo --cpu 50
```

Com um executável compilado a partir desta fonte:

```powershell
.\cpu-express.exe --demo --cpu 50
.\cpu-express.exe --demo --cpu 0 --smoke-seconds 10
.\cpu-express.exe --cpu 100
.\cpu-express.exe --config .\preferencias.json
```

`--cpu` sem `--demo` mantém a coleta real de CPU/RAM em segundo plano, como na base. Para começar sem consultar contadores, use `--demo`. A janela mostra DEMO ou SIMULADO em todos os modos de HUD.

Use Espaço, M, P, O, F1, R, Esc e Ctrl+Q conforme `docs/ux_guia.md`. A ajuda visual é F1; a ajuda de CLI de um executável GUI Windows depende de uma saída padrão conectada/redirecionada.

## Validação, integração e limitações

Os resultados executados, comandos completos e checklist nativo estão em `TESTES.md`. As imagens novas foram produzidas a partir da UI real do egui, executada sem janela nativa, e são identificadas dessa forma. Não são screenshots de Windows.

`INTEGRACAO.md` contém as novas APIs públicas, os pontos de conexão da bandeja e dos painéis de estação/bioma, três riscos concretos e o procedimento de rollback. Esta entrega não pressupõe que qualquer outra frente esteja concluída. O botão de ocultar na bandeja fica indisponível até existir uma interface real de restauração; minimizar pela barra de tarefas continua disponível.
