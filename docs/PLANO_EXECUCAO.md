# CPU Express — plano de execução da expansão

Atualizado em 2026-11-14. Base de referência: **0.3.0-rc.7**. **Plano aprovado; expansão 0.4 integrada. Aceite por item abaixo; a matriz completa 1.0 continua aberta.** A existência de alterações paralelas não substitui revisão, testes e integração. Este é um tracker local; não cria tarefas externas nem agenda automações.

## 1. Como acompanhar

- **BASE RC7:** comportamento existente, com escopo da evidência explicitado no [roadmap](ROADMAP.md).
- **EM ANDAMENTO:** frente aberta para execução; não significa que todos os itens já foram iniciados.
- **PENDENTE:** item ainda sem conclusão comprovada; checkbox desmarcado inclui trabalho parcialmente implementado.
- **BLOQUEADO:** registrar dependência concreta, responsável e próximo passo.
- **ACEITO:** marcar `[x]` somente depois de implementação, validação pertinente e evidência vinculada à fonte/binário. Registrar data e integrador.

Cada entrega deve atualizar: ID, responsável, estado, arquivos, dependências, teste executado e resultado, artefato/hash quando aplicável, riscos e aceite. Não preencher nomes ou resultados por inferência. Novos schemas e decisões de balanceamento precisam de registro explícito antes de integrar consumidores.

## 2. Referência verificada, sem herança automática

[BUILD-INFO rc7](historico/BUILD-INFO_RC7.json) identifica release Windows GNU, sem assinatura, SHA-256 `a1febb5706989ab5578496261df37cd8e817e8f133fb1aeac8772706b1e842f6`. Registra 203 testes aprovados, zero falhas e dois ignorados; format e Clippy com saída zero. Isso é evidência existente, não execução feita ao redigir este plano.

Há [smoke nativo](../evidence/current/native-smoke-result.json), [transições](../evidence/current/transitions-rc7/result.json), [métricas reais](../evidence/current/real-metrics-result.json), [clima online](../evidence/current/weather/result.json) e [painéis egui](../evidence/current/game-ui-result.json). Painéis são exportações headless. O smoke não capturou o painel (`panel_capture=false`). A comparação curta de cadência mede frames renderizados, não compositor nem energia. A matriz de hardware e longa duração continuam abertas.

A rc7 já tem save local protegido, circuito de quatro estações, paradas, visitas, objetivos e pinturas, cinco aparências de vagão, clima local e oficina. Os passageiros visuais não são ainda um inventário de pessoas; não há economia de carga/contratos inferida dessas animações. Não reimplementar esses fundamentos como se fossem ausentes.

## 3. Contratos permanentes

- CPU influencia velocidade visual; progresso de jornada e recompensas usam tempo ativo, sem incentivo a gerar carga.
- Pausa/ocultação/minimização não acumulam progresso offline. Rede e relógio civil não concedem avanço lógico.
- Métricas e clima ficam fora do caminho de renderização; UI distingue dado atual, antigo, indisponível e simulado.
- Renderer determinístico, grade 320 × 70, nearest, sem alocação por render; preservar rodas, engates, baseline e sockets.
- Plataforma tem um proprietário de visibilidade/topo. Preservar a aplicação de bordas antes de restaurar geometria rc4.
- Checkpoint separado das preferências, versão explícita, escrita protegida e escritor exclusivo; arquivos futuros/inválidos não são sobrescritos silenciosamente.
- Conteúdo autoral ou licenciado. Nenhum sprite, som, mapa ou texto extraído do APK entra no produto.
- O produto continua local e utilizável sem conta ou rede. Novos efeitos são discretos e desligáveis; nenhuma compra/moeda fictícia na UI.

## 4. Sequência, ownership e gates

| Frente | Estado da expansão | Dependência principal | Responsabilidade de integração |
| --- | --- | --- | --- |
| DES/UI/ACC | EM ANDAMENTO, aceite pendente | Base rc7 | Plataforma e UI combinam contratos de foco/geometria |
| SAV/DATA | EM ANDAMENTO, aceite pendente | Inventário do schema atual | Integrador controla formato e migração |
| PAS/WAG/CAR/CON | EM ANDAMENTO, aceite pendente | SAV/DATA antes de persistir estado novo | Domínio fornece estado; UI/renderer apenas o apresentam |
| REW/STA/BRN | EM ANDAMENTO, aceite pendente | Serviço mínimo integrado; catálogo versionado | Integrador decide compatibilidade da rota |
| ART/WEA/AUD | EM ANDAMENTO, aceite pendente | Contratos de eventos/apresentação | Módulos independentes; não alterar regras de viagem |
| PERF/MOD/MAI/DIST | EM ANDAMENTO, aceite pendente | Medição desde o início; release após gates | Integrador executa suíte/release e identifica pacote |

