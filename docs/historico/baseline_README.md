# CPU Express 0.1

Uma pequena locomotiva a vapor atravessa uma paisagem pixel art enquanto reage à utilização real da CPU do computador. Aplicação nativa em Rust, com janela compacta de aproximadamente **640 × 140 pontos lógicos**, três vagões e arte original desenhada pelo próprio programa.

**Destino da versão 0.1: Windows 10 versão 1607 ou posterior e Windows 11, x64.** O desenvolvimento e os testes disponíveis ocorreram em Linux. A geração de um executável Windows e a execução de uma janela no laboratório Linux não substituem a validação em Windows real. Consulte [VALIDATION.md](docs/VALIDATION.md) para os comandos, resultados e pendências da entrega; [BENCHMARKS.md](docs/BENCHMARKS.md) registra exclusivamente as medições realizadas.

## Abrir o aplicativo

1. Extraia o pacote ZIP da versão Windows para uma pasta.
2. Mantenha juntos os arquivos que vieram no pacote.
3. Abra `cpu-express.exe` com dois cliques.
4. Aguarde a primeira leitura de CPU. A janela continua animada durante essa preparação.

O executável não precisa de Rust, Cargo, navegador, servidor local, conta ou conexão de rede para funcionar. Rust e ferramentas de compilação só são necessários para desenvolver o código.

### Controles

| Controle | Ação |
|---|---|
| Arrastar uma área livre do cenário | Mover a janela |
| `≡` | Abrir as opções |
| `P` no topo ou tecla `P` | Alternar sempre no topo |
| `−` | Minimizar |
| `×` | Fechar |
| Tecla `Esc` | Fechar os controles, se abertos; caso contrário, fechar o aplicativo |
| Tecla `M` | Mostrar ou ocultar os indicadores |
| Opção de CPU manual | Substituir a CPU usada na viagem por um valor de 0 a 100% |
| Opção de reposicionar | Voltar a uma posição dentro da área útil do monitor |

Os atalhos de teclado funcionam quando a janela está com foco. A versão 0.1 não instala atalhos globais nem ícone na bandeja. Depois de minimizar, restaure a janela pela barra de tarefas ou pelo seletor de janelas do Windows.

No modo automático, CPU, RAM e velocidade aparecem nos indicadores. No modo manual, a CPU usada no movimento é identificada como simulada; isso não gera carga artificial na máquina. A coleta real pode continuar atualizando a RAM.

## Como a CPU movimenta o trem

O alvo inicial usa `target_speed = 12.0 + cpu_usage * 0.88`:

| Uso de CPU | Velocidade desejada |
|---|---:|
| 0% | 12 km/h |
| 50% | 56 km/h |
| 100% | 100 km/h |

Esses km/h são unidades fictícias da viagem. Não representam frequência do processador, capacidade de hardware ou velocidade de algum processo específico.

A velocidade atual se aproxima do alvo com suavização exponencial dependente do tempo, inicialmente `k = 1.8 s⁻¹`. A simulação integra também a distância durante o intervalo; rodas, trilhos e camadas do cenário acompanham essa distância. A fumaça tem emissão e quantidade limitadas.

O valor de CPU não altera o limite de frames. A renderização tem alvo de aproximadamente 30 FPS, enquanto um worker separado coleta métricas aproximadamente uma vez por segundo. A primeira leitura de CPU é identificada como preparação, pois ainda não existe uma referência anterior confiável. Leituras ausentes ou antigas não encerram a aplicação; a viagem usa um comportamento de contingência e a interface indica o estado.

O próprio CPU Express participa da utilização total de CPU que está medindo.

Na execução de referência de cinco minutos no laboratório Linux/Mesa por software, foram medidos 29,17 FPS, RSS mediana de 73,95 MiB e CPU média de 10,25% de um processador lógico, equivalente a 1,14% da capacidade dos 9 processadores lógicos disponíveis. O consumo medido durante a minimização caiu para 0,102% de um processador lógico, sem desenhar novos frames. Esses valores não são benchmarks de Windows; método, dados e limites estão em [BENCHMARKS.md](docs/BENCHMARKS.md).

## O que está na versão 0.1

- Locomotiva, três vagões, rodas e biela animadas.
- Céu, nuvens, montanhas, árvores, colinas, cerca, trilhos e vegetação em camadas de parallax.
- Fumaça com quantidade limitada e desaparecimento gradual.
- CPU global e RAM pelo `sysinfo`, coletadas fora da thread da janela.
- Controle manual de CPU para experimentar aceleração e frenagem.
- Janela compacta, sem decoração tradicional, arrastável, com opções de sempre no topo, indicadores, minimizar e fechar.
- Posicionamento Windows baseado na área útil do monitor, com tratamento de coordenadas e escala; integração isolada em `platform/windows.rs`.
- Configurações locais e diagnósticos opcionais em arquivo.
- Testes do núcleo matemático, dos limites e dos caminhos relevantes da coleta.

Não há rede, cloud, banco de dados, telemetria remota, economia, estações interativas ou progressão. O executável é uma aplicação desktop; a cena não é uma página web.

