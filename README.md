# CPU Express 0.4.0-rc.2

Jogo ferroviário de desktop para Windows, com faixa na barra de tarefas, janela flutuante e menus independentes.

## Usar

Abra `cpu-express.exe` na distribuição portátil, ou `Iniciar_CPU_Express.cmd` na raiz do projeto. Clique no trem para abrir a central; botão direito pausa/retoma.

- [Guia atual](docs/GUIA_04.md): passageiros, despacho automático/manual, devoluções, vagões, locomotivas, excursões, clima e saves.
- [Validação e limites](docs/VALIDACAO_04.md): 288 testes aprovados e verificações nativas executadas.
- [Plano de execução](docs/PLANO_EXECUCAO.md): entregas e pendências.
- [Prioridades](docs/ROADMAP.md).
- [Primeira abertura e SmartScreen](docs/windows_primeira_abertura.md): texto para o próximo pacote e comportamento da fonte posterior à rc.2.

## Desenvolver

Esta é a única base atual de código. `src/` contém domínio, renderer, UI e integração Windows; `tests/` e `examples/` fornecem verificações e exportadores. `scripts/dev.ps1` encontra a toolchain local sem alterar o PATH permanentemente. Cargo.lock fixa as dependências.

```powershell
.\scripts\verify-current.ps1 -Jobs 4
.\scripts\dev.ps1 build --locked --release -j 4
.\scripts\dev.ps1 run --locked
```

Os caches de compilação são recriados quando necessários; a primeira compilação após a limpeza leva mais tempo. `target/release/cpu-express.exe` mantém uma cópia do binário validado para empacotamento. `evidence/current/` contém as evidências da versão atual.

O pacote portátil, o código-fonte e as evidências são distribuídos em ZIPs separados. `scripts/package-current.py` exige hashes e validações correspondentes; `scripts/validate-portable.py` testa extração e execução com perfil isolado. Licenças em LICENSE, THIRD_PARTY_NOTICES.md e licenses/.

## Estado e dados

O progresso normal fica em `%LOCALAPPDATA%\CPU Express`, separado deste projeto. O formato v3 migra v1/v2 com backup; não substitua saves por arquivos antigos. A CPU afeta a velocidade visual, não a progressão lógica. Pausa e fechamento não acumulam progresso offline.
