# CPU Express — orientação para próximas contribuições

Esta árvore é a base integrada **0.4.0-rc.2**. Leia `README.md` e `docs/GUIA_04.md` antes de alterar o projeto. Patches e evidências da rc.1 são históricos.

## Contratos

- Coleta pertence a `metrics`; não consultar `sysinfo` no renderer nem na UI por frame.
- Movimento e jornada usam o mesmo tempo ativo. O produto usa `World::advance_expedition`: progresso lógico independente da CPU, com paradas de serviço. `advance_train` conserva o adaptador matemático legado.
- A rota decide o ambiente; `SceneContext::from_journey` apenas o adapta ao renderer.
- A UI revalida a amostra em cache e permite métricas parciais válidas.
- Win32 e bandeja ficam em `platform`; `DesktopController` é o único proprietário das ações de visibilidade/topo de uma janela nativa.
- Pausa, ocultação e minimização não acumulam progresso offline.
- Preferências têm defaults e gravação protegida com retry limitado. `progress.rs` salva checkpoints versionados separados da configuração e protege o perfil contra dois escritores Windows. Arquivos incompatíveis não podem ser sobrescritos.
- Arte procedural original, resolução interna 320 × 70 e filtragem nearest. Não copiar recursos do APK de referência.

## Trabalho paralelo

Defina um integrador para `app.rs`, `config.rs`, `main.rs`, Cargo e novos formatos persistidos. As demais frentes podem alterar seus módulos, com mudanças de contrato comunicadas aos consumidores. Não substitua uma árvore completa por uma entrega antiga. Mantenha exemplos/exportadores compatíveis com os limites do domínio.

## Verificação e entrega

Execute formatação, Clippy sem warnings e testes pertinentes. A suíte completa e a compilação Windows são gates para uma distribuição. Só descreva um teste como executado quando houver resultado desta fonte. Compilação cruzada e imagens headless não equivalem a execução em Windows real.

Mantenha `BUILD-INFO.json`, manifesto e documentação coerentes com o binário distribuído. Não reutilize benchmarks, hashes ou screenshots de outro executável como evidência atual. Preserve histórico separado e entregue uma única base atualizada.


RC2: a faixa usa uma janela independente no retangulo da barra, na mesma altura e alinhada a esquerda. O encaixe como filha do Explorer foi removido porque uma janela nativa visivel e um framebuffer valido nao garantiam exibicao pelo compositor. A superficie ocupa essa area sem reservar espaco para widgets. A confirmacao visual no desktop e separada dos testes de geometria.

## Serviços e excursões

- ServiceState pertence ao World e aplica transações na chegada, antes das notificações. Não conceder recompensas na UI.
- Preservar aurora-v1. Excursões são estado separado; não reinterpretar distância acumulada.
- Checkpoint v3 inclui jornada, serviço, despacho e devolução (migra v1/v2 com backup); o aplicativo restaura com load_world. O load legado não restaura serviços.
- Não clonar Config por frame: preferências incluem strings da localização.
- Os módulos src/app/ dividem jornada, oficina, serviços, excursões e recuperação; manter cada responsabilidade localizada.