Responsáveis nominais: atribuir por tarefa na integração. Um único integrador controla `app.rs`, `config.rs`, `main.rs`, Cargo e formatos persistidos. Respeitar ownership de agentes, ler estado atual antes de editar, nunca substituir árvores completas ou reverter mudanças alheias.

Gates: **G0** estabilidade e contratos preservados; **G1** save migra; **G2** um serviço completo; **G3** progressão e primeira ramificação; **G4** apresentação/acessibilidade/performance; **G5** pacote e aceite Windows. Frentes independentes podem avançar em paralelo, mas não atravessar dependências de schema sem acordo.

## 5. DES — estabilidade do desktop

Base: faixa independente, painel separado e correções rc4 já existem. Expansão: endurecer recuperação e completar cobertura.

- [x] DES-01 Reproduzir alternância barra/flutuante com central aberta e fechada, preservando retângulos independentes.
- [x] DES-02 Cobrir transição de bordas, eventos atrasados de resize e restauração; não gravar geometria intermediária.
- [ ] DES-03 Testar minimizar, ocultar, restaurar e sair em todos os estados; impedir instância inacessível.
- [ ] DES-04 Validar reinício do Explorer, falha de bandeja, alteração de resolução e desconexão de monitor.
- [ ] DES-05 Completar matriz 100/125/150/200%, coordenadas negativas, DPI misto, autohide e widgets.
- [x] DES-06 Documentar fallback de barra não suportada, sem prometer encaixe vertical/secundário.

**Aceite:** G01–G04/T01–T10/W01–W08 do [checklist Windows](windows_checklist.md) com resultado por ambiente, hash e limitações; zero perda de acesso/dados e nenhuma geometria transitória persistida. Ambientes não disponíveis permanecem pendentes.

## 6. UI — experiência compacta e menus

Base: caderno rc7, oficina por cartões e painel 760 × 720 pontos com mínimo 480 × 480. Não restaurar controles de clássico/reduzido retirados do produto.

- [x] UI-01 Garantir leitura do trem na faixa física de 60 px, com céu recortável e trilho/rodas preservados.
- [ ] UI-02 Verificar central mínima e escalada: navegação, mensagens, escolhas e fechamento acessíveis sem expansão involuntária.
- [x] UI-03 Preservar fallback compacto somente no backend com viewports embutidos; CPU manual deve continuar editável.
- [x] UI-04 Manter scroll por página, foco visível e `scroll_to_me` quando Tab alcança item fora da área.
- [ ] UI-05 Conferir menus/popups: cancelar, selecionar, trocar abas e fechar sem ação acidental.
- [x] UI-06 Distinguir CPU/RAM atuais, stale e indisponíveis; mostrar simulação e pausa sem depender apenas de cor.
- [ ] UI-07 Fazer cada nova ação refletir o domínio: equipamento, contrato, carga, prêmio e erro com confirmação discreta.

**Aceite:** exportar páginas normal/mínima/rolada e estados vazio/erro/bloqueado; executar navegação por teclado e mouse no nativo. Renders headless complementam, não substituem, teste de foco/compositor. Nenhum controle essencial escondido pelo rodapé.

## 7. SAV — persistência e migração

Base: já existe checkpoint versionado, escritor exclusivo e proteção de arquivos inválidos/futuros. Backup e migração ampliada precisam de aceite próprio.

- [x] SAV-01 Inventariar campos rc7 e invariantes: rota, seed, distância, tempo ativo, serviço, preferências e desbloqueios derivados.
- [ ] SAV-02 Definir schema novo e versão de catálogo com o integrador antes de persistir passageiros/cargas/contratos.
- [ ] SAV-03 Criar fixtures rc7: perfil novo, trecho intermediário, parada, recompensas liberadas e preferências antigas.
- [x] SAV-04 Implementar migração explícita; definir defaults de novos campos sem inventar serviço já realizado.
- [x] SAV-05 Validar limites, referências e relações entre campos; rejeitar NaN, valores impossíveis, IDs ausentes e versões futuras.
- [ ] SAV-06 Definir backup/recuperação e falha entre gravação temporária e substituição; nunca destruir o único progresso válido.
- [ ] SAV-07 Testar writer concorrente, falha de permissão/espaço, arquivo truncado, reinício abrupto e retry limitado.
- [x] SAV-08 Definir rollback de versão: preservar arquivo novo incompatível; não fazer downgrade silencioso.

