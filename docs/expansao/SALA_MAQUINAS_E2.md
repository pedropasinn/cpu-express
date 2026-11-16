# Sala de máquinas — E2

Em Trem → Máquina, seis medidores mostram a força de cada efeito (0–100%), não a porcentagem bruta de uso do computador. O total destacado vale somente para renda recorrente de chegadas e entregas. Prêmios de missões, caixas e reembolsos mantêm seus valores.

| Sistema | Fonte e saturação | Efeito |
| --- | --- | --- |
| Caldeira | CPU global, satura em 15% | Até +10% de moedas; ritmo visual já existente |
| Foguista | Presença de entrada em 20 amostras do último minuto | Até +15%; após cinco minutos sem entrada, marcha visual tranquila |
| Plataforma | RAM física, satura em 60% | Animação de embarque ligeiramente adiantada; sem passageiros extras |
| Guindaste | Leitura + escrita de discos físicos, satura em 5 MB/s | Guindaste decorativo nas paradas; sem encurtar serviço |
| Telégrafo | Tráfego agregado, barra satura em 200 kB/s | Fios/luz no pátio; qualquer tráfego válido na média ativa Linha viva (+5%) |
| Tênder | Tomada ou carga da bateria | Cheio na tomada; abaixo de 20% na bateria, menos movimento e no máximo 20 FPS |

O teto único é **+30%**. Cada observação satura antes de entrar na média: sobrecarregar a CPU não guarda bônus para depois. A média usa até 60 observações, em armazenamento fixo; repetições com menos de um segundo não contam novamente. O Foguista começa vazio na execução real e precisa das 20 observações com atividade. Uma leitura ausente desativa imediatamente aquele sistema; amostra vencida/futura desativa todos. As leituras não são gravadas em config, checkpoint ou diagnóstico.

O hodômetro mostra o tempo ativo da jornada, derivado do `SimulationClock` e do World. Não há segundo relógio de progressão no worker. Pausa, ocultação, minimização e fechamento continuam sem renda offline.

## Coleta e privacidade

O worker existente consulta uma vez por segundo. CPU/RAM permanecem no sysinfo. Windows acrescenta PDH com caminhos ingleses independentes do idioma (`PhysicalDisk(_Total)/Disk Bytes/sec`, `Network Interface(*)/Bytes Total/sec`), `GetSystemPowerStatus` e `GetLastInputInfo`. Foram habilitadas quatro features da dependência windows-sys já instalada; sem dependência, download ou versão nova de Cargo.lock.

O aplicativo fixa o intervalo em 1 s para essa coleta conjunta. `metrics_interval_ms` continua aceito nos arquivos antigos por compatibilidade, mas não altera a frequência desta tela. A API de workers injetados conserva os intervalos usados pelos testes; o provider real também recusa atualizar antes de 1 s.

Não há leitura de teclas, processos, títulos de janela, arquivos ou conteúdo de tráfego. `GetLastInputInfo` fornece somente o instante agregado da última entrada da sessão Windows. O coletor compara os instantes e calcula inatividade, sem ganchos. A opção **Usar atividade do teclado e mouse** está ligada por padrão, avisada e desligável na acolhida, em Ajustes/Fluidez e na própria Sala. Desligar sinaliza o worker e apaga imediatamente a média do Foguista; um resultado em trânsito também é ignorado.

As leituras da máquina não saem do computador. O clima conserva sua consulta meteorológica independente. Demonstração não constrói o coletor real: simula sinais e identifica isso na tela.

O armazenamento adicional é fixo: buffer alinhado de 64 KiB para as instâncias de rede, 60 observações e um snapshot no slot existente. Não há alocação Rust no código da nova consulta por amostra; isso não promete ausência de alocações internas das APIs do Windows/sysinfo. Um buffer insuficiente, contador ausente, status PDH inválido, bateria desconhecida ou API sem resposta válida resulta em `None`/“Sem leitura”, sem inventar zero. Interfaces virtuais podem participar do agregado de rede; a soma serve a uma ambientação saturada, não a contabilidade de tráfego. A primeira consulta de taxas precisa de baseline. Falha na abertura/registro PDH conserva indisponibilidade até reiniciar; falhas de leitura são tentadas novamente na próxima amostra.

## Telegramas e integração

Com rede válida, a Sala pede um telegrama a cada **30 minutos ativos da sessão**; fechar/reabrir reinicia essa espera. É uma decisão conservadora para revisão do Pedro. Não acumula pedidos. `World::request_telegram` mantém a solicitação até uma chegada elegível; só a economia cria a quarta ordem, depois das três comuns e respeitando dez minutos desde a última entrada. Uma ordem sem vaga pode esperar mesmo que a rede deixe de ter tráfego. O pedido em memória não é uma recompensa e não precisa de migração; a ordem criada é salva pelo checkpoint v5 existente. A segunda página de Ordens já a apresenta.

`src/simulation/machine.rs` é puro e testável sem Windows. O adaptador em `src/app/machine.rs` revalida idade, aplica `set_machine_bonus` antes da chegada transacional e envia apenas efeitos ao renderer. Memória muda a fase visual do embarque, nunca os contadores ou os oito segundos de parada. Guindaste e telégrafo são desenhos originais discretos no pátio durante paradas Aurora; excursões não os recebem. Movimento reduzido e bateria baixa fixam suas poses. A geometria antiga sem sinais permanece intacta.

## Reprodução local, sem perfil real

Usar a toolchain de `scripts/dev.ps1`, `CARGO_NET_OFFLINE=true` e `CPU_EXPRESS_CONFIG` apontando para um arquivo temporário.

```powershell
cargo fmt --all
cargo clippy --locked --all-targets -j 2 -- -D warnings
cargo test --locked --all-targets -j 2
cargo run --locked --example machine_probe -j 2
$env:CPU_EXPRESS_VISUAL_DIR = "$PWD/docs/expansao/telas"
cargo test --locked --test ux_visual -j 2 export_machine_menus -- --ignored
Get-ChildItem docs/expansao/telas/floating_e2_*.json | ForEach-Object { node docs/ux_rasterizar.cjs $_.FullName }
```

`machine_probe` faz quatro consultas agregadas, não abre perfil nem clima. As quatro telas `floating_e2_maquina_0` a `_3` representam demonstração no teto, ausência de sinais, bateria baixa/inatividade e entrada desligada. São renderizações headless do egui, não capturas do compositor. Resultados efetivamente executados e limites ficam no relato E2 ao final do plano.
