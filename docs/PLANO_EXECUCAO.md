# CPU Express — plano de execução da expansão

Atualizado em 2026-11-14. Base de referência: **0.3.0-rc.7**. **Plano aprovado; expansão 0.4 integrada. Aceite por item abaixo; a matriz completa 1.0 continua aberta.** A existência de alterações paralelas não substitui revisão, testes e integração. Este é um tracker local; não cria tarefas externas nem agenda automações.

## 1. Como acompanhar

- **BASE RC7:** comportamento existente, com escopo da evidência explicitado no [roadmap](ROADMAP.md).
- **EM ANDAMENTO:** frente aberta para execução; não significa que todos os itens já foram iniciados.
- **PENDENTE:** item ainda sem conclusão comprovada; checkbox desmarcado inclui trabalho parcialmente implementado.
- **BLOQUEADO:** registrar dependência concreta, responsável e próximo passo.
- **ACEITO:** marcar `[x]` somente depois de implementação, validação pertinente e evidência vinculada à fonte/binário. Registrar data e integrador.

Cada entrega deve atualizar: ID, responsável, estado, arquivos, dependências, teste executado e resultado, artefato/hash quando aplicável, riscos e aceite. Não preencher nomes ou resultados por inferência. Novos schemas e decisões de balanceamento precisam de registro explícito antes de integrar consumidores.

## 2. Referência atual

A base atual é 0.4.0-rc.2. Consulte [BUILD-INFO](../BUILD-INFO.json), [guia](GUIA_04.md) e [validação](VALIDACAO_04.md). Referências a rc7 nos requisitos abaixo indicam a origem histórica de um comportamento, não uma segunda base de desenvolvimento. As cópias antigas de entrega foram removidas da árvore por solicitação do usuário.

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
- [x] SAV-02 Definir schema novo e versão de catálogo com o integrador antes de persistir passageiros/cargas/contratos.
- [x] SAV-03 Criar fixtures rc7: perfil novo, trecho intermediário, parada, recompensas liberadas e preferências antigas.
- [x] SAV-04 Implementar migração explícita; definir defaults de novos campos sem inventar serviço já realizado.
- [x] SAV-05 Validar limites, referências e relações entre campos; rejeitar NaN, valores impossíveis, IDs ausentes e versões futuras.
- [x] SAV-06 Definir backup/recuperação e falha entre gravação temporária e substituição; nunca destruir o único progresso válido.
- [ ] SAV-07 Testar writer concorrente, falha de permissão/espaço, arquivo truncado, reinício abrupto e retry limitado.
- [x] SAV-08 Definir rollback de versão: preservar arquivo novo incompatível; não fazer downgrade silencioso.

**Aceite:** migrar fixture e reabrir duas vezes produz mesmo domínio; visita/prêmio/entrega não duplica; arquivos rejeitados mantêm hash; operação normal recupera o último checkpoint válido. DEMO/smoke permanecem isolados. Migração aprovada é pré-requisito de G1.

## 8. DATA — conteúdo e catálogos

- [ ] DATA-01 Definir IDs estáveis para estações, trechos, tipos de vagão, cargas, contratos e recompensas.
- [ ] DATA-02 Separar definição de conteúdo de estado da partida e texto de interface.
- [x] DATA-03 Validar unicidade, referências, capacidade, conectividade e caminhos de retorno.
- [ ] DATA-04 Versionar catálogo; definir tratamento de item removido, renomeado ou desativado em save existente.
- [x] DATA-05 Tornar erros legíveis, indicando campo/ID; manter conteúdo inválido fora da sessão ativa.
- [x] DATA-06 Começar com conjunto pequeno autoral; não adicionar editor/modding público antes de estabilizar contrato.

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

- [x] CON-01 Definir oferta, aceito, em execução, concluído e cancelado, com transições válidas e ID estável.
- [ ] CON-02 Começar com um contrato de passageiros e um de carga; condições devem corresponder a ações existentes.
- [x] CON-03 Validar rota, capacidade e pré-requisitos antes de aceitar, com motivo de bloqueio legível.
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
- [x] BRN-05 Adaptar mapa, ETA, estação seguinte e transição de bioma a partir da rota real.
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

- [x] MAI-01 Versionar diagnóstico quando mudar formato; registrar modo, frescor, pausa, jornada e versão sem dados sensíveis desnecessários.
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