**Aceite:** migrar fixture e reabrir duas vezes produz mesmo domínio; visita/prêmio/entrega não duplica; arquivos rejeitados mantêm hash; operação normal recupera o último checkpoint válido. DEMO/smoke permanecem isolados. Migração aprovada é pré-requisito de G1.

## 8. DATA — conteúdo e catálogos

- [ ] DATA-01 Definir IDs estáveis para estações, trechos, tipos de vagão, cargas, contratos e recompensas.
- [ ] DATA-02 Separar definição de conteúdo de estado da partida e texto de interface.
- [ ] DATA-03 Validar unicidade, referências, capacidade, conectividade e caminhos de retorno.
- [ ] DATA-04 Versionar catálogo; definir tratamento de item removido, renomeado ou desativado em save existente.
- [ ] DATA-05 Tornar erros legíveis, indicando campo/ID; manter conteúdo inválido fora da sessão ativa.
- [ ] DATA-06 Começar com conjunto pequeno autoral; não adicionar editor/modding público antes de estabilizar contrato.

**Aceite:** adicionar um serviço/estação pelo catálogo sem reescrever o algoritmo de viagem; fixtures inválidas falham de modo explicável; seed e conteúdo iguais preservam determinismo.

## 9. PAS — passageiros com estado real

- [x] PAS-01 Definir grupos com origem/destino, quantidade e estado; evitar dados pessoais e simulação individual desnecessária.
- [x] PAS-02 Modelar demanda por estação e embarque/desembarque em evento de serviço, com capacidade limitada.
- [x] PAS-03 Garantir conservação: embarcados − desembarcados = a bordo; contagens nunca negativas.
- [x] PAS-04 Definir comportamento quando não há vaga, destino está fora da rota ou composição muda.
- [x] PAS-05 Expor estado na cabine/estação; animação de plataforma deve acompanhar fatos, sem inventar receita.
- [x] PAS-06 Salvar ocupação/destinos e retomar durante serviço sem repetir embarque.

**Aceite:** um grupo embarca em A, permanece a bordo nos trechos corretos e desembarca em B uma vez; capacidade, pausa, mudança de vagão e restart cobertos. Demanda não aumenta por CPU alta.

## 10. WAG — papéis dos vagões

- [x] WAG-01 Definir papéis explícitos para Passageiros, Dormitório, Carga, Tanque e Panorâmico; valores de balanceamento ainda a decidir.
- [x] WAG-02 Separar modelo/papel, posição equipada, capacidade e pintura; não usar cor do sprite como regra.
- [ ] WAG-03 Definir compatibilidade de passageiro/carga e benefício de cada escolha sem bloquear o perfil inicial.
- [x] WAG-04 Tratar troca de vagão ocupado: recusar com motivo ou adiar para estação; não apagar ocupantes/carga.
- [ ] WAG-05 Mostrar capacidade/uso/efeito antes de equipar e feedback após equipar.
- [x] WAG-06 Migrar composição rc7 para estado válido sem cobrar recursos inexistentes.

**Aceite:** cinco tipos diferenciados por regra documentada; troca inválida é explicada; slot correto persiste; geometria visual não muda; jogador inicial consegue completar o serviço mínimo.

## 11. CAR — cargas

- [x] CAR-01 Implementar primeiro um tipo de carga geral e um fluxo origem → destino; ampliar depois para líquidos compatíveis com tanque.
- [x] CAR-02 Definir unidade, lote, capacidade, categoria e estação de entrega com limites explícitos.
- [x] CAR-03 Reservar espaço antes de carregar; impedir duplicação, sobrecarga e quantidade negativa.
- [ ] CAR-04 Separar carga disponível, reservada, transportada e entregue; estados não dependem de renderização.
- [ ] CAR-05 Definir cancelamento/troca de composição e carga incompatível após migração.
- [x] CAR-06 Exibir manifesto compacto e persistir lotes/estado de entrega.

