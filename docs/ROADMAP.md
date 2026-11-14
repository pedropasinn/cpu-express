# CPU Express — roadmap a partir da 0.3.0-rc.7

Atualizado em 2026-11-14. **Expansão 0.4 integrada; aceite técnico por item em PLANO_EXECUCAO. A matriz 1.0 permanece aberta.** Este documento orienta prioridades. O acompanhamento por entrega está em [PLANO_EXECUCAO.md](PLANO_EXECUCAO.md); a aceitação no desktop está em [windows_checklist.md](windows_checklist.md).

## Entrega atual

A 0.4 integra passageiros, carga tipada, capacidade de composição, postais, diesel/elétrica, excursões, importação/exportação, cache de clima e iluminação solar, sons opcionais e recuperação de menus. Guia: [GUIA_04.md](GUIA_04.md). Evidência executada: [VALIDACAO_04.md](VALIDACAO_04.md). As referências rc7 abaixo são históricas.

## Base estabelecida e limites da evidência

A referência integrada é a **0.3.0-rc.7**, conforme [README](../README.md), [AGENTS](../AGENTS.md), [funcionamento](VERSAO_03.md) e [BUILD-INFO](historico/BUILD-INFO_RC7.json). Não partir da 0.2.0-rc.1 nem reaplicar patches antigos. Alterações paralelas posteriores à rc7 não herdam automaticamente seu aceite.

| Área | Já existe na rc7 | Evidência e limite |
| --- | --- | --- |
| Jornada | Circuito de quatro estações, paradas, diário e objetivos; progresso lógico independente de CPU/FPS | Contratos descritos em VERSAO_03; suíte registrada em [tests.log](../evidence/current/tests.log). Passageiros desenhados ainda não representam saldo ou economia. |
| Persistência | Checkpoint separado e versionado, escritor exclusivo, proteção contra arquivos incompatíveis, retomada local | Não é tarefa futura criar o primeiro save. Expandir e migrar o schema é trabalho novo; não há progresso offline ou sincronização em nuvem. |
| Desktop | Faixa independente na área da barra, painel nativo separado, ajustes de posição e tamanho; estabilização rc4 | [Transições rc7](../evidence/current/transitions-rc7/result.json): cinco estágios barra/flutuante/barra/flutuante/barra no ambiente registrado. Não é cobertura de todas as configurações Windows. |
| Arte e fluidez | Arte procedural refinada, cinco vagões, quatro pinturas, 320 × 70 nearest; 60 FPS padrão para perfis novos | [Comparação](../evidence/current/visual-refresh/comparison.json): cerca de 52,87 → 59,81 frames renderizados/s em execuções curtas a 60 FPS. Não mede apresentação do compositor, custo de CPU/GPU ou bateria. |
| Interface rc7 | Caderno ferroviário, cabine, mapa/objetivos/diário, oficina com cartões e miniaturas do renderer | [Manifesto visual](../evidence/current/game-ui-result.json) e [painéis](../evidence/current/panels-rc7): dez exportações egui, não capturas do compositor. |
| Clima rc6/rc7 | São Paulo configurada, Open-Meteo fora da UI, indicação de dados antigos/falha/estado ilustrativo | [Consulta rc7](../evidence/current/weather/result.json) registra resposta online. Não certifica todas as condições meteorológicas ou falhas de rede reais. |
| Qualidade e pacote | BUILD-INFO registra 203 testes aprovados, zero falhas, dois ignorados; format e Clippy com saída zero; release Windows GNU | Resultados da entrega registrada, não execução nova nesta atualização documental. Binário sem assinatura. Matriz de hardware, longa duração e distribuição ampliada pendentes. |

O hash da referência rc7 é `a1febb5706989ab5578496261df37cd8e817e8f133fb1aeac8772706b1e842f6`. Resultados de outro hash devem ser registrados separadamente. Logs nativos confirmam comportamento no ambiente ensaiado; não substituem aceitação visual e interação no computador do usuário.

## Sequência de implementação

| Etapa | Entrega prática | Dependências | Gate de saída |
| --- | --- | --- | --- |
| 1. Estabilidade e experiência compacta | Preservar recuperação de janela, geometria rc4, pausa, foco, menus e legibilidade em faixa de 60 px | Base rc7 | Casos prioritários do checklist Windows e regressões específicas aprovados; nenhuma janela inacessível ou perda de dados |
| 2. Persistência extensível | Migração explícita de saves, catálogos com IDs estáveis e recuperação verificável | Etapa 1; coordenação do integrador | Save rc7 migra sem duplicar visitas/recompensas; versões futuras permanecem intactas |
| 3. Serviço ferroviário mínimo | Passageiros reais, papéis de vagões, uma carga e um contrato completos | Etapa 2 | Ciclo aceitar → embarcar/carregar → transportar → entregar → salvar/reabrir funciona e é idempotente |
| 4. Progressão e lugares | Recompensas, serviços e identidade das estações; primeira ramificação pequena | Etapa 3; versão do catálogo/rota | Escolhas têm consequências claras, sem exigir CPU alta, atenção contínua ou avanço offline |
| 5. Apresentação e acesso | Arte adicional, clima robusto, som opcional e acessibilidade | Contratos de serviço e estado estabilizados | Toda informação tem representação legível; efeitos não alteram regras; som pode ser desligado |
| 6. Desempenho, manutenção e entrega | Modularização, medição comparável, pacote verificável e procedimentos de atualização | Gates anteriores; medições contínuas desde etapa 1 | Evidências vinculadas ao binário, limitações publicadas e dados preservados durante atualização/rollback |

Trabalho independente pode ocorrer em paralelo; dependências de schema, rota e contratos são gates de integração, não motivo para reescrever a aplicação inteira. Cada etapa deve produzir uma fatia utilizável antes de multiplicar conteúdo.

## Direção do produto

- O trem continua sendo um companheiro de desktop tranquilo. Não premiar carga de CPU, punir ausência nem fabricar urgência com relógio de parede.
- Vagões passam de aparência a escolhas compreensíveis de serviço, sem transformar a composição antiga em configuração inválida.
- Passageiros e cargas devem ter origem, destino e capacidade verificáveis; a animação deve refletir o domínio real.
- Contratos e recompensas são locais, com IDs e entrega única. Não apresentar moeda, loja, compra ou benefício ainda inexistente.
- Estações e ramificações começam pequenas, com conteúdo autoral e regras em dados validados. Preservar o circuito antigo por migração explícita.
- Clima externo é apresentação. Não deve bloquear viagem, introduzir localização automática ou decidir recompensas.
- Arte e áudio próprios ou licenciados; não reutilizar assets do APK de referência.

## Organização e aceitação

Um integrador controla `app.rs`, `config.rs`, `main.rs`, Cargo e formatos persistidos. Responsáveis por domínio, renderer, plataforma e documentação devem declarar ownership e combinar mudanças de contrato antes de integrar. Não reverter trabalho alheio nem criar distribuições concorrentes.

A entrega de cada tarefa inclui comportamento antes/depois, arquivos envolvidos, testes pertinentes, evidência visual quando aplicável e limitações. O integrador executa os gates finais de suíte/release. Testes devem demonstrar riscos reais; contagem isolada não prova qualidade.

Assinatura, instalador, inicialização com Windows, atualização automática, sincronização em nuvem, novas plataformas e variantes de barra são **pendentes de implementação/validação e decisões específicas de produto**. A aprovação deste plano não certifica esses recursos como disponíveis. O produto local deve continuar utilizável sem conta, som ou rede.
