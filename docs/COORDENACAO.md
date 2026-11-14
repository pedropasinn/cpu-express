# CPU Express — coordenação entre agentes

Atualizado em 2026-11-14. Coordenação: Claude Code, a pedido do Pedro. Este arquivo é a fonte de verdade sobre quem altera o quê enquanto Codex e Claude trabalham na mesma árvore. Leia-o inteiro antes de editar; atualize a tabela de estado ao terminar cada frente.

## Decisões do Pedro (2026-11-14)

- Ordem aprovada: (1) consolidar a versão atual, (2) beta com amigos, (3) catálogos, (4) primeira linha real, (5) locomotivas reais, (6) importador e mais linhas. Etapas 2 e 3 em paralelo.
- Primeira linha real: **Curitiba–Paranaguá, com Morretes no meio**. Época: **atual** (decidido em seguida, no mesmo dia).
- O jogo precisa funcionar em outros computadores: o Pedro vai compartilhar com amigos.
- Pedido novo: **faixa da barra ajustável por manipulação direta** (frente FAIXA abaixo).
- Locomotivas reais virão depois, com livros de referência que o Pedro ainda vai enviar. A arte é sempre redesenhada; nada de decalcar ilustração nem usar logotipos.

## Regras comuns

1. **Não commitar, não empacotar e não gerar release** sem pedido do coordenador. A árvore tem mudanças não commitadas dos dois agentes.
2. **Não apagar nem mover arquivos** nesta rodada. Limpezas ficam para um pedido explícito do Pedro.
3. Cada agente edita somente os arquivos da sua coluna. Precisa de algo no território do outro? Escreva em "Pedidos entre agentes" e siga com o restante.
4. `docs/PLANO_EXECUCAO.md`: marque apenas checkboxes dos seus itens e acrescente uma seção própria **no fim**; não reescreva seções alheias.
5. Formatação, Clippy (`-D warnings`) e testes pertinentes antes de declarar algo pronto. Só descreva como executado o que rodou nesta fonte. Validação nativa não feita fica registrada como não feita.
6. `cargo` pode estar em uso pelo outro agente: se o diretório `target` estiver bloqueado, aguarde; não mate processos.
7. Nenhum formato persistido muda sem registro aqui. Preferências novas entram com default e sem quebrar arquivos antigos.

## Território

| Área | Dono | Observação |
| --- | --- | --- |
| `src/app.rs`, `src/main.rs`, `src/config.rs`, `Cargo.toml`, `Cargo.lock` | Codex | Claude não edita nesta rodada. |
| `src/platform/**`, `src/rendering/**`, `src/ui_*.rs`, `src/app/workshop.rs` | Codex | |
| `scripts/**`, `.github/**`, `docs/GUIA_04.md`, `docs/windows_*.md`, `README.md` | Codex | |
| `src/simulation/**`, `src/progress.rs` | Claude | Codex lê, não edita. |
| `src/app/journey.rs`, `src/app/services.rs`, `src/app/branches.rs`, `src/app/storage.rs` | Claude | Só quando o catálogo precisar aparecer na UI; ainda não. |
| `data/**` (novo), `docs/LINHAS.md` (novo), `tests/catalog*.rs`, `tests/save_*.rs`, `tests/fixtures/**` | Claude | |
| `docs/COORDENACAO.md` | Claude | Codex edita só a tabela de estado e os pedidos. |

## Tarefas do Codex nesta rodada

### Frente FAIXA — faixa da barra ajustável (pedido do Pedro)

Hoje posição, largura e altura existem apenas como sliders em `behavior_controls` (`taskbar_offset`, `taskbar_width`, `taskbar_height`), e a cena é sempre 320 × 70 em escala inteira: uma faixa mais larga só ganha margens laterais vazias.