**Aceite:** ciclo completo conserva quantidades e capacidade; entrega repetida ou restart na parada não duplica recompensa; sem expiração por tempo offline.

## 12. CON — contratos de serviço

- [ ] CON-01 Definir oferta, aceito, em execução, concluído e cancelado, com transições válidas e ID estável.
- [ ] CON-02 Começar com um contrato de passageiros e um de carga; condições devem corresponder a ações existentes.
- [ ] CON-03 Validar rota, capacidade e pré-requisitos antes de aceitar, com motivo de bloqueio legível.
- [x] CON-04 Limitar contratos simultâneos e definir reserva de capacidade/conflito entre contratos.
- [x] CON-05 Adotar acompanhamento tranquilo; se houver prazo futuro, usar tempo ativo e decisão explícita, nunca relógio offline punitivo.
- [x] CON-06 Tornar entrega idempotente e ligar recompensa ao fato concluído, não à abertura de painel.
- [ ] CON-07 Definir cancelamento sem softlock e recuperação de contratos cujo catálogo mudou.

**Aceite:** aceitar → carregar/embarcar → viajar → entregar → receber → salvar/reabrir funciona ponta a ponta; eventos duplicados não premiam duas vezes; falhas recuperáveis não apagam a partida. G2 só passa com esse fluxo integrado.

## 13. REW — objetivos e recompensas

- [x] REW-01 Preservar pinturas de 1/4/12 chegadas e disponibilidade dos perfis rc7.
- [x] REW-02 Definir recompensas novas por serviço/descoberta e catálogo de desbloqueios; evitar moedas sem função implementada.
- [x] REW-03 Escolher mecanismo de concessão única persistente e validar IDs recebidos.
- [x] REW-04 Mostrar requisito, progresso, desbloqueado e equipado sem confusão entre obter e usar.
- [x] REW-05 Enviar ação de recompensa à coleção correta e manter feedback opcional/discreto.
- [ ] REW-06 Balancear caminhos acessíveis em CPU ociosa, 30/60 FPS, pausa e sessões curtas.

**Aceite:** mesma jornada lógica concede mesmo progresso em diferentes cargas/FPS; reabertura não remove desbloqueio nem concede duplicata; UI não anuncia item que o domínio não permite usar.

## 14. STA — estações e serviços

- [ ] STA-01 Dar identidade às quatro estações por serviços/demanda e apresentação, além da arquitetura existente.
- [x] STA-02 Centralizar chegada, início/fim de serviço e partida; efeitos de UI/áudio consomem eventos sem comandar domínio.
- [x] STA-03 Definir ações automáticas e opcionais; funcionamento passivo não depende de clique obrigatório a cada parada.
- [x] STA-04 Relacionar plataforma, passageiros, carga e indicadores ao estado real de serviço.
- [x] STA-05 Cobrir pausa/save no meio da parada e reabertura de painel sem reaplicar ações.
- [ ] STA-06 Adicionar descoberta contextual e mensagens limitadas, sem flood de notificações.

**Aceite:** uma estação implementa serviço completo e recuperável antes de expandir todas; fila e duração são previsíveis; nenhuma parada prende o jogador indefinidamente.

## 15. BRN — primeira ramificação

- [x] BRN-01 Desenhar grafo autoral pequeno com desvio e retorno; preservar Circuito Aurora como caminho válido.
- [x] BRN-02 Definir momento de escolha, alternativa padrão e efeito sobre passageiros/contratos já aceitos.
- [x] BRN-03 Persistir trecho/nó/decisão com versão da rota; migrar posição rc7 sem reiniciar partida silenciosamente.
- [x] BRN-04 Validar conectividade, destinos alcançáveis e ausência de ciclos sem saída.
- [ ] BRN-05 Adaptar mapa, ETA, estação seguinte e transição de bioma a partir da rota real.
- [x] BRN-06 Testar escolha, pausa, save/restart e percurso de retorno em cada ramo.

**Aceite:** jogador entende a escolha antes de confirmá-la; contratos permanecem possíveis ou recebem resolução explícita; determinismo e continuidade do cenário preservados. Não anunciar mapa aberto antes de G3.

## 16. ART — arte e animação

