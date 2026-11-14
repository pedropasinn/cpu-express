# CPU Express 0.2.0-rc.1 — revisão visual da interface consolidada

Estas imagens saíram do código real de `CpuExpressApp`/egui e `SceneRenderer`. O teste captura a tesselação e as texturas, e o rasterizador de QA produz PNGs. Não são mockups nem capturas de Windows. O teste usa DEMO, não consulta CPU/RAM reais e não inicializa integração de janela nativa.

Comando de exportação: `CPU_EXPRESS_VISUAL_DIR=<diretório> cargo test --locked --test ux_visual -- --ignored --nocapture`.

Rasterização: `python docs/ux_rasterizar.py <diretório>`.

## Cenários

| Arquivo | Cenário |
|---|---|
| 01_compacto.png | 640×140, métricas e progresso da viagem |
| 02_detalhado.png | 640×140, HUD completo |
| 03_limpo.png | 640×140, HUD limpo com controles visíveis |
| 04_pausa.png | 640×140, pausa explícita |
| 05_viagem.png | Aba Viagem, início do conteúdo |
| 06_visual.png | Aba Visual, início do conteúdo |
| 07_janela.png | Aba Janela, início do conteúdo |
| 08_compacto_320.png | 320×140, controles estreitos |
| 09_limpo_sem_foco.png | HUD limpo, sem foco e controles recolhidos |
| 10_visual_rolado.png | Aba Visual, vagões e parâmetros inferiores |
| 11_viagem_rolada.png | Aba Viagem, conquistas e aviso de progresso por sessão |
| 12_ajuda.png | Ajuda e atalhos |
| 13_viagem_320.png | Aba Viagem a 320×140 |
| 14_trem.png | Aba Trem, CPU simulada e slider |
| 15_visual_meio.png | Aba Visual, nuvens/detalhes/clima |
| 16_janela_rolada.png | Aba Janela, minimizar/ocultar e política de pausa |
| 17_detalhado_320.png | HUD completo degradado para compacto no espaço estreito |
| 18_pausa_320.png | Pausa com sinalização preservada a 320×140 |

## Problemas encontrados e corrigidos pelo coordenador

- Os botões sobrepunham o título e o selo de simulação em 320×140. O cabeçalho estreito agora mantém os sinais e os comandos essenciais.
- O framebuffer ficava deformado no formato estreito. A imagem agora preserva sua proporção.
- Caracteres de seta/círculos não presentes na fonte apareciam como quadrados. Foram trocados por texto legível e marcadores ASCII.
- O HUD detalhado sobrepunha RAM e velocidade no formato estreito. Agora usa a apresentação compacta abaixo de 600 pontos.
- O novo resumo da viagem escondia o slider da aba original. A interface foi separada em Trem, Viagem, Visual e Janela; Trem é a aba inicial.

O controle de ocultar na bandeja aparece desativado neste harness porque não existe janela/ícone nativo. Isso não é evidência de falha do adaptador Windows.

Os valores de velocidade variam um pouco porque o app usa seu relógio monotônico real, enquanto a captura percorre os quadros egui rapidamente. O rótulo de CPU é explicitamente simulado. Nenhuma medição de consumo real é inferida destas imagens.
