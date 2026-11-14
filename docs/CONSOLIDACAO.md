# CPU Express — consolidação das cinco entregas

**Versão resultante:** 0.2.0-rc.1  
**Data:** 13 de novembro de 2026  
**Plataforma de destino:** Windows 10/11 x64

## Resultado e decisão

As cinco contribuições foram reunidas em uma única árvore baseada na linha A previamente escolhida. A entrega inclui um novo executável Windows, código-fonte, instruções de uso, roteiro de aceitação e evidências desta consolidação. Ela passa a ser a base recomendada para os próximos agentes.

O estado é de **candidata a lançamento**: os testes de lógica e interface passaram no host; o programa foi compilado para Windows; ainda falta executar o conjunto em um Windows real. Os pacotes antigos não foram usados como substitutos do executável integrado.

## 1. O que veio de cada frente

| Frente recebida | O que foi incorporado | Ligação realizada nesta consolidação |
| --- | --- | --- |
| 01 — Windows | Geometria da área útil, DPI, posicionamento, recuperação, ciclo de vida, bandeja e CI. | A interface agora usa o controlador para topo, minimizar, ocultar, reposicionar e sair; a bandeja pode restaurar a janela. |
| 02 — Arte | Renderer procedural original, ambientes, parallax, luz, chuva, qualidade reduzida, cinco estilos de vagão e tema clássico. | O cenário segue a jornada real; as opções visuais aparecem na interface e são salvas. Foi acrescentada floresta para completar o domínio. |
| 03 — Mundo | Circuito de quatro estações, jornada determinística, eventos, descobertas, conquistas e limites de tempo/deslocamento. | Progresso e conquistas ficam visíveis; eventos geram avisos; a mesma pausa controla trem, cenário e viagem. |
| 04 — Métricas | Worker com amostra mais recente, frescor, tratamento de valores inválidos, dados parciais, diagnósticos e ferramentas de medição. | A UI revalida sua cópia em cache e conserva uma leitura válida de CPU ou RAM quando apenas a outra falha. |
| 05 — UX | Atalhos, pausa, demo, HUD, controles, preferências protegidas e testes da interface. | Quatro abas reúnem todas as frentes; layout estreito e proporção da paisagem foram corrigidos; ocultar na bandeja deixou de ser um controle apenas previsto. |

As fontes originais continuam identificadas em `docs/entregas_originais/`. Não há cinco aplicativos concorrentes no pacote novo: há uma aplicação integrada.

## 2. O que o usuário encontra

### Um companheiro de desktop

A janela de 640 × 140 pontos lógicos aparece perto da barra de tarefas e pode ser arrastada por uma área livre da paisagem. Ela oferece sempre no topo, minimizar, ocultar/restaurar pela bandeja e encerramento explícito. Não é um componente inserido dentro da barra de tarefas.

Pausar, minimizar ou ocultar congela a simulação. O retorno não compensa o tempo ausente. A coleta permanece independente, e a interface verifica se a amostra ainda está atual antes de apresentá-la.

### Uma jornada visível

O Circuito Aurora liga Vila Aurora, Bosque dos Pinheiros, Serra Clara e Porto do Sol. A interface mostra o próximo destino, progresso do trecho, distância fictícia restante, visitas e circuitos concluídos. As três conquistas são primeira estação, primeiro circuito e longa jornada.

Os cenários acompanham a rota: vale, floresta, serra árida e litoral. A transição ocorre nos últimos 25% de cada trecho, sem reiniciar as camadas de movimento. O bioma Highland usa a arte de serra árida já existente; não foi tratado como uma quinta paisagem.

Estações ainda são marcos de passagem. O trem não possui embarque, comércio ou uma rotina de parada nas plataformas nesta versão.

### Personalização disponível

Em **Trem**, o usuário pausa e controla a CPU simulada. Em **Viagem**, consulta rota, descobertas e conquistas. Em **Visual**, escolhe HUD, tema, detalhes, luz, nuvens, chuva e vagões. Em **Janela**, ajusta topo, pausas automáticas e ações de desktop.

Os vagões oferecem passageiros, dormitório, carga, tanque e panorâmico. Os três lugares podem ser escolhidos individualmente, da cauda em direção à locomotiva. Essas escolhas são de aparência e não modificam a leitura de CPU ou recompensas.

O modo DEMO não cria o coletor real de CPU/RAM. O modo manual fora do DEMO muda a entrada do trem e mantém a coleta real em segundo plano. Nenhum desses modos gera trabalho artificial para aquecer ou ocupar a CPU.

## 3. Correções necessárias para a integração

### Controle de janela e primeiro desenho

O eframe começa com a janela oculta antes do primeiro desenho. Interpretar essa condição como uma ocultação pedida pelo usuário impediria a inicialização visual. A plataforma ganhou uma consulta que diferencia o primeiro desenho das suspensões posteriores. O controlador processa restauração e saída antes de qualquer retorno por estado oculto.

O código evita comandos concorrentes de visibilidade/topo enviados pela UI e pelo controlador. A preferência de topo alterada pela bandeja também volta à configuração. O tratamento de erro da bandeja preserva a janela utilizável.

### Rota e renderer

O renderer original da frente 02 tinha um circuito visual próprio baseado na distância. A frente 03 tinha quatro trechos reais. Apenas copiar os dois módulos não faria seus ambientes coincidirem.