## Entrega atual 0.4.0-rc.2

Integrador: Codex. Halley: mapa de excursão; Aristotle: prévia da oficina e revisão; Schrodinger: regressões de despacho e migração. Código novo: oferta manual opcional, aceite com capacidade compatível, recusa, cancelamento e devolução física à origem sem recompensa. Checkpoint v3 preserva v1/v2 com backup exato, mantendo o modo automático nos perfis antigos. Oficina compara capacidade por opção/favorito; mapa de excursão apresenta fases e ETA próprios. Não há contrato especial de passageiros, mercado de múltiplas ofertas ou simulação de conforto/turismo nesta entrega.

Validação da continuação: 288 testes aprovados, exportador executado separadamente, smoke e quatro alternâncias passaram, migração v2 → v3 preservou backup e propriedade, clima respondeu e duas execuções de 120 s encerraram normalmente. As evidências rc.1 permanecem no pacote anterior; os resultados atuais identificam o SHA-256 da rc.2. Fluxo nativo de cliques continua sem aceite por indisponibilidade do conector.

Aceite de domínio rc.2: CON-01 e CON-03 exercitados por testes de oferta/aceite/incapacidade/recusa/retorno e reinício. CON-07 tem cancelamento seguro implementado, mas continua aberto para mudanças futuras de catálogo. BRN-05 apresenta mapa e ETA por fase; cenário já usa o ramal. WAG-05 tem prévia de capacidade e feedback de bloqueio, com a interação nativa ainda pendente. Registros detalhados e limites em VALIDACAO_04.md. A indisponibilidade do conector nativo não foi transformada em aprovação da matriz Windows.

Pacote rc.2: CRC dos três ZIPs verificado; portátil extraído em pasta com espaços e executado com perfil isolado e PATH somente do Windows, saída zero. Script reproduzível: scripts/validate-portable.py. O teste não equivale a uma conta Windows nova.

## Continuação pós-rc.2 em 2026-11-14 (fonte; ainda sem binário distribuído)

Responsável: Claude Code, a pedido do Pedro. Escopo: `src/progress.rs`, `src/diagnostics.rs`, chamadas em `src/app.rs` e `src/app/storage.rs`, testes e fixtures. Nenhum formato persistido mudou: o checkpoint continua v3. O executável e os pacotes rc.2 em `release/` **não** contêm estas alterações; BUILD-INFO, SOURCE-MANIFEST e VALIDACAO_04 continuam descrevendo o binário rc.2.

| ID | Entregue | Verificação executada |
| --- | --- | --- |
| SAV-03 | `tests/fixtures/rc7/`: quatro checkpoints v1 sintéticos (perfil novo, trecho intermediário, parada com 5,25 s, 49 chegadas) e preferências rc7. | `tests/save_fixtures.rs`, 5 testes: carga sem serviço inventado e sem tocar o arquivo; migração e duas reaberturas com o mesmo domínio; backup v1 byte a byte; parada migrada não repete a chegada; determinismo com save no meio; preferências antigas recebem defaults e mantêm as chaves. |
| SAV-06 | `progress::save_checkpoint` guarda o documento substituído em `config.journey.previous.json`; `progress::load_profile` isola um checkpoint danificado como `config.journey.damaged-*.json` (bytes intactos) e retoma do anterior, com aviso ao jogador. Saves de versão mais nova, ilegíveis ou maiores que 1 MiB não são movidos. Após importar, a cópia de recuperação é renovada. | `tests/save_recovery.rs`: sete formas de dano (vazio, truncado em três pontos, zeros, `{}`, estado inválido) recuperam o checkpoint anterior; versão futura permanece bloqueando; dano sem cópia válida fica intocado; checkpoint ausente continua sendo perfil novo. |
| SAV-07 (parcial) | Sem código novo além do acima. | Substituição bloqueada por outro processo (compartilhamento somente leitura) e arquivo somente leitura mantêm o checkpoint e não deixam temporário; nova tentativa funciona. Temporário órfão de escritor interrompido é inerte. Quatro escritores em corrida sem o lock nunca deixam documento inválido. `app::tests::failed_checkpoint_is_reported_once_and_retried_only_on_the_next_interval` cobre o retry limitado a 30 s. |
| MAI-01 | Diagnóstico em schema v3 com `app_version`; v3 registra o formato que a 0.4 já gravava (serviços, parada, clima, tempos de frame) sem alterar campos v2. | Testes de `diagnostics.rs` atualizados e `app::tests::diagnostic_record_identifies_version_mode_freshness_pause_and_journey`. |
| PERF-05 (parcial) | Sem código novo. | Teste de 24 h de tempo ativo simulado sem consumir eventos: fila ≤ 64, diário ≤ 32, grupos ≤ 12, fumaça ≤ 48, checkpoint < 16 KiB e apenas dois arquivos no perfil. |

