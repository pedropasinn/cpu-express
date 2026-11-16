# Economia — simulação E1b

Execução local em 2026-11-16, branch `expansao`, sem perfil real, rede ou downloads. A saída abaixo veio do simulador desta fonte; foram 1.000 sementes econômicas por cenário, doze cenários (1/3/8 h por dia, com/sem serviços, bônus ×1,00/×1,30). São quatro trajetórias de 240 h por semente, observadas também em 30 e 90 h; as amostras dos três horizontes são correlacionadas.

## Resultado e ajuste

O alvo foi atingido para a composição inicial com serviços automáticos: medianas de **101,08–103,93 moedas/h sem bônus**, **120,36–123,57 com bônus máximo**, incluindo ordens. P10–P90 ficaram entre 100,48 e 125,23 nos seis cenários com serviços. A primeira caixa paga chegou em **20,27 minutos ativos** em P10/P50/P90, além da caixa grátis em t=0. O alvo de primeira compra refere-se ao perfil novo com a composição inicial e resgate dos marcos; não é uma promessa para quem retira todos os serviços.

A execução preliminar com dez sementes e os preços/renda do E1a mediu 220–225 moedas/h sem bônus e 275–280 com bônus, primeira compra em 6,68 min. Por isso:
- apliquei fator **0,35** à renda de chegada/entregas antes do bônus, mantendo restos duráveis;
- preços de Pátio/Cargas/Plataforma/Garagem/Ateliê passaram de **60/90/90/180/75 para 150/225/225/450/188**;
- mantive os 405 de missões únicas, as 30 iniciais, compra direta e selos;
- ordens usam metas de 24 chegadas, 120 passageiros, oito contratos, 48 unidades por espécie ou 1 h ativa, com prêmios de 10/15. Não são os exemplos curtos de seis chegadas/dois contratos;
- as faixas sem épico de Plataforma e Ateliê transferem 1 ponto percentual para raro.

Os preços antigos de resultados já comprados continuam válidos na revelação; não houve recálculo de saldo histórico. Detalhes e API em [ECONOMIA_E1B.md](ECONOMIA_E1B.md).

## Método e limites

O programa primeiro executa o **World real**, em passos de um segundo, por 240 h, com o seed padrão do Aurora. Guarda snapshots e serviços apenas nas chegadas, usando o instante exato dentro do frame (desconta a parcela já consumida dos oito segundos de parada). Um teste compara o estado reproduzido com o World real, incluindo timestamps das ordens, carteira e RNG. Depois reproduz esses eventos nos mesmos módulos de economia e ordens usados pelo produto, sem copiar fórmulas para o exemplo. A inclusão desses módulos no executável dá acesso aos redutores internos sem expor um atalho de tempo no produto. Não é simulação de compositor nem teste de persistência física; esses fluxos têm testes separados.

Com serviços: Passageiros/Passageiros/Carga, despacho automático. Sem serviços: Tanque/Tanque/Tanque, despacho manual sem aceitar ofertas, portanto zero entregas. Nenhum equipamento muda durante uma execução. Compras ampliam posse e elegibilidade de ordens; o simulador troca uma ordem que a composição fixa não atende quando o domínio permite. Resgata missões e ordens em cada chegada. Não agenda excursões nem telegramas. O bônus é fixo em cada cenário, não leitura real da máquina.

Estratégia obrigatória: comprar a caixa mais barata que ainda possa dar novidade; quando nenhuma puder, comprar diretamente a peça mais barata faltante; após completar os 39 modelos, comprar selos. Aguarda saldo para a opção escolhida; não escolhe uma compra direta só por ser mais eficiente. Empates de caixas seguem `CrateKind::ALL`; de peças seguem a ordem estável do catálogo. Com os conjuntos atuais cobrindo todo o catálogo, o ramo de compra direta não chega a ser necessário. A proteção de novidades vale por raridade, portanto uma caixa com um épico faltante ainda pode gerar muitas repetidas comuns.