## Configuração local

No Windows, o arquivo padrão é:

```text
%LOCALAPPDATA%\CPU Express\config.json
```

Exemplo dos parâmetros principais:

```json
{
  "min_speed_kmh": 12.0,
  "max_speed_kmh": 100.0,
  "smoothing_k": 1.8,
  "fps": 30,
  "metrics_interval_ms": 1000,
  "show_metrics": true,
  "always_on_top": true
}
```

Feche o aplicativo antes de editar o arquivo manualmente. Campos ausentes recebem valores padrão; valores numéricos são normalizados para limites seguros. O arquivo permite experimentar o comportamento sem editar o núcleo. O coletor sempre respeita também o intervalo mínimo exigido pelo `sysinfo`.

Use `--config` para escolher outro arquivo. Essa é uma opção do aplicativo: ao usar `cargo run`, ela deve aparecer **depois** de `--`.

Também é possível definir `CPU_EXPRESS_CONFIG`; a opção `--config` tem prioridade. Mostrar métricas e manter no topo são salvos quando você os altera. O modo manual de CPU vale para a sessão: uma abertura normal volta à CPU real. Um arquivo inválido não é regravado só porque foi lido; a interface mostra um aviso e usa valores padrão.

### Linha de comando

Exemplos em PowerShell, na pasta do executável:

```powershell
.\cpu-express.exe --cpu 50
.\cpu-express.exe --config ".\minha-config.json"
.\cpu-express.exe --smoke-seconds 10 --diagnostics ".\diagnostico.jsonl"
```

| Opção | Finalidade |
|---|---|
| `--cpu 0..100` | Iniciar com CPU manual; não força utilização real |
| `--config CAMINHO` | Usar um arquivo de configuração específico |
| `--smoke-seconds N` | Encerrar automaticamente após N segundos, para uma verificação curta |
| `--smoke` | Forma curta de execução com encerramento automático |
| `--diagnostics CAMINHO` | Gravar amostras e estado da viagem em JSONL local |
| `--help` | Imprimir ajuda na saída padrão, quando houver console; para o executável GUI Windows, consulte esta tabela |

Aqui, *smoke test* significa teste curto de abertura/execução/encerramento. O nome da opção não liga nem desliga a fumaça da locomotiva.

O executável usa o subsistema GUI do Windows e não abre nem conecta um console. Por isso, a exibição direta de `--help` no PowerShell não é garantida; as opções estão documentadas acima.

## Compilar no Windows

### Preparar as ferramentas