Resultado nesta fonte: formatação e Clippy (`-D warnings`) com saída zero; **306 testes aprovados**, zero falhas, dois exportadores ignorados (288 da rc.2 mais 18 novos). Execução em Windows 11 local com a toolchain de `.tools`.

Não executado, por isso SAV-07 e PERF-05 seguem abertos: disco cheio real, encerramento forçado do processo real durante a gravação, e auditoria nativa de workers (o canal de eventos da bandeja é ilimitado). A recuperação foi exercitada pela API de domínio e pelo teste de app, não por uma execução nativa com perfil danificado. O diagnóstico opt-in continua registrando a cidade e as coordenadas informadas em **Visual → Clima**; não registra conta, caminhos ou identificação da máquina.

Gate G1: os critérios de aceite da frente SAV estão cobertos por testes, exceto SAV-02 (versão de catálogo, depende de DATA) e o restante de SAV-07. INT-06 permanece aberto. Próximo passo sugerido: gerar um binário com estas mudanças, repetir `validate-profile-migration.ps1` e ensaiar um perfil danificado no aplicativo real antes de declarar G1.

## Catálogo de conteúdo em 2026-11-14 (fonte; sem binário distribuído)

Responsável: Claude Code. Arquivos: `src/simulation/catalog.rs`, `data/catalog/core.json`, `data/lines/*.json`, `tests/catalog.rs`, `docs/LINHAS.md`; em `src/simulation/service.rs` as metas dos postais viraram constantes públicas, sem mudar valores. Divisão de trabalho com o Codex em `docs/COORDENACAO.md`.

| ID | Estado | Entregue e verificado |
| --- | --- | --- |
| DATA-03 | aceito | Unicidade, referências, capacidade, conectividade e caminho de retorno (circuito fechado ou linha de ponta a ponta), com casos inválidos testados. |
| DATA-05 | aceito | Erros indicam linha, estação, trecho ou item; núcleo inválido não carrega e linha inválida fica fora da sessão sem derrubar as demais. |
| DATA-06 | aceito | Conjunto pequeno: o Circuito Aurora e um rascunho; nenhum editor ou modding público. |
| DATA-01 | parcial | IDs estáveis para linha, estações, trechos, vagões, cargas, pinturas, recompensas e locomotivas. Faltam tipos de contrato, que ainda não existem como conteúdo. |
| DATA-02 | parcial | Conteúdo em `data/`, separado do estado. O motor de viagem ainda usa as definições fixas do código; testes de paridade garantem que os dois coincidem. |
| DATA-04 | parcial | Versão de catálogo e política para item renomeado, desativado ou desconhecido implementadas e testadas no catálogo; ainda não aplicadas ao checkpoint. |
| SAV-02 | definido, não implementado | Proposta do checkpoint v4 em `docs/LINHAS.md`. |

Verificação: `tests/catalog.rs`, 10 testes aprovados, junto com as suítes de save e serviço que tocam os mesmos módulos. A suíte completa e o Clippy da árvore inteira ficam para depois da rodada do Codex, que está alterando `app.rs`, `config.rs`, plataforma e renderer em paralelo.

Linha Curitiba–Paranaguá (época atual): dados reais coletados uma vez em 2026-11-14 do OpenStreetMap, Open-Meteo e Wikidata — 14 pontos com quilômetro e altitude, 13 trechos com túneis e pontes contados, conferidos contra marcos quilométricos; cópias e método em `data/sources/curitiba-paranagua/`. Continua rascunho e não aparece no jogo; `tests/catalog.rs` passou a 11 testes. Critério de aceite da frente DATA ("adicionar um serviço/estação pelo catálogo sem reescrever o algoritmo de viagem") continua aberto até o motor ler a linha do catálogo.