“Primeira rara+ possuída” inclui a caixa grátis e a pintura Expresso Noturno concedida nas doze chegadas. Por isso P10/P50/P90 coincidem em 20,27 min; **não** significa garantia de caixa rara nesse minuto. Caixa rara antes disso é possível nas sementes da cauda inferior.

Coleção completa significa vagões, motores e pinturas, não postais. Valores não atingidos até o horizonte entram como +infinito nos percentis, sem selecionar somente os jogadores que completaram. Os tempos são ativos; o intervalo fechado entre dias não gera renda nem reinicia contadores. A apresentação em horas/dia converte horizontes de 30/90/240 h em trinta dias.

**Para Pedro rever:** sem serviços, a primeira compra mediana leva 168–205 min, bem acima do alvo de entrada do perfil padrão; a renda fica em 23–29/h. Mesmo com serviços, 97,5% sem bônus e 85,6% com bônus não completam em 3 h/dia durante trinta dias. Em 8 h/dia, as frações são 34,7% e 24,1%. A estratégia obrigatória insiste em caixas até o último épico; compra direta é uma alternativa humana mais previsível. Não ajustei raridades ou garantia para ocultar essa cauda. Este estudo varia a semente econômica, **não** a rota, a composição, o comportamento humano, excursões ou telegramas. Não afirma renda universal para qualquer trem.

## Reprodução

Cargo offline e uma compilação por vez, até dois jobs:

```powershell
$env:CARGO_NET_OFFLINE = 'true'
$env:CPU_EXPRESS_OFFLINE = '1'
$env:CPU_EXPRESS_CONFIG = Join-Path $env:TEMP 'cpu-express-e1b-isolado/config.json'
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('rustc','--locked','--example','economy_sim','-j','2','--','-C','opt-level=2')"
./target/debug/examples/economy_sim.exe 1000
```

Foi otimizado somente o executável do exemplo, mantendo as dependências locais de desenvolvimento. `cargo run --locked --example economy_sim -j 2 -- 1000` também funciona, com maior tempo de execução. A execução final do simulador terminou com código 0. O log local bruto é `e1b-simulacao-1000-final.log` (ignorado pelo Git).

## Saída integral