Foi criada a adaptação `SceneContext::from_journey`, e a jornada passou a expor a dica do próximo bioma. Foi acrescentada uma floresta original, mantendo o desenho da locomotiva e o contato das rodas com os trilhos. Os deslocamentos de parallax continuam vinculados ao mundo.

### Relógios e ferramentas visuais

Algumas ferramentas da arte faziam um grande salto de tempo, ou deslocamento impossível em um intervalo muito pequeno, para montar capturas. O novo domínio rejeita esses passos. Isso tornava algumas imagens incorretas e uma verificação de continuidade pouco informativa.

As fixtures e o exportador agora avançam em passos aceitos pelo domínio. A sequência de apresentação pode seguir a jornada efetiva. A UI utiliza `World::advance_train` para aplicar o mesmo passo ao trem, ao cenário e à viagem.

### Métricas parciais e vencidas

O status agregado do coletor pode indicar falha enquanto uma das métricas continua válida. A UX anterior exigia status totalmente pronto e descartava CPU válida quando apenas a RAM falhava; também ocultava RAM válida durante o aquecimento da CPU.

A interface passou a avaliar cada leitura, seu frescor e os dados disponíveis. Um aviso informa a leitura parcial sem substituir um percentual válido por um zero fictício. A cópia em cache continua envelhecendo entre publicações e é reavaliada.

### Configuração, espaço e legibilidade

As opções visuais foram acrescentadas com defaults para configurações antigas. A gravação preserva campos desconhecidos inclusive dentro dos objetos novos. Arquivos inválidos ou incompatíveis permanecem intactos.

Em 320 × 140, o cabeçalho usa menos botões, o HUD completo reduz o conteúdo e a paisagem mantém sua proporção. Marcadores sem cobertura na fonte foram substituídos por texto legível. Os avisos de DEMO e pausa continuam identificáveis. Os controles de CPU permanecem na primeira aba, separados do resumo da viagem.

## 4. Validação executada

Os resultados detalhados e comandos estão em `docs/VALIDATION.md` e `docs/evidencias_consolidacao/`.

| Verificação | Evidência desta consolidação |
| --- | --- |
| Testes Rust no host Linux | 169 aprovados; 1 exportador visual ignorado na execução padrão. |
| Exportação visual explícita | Exportador executado separadamente; 18 cenários renderizados a partir do egui real e revisados. |
| Ferramentas de análise de benchmark | 19 testes Python aprovados. |
| Formatação e Clippy | Verificados na fonte consolidada; Clippy também para o alvo Windows. |
| Executável Windows | Release PE32+ x64 produzido a partir desta fonte. |
| Testes Windows | 18 executáveis de teste compilados e linkados; não executados. |
| Avisos das dependências | Inventário GNU/MSVC reunido e integridade dos textos conferida. |

As imagens da interface são renderizações sem janela nativa: o egui produziu as texturas e a geometria, que foram rasterizadas para inspeção. Não são screenshots do Windows. O ambiente não permitiu abrir um display gráfico, pois a criação dos sockets necessários foi bloqueada. Não houve execução do jogo em Wine nem validação em Windows 10/11.

Os registros antigos foram separados em `docs/historico/`. Resultados de CPU, memória, FPS, contagens de testes e hashes das entregas anteriores não são apresentados como resultados deste executável.

## 5. O que permanece pendente

1. **Aceitação Windows real.** Foco, bandeja, Explorer, DPI, dois monitores, área útil, minimização/restauração e consumo devem ser verificados no PC de destino. O checklist entregue começa com todos esses resultados pendentes.
2. **Salvar o progresso.** Preferências são persistidas; posição na rota, visitas e conquistas reiniciam ao fechar o aplicativo. A implementação de save da jornada deve ser o próximo avanço funcional depois da aceitação Windows.
3. **Estações e conteúdo de jogo.** Paradas, passageiros, cargas, comércio, ramificações de rota e novos objetivos ainda não existem nesta candidata.
4. **Distribuição final.** O executável não tem assinatura digital nem instalador. O workflow MSVC e o script de empacotamento foram preparados, mas não executados neste ambiente.
5. **Linux como produto.** O host permite testar código independente de plataforma; um port de desktop Linux continua sendo uma frente futura.

Não é necessário escolher novamente entre os cinco pacotes. O próximo trabalho deve partir do código consolidado e de um resultado concreto do teste no Windows.

## 6. Pacotes e rastreabilidade

O pacote Windows contém o novo `cpu-express.exe`, um iniciador DEMO, instruções, checklist, metadados de build e avisos de licença. O pacote de código contém a árvore completa, testes, exemplos, scripts, CI, documentação corrente e histórico identificado. Não inclui diretórios de compilação nem executáveis antigos.

`BUILD-INFO.json` identifica o executável, target, toolchain, hash e limitações. `SOURCE-MANIFEST.json` identifica os arquivos entregues. A compilação foi feita com dependências bloqueadas e sem acrescentar novas bibliotecas ao projeto.

**Recomendação:** usar 0.2.0-rc.1 como a única base; validar a experiência Windows; corrigir os problemas observados; então implementar persistência de jornada e uma primeira estação completa. O roadmap detalhado está em `docs/ROADMAP.md`.