## Codex — FAIXA e PRIMEIRA, 2026-11-14 (fonte posterior à rc.2)

Responsável pela integração destas frentes: Codex. Execução na ordem FAIXA F1–F7, depois PRIMEIRA P1–P3. O território do Claude foi preservado: nenhuma edição em simulação, progress.rs, data, testes de catálogo/save ou fixtures. Não houve commit, empacotamento, release, exclusão ou movimentação de arquivos do projeto. BUILD-INFO e manifestos da distribuição rc.2 continuam descrevendo aquela distribuição.

| Itens | Implementação e escopo |
| --- | --- |
| F1–F2 | Arraste horizontal, limiar de 4 px físicos, clique/direito preservados, pegas laterais de 6 px e cursor apropriado. Geometria pura em src/platform/strip.rs; a esquerda mantém a direita fixa. Barra menor que o mínimo usa sua largura disponível. |
| F3 | Canvas e renderer com 160–32768 colunas e 70 linhas; escala inteira/nearest, paisagem ampliada sem margens laterais, períodos de parallax fixos, buffers redimensionados somente ao mudar largura. Em 160 px parte da traseira fica recortada, sem diminuir a locomotiva. Estações usam o limite real de culling. |
| F4–F5 | Cantos retos por padrão; arredondamento com suporte DWM do Windows 11 e região de desenho/hit-test, removida ao voltar ao flutuante. Encaixe de altura ligado por padrão; altura livre 24–240 px cresce para cima, sobre a área de trabalho, com base fixa na barra. |
| F6–F7 | Gesto e sliders mantêm geometria provisória fora de Config; confirmar/soltar grava valores finais. Restaurar geometria religa encaixe de altura. Defaults novos compatíveis com JSON antigo; largura persistida admite barras maiores que 2400 px. Roteiro manual acrescentado a docs/windows_checklist.md. |
| P1 | Acolhida somente em perfil normal com configuração ausente; cidade/coordenadas ou São Paulo e escolha barra/flutuante. Confirmação salva; fechar antes não grava. Sem GPS/IP/geocodificação. Config inválida não vira perfil novo. |
| P2 | Mantidas três tentativas de taskbar_failures. Acrescentadas confirmação do retângulo, cliente não vazio, visibilidade e estado DWM após a primeira pintura. Falha abre flutuante e central com aviso; ocultação/minimização intencional não é falha. |
| P3 | Texto curto para futuro pacote em docs/windows_primeira_abertura.md, com fonte oficial Microsoft; não promete que assinatura elimina SmartScreen. |

A suíte revelou e orientou duas correções adicionais no território Codex: o guardião de caminhos em main.rs agora resolve primeiro o destino/ancestral acessível (sem exigir acesso a todos os ancestrais), mantendo proteção contra aliases; exports headless grandes conservam toolbar, indicador DEMO e arraste flutuante. Os 12 testes de ux_app passaram após essa correção.

Verificações automatizadas específicas: referência RGBA de 12 frames capturada nesta fonte antes da ampliação e preservada em 320 × 70; transparência/limites/determinismo/endereço de buffers e continuidade no wrap; contador de alocações com largura estável; geometria, DPI e barra menor que faixa salva; Config antiga; geometria provisória não gravada; acolhida confirmada por clique headless; fallback após três falhas; quatro alternâncias de modo e tamanho no harness egui. Isso não equivale a cliques/compositor Windows.

Validação final nesta fonte: **329 testes aprovados, zero falhas e dois exportadores opcionais ignorados**; formatação e Clippy sem warnings, todos com saída zero. Nenhuma falha final em tests/catalog.rs ou tests/save_*.rs. A suíte inclui os 11 testes atuais de catálogo do Claude. Comandos via scripts/dev.ps1 (PowerShell com ExecutionPolicy Bypass somente no processo):

- cargo fmt --all -- --check
- cargo clippy --locked --all-targets -j 2 -- -D warnings
- cargo test --locked --all-targets -j 2

