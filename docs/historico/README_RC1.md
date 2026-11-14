# Histórico — CPU Express 0.2.0-rc.1

Um pequeno trem de desktop em Rust, com pixel art original e velocidade que reage à atividade real da CPU. Esta é a **base única consolidada** das entregas Windows, Arte, Mundo, Métricas e UX.

**Destino: Windows 10/11 x64.** A aplicação abre uma janela de 640 × 140 pontos lógicos perto da barra de tarefas. Ela não é inserida dentro da barra de tarefas. A versão é uma candidata de testes: foi compilada para Windows; a aceitação em um Windows real permanece pendente. Veja [validação atual](docs/VALIDATION.md).

## Executar

Extraia o pacote Windows inteiro e abra `cpu-express.exe`. Não é necessário instalar Rust para usar o executável. O pacote acompanha os avisos de licença das dependências.

Para explorar a aparência sem consultar os contadores reais, execute `Iniciar_DEMO.cmd`, ou use:

```powershell
.\cpu-express.exe --demo
```

O demo começa em 50% de CPU simulada. O controle manual não cria carga artificial. Para uma simulação específica:

```powershell
.\cpu-express.exe --demo --cpu 100
```

`--cpu 50` sem `--demo` controla apenas a entrada do trem; a coleta real de CPU/RAM continua em segundo plano. A interface distingue os modos.

## O que foi integrado

| Frente | Comportamento no consolidado |
| --- | --- |
| Windows | Posicionamento na área útil, DPI, recuperação após mudanças de monitor, sempre no topo, minimizar e ocultar/restaurar pela bandeja. |
| Arte | Vale, floresta, serra árida e litoral; transições entre trechos; ciclo de luz, chuva, nuvens e cinco estilos de vagão. |
| Mundo | Circuito de quatro estações, distância/progresso do trecho, descobertas, eventos e três conquistas locais. |
| Métricas | Worker com uma amostra substituível, proteção contra leituras vencidas, preservação de leituras parciais e diagnóstico opt-in. |
| Interface | Pausa, demo, três modos de HUD e quatro abas: Trem, Viagem, Visual e Janela. |

O cenário acompanha a **rota efetiva da jornada**, com transição nos últimos 25% do trecho. A integração preserva o movimento de parallax, a posição das rodas nos trilhos e a separação entre coleta, domínio e desenho.

### Controles

| Ação | Atalho ou controle |
| --- | --- |
| Pausar/seguir | Espaço ou botão Pausar/Seguir |
| Alternar HUD compacto/completo/limpo | M |
| Sempre no topo | P |
| Opções / ajuda | O / F1 |
| Reposicionar perto da barra | R |
| Fechar painel | Esc |
| Encerrar | Ctrl+Q ou Sair |
| Mover janela | Arrastar uma área livre da paisagem |
| Ocultar e recuperar | Opções → Janela → Ocultar na bandeja; depois usar o ícone do CPU Express |

Atalhos são locais à janela e cedem aos campos em edição. Fechar/Sair encerra o processo. Ocultar é uma ação separada e só fica disponível quando a bandeja oferece uma forma de recuperar a janela. Em janelas estreitas, o cabeçalho mantém os controles essenciais, o HUD reduz a quantidade de informação e o cenário preserva a proporção.

## Viagem e personalização

O circuito liga Vila Aurora, Bosque dos Pinheiros, Serra Clara e Porto do Sol. Os quatro ambientes seguem os trechos dessa rota. A floresta foi acrescentada na integração para resolver a diferença entre os quatro biomas do domínio e os três cenários entregues originalmente pela frente de arte. Serra Clara usa o cenário de serra árida.

Há três conquistas: primeira estação, primeiro circuito e longa jornada. Chegadas e conquistas produzem avisos breves. As estações são marcos da rota; esta versão ainda não implementa parada, embarque, comércio ou escolha de destino.