```text
CPU Express — simulação E1b; 1000 sementes por cenário; percentis P10 / P50 / P90.
30 dias, horas ativas; sem progresso entre sessões. Renda recorrente inclui ordens e bônus, exclui boas-vindas, missões únicas e reembolsos.
Estratégia: caixa mais barata com novidade possível; depois compra direta mais barata faltante; depois selos. Empates: ordem CrateKind::ALL/ID. Grátis aberta em t=0.
Traço Aurora padrão real em passos <=1 s; composição fixa Passageiros/Passageiros/Carga ou Tanque/Tanque/Tanque sem despacho. Resgates em cada chegada; troca de ordem incompatível quando permitida. Sem excursões/telegramas. Sementes econômicas 0..N; rota fixa.

1 h/dia; serviços=false; bônus=×1.00; incompletas=1000/1000
Renda recorrente moedas/h: 23.67 / 25.00 / 26.33
Saldo final: 35.00 / 75.00 / 115.00
Caixas (inclui grátis): 6.00 / 6.00 / 6.00
Primeira compra (min ativos): 169.88 / 205.34 / 218.94
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

3 h/dia; serviços=false; bônus=×1.00; incompletas=1000/1000
Renda recorrente moedas/h: 23.34 / 24.34 / 25.34
Saldo final: 36.00 / 86.00 / 136.00
Caixas (inclui grátis): 16.00 / 17.00 / 17.00
Primeira compra (min ativos): 169.88 / 205.34 / 218.94
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

8 h/dia; serviços=false; bônus=×1.00; incompletas=1000/1000
Renda recorrente moedas/h: 22.68 / 23.39 / 24.18
Saldo final: 39.00 / 89.00 / 139.00
Caixas (inclui grátis): 41.00 / 43.00 / 45.00
Primeira compra (min ativos): 169.88 / 205.34 / 218.94
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

1 h/dia; serviços=false; bônus=×1.30; incompletas=1000/1000
Renda recorrente moedas/h: 27.03 / 28.70 / 30.03
Saldo final: 16.00 / 56.00 / 136.00
Caixas (inclui grátis): 6.00 / 7.00 / 7.00
Primeira compra (min ativos): 164.54 / 167.86 / 205.34
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

3 h/dia; serviços=false; bônus=×1.30; incompletas=1000/1000
Renda recorrente moedas/h: 26.94 / 28.06 / 29.06
Saldo final: 30.00 / 80.00 / 130.00
Caixas (inclui grátis): 18.00 / 19.00 / 20.00
Primeira compra (min ativos): 164.54 / 167.86 / 205.34
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

8 h/dia; serviços=false; bônus=×1.30; incompletas=1000/1000
Renda recorrente moedas/h: 26.18 / 26.93 / 27.68
Saldo final: 38.00 / 88.00 / 138.00
Caixas (inclui grátis): 48.00 / 50.00 / 51.00
Primeira compra (min ativos): 164.54 / 167.86 / 205.34
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

1 h/dia; serviços=true; bônus=×1.00; incompletas=1000/1000
Renda recorrente moedas/h: 102.10 / 103.93 / 105.77
Saldo final: 38.00 / 88.00 / 138.00
Caixas (inclui grátis): 25.00 / 25.00 / 26.00
Primeira compra (min ativos): 20.27 / 20.27 / 20.27
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

3 h/dia; serviços=true; bônus=×1.00; incompletas=975/1000
Renda recorrente moedas/h: 102.31 / 103.37 / 104.48
Saldo final: 43.00 / 88.00 / 138.00
Caixas (inclui grátis): 74.00 / 75.00 / 76.00
Primeira compra (min ativos): 20.27 / 20.27 / 20.27
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

8 h/dia; serviços=true; bônus=×1.00; incompletas=347/1000
Renda recorrente moedas/h: 100.48 / 101.08 / 101.67
Saldo final: 17.00 / 70.00 / 119.00
Caixas (inclui grátis): 83.00 / 163.00 / 200.00
Primeira compra (min ativos): 20.27 / 20.27 / 20.27
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): 104.98 / 200.70 / não atingido
Selos: 0.00 / 32.00 / 111.00

1 h/dia; serviços=true; bônus=×1.30; incompletas=1000/1000
Renda recorrente moedas/h: 121.73 / 123.57 / 125.23
Saldo final: 32.00 / 82.00 / 137.00
Caixas (inclui grátis): 29.00 / 30.00 / 31.00
Primeira compra (min ativos): 20.27 / 20.27 / 20.27
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): não atingido / não atingido / não atingido
Selos: 0.00 / 0.00 / 0.00

3 h/dia; serviços=true; bônus=×1.30; incompletas=856/1000
Renda recorrente moedas/h: 121.88 / 122.93 / 123.88
Saldo final: 39.00 / 89.00 / 139.00
Caixas (inclui grátis): 79.00 / 90.00 / 90.00
Primeira compra (min ativos): 20.27 / 20.27 / 20.27
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): 83.59 / não atingido / não atingido
Selos: 0.00 / 0.00 / 6.00

8 h/dia; serviços=true; bônus=×1.30; incompletas=241/1000
Renda recorrente moedas/h: 119.78 / 120.36 / 120.94
Saldo final: 14.00 / 67.00 / 116.00
Caixas (inclui grátis): 79.00 / 163.00 / 238.00
Primeira compra (min ativos): 20.27 / 20.27 / 20.27
Primeira rara+ possuída (min ativos): 20.27 / 20.27 / 20.27
Coleção completa (h ativas; censura em +infinito): 83.59 / 167.60 / não atingido
Selos: 0.00 / 71.00 / 155.00
```