Os arquivos alterados foram formatados com rustfmt da mesma toolchain, usando skip_children=true para não escrever nos módulos de outro agente. A checagem global cargo fmt passou depois. Logs desta rodada: target/codex-faixa-primeira-20261114-102506/{fmt-final.log,clippy-approved.log,tests-approved.log}. Tentativas anteriores que falharam estão separadas; não são apresentadas como gates aprovados.

**Aplicativo Windows real, sem certificação visual:** scripts/validate-faixa-primeira.ps1 executou uma primeira abertura de 12 s e o driver de quatro alternâncias de 20 s. Ambos retornaram 0 e registraram shutdown. Consultas Win32 observaram cliente flutuante válido e central visível; o perfil da primeira abertura continuou sem config.json, pois não houve confirmação da acolhida. As quatro mudanças solicitadas ocorreram aproximadamente em 4,8/8,9/12,2/16,3 s. A barra retornou largura/altura zero neste ambiente: foi exercitado o fallback, não um encaixe bem-sucedido. Não foram executados cliques/arrastes no aplicativo real. O teste headless de confirmação da acolhida é separado destes processos nativos.

Binário de debug destas execuções: SHA-256 6A9137E070FE6F9CB26DCE13F74A63E3DE992FDF1CC4BE9402C1DD079128299F. Registros em target/codex-native-20261114-114630/result.json e nas subpastas primeira/alternancias. A descrição textual desse JSON sofreu a leitura ANSI do PowerShell 5; os campos numéricos, booleanos e o hash permanecem legíveis. O script passou a usar UTF-8 com BOM para preservar o português nas próximas execuções. Nenhum resultado anterior foi sobrescrito ou reatribuído a outro binário.

O script scripts/validate-faixa-primeira.ps1 copia o executável para uma pasta nova em target, usa perfis isolados e registra hash/diagnósticos sem alterar evidence/current. A primeira tentativa longa foi executada antes dos últimos ajustes e não é evidência do binário final. O inventário de uso do computador não exibiu a instância de teste; a instância rc.2 do usuário não foi tocada. Permanecem sem aceite visual: arraste, pegas, cantos, altura e alternâncias com uma barra real; matriz Windows 10/11, DPI misto, monitores, autohide e SmartScreen em outro computador.

## Rota pelo catálogo, percurso genérico e checkpoint v4 em 2026-11-14 (fonte; sem binário distribuído)

Responsável: Claude Code. Arquivos: `src/simulation/route.rs`, `src/simulation/itinerary.rs` (novo), `src/progress.rs`, `tests/itinerary.rs` (novo), ajustes em `tests/save_*.rs`, `tests/service_dispatch.rs`, `tests/service_import.rs` e nos dois exemplos que incluem a rota isoladamente.

| ID | Estado | Entregue e verificado |
| --- | --- | --- |
| SAV-02 | aceito | Checkpoint v4 com `catalog_version` e `line`; v1 a v3 migram com backup exato; linha ou catálogo que esta edição não compreende é recusado e preservado. Teste de migração v3 → v4 byte a byte e cinco documentos v4 estranhos. |
| DATA-02 | parcial | O Circuito Aurora roda com nomes, comprimentos e biomas lidos do catálogo; teste fixa os comprimentos da seed padrão (1900, 2375, 2025, 2725) e a igualdade com a reserva embutida. Vagões, cargas e recompensas ainda vêm do código. |
| DATA (aceite) | aberto | `Itinerary` calcula posição em linhas de qualquer tamanho, circuito ou ponta a ponta, com paridade contra o motor atual. Ainda não ligado à viagem, aos serviços nem à interface: uma linha nova não roda só pelo catálogo. |

Verificação nesta fonte: formatação e Clippy (`-D warnings`) com saída zero; **340 testes aprovados**, zero falhas, dois exportadores ignorados. Não executado: aplicativo real com perfil v3 (a migração foi exercitada pela API), nem `scripts/validate-profile-migration.ps1`, que ainda descreve o v3.

Mudança de formato persistido, registrada conforme o contrato: o save passa a v4 e o executável rc.2 não o lê. Escala de tempo das linhas reais: 480 unidades por quilômetro, confirmada pelo Pedro em 2026-11-14.


## Codex — Rodada 2: pacote rc.3 e beta, 2026-11-14

