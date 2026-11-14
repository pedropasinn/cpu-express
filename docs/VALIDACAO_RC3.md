# Validação CPU Express 0.4.0-rc.3

Executado por Codex em 2026-11-14 no Windows local, build do sistema `10.0.26300.0`. Branch `pos-rc2`, base `efeb74ac31d97b9b5d9d8013dd3cd28a53f7196c` (o commit posterior a `a8beb80` registra a própria Rodada 2). As alterações desta entrega não foram commitadas. `SOURCE-MANIFEST.json` identifica a fonte distribuída; não se afirma que o pacote corresponde apenas ao HEAD.

Executável **Windows x86_64 GNU, release, sem assinatura**, compilado com Rust 1.100.0. SHA-256:

```text
85f6b91569eca70b256b5fdc5a6d4637b0e187d268b27f3a5136e7c9a0f9fd4f
```

## Verificações executadas

| Verificação | Resultado e alcance |
| --- | --- |
| Build Windows | `cargo build --offline --locked --release -j 2`, saída 0; sem compilação cruzada. Diretório `target/rc3`, separado do executável de debug aberto pelo usuário. |
| Formatação | `cargo fmt --all -- --check`, saída 0. |
| Clippy | `cargo clippy --offline --locked --all-targets -j 2 -- -D warnings`, saída 0. |
| Suíte completa | `cargo test --offline --locked --all-targets -j 2`: **340 aprovados, zero falhas, dois exportadores opcionais ignorados**. Estes dois não contam como aprovados. Temporários dos testes em `target/rc3/tmp`. |
| Correspondência da fonte | Hashes de Cargo, `src/`, `data/`, `tests/`, `examples/` e configuração Cargo conferidos antes/depois dos gates e no empacotamento. Manifesto do ZIP-fonte conferido na extração. Os gates são da fonte exata empacotada; não foi feita uma segunda compilação do ZIP em outra máquina. |
| Portátil | Três ZIPs extraídos em pasta com espaços dentro de `target/`; hashes dos arquivos conferidos. Executável extraído com perfil isolado e PATH apenas do Windows: 8 s, saída 0, versão rc.3 e `shutdown` no diagnóstico. Não equivale a conta Windows nova. |
| Migração nativa | **Cópia de fixture sintética v3 → v4**, usando o executável extraído em modo normal, pausado, sem rede, por 14 s; fechamento normal, saída 0. `aurora-v1` e catálogo 1; distância 3100,5 e tempo ativo mantidos, serviço integralmente preservado, pinturas Aurora/Ocean mantidas. `.v3.backup.json` idêntico byte a byte à origem; configuração e jornada de origem intactas. Não é perfil real do Pedro nem um save produzido por uma execução rc.2. |
| Smoke nativo | 12 s, saída 0 e `shutdown`; 13 registros, cliente flutuante válido e central independente detectados por Win32. A barra retornou largura/altura zero: **fallback confirmado; encaixe na barra não confirmado**. Captura de framebuffer da cena presente; captura da central ausente. Nenhuma confirmação do compositor. |
| Rede e perfil | Cargo offline e `CPU_EXPRESS_OFFLINE=1` nos ensaios nativos. Na migração, todos os registros confirmam `local_weather.enabled=false`. Nenhum envio externo. O perfil real em `%LOCALAPPDATA%\CPU Express` não foi lido nem usado como destino; o processo do usuário foi preservado. |
| Licenças e integridade | Textos de licença existentes conferidos por SHA-256 contra `licenses/manifest.json`. Portátil e fonte incluem `data/` completo e seu README de procedência/licença: © OpenStreetMap contributors, ODbL 1.0, além das atribuições de Wikidata e Open-Meteo/Copernicus. A linha real continua rascunho, não jogável. |

## Pacote e reprodução

Artefatos na raiz do projeto, em `release/`:

- `CPU-Express-0.4.0-rc.3.zip`: portátil, dados, licenças, guia, roteiro e iniciador de diagnóstico.
- `CPU-Express-0.4.0-rc.3-Fonte.zip`: fonte com `BUILD-INFO.json` e `SOURCE-MANIFEST.json`.
- `CPU-Express-0.4.0-rc.3-Evidencias.zip`: somente evidências desta rodada em `evidence/rc3/`, com `EVIDENCE-MANIFEST.json`.
- `CPU-Express-0.4.0-rc.3-SHA256SUMS.txt`: hashes dos três ZIPs finais. O portátil inclui também hashes de todos os seus arquivos em `SHA256SUMS.txt`.

O fluxo adapta os scripts existentes: `verify-current.ps1`, `smoke-current.ps1` e `validate-profile-migration.ps1` foram atualizados; `build-rc3.ps1`, `package-rc3.ps1` e `validate-portable.ps1` fornecem a alternativa PowerShell ao empacotamento Python, indisponível para execução neste ambiente. Todos os caminhos de perfil de ensaio ficam em `target/`. Resultados antigos de `evidence/current/` e `VALIDACAO_04.md` são históricos da rc.2, não evidência deste executável.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-rc3.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate-rc3.ps1
# Após concluir documentação e relatório:
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-rc3.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate-portable.ps1 -IntegrityOnly
```

`validate-rc3.ps1` cria o pacote preliminar, extrai/executa o portátil, prepara a fixture v3, testa sua cópia e executa o smoke, nesta ordem. A última extração de integridade confere os ZIPs finais após a inclusão da documentação e das evidências; seu relatório fica em `target/portable acceptance …/integrity-result.json`, junto aos hashes desses ZIPs. Ele fica fora do ZIP de evidências para evitar uma referência circular ao hash do próprio arquivo. Os comandos não publicam nem fazem commit.

## Não verificado nesta rc.3

Não foram executados cliques de acolhida, arrastes, redimensionamento, cantos ou altura no desktop; quatro alternâncias nativas completas; nova revisão visual de menus; SmartScreen em outro computador; Windows 10; DPI misto/segundo monitor; autohide; reinício do Explorer; leitores de tela; áudio audível; clima online; benchmarks ou soak prolongado. O iniciador `Iniciar_DIAGNOSTICO.cmd` foi revisado, mas não acionado interativamente; a criação de diagnósticos pela CLI foi exercitada nos ensaios nativos.

O aceite visual anterior do Pedro vale somente para o Windows 11 do computador dele e para o executável que ele testou naquela ocasião. Não é atribuído ao hash rc.3 acima. As capturas de framebuffer e os testes geométricos não substituem a observação do compositor.

O workflow remoto permanece pendente: não houve publicação, GitHub CLI nem repositório remoto criado nesta rodada. Também não foram testados instalação opcional, desinstalação, assinatura, atualização automática, disco cheio real ou interrupção forçada durante gravação. Nenhum teste sintético é apresentado como teste de todo perfil real possível.