- [ ] ART-01 Manter galeria comparativa antes/depois gerada pelo renderer real, em 1× e escalas inteiras.
- [ ] ART-02 Priorizar leitura de locomotiva/cinco vagões a 60 px, contraste de material e silhueta.
- [ ] ART-03 Adicionar sinais de ocupação/carga sem ruído excessivo e somente quando sustentados pelo domínio.
- [ ] ART-04 Expandir estações e pontos de interesse por bioma, respeitando parallax e períodos de wrap.
- [x] ART-05 Validar novas locomotivas por sockets de chaminé/rodas/bielas/engates; sem deslocar baseline.
- [ ] ART-06 Conferir dia/noite, clima, pausa, transições e todos os equipamentos; registrar autoria/licença.

**Aceite:** renderer continua determinístico e sem alocação por frame; nenhuma borda de asset fora do footprint contratado; leitura confirmada no desktop, além de exportação headless. Miniaturas não são recriadas por frame.

## 17. WEA — clima e atmosfera

Base: Open-Meteo para São Paulo, worker, refresh periódico e estados de disponibilidade já existem. Não tratar consulta online rc7 como tarefa ainda ausente.

- [x] WEA-01 Fortalecer estados inicial/ready/stale/erro e transição ilustrativo → dado real sem bloquear UI.
- [x] WEA-02 Testar timeout, resposta inválida, rede intermitente e atualizações manuais repetidas com worker limitado.
- [x] WEA-03 Verificar dia/noite e fuso sem misturar relógio civil com avanço lógico.
- [x] WEA-04 Preservar modo offline/DEMO e fixtures determinísticas sem consulta externa por padrão.
- [ ] WEA-05 Revisar contraste de neblina/tempestade/neve e efeito de reduzir movimento/intensidade.
- [x] WEA-06 Se ampliar seleção de cidade, definir UX, coordenadas explícitas, privacidade e migração antes de ativar; não inferir localização do dispositivo.
- [ ] WEA-07 Conferir atribuição e condições do provedor para a distribuição pretendida; não presumir autorização comercial.

**Aceite:** viagem funciona sem rede; stale não aparece atual; falhas não acumulam threads/requests; todos os climas têm fixtures legíveis. Licença/condições devem ser verificadas na preparação da distribuição, não presumidas a partir deste tracker.

## 18. AUD — som discreto opcional

- [x] AUD-01 Definir sons curtos de chegada, recompensa e equipamento, autorais/procedurais ou com licença comprovada.
- [x] AUD-02 Criar API de cues desacoplada da UI; emitir por evento, nunca por render/reabertura de página.
- [x] AUD-03 Oferecer mute e volume do aplicativo, com modo silencioso respeitado; não modificar volume global do Windows.
- [x] AUD-04 Manter reprodução fora da thread da UI, fila limitada, política de descarte e encerramento não bloqueante.
- [x] AUD-05 Evitar sobreposição excessiva e repetição em retomada/replay; não reproduzir backlog antigo.
- [ ] AUD-06 Validar amplitude, duração, volume zero, falha do dispositivo e estabilidade de memória durante reprodução assíncrona.

**Aceite:** silêncio por padrão conforme decisão do integrador; volume zero não inicia reprodução; mutar não altera jornada; interface permanece responsiva; nenhum asset externo sem origem. Testes de bytes/worker não certificam audibilidade, volume percebido ou dispositivo real.

## 19. ACC — acessibilidade

- [ ] ACC-01 Cobrir navegação por teclado, ordem de foco, foco visível e scroll automático em todas as páginas.
- [x] ACC-02 Fornecer rótulos e estado textual para ícones, cartões equipados/bloqueados, métricas e ações.
- [ ] ACC-03 Conferir contraste e identificação sem depender exclusivamente de cor, incluindo noite/clima.
- [ ] ACC-04 Validar escalas e texto maior sem ocultar ações essenciais; definir tamanho de alvo adequado no painel.
- [x] ACC-05 Criar preferência de menor movimento/intensidade sem remover informação necessária nem mudar progressão.
- [ ] ACC-06 Garantir que todo feedback sonoro tenha equivalente visual; testar mute e interação com leitor de tela quando suportado.

**Aceite:** fluxo equipar/pausar/aceitar serviço/salvar/sair realizável por teclado; avisos compreensíveis sem som/cor. Suporte assistivo só é anunciado após teste no ambiente correspondente.

## 20. PERF — desempenho e cadência