Tarefas 1 a 6 executadas na ordem: versão, textos/checkpoint, build/pacote preliminar, validação do pacote, roteiro do beta e relatório. Esta entrega **0.4.0-rc.3** substitui como base atual as indicações de rc.2 nas seções históricas acima, que foram preservadas. Base real: efeb74ac31d97b9b5d9d8013dd3cd28a53f7196c, posterior ao a8beb80 da descrição da rodada; sem commit novo. Não se atribui a fonte modificada apenas ao HEAD: SOURCE-MANIFEST registra os bytes empacotados.

- Cargo.toml/Cargo.lock e textos atuais em rc.3; guia/checklist sem botão do jogo na barra, acesso por faixa/bandeja. Checkpoint v4 e backup .v3.backup.json documentados. Nenhuma edição em src/simulation/**, src/progress.rs ou data/**; nenhum formato persistido novo nesta rodada.
- Compilação Windows release com Rust 1.100.0, offline, no máximo dois jobs e sem duas compilações simultâneas; target/rc3 separado do executável de debug em uso. Variável CPU_EXPRESS_OFFLINE=1 acrescentada na criação do serviço de clima para permitir validação nativa normal, com salvamento, sem rede. Sem persistir essa opção.
- Scripts existentes de gates, smoke e migração atualizados. Python local não executável neste ambiente; fluxo PowerShell em build-rc3.ps1, package-rc3.ps1, validate-portable.ps1 e validate-rc3.ps1. Gates vinculados por hashes às fontes e ao binário; pacote inclui data, suas atribuições/ODbL, licenças e manifestos. A rc.2 e suas evidências foram preservadas.
- fmt e Clippy -D warnings: saída 0. Suíte completa: **340 aprovados, zero falhas, dois exportadores opcionais ignorados**. Fonte exata empacotada conferida por hashes; não houve recompilação em outra máquina. Logs em evidence/rc3/; temporários dos testes em target/rc3/tmp.
- Executável extraído de ZIP em pasta com espaços, perfil isolado, PATH Windows: 8 s, saída 0, shutdown e versão rc.3 confirmados. Os três ZIPs têm seus conteúdos conferidos por SHA-256; hashes dos ZIPs em release/CPU-Express-0.4.0-rc.3-SHA256SUMS.txt.
- Migração nativa por 14 s de **cópia de fixture sintética v3**, pausada: v4, catálogo 1/aurora-v1, backup exato, origem/config intactas, distância 3100,5 e tempo ativo mantidos, serviço integralmente preservado e pinturas Aurora/Ocean mantidas. Nenhum perfil real foi lido nem usado como destino. Não se apresenta essa fixture como save gerado pela rc.2.
- Smoke nativo de 12 s: saída 0, fallback flutuante e central independente confirmados. Barra indisponível para a instância, portanto sem confirmação de encaixe. Framebuffer da cena capturado; central/compositor não capturados. Sem reutilizar imagens ou medições antigas.
- Roteiro de dez minutos em docs/ROTEIRO_BETA.md; Iniciar_DIAGNOSTICO.cmd incluído no portátil, com perfil-beta separado e logs locais, sem envio automático. Iniciador revisado, não acionado interativamente; diagnóstico CLI exercitado nos ensaios. Relatório completo em docs/VALIDACAO_RC3.md; tabela Estado atualizada.

SHA-256 do executável: 85f6b91569eca70b256b5fdc5a6d4637b0e187d268b27f3a5136e7c9a0f9fd4f. Artefatos em release/CPU-Express-0.4.0-rc.3{,-Fonte,-Evidencias}.zip, dois níveis acima desta fonte. BUILD-INFO e manifestos gerados para esse binário. A extração final de integridade fica registrada em target/portable acceptance …/integrity-result.json, fora dos ZIPs para evitar hash circular.

Não executados: novo aceite visual/acolhida/cliques/arrastes/altura/cantos, encaixe real, quatro alternâncias nativas, SmartScreen/outro computador, Windows 10, DPI misto/monitores/autohide/Explorer, clima online, soak/benchmark, áudio/leitores de tela, instalação opcional e workflow remoto. O aceite anterior do Pedro vale só para o computador e o executável dele naquela ocasião. Nenhum commit, envio externo ou encerramento do processo do usuário; nenhuma exclusão/movimentação fora de release/ e target/. O perfil real em %LOCALAPPDATA%\CPU Express permaneceu intocado.