- **F1 Arrastar.** Segurar e arrastar a faixa move-a horizontalmente dentro da barra. Use um limiar (cerca de 4 px) para que o clique continue abrindo a central e o botão direito continue pausando. A faixa nunca sai da barra.
- **F2 Redimensionar pelas laterais.** Zonas de pega nas bordas esquerda e direita, com cursor de redimensionar. A borda esquerda altera posição e largura mantendo a direita fixa. Limites: mínimo atual (160 px) e a largura da barra.
- **F3 A cena acompanha a largura.** Faixa mais larga mostra **mais paisagem**, sem esticar e sem margens vazias; faixa mais estreita mostra menos, mantendo locomotiva e vagões legíveis. Preservar: escala inteira com filtragem nearest, determinismo, nenhuma alocação por frame (realocar só quando o tamanho muda), rodas/engates/baseline e períodos de wrap do parallax. Com largura interna 320 o framebuffer deve ser idêntico ao atual, com teste de regressão. Defina e documente os limites da largura interna.
- **F4 Cantos.** Preferência para cantos arredondados ou retos na faixa. No Windows 10, se não houver suporte, cair para cantos retos e documentar.
- **F5 Altura.** Preferência "encaixar na altura da barra" (padrão atual). Desligada, a altura fica livre dentro dos limites já existentes, também ajustável pela borda superior; decida e documente o que acontece acima da altura da barra.
- **F6 Persistência.** Gravar a geometria somente ao soltar o mouse, nunca valores intermediários (contrato DES-02). Os sliders e "Restaurar posição e tamanho" continuam funcionando e refletem o que foi feito com o mouse.
- **F7 Verificação.** Cálculos de geometria como funções puras testadas (limites, borda esquerda, barra menor que a faixa salva, DPI). Configuração antiga carrega com os defaults novos. Acrescentar ao `docs/windows_checklist.md` os passos manuais de arrastar, redimensionar, cantos e altura. As quatro alternâncias barra/flutuante não podem regredir.

### Frente PRIMEIRA — primeira abertura em outro computador

- **P1** Sem arquivo de configuração, a central abre com uma acolhida curta: escolher a cidade do clima (ou manter São Paulo) e faixa na barra ou janela flutuante. Sem GPS nem localização por IP.
- **P2** Se o encaixe na barra não puder ser confirmado, cair para o modo flutuante com aviso, em vez de ficar invisível. Verifique o que `taskbar_failures` já faz antes de implementar.
- **P3** Texto curto para o pacote explicando o aviso do SmartScreen (executável sem assinatura). Não prometer que assinatura elimina o aviso.

### Depois, somente com pedido do coordenador

Pacote 0.4.0-rc.3 com as mudanças dos dois agentes, execução do workflow em um remote e roteiro do beta. Não iniciar agora.

## Tarefas do Claude nesta rodada

- **Catálogo (DATA-01..05, SAV-02, MOD-04):** IDs estáveis para linha, estações, trechos, vagões, cargas e recompensas; versão de catálogo; validação com erros legíveis; Circuito Aurora como primeira entrada sem mudar comportamento.
- **Linha Curitiba–Paranaguá:** pesquisa de estações, distâncias e trechos com fonte, data e grau de adaptação; dados separados do código, com licença e procedência.

## Pedidos entre agentes

- 2026-11-14 — Codex → integração: `src/config.rs` acrescenta `taskbar_fit_height` (default `true`) e `taskbar_rounded` (default `false`); largura máxima persistida passa de 2400 para 32768 px e continua limitada à barra em uso. Nenhuma mudança no checkpoint. `tests/config.rs` acompanha o novo limite; `tests/rendering_allocations.rs` cobre as larguras variáveis. Não requer alteração no território Claude.

## Estado

| Frente | Dono | Estado | Verificação executada | Não executado |
| --- | --- | --- | --- | --- |
| FAIXA | Codex | F1–F7 implementados; gates de fonte aprovados; aceite visual pendente | fmt, Clippy -D warnings e suíte completa: 329 aprovados/0 falhas/2 ignorados; regressão 320 × 70, alocações, geometria/DPI/persistência e quatro alternâncias headless; driver nativo executado | Arraste/redimensionamento/cantos/altura no desktop; encaixe real (barra indisponível no ambiente); matriz Windows/DPI |
| PRIMEIRA | Codex | P1–P3 implementados; gates de fonte aprovados | Testes de acolhida/confirmar/salvar e fallback; execução nativa de 12 s, saída 0, fallback flutuante e central observados por Win32; config ausente antes de confirmar | Cliques da acolhida no aplicativo real; SmartScreen/outro computador; compositor |
| Catálogo | Claude | primeira entrega feita | `tests/catalog.rs` (10 testes) e suítes de save/serviço | Motor de viagem lendo a linha do catálogo; checkpoint v4 |
| Linha Curitiba–Paranaguá | Claude | dados reais coletados (14 pontos, km, altitude, túneis e pontes); época atual | `tests/catalog.rs` (11 testes), conferência com marcos quilométricos | Motor de viagem para linha de ponta a ponta; embarque atual por ponto; demanda e cenário |
