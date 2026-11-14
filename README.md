# CPU Express 0.4.0-rc.1

Uma ferrovia de desktop em Rust: animação ligada à CPU, viagem com estações e objetivos, salvamento local e faixa integrada à esquerda da barra de tarefas do Windows.

## Expansão 0.4

Passageiros e cargas reais, composição com capacidade, bilhetes persistentes, postais, locomotivas originais e excursões. Clima configurável com cache e iluminação solar gradual. Veja o [guia da versão atual](docs/GUIA_04.md). Os tópicos rc.5–rc.7 abaixo registram a evolução anterior.

## Usar

Abra `cpu-express.exe` na distribuição portátil. Clique no trem para abrir a central em janela independente; botão direito pausa/retoma. Em **Viagem**, acompanhe paradas, diário e objetivos. Em **Visual**, equipe vagões e pinturas liberadas. **Janela** alterna entre integração na barra e modo flutuante.

Execute `cpu-express.exe --demo` para experimentar sem consultar métricas reais. DEMO libera as pinturas; não salva a jornada. O modo normal não gera carga artificial, e CPU alta não acelera a progressão lógica.

Veja [funcionamento, limites e próximas áreas de trabalho](docs/VERSAO_03.md).

## Desenvolver

Esta árvore é a base atual. Não reaplique patches de agentes anteriores. Rust 1.100.0, eframe 0.37.0 e sysinfo 0.40.3; dependências fixadas no Cargo.lock. No workspace fornecido, `scripts/dev.ps1` localiza Rust e MinGW em `../../.tools/`, sem alterar o PATH do Windows permanentemente.

```powershell
.\scripts\verify-current.ps1 -Jobs 2
.\scripts\dev.ps1 build --locked --release -j 2
.\scripts\dev.ps1 run --locked
```

Com toolchain própria, os comandos cargo equivalentes também funcionam. O executável fica em `target/release/cpu-express.exe`. O aplicativo distribuído não precisa das ferramentas de desenvolvimento.

## Organização

| Área | Fonte |
| --- | --- |
| Interface e coordenação | `src/app.rs` |
| Preferências compatíveis | `src/config.rs` |
| Checkpoint e escritor exclusivo | `src/progress.rs` |
| Itinerário, paradas e objetivos | `src/simulation/` |
| Arte procedural, estações e pinturas | `src/rendering/` |
| Win32, encaixe e bandeja | `src/platform/` |
| Coleta fora da UI | `src/metrics/` |

Os relatórios em `docs/historico/` e `docs/evidencias_consolidacao/` não certificam esta versão. Resultados atuais acompanham o novo BUILD-INFO e as evidências da distribuição.

## Diagnóstico

```powershell
.\cpu-express.exe --config teste.json --smoke-seconds 15 --diagnostics teste.jsonl
```

O registro inclui métricas, viagem e geometria/parent da janela nativa. As variáveis opcionais `CPU_EXPRESS_CAPTURE_DIR` e `CPU_EXPRESS_OPEN_PANEL` servem aos testes: a primeira salva o framebuffer do próprio app em PPM após três segundos, sem sobrescrever arquivos; a segunda abre a central inicialmente. Não capturam outros aplicativos. Retire essas variáveis após validar.

Código e arte original sob MIT. Avisos de terceiros em `THIRD_PARTY_NOTICES.md` e `licenses/`. Esta distribuição é candidata de testes; versões distintas do Explorer, DPI misto, autohide e vários monitores requerem aceitação específica.


RC2: a faixa usa uma janela independente no retangulo da barra, na mesma altura e alinhada a esquerda. O encaixe como filha do Explorer foi removido porque uma janela nativa visivel e um framebuffer valido nao garantiam exibicao pelo compositor. A superficie ocupa essa area sem reservar espaco para widgets. A confirmacao visual no desktop e separada dos testes de geometria.


## Novidades visuais da rc.5

- Locomotiva e cinco tipos de vagão com novos acabamentos; estações próprias por bioma, fumaça com volume e cenários refinados.
- Central com prévia ao vivo, cartões de métricas e objetivos, amostras de pintura, controles agrupados e rodapé fixo.
- Visual → Fluidez oferece 60 FPS ou 30 FPS. Perfis novos usam 60; preferências existentes são respeitadas pelo aplicativo.
- O agendamento mantém cadência sem acumular atrasos nem adiar animação durante ajustes rápidos.

Galeria e medições: `evidence/current/visual-refresh/`. Painéis: `evidence/current/panels-rc5/`. Para repetir a validação nativa desta entrega, compile release e use `scripts/validate-visual-refresh.ps1`; ela trabalha com perfis isolados e mantém o baseline da rc.4 local.

## Clima local da rc.6

A paisagem usa somente a apresentação completa. Em Visual → Paisagem e atmosfera, São Paulo, SP, Brasil é a cidade configurada nesta versão. O aplicativo consulta condições estimadas pelo Open-Meteo em segundo plano a cada 15 minutos; dia/noite e efeitos de céu limpo, nublado, neblina, chuva, tempestade e neve seguem os dados recebidos. A consulta usa coordenadas do centro da cidade, sem localizar o dispositivo por GPS ou IP. Não representa uma medição na rua do usuário.

O painel identifica carregamento, falhas e dados antigos. Sem resposta válida inicial, o cenário é ilustrativo. O botão de atualização respeita intervalo mínimo para evitar consultas repetidas. Os modos clássico e reduzido saíram da interface; configurações antigas migram para a apresentação completa. O renderer conserva adaptadores históricos apenas para compatibilidade de testes e exemplos.

DEMO e testes de janela não consultam a rede normalmente. Para validar a integração real em perfil isolado, use `CPU_EXPRESS_LIVE_WEATHER=1` junto de `--smoke-seconds 25` e `--diagnostics`. Retire a variável após o teste.

Dados meteorológicos: [Open-Meteo](https://open-meteo.com/), licença [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). A API gratuita usada nesta edição é destinada a uso não comercial.

## Interface ferroviária da rc.7

A central agora é um caderno de jogo com molduras de madeira e latão, papel claro e ícones pixelados originais. **Trem** abre a cabine do maquinista, **Viagem** mostra mapa e objetivos, **Visual** abre a oficina e **Janela** reúne os ajustes do desktop.

Na oficina, escolha um dos três lugares e clique no cartão de um vagão para equipá-lo. **Pinturas** mostra as quatro locomotivas com requisito de desbloqueio e marca de item equipado. **Clima** mantém as condições locais de São Paulo; **Fluidez** reúne os ajustes de animação. As miniaturas vêm do próprio renderer e são criadas uma vez, sem recriar texturas por frame. Recompensas de objetivos levam diretamente à coleção de pinturas.

Botões têm resposta de foco, hover e pressão; mudanças de equipamento recebem confirmação. A janela de menus continua independente, redimensionável, com conteúdo rolável e rodapé fixo. A aparência foi orientada pelas 17 capturas fornecidas pelo usuário, usando implementação e arte próprias. Não foram adicionados compras, moedas ou sistemas de jogo fictícios à interface.

Evidências da interface atual: `evidence/current/panels-rc7`. Os PNGs são exportações do egui, não screenshots do compositor do Windows.