Em Visual é possível escolher o tema Viagem ou Clássico, detalhes completos/reduzidos, ciclo de luz, nuvens, chuva e três vagões independentes. Os cinco estilos são passageiros, dormitório, carga, tanque e panorâmico. São escolhas visuais: não alteram a coleta ou o modelo de velocidade. O ciclo de luz dura dez minutos ativos; não representa a hora local.

**O progresso da viagem e as conquistas pertencem à sessão.** Fechar e abrir o jogo reinicia a jornada. Pausar, minimizar ou ocultar congela a simulação; não há avanço offline ou recuperação do tempo ausente.

## Preferências e diagnósticos

No Windows, as preferências ficam em `%LOCALAPPDATA%\CPU Express\config.json`. É possível escolher outro arquivo com `--config CAMINHO` ou pela variável `CPU_EXPRESS_CONFIG`.

Arquivos antigos recebem defaults para os campos novos. Campos desconhecidos, inclusive dentro das preferências visuais, são preservados. Um arquivo inválido ou incompatível não é sobrescrito automaticamente: a aplicação usa defaults e exibe aviso. Mudanças na interface são agrupadas antes da gravação.

O jogo não exige serviço online. Diagnósticos são locais e só são gerados quando solicitados:

```powershell
.\cpu-express.exe --diagnostics diagnostico.jsonl --smoke-seconds 30
```

O arquivo registra origem/frescor das leituras, entrada efetiva da simulação, frames, pausa, estado oculto e progresso. Dados ausentes/vencidos são `null`, não medições de zero. Use um arquivo diferente do JSON de configuração.

## Compilar e testar

Use a versão de Rust fixada em `rust-toolchain.toml` (1.100.0), com rustfmt e Clippy. Para MSVC, o ambiente precisa dos componentes C++ e Windows SDK adequados do Visual Studio Build Tools. As versões de dependências estão fixadas em `Cargo.lock`.

No Windows:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo run --locked
```

Para validar e gerar um pacote portátil com os avisos das dependências:

```powershell
.\scripts\build-windows.ps1
```

O script aceita `-Target x86_64-pc-windows-gnu` quando esse target/linker estiverem instalados. O workflow em `.github/workflows/windows.yml` executa o caminho MSVC e produz artefatos de teste; não publica releases.

Os testes de lógica também rodam no host Linux. `--features host-validation` permite abrir uma janela de laboratório quando existe um display apropriado. Isso não constitui um port Linux do produto.

## Estrutura

```text
src/app.rs          Interface e adaptação entre as cinco frentes
src/config.rs       Preferências, migração e gravação protegida
src/metrics/        Coleta independente da UI
src/simulation/     Trem, tempo ativo, circuito, eventos e conquistas
src/rendering/      Framebuffer, ambientes, clima e vagões originais
src/platform/       Geometria, ciclo de vida, bandeja e Win32
src/diagnostics.rs  Registro local opcional
tests/              Contratos e regressões da integração
docs/               Decisões, validação e guia de uso
licenses/           Textos e proveniência das dependências
```

Leia [consolidação](docs/CONSOLIDACAO.md), [guia de uso](docs/ux_guia.md), [checklist Windows](docs/windows_checklist.md) e [próximos passos](docs/ROADMAP.md).

Documentos e patches antigos foram mantidos em `docs/historico/` e `docs/entregas_originais/`. Seus números, hashes e capturas descrevem aquelas entregas. Não reaplique esses patches sobre esta árvore.

## Limites desta candidata

A compilação Windows e os testes automatizados estão documentados, mas bandeja, foco, DPI, múltiplos monitores, recuperação do Explorer e consumo real precisam de aceitação no PC Windows de destino. Não há instalador, assinatura digital, transparência/click-through ou integração dentro da barra de tarefas. O executável e todos os recursos de arte desta versão foram produzidos a partir desta fonte consolidada; recursos gráficos/sonoros do APK de Tiny Rails não foram incorporados.

Código do projeto sob licença MIT. Avisos de terceiros em `THIRD_PARTY_NOTICES.md` e `licenses/`.
