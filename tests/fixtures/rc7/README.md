# Fixtures rc7 (SAV-03)

Documentos sintéticos no formato gravado pela 0.3.0-rc.7: checkpoint v1 (`*.journey.json`) e preferências sem os campos da 0.4 (`old-preferences.config.json`). Usam a seed padrão; nenhum arquivo é cópia de um perfil real.

| Arquivo | Situação |
| --- | --- |
| `new-profile.journey.json` | Perfil recém-criado, em Vila Aurora, sem chegadas. |
| `mid-segment.journey.json` | Trecho intermediário entre Bosque dos Pinheiros e Serra Clara, uma chegada. |
| `station-stop.journey.json` | Parada em Serra Clara com 5,25 s de serviço restantes, duas chegadas. |
| `rewards-unlocked.journey.json` | 12 voltas completas e 49 chegadas: quatro pinturas liberadas. |
| `old-preferences.config.json` | Preferências rc7: modo flutuante, HUD detalhado, pintura Expresso Noturno. |

Os testes em `tests/save_fixtures.rs` copiam estes bytes para uma pasta temporária; os originais não são regravados. Não edite um arquivo existente para acomodar um formato novo: acrescente outra fixture.