- [ ] PERF-01 Registrar baseline rc7 e nova entrega no mesmo hardware, energia, janela, perfil e instrumentação.
- [ ] PERF-02 Medir 30/60 FPS, painel fechado/aberto, todos os biomas/climas, pausa/oculto/minimizado.
- [x] PERF-03 Preservar deadlines: atrasos saltam frames, redraw antecipado fora da tolerância não adia frame agendado.
- [x] PERF-04 Verificar renderer sem alocação e catálogo sem criação contínua de texturas; medir UI separadamente.
- [ ] PERF-05 Limitar filas, eventos, diário, áudio e workers; testar falhas prolongadas sem crescimento ilimitado.
- [ ] PERF-06 Acompanhar CPU do processo, working set/memória privada, handles/threads e frames; definir orçamento antes de aceitar regressões.
- [ ] PERF-07 Executar sessão prolongada com ciclos de serviço/rede/menus e registrar tendência após aquecimento.

**Aceite:** comparação reproduzível com duração e limites; sem regressão não explicada ou trabalho contínuo oculto. FPS renderizado não é FPS do compositor; não prometer ganho de bateria sem medição apropriada. Usar [método de medição](performance_windows.md), não resultados históricos dele como atuais.

## 21. MOD — modularização sem reescrita

- [x] MOD-01 Mapear responsabilidades atuais de app, UI, domínio, persistência, renderer, metrics/weather e platform.
- [x] MOD-02 Extrair uma responsabilidade por mudança, preservando comportamento antes de adicionar recurso.
- [x] MOD-03 Manter UI como apresentação/comandos, sem transições de domínio escondidas em desenho de widget.
- [ ] MOD-04 Centralizar IDs/catálogos e eventos públicos mínimos, sem dependências circulares ou tipos de UI no domínio.
- [x] MOD-05 Evitar I/O/FFI no renderer e duplicação de proprietários de relógio/janela/estado persistido.
- [x] MOD-06 Atualizar consumidores e testes de contrato junto de qualquer alteração de API.

**Aceite:** cada extração tem escopo pequeno, revisão e regressões pertinentes; sem mudança visual/lógica acidental; não usar modularização para substituir trabalho paralelo.

## 22. MAI — manutenção e observabilidade

- [ ] MAI-01 Versionar diagnóstico quando mudar formato; registrar modo, frescor, pausa, jornada e versão sem dados sensíveis desnecessários.
- [x] MAI-02 Manter fixtures de migração e catálogo, testes de determinismo/conservação/idempotência e falhas reais.
- [x] MAI-03 Distinguir teste unitário, integração headless, execução nativa e aceite humano nos relatórios.
- [ ] MAI-04 Registrar bugs por reprodução/impacto/versão, sem converter hipótese em falha confirmada.
- [ ] MAI-05 Revisar dependências/licenças e toolchain com mudanças delimitadas; preservar lockfile e builds reproduzíveis.
- [ ] MAI-06 Atualizar este tracker por entrega com evidência e pendências; documentos históricos permanecem históricos.

**Aceite:** terceiro consegue repetir um defeito e vincular teste/artefato ao binário correto; nenhum gate verde derivado apenas de documento antigo; testes não são adicionados só para aumentar contagem.

## 23. DIST — distribuição e atualização

- [x] DIST-01 Integrador executa formatação, Clippy sem warnings, suíte pertinente/completa e release final após integração.
- [x] DIST-02 Gerar pacote portátil único, BUILD-INFO, hashes, licenças e notas coerentes com o executável entregue.
- [ ] DIST-03 Testar extração/execução em usuário comum e pasta com espaços, sem depender da toolchain de desenvolvimento.
- [ ] DIST-04 Ensaiar atualização preservando preferências/jornada; documentar backup e rollback compatível.
- [ ] DIST-05 Preencher aceite Windows por ambiente e limitações antes de anunciar estabilidade ampla.
- [ ] DIST-06 Avaliar assinatura e eventual build MSVC com procedimento verificável; registrar explicitamente se pacote segue não assinado.
- [ ] DIST-07 Projetar instalador/inicialização com Windows/atualização automática apenas com opt-in, remoção e recuperação claros; não anunciá-los antes de existir.
- [ ] DIST-08 Tratar nuvem como frente futura separada: consentimento, autenticação, conflitos de saves, privacidade e modo offline antes de qualquer promessa.
- [ ] DIST-09 Tratar Linux/outros shells/variantes de barra como frentes próprias, não efeito implícito da compilação.