Instale Rust pelo [instalador oficial](https://rust-lang.org/tools/install/). Para o alvo Windows MSVC, instale as ferramentas C++ do Visual Studio e o Windows SDK. A opção **Desenvolvimento para desktop com C++** do instalador do Visual Studio reúne os componentes necessários; o IDE completo não é uma dependência do código. Consulte as [instruções oficiais de pré-requisitos MSVC do rustup](https://rust-lang.github.io/rustup/installation/windows-msvc.html).

A entrega usa **Rust 1.100.0, edição 2024**. Para usar a mesma toolchain:

```powershell
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
rustup override set 1.100.0
rustc --version
cargo --version
```

Execute o `override` dentro da pasta deste projeto. Abra novamente o terminal se o instalador de Rust tiver acabado de atualizar o PATH.

### Executar, verificar e gerar release

Na pasta que contém `Cargo.toml`:

```powershell
cargo run --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

O executável da compilação nativa fica em:

```text
target\release\cpu-express.exe
```

O script `.\scripts\build-windows.ps1` reúne as verificações e cria um pacote release. Para medir o processo no Windows sem gerar carga artificial, execute `.\scripts\measure-windows.ps1 -Exe .\cpu-express.exe -Seconds 300 -IncludeDiagnostics` na pasta do pacote Windows. Os dois scripts foram revisados, mas não executados em PowerShell neste ambiente.

`cargo run` e `cargo build --release` também funcionam sem `--locked`. Recomendamos manter a opção para reproduzir as versões registradas no `Cargo.lock`; o Cargo recusa alterações inesperadas no lockfile. A opção `--release` ativa o perfil otimizado. Referência: [manual oficial de cargo build](https://doc.rust-lang.org/cargo/commands/cargo-build.html).

Para experimentar sem carga real:

```powershell
cargo run --locked -- --cpu 0
cargo run --locked -- --cpu 50
cargo run --locked -- --cpu 100
```

## Compilação cruzada e laboratório Linux

O alvo de compilação cruzada da entrega é `x86_64-pc-windows-gnu`. Ele requer a biblioteca padrão desse alvo e um linker MinGW-w64 x64:

```bash
rustup target add x86_64-pc-windows-gnu --toolchain 1.100.0
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo +1.100.0 build --locked --release --target x86_64-pc-windows-gnu
```

Saída esperada: `target/x86_64-pc-windows-gnu/release/cpu-express.exe`. Este comando descreve como reproduzir o build; o resultado efetivamente obtido e a inspeção de suas dependências estão em [VALIDATION.md](docs/VALIDATION.md).

A feature `host-validation` permite abrir a aplicação no ambiente Linux de desenvolvimento para verificar a cena e o fluxo de métricas:

```bash
cargo run --locked --features host-validation -- --cpu 50
```

**Essa feature é apenas um instrumento de laboratório.** A versão 0.1 não oferece integração desktop Linux como produto. Não há promessa de suporte a Wayland, posição de painéis, regras de compositores ou empacotamento Linux; isso pertence à versão 0.5.

## Organização do código

| Módulo | Responsabilidade |
|---|---|
| `src/main.rs` | Argumentos e inicialização |
| `src/app.rs` | Estado da aplicação, controles e agenda de frames |
| `src/config.rs` | Preferências, valores padrão e normalização |
| `src/metrics/` | Interface dos provedores, sysinfo, canal limitado e ciclo do worker |
| `src/simulation/train.rs` | CPU → alvo, suavização, distância e rodas |
| `src/simulation/world.rs` | Deslocamento contínuo e partículas limitadas |
| `src/rendering/` | Desenho pixel art original, sem leitura do sistema |
| `src/platform/` | Integração Windows da janela e área útil |
| `examples/export_frame.rs` | Exportação de um frame usando o mesmo renderer |
| `tests/` | Verificações automatizadas |

Para aprender ownership, borrowing, traits, `Option`, `Result`, threads e a matemática do movimento, leia [APRENDER_RUST.md](docs/APRENDER_RUST.md). O guia inclui experimentos curtos com resultados esperados.

## Dependências e decisão técnica

| Dependência | Versão definida | Uso |
|---|---:|---|
| Rust | 1.100.0 na entrega | Toolchain estável; edição 2024 |
| `eframe` / `egui` | 0.37.0 | Janela, interface e upload da cena; backend Glow |
| `sysinfo` | 0.40.3 | CPU global e memória |
| `serde` / `serde_json` | Linha 1, exata no lockfile | Configuração e diagnósticos locais |
| `windows-sys` | 0.61.2, somente Windows | Área útil, DPI, posicionamento e mensagem de erro |
| `raw-window-handle` | 0.6.2, somente Windows | Empréstimo seguro do handle da janela criado pelo eframe |

Os cinco repositórios solicitados foram investigados antes da implementação. A decisão foi criar uma aplicação nativa em Rust: os forks Tauri reaproveitariam principalmente recursos periféricos e exigiriam substituir a simulação, o rendering do trem e a arquitetura do overlay. Veja a comparação fundamentada em código e testes em [REPOSITORY_COMPARISON.md](docs/REPOSITORY_COMPARISON.md).

O código de CPU Express e sua arte foram escritos para este projeto. **Nenhum arquivo dos cinco repositórios foi copiado para o produto.** A licença dos projetos investigados não transfere a licença de suas imagens; por isso nenhum sprite deles foi incorporado.

## Situação da validação e limites reais

| Item | Situação |
|---|---|
| Janela mínima, cena animada e CPU/RAM reais | Fluxo executado no laboratório Linux; detalhes no relatório de validação |
| Núcleo matemático e caminhos de falha | Testes automatizados; resultado final e comandos no relatório |
| Binário Windows x64 | Alvo de entrega GNU; compilação cruzada e inspeção documentadas separadamente |
| Execução nativa em Windows 10/11 | Pendente de validação em um sistema Windows real |
| DPI misto, vários monitores e posições de barra de tarefas | Tratamento implementado; comportamento real Windows ainda deve ser verificado |
| CPU, RAM, tamanho e estabilidade | Somente números medidos em [BENCHMARKS.md](docs/BENCHMARKS.md); sem extrapolar o host para a máquina do usuário |

O encerramento do coletor é cooperativo: um timer de espera é interrompido ao fechar, mas uma consulta ao sistema operacional que já está executando precisa retornar. No Windows, o winit 0.30 não informa oclusão geral: uma janela apenas coberta por outra pode continuar renderizando. Minimizar suspende a renderização e mantém a coleta aproximadamente a 1 Hz. Erros internos de alguns contadores PDH não são expostos pelo sysinfo; o programa identifica ausência de dados, valores inválidos e amostras antigas, mas não diagnostica toda falha possível de contador.

Em uma área útil excepcionalmente menor que a janela, o posicionamento reduz o tamanho físico para caber. O renderer Glow exige um driver compatível com OpenGL 2.0 ou posterior. A entrega é portátil e não possui instalador ou assinatura digital.

A versão 0.1 usa janela opaca, tamanho lógico fixo e uma paisagem. Não implementa transparência avançada, click-through, encaixe na barra de tarefas real ou bandeja. Drivers e integração de janela devem ser verificados no Windows de destino; funcionamento no host Linux não comprova esses detalhes.

As evoluções previstas estão em [ROADMAP.md](docs/ROADMAP.md).

## Licença

Código e arte original do CPU Express: [MIT](LICENSE). Dependências mantêm suas próprias licenças; os avisos e textos acompanham a entrega em [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) e `licenses/`. A origem dos desenhos está documentada em [assets/sprites/README.md](assets/sprites/README.md).
