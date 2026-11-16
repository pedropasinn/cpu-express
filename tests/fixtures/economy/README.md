# Fixtures da economia v5

Geradas localmente em 2026-11-16 pelo exemplo `economy_fixtures`, sem abrir o perfil do jogador. Dados fictícios; seed 42. O exportador recusa sobrescrever arquivos e pode ser executado novamente com **outra pasta**:

```powershell
$env:CARGO_NET_OFFLINE = 'true'
./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','economy_fixtures','-j','2','--','pasta-nova')
```

- `v1.json`: jornada sem serviços; migração não inventa passageiros, carga ou postais.
- `v2.json`: perfil com 20 passageiros e 20 unidades entregues, dois contratos e 240 pontos; sem os campos posteriores de despacho/excursões.
- `v3.json`: mesmo perfil, incluindo despacho e excursões.
- `v4.json`: mesmo perfil, incluindo catálogo e linha Aurora. Composição legada e pinturas elegíveis preservadas.
- `v5-new.json`: carteira inicial, inventário mínimo, campos reservados vazios e progresso de Aurora.
- `v5-pending.json`: migração v4 com uma remessa grátis sorteada, repetida e ainda não revelada. Reembolso zero, resultado/RNG/garantia já persistidos.

Os testes copiam os bytes para diretórios temporários exclusivos. Conferem backups exatos de v1 a v4, reabertura, preservação dos direitos adquiridos, ausência de carga retroativa por espécie, campos obrigatórios, versões futuras e recuperação de compra pendente. As fixtures rc7 anteriores continuam separadas e intactas.