**Aceite:** pacote funciona fora da árvore de desenvolvimento, hash corresponde aos relatórios, atualização não perde dados e limitações estão publicadas. Assinatura, nuvem, matriz completa e instalador continuam pendentes até evidência específica. G5 não pode ser fechado por screenshots ou build isolado.

## 24. Primeira fatia integrada e critério de encerramento

Primeira fatia proposta: perfil rc7 migrado → composição válida → contrato simples em estação existente → embarque/carga → viagem ativa → entrega única → recompensa visível → salvar e reabrir. Só depois ampliar quantidade de conteúdo e ramificação.

- [x] INT-01 Definir responsáveis e contratos da fatia antes de integrar.
- [x] INT-02 Demonstrar caminho feliz e capacidade insuficiente sem atenção contínua obrigatória.
- [x] INT-03 Demonstrar pausa, saída/reabertura na parada e evento duplicado.
- [x] INT-04 Conferir UI compacta/nativa, arte real, som desligado e operação sem rede.
- [x] INT-05 Registrar testes, comparação visual, riscos e resultado do integrador.
- [ ] INT-06 Liberar próximas etapas somente com G1/G2 comprovados.

Modelo de atualização por item:

| Campo | Preencher |
| --- | --- |
| ID / responsável / data | PENDENTE |
| Estado e dependências | PENDENTE |
| Comportamento entregue e arquivos | PENDENTE |
| Verificação executada / comando / resultado | PENDENTE |
| Evidência / hash / ambiente | PENDENTE |
| Riscos, não executado e próximo passo | PENDENTE |
| Aceite do integrador | PENDENTE |

Não marcar a expansão concluída porque o plano foi escrito, um módulo compila ou uma galeria foi exportada. Conclusão exige comportamento integrado, critérios de aceite atendidos e limitações honestamente registradas.

## Integração 0.4 em 2026-11-14

Integrador: Codex; seis frentes paralelas com ownership separado. Itens marcados acima correspondem ao código integrado e à suíte atual, não a certificação de outros computadores. Foram executados 272 testes Rust (zero falhas, dois exportadores ignorados na suíte normal), formatação e Clippy sem warnings. Os exportadores foram executados separadamente com 16 cenários, incluindo serviços preenchidos, diário e excursão. Resultado nativo final: consultar [VALIDACAO_04.md](VALIDACAO_04.md) e [BUILD-INFO atual](../BUILD-INFO.json).

A ramificação foi implementada como excursão limitada, persistida e com retorno a Aurora, preservando aurora-v1. Não há grafo arbitrário de rotas nem transporte com destino aos novos ramais nesta edição. Contratos de carga são automáticos, um por vez, com três categorias; oferta manual, cancelamento e contratos especiais de passageiros permanecem abertos. Os pontos são um registro de serviço, não moeda. Dormitório e Panorâmico diferem em capacidade e desenho; sistemas de conforto/turismo não são simulados.

A revisão cruzada encontrou e corrigiu: recriação indevida da chave Run no instalador opcional, perda potencial da sessão ainda não salva durante importação, erros escondidos após o histórico, miniaturas pequenas, inspeção silenciosa de itens bloqueados e texto que prometia histórico ilimitado. A importação preserva agora tanto a sessão atual quanto o arquivo anterior; há teste de regressão com destino corrompido. O instalador não foi aplicado ao perfil do usuário; seu bloco de autostart foi testado em fixture isolada.

Hardware adicional, leitor de tela real, reprodução audível/dispositivo, reinício real do Explorer e sessão de um dia completo continuam sem aceite. Nuvem, outros sistemas, assinatura e atualização automática não estão implementados. Nenhum desses itens deve ser marcado como concluído por inferência.

Validação nativa final: quatro alternâncias passaram, clima online com 25 amostras prontas, migração de cópia do perfil v1 preservou quatro pinturas e backup, dois cenários de 120 s encerraram normalmente. Métricas e limites em VALIDACAO_04. A remoção recursiva de caches foi bloqueada pela revisão automática; os caches foram preservados e continuam excluídos dos pacotes.

Distribuição final: portátil extraído e executado em pasta com espaços, saída zero e CRC dos três arquivos ZIP verificado. DIST-03 permanece parcialmente aberto porque não foi criada uma conta Windows separada.
