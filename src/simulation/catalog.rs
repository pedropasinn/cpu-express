//! Versioned content catalogue: stable IDs and authored definitions, kept apart
//! from save state and from the code that runs a journey.
//!
//! The documents live in `data/` and are embedded at build time, so the game
//! needs no file or network access to read them. Display names are content,
//! never persistence keys: saves and references use `id` only.
//!
//! Loading never panics. The core document must be valid; each line is checked
//! on its own and an invalid one is left out of the session with readable errors.
use serde::Deserialize;
use std::{collections::BTreeSet, fmt, sync::OnceLock};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
}
impl Rarity {
    pub const ALL: [Self; 4] = [Self::Common, Self::Uncommon, Self::Rare, Self::Epic];
    pub const fn direct_price(self) -> u64 {
        match self {
            Self::Common => 120,
            Self::Uncommon => 240,
            Self::Rare => 480,
            Self::Epic => 900,
        }
    }
}

/// Highest catalogue format this build understands.
pub const CATALOG_VERSION: u32 = 1;

const CORE: &str = include_str!("../../data/catalog/core.json");
const LINES: [&str; 3] = [
    include_str!("../../data/lines/aurora-v1.json"),
    include_str!("../../data/lines/curitiba-paranagua.json"),
    include_str!("../../data/lines/vitoria-minas.json"),
];

/// One problem, located by document path so the author can find it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogError {
    /// For example ``linha `aurora-v1` › trecho `pines-ridge` ``.
    pub at: String,
    pub message: String,
}
impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.at, self.message)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WagonDef {
    #[serde(default)]
    pub rarity: Rarity,
    pub id: String,
    pub name: String,
    pub seats: u32,
    pub cargo_units: u32,
    #[serde(default = "fictional_kind")]
    pub kind: ContentKind,
    #[serde(default)]
    pub required_stops: u64,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub retired: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CargoDef {
    pub id: String,
    pub name: String,
    /// Wagon type able to carry this cargo.
    pub wagon: String,
    /// Additional compatible stock; `wagon` remains the primary recommendation.
    #[serde(default)]
    pub compatible_wagons: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub retired: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LiveryDef {
    #[serde(default)]
    pub rarity: Rarity,
    pub id: String,
    pub name: String,
    pub required_stops: u64,
    #[serde(default = "fictional_kind")]
    pub kind: ContentKind,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub retired: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Counter {
    PassengersDelivered,
    CargoDelivered,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ServiceRewardDef {
    pub id: String,
    pub name: String,
    pub counter: Counter,
    pub goal: u64,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub retired: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Traction {
    Steam,
    Diesel,
    Electric,
}

/// Fictional engines leave the historical fields empty. A real engine must name
/// its builder, year and wheel arrangement and cite where the facts came from.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LocomotiveDef {
    #[serde(default)]
    pub rarity: Rarity,
    pub id: String,
    pub name: String,
    pub traction: Traction,
    pub kind: ContentKind,
    /// Service reward that unlocks it; `None` is available from the start.
    #[serde(default)]
    pub unlocked_by: Option<String>,
    #[serde(default)]
    pub builder: Option<String>,
    #[serde(default)]
    pub year: Option<u32>,
    /// Whyte notation for steam (`4-6-2`) or the UIC/AAR form for the others.
    #[serde(default)]
    pub wheel_arrangement: Option<String>,
    #[serde(default)]
    pub gauge_mm: Option<u32>,
    #[serde(default)]
    pub sources: Vec<SourceDef>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub retired: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Fictional,
    Real,
}

fn fictional_kind() -> ContentKind {
    ContentKind::Fictional
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LineStatus {
    /// Complete and selectable in the game.
    Playable,
    /// Research in progress: structurally checked, never offered to the player.
    Draft,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// The last segment returns to the first station.
    Circuit,
    /// End to end; the train returns over the same segments.
    Shuttle,
}

/// How closely a segment follows the real railway.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Adaptation {
    /// Order, distance and features follow the cited sources.
    Faithful,
    /// Real place, with distances or features reduced for play.
    Simplified,
    /// Authored for the game.
    Invented,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SceneryTag {
    Plain,
    Farmland,
    Forest,
    Mountain,
    River,
    Bridge,
    Viaduct,
    Tunnel,
    Urban,
    Port,
    Coast,
    Plateau,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SourceDef {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub url: Option<String>,
    /// ISO date on which the source was consulted.
    pub accessed: String,
    /// Licence of the source when data was derived from it (for example `ODbL-1.0`).
    #[serde(default)]
    pub license: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EraDef {
    pub label: String,
    pub from_year: u32,
    pub to_year: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StationDef {
    pub id: String,
    pub name: String,
    /// Real distance from the first station. Drafts may leave it open.
    #[serde(default)]
    pub km: Option<f64>,
    #[serde(default)]
    pub elevation_m: Option<f64>,
    /// False until the station, its order and distance were checked against a source.
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub retired: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SegmentDef {
    pub id: String,
    pub from: String,
    pub to: String,
    /// Fictional length in journey units, before the seed variation.
    #[serde(default)]
    pub base_units: Option<u32>,
    /// Largest seed variation of `base_units`, in journey units.
    #[serde(default)]
    pub variation_units: Option<u32>,
    /// Real length. Must agree with the stations' `km` when both are known.
    #[serde(default)]
    pub distance_km: Option<f64>,
    /// Renderer hint: `meadow`, `forest`, `highland` or `coast`.
    pub biome: String,
    #[serde(default)]
    pub scenery: Vec<SceneryTag>,
    pub adaptation: Adaptation,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LineDef {
    pub line_format: u32,
    pub id: String,
    pub name: String,
    pub status: LineStatus,
    pub kind: ContentKind,
    pub layout: Layout,
    #[serde(default)]
    pub era: Option<EraDef>,
    #[serde(default)]
    pub gauge_mm: Option<u32>,
    #[serde(default)]
    pub country: Option<String>,
    /// Licence of this document when it is a derived database.
    #[serde(default)]
    pub data_license: Option<String>,
    /// Credit line that must be shown with the line, if any.
    #[serde(default)]
    pub attribution: Option<String>,
    #[serde(default)]
    pub sources: Vec<SourceDef>,
    pub stations: Vec<StationDef>,
    pub segments: Vec<SegmentDef>,
    #[serde(default)]
    pub open_questions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct CoreDocument {
    catalog_version: u32,
    wagons: Vec<WagonDef>,
    cargo: Vec<CargoDef>,
    liveries: Vec<LiveryDef>,
    service_rewards: Vec<ServiceRewardDef>,
    locomotives: Vec<LocomotiveDef>,
}

/// Validated content for one session.
#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    pub catalog_version: u32,
    pub wagons: Vec<WagonDef>,
    pub cargo: Vec<CargoDef>,
    pub liveries: Vec<LiveryDef>,
    pub service_rewards: Vec<ServiceRewardDef>,
    pub locomotives: Vec<LocomotiveDef>,
    /// Only lines that passed validation, drafts included.
    pub lines: Vec<LineDef>,
    /// Problems of the lines that were left out.
    pub rejected: Vec<CatalogError>,
}

/// What a saved identifier means in the current catalogue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution<'a> {
    Current(&'a str),
    /// The identifier was renamed; use the new one and keep the save's meaning.
    Renamed(&'a str),
    /// Still understood, no longer offered. Keep what the save owns; do not grant it anew.
    Retired(&'a str),
    /// Not in this catalogue: the save comes from other content and must be preserved.
    Unknown,
}

const BIOMES: [&str; 4] = ["meadow", "forest", "highland", "coast"];

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 48
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        && id.as_bytes()[0].is_ascii_lowercase()
}

struct Report {
    errors: Vec<CatalogError>,
}
impl Report {
    fn push(&mut self, at: &str, message: impl Into<String>) {
        self.errors.push(CatalogError {
            at: at.to_owned(),
            message: message.into(),
        });
    }
    /// IDs and aliases of one list share a namespace: an alias may never shadow an ID.
    fn ids<'a>(
        &mut self,
        at: &str,
        what: &str,
        items: impl Iterator<Item = (&'a str, &'a [String])>,
    ) -> BTreeSet<&'a str> {
        // Identifiers first, so a clashing old name is reported on the item that declares it.
        let items: Vec<(&'a str, &'a [String])> = items.collect();
        let mut seen = BTreeSet::new();
        for (id, _) in &items {
            if !valid_id(id) {
                self.push(
                    at,
                    format!("{what} `{id}`: use letras minúsculas, dígitos, `-` ou `_`, começando por letra (até 48 caracteres)"),
                );
            }
            if !seen.insert(*id) {
                self.push(at, format!("{what} `{id}` aparece mais de uma vez"));
            }
        }
        let mut all = seen.clone();
        for (id, aliases) in &items {
            for alias in aliases.iter() {
                if !valid_id(alias) || !all.insert(alias.as_str()) {
                    self.push(
                        at,
                        format!(
                            "{what} `{id}`: o nome antigo `{alias}` é inválido ou já está em uso"
                        ),
                    );
                }
            }
        }
        seen
    }
}

fn resolve<'a>(
    id: &str,
    items: impl Iterator<Item = (&'a str, &'a [String], bool)>,
) -> Resolution<'a> {
    for (current, aliases, retired) in items {
        if current == id {
            return if retired {
                Resolution::Retired(current)
            } else {
                Resolution::Current(current)
            };
        }
        if aliases.iter().any(|alias| alias == id) {
            return if retired {
                Resolution::Retired(current)
            } else {
                Resolution::Renamed(current)
            };
        }
    }
    Resolution::Unknown
}

fn validate_sources(report: &mut Report, at: &str, sources: &[SourceDef]) {
    report.ids(
        at,
        "fonte",
        sources.iter().map(|s| (s.id.as_str(), &[][..])),
    );
    for source in sources {
        let date = source.accessed.as_bytes();
        let iso = date.len() == 10
            && date[4] == b'-'
            && date[7] == b'-'
            && date
                .iter()
                .enumerate()
                .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
        if !iso {
            report.push(
                at,
                format!(
                    "fonte `{}`: `accessed` deve ser uma data AAAA-MM-DD",
                    source.id
                ),
            );
        }
        if source.title.trim().is_empty() {
            report.push(at, format!("fonte `{}`: falta o título", source.id));
        }
    }
}

fn validate_core(core: &CoreDocument) -> Vec<CatalogError> {
    let mut report = Report { errors: Vec::new() };
    let at = "catálogo";
    if core.catalog_version == 0 || core.catalog_version > CATALOG_VERSION {
        report.push(
            at,
            format!(
                "versão {} não é compreendida por esta edição (máxima: {CATALOG_VERSION})",
                core.catalog_version
            ),
        );
    }
    report.ids(
        at,
        "vagão",
        core.wagons.iter().map(|w| (w.id.as_str(), &w.aliases[..])),
    );
    report.ids(
        at,
        "carga",
        core.cargo.iter().map(|c| (c.id.as_str(), &c.aliases[..])),
    );
    report.ids(
        at,
        "pintura",
        core.liveries
            .iter()
            .map(|l| (l.id.as_str(), &l.aliases[..])),
    );
    let rewards = report.ids(
        at,
        "recompensa",
        core.service_rewards
            .iter()
            .map(|r| (r.id.as_str(), &r.aliases[..])),
    );
    report.ids(
        at,
        "locomotiva",
        core.locomotives
            .iter()
            .map(|l| (l.id.as_str(), &l.aliases[..])),
    );
    for wagon in &core.wagons {
        if wagon.kind != ContentKind::Fictional {
            report.push(
                at,
                format!(
                    "vagão `{}`: material rodante real exige um futuro esquema de fontes",
                    wagon.id
                ),
            );
        }
        if wagon.seats == 0 && wagon.cargo_units == 0 {
            report.push(at, format!("vagão `{}`: sem lugares e sem carga", wagon.id));
        }
        if wagon.seats > 0 && wagon.cargo_units > 0 {
            report.push(
                at,
                format!(
                    "vagão `{}`: escolha passageiros ou carga, não ambos",
                    wagon.id
                ),
            );
        }
        if wagon.seats > 200 || wagon.cargo_units > 200 {
            report.push(at, format!("vagão `{}`: capacidade acima de 200", wagon.id));
        }
    }
    for cargo in &core.cargo {
        let mut seen = BTreeSet::new();
        for wagon_id in std::iter::once(&cargo.wagon).chain(&cargo.compatible_wagons) {
            if !seen.insert(wagon_id) {
                report.push(
                    at,
                    format!("carga `{}`: vagão `{wagon_id}` repetido", cargo.id),
                );
            }
            match core.wagons.iter().find(|w| w.id == *wagon_id) {
                None => report.push(
                    at,
                    format!("carga `{}`: o vagão `{}` não existe", cargo.id, wagon_id),
                ),
                Some(wagon) if wagon.cargo_units == 0 => report.push(
                    at,
                    format!(
                        "carga `{}`: o vagão `{}` não transporta carga",
                        cargo.id, wagon_id
                    ),
                ),
                Some(_) => {}
            }
        }
    }
    if !core
        .liveries
        .iter()
        .any(|l| l.required_stops == 0 && !l.retired)
    {
        report.push(at, "nenhuma pintura disponível desde o início");
    }
    for reward in &core.service_rewards {
        if reward.goal == 0 {
            report.push(at, format!("recompensa `{}`: meta zero", reward.id));
        }
    }
    if !core
        .locomotives
        .iter()
        .any(|l| l.unlocked_by.is_none() && !l.retired)
    {
        report.push(at, "nenhuma locomotiva disponível desde o início");
    }
    for locomotive in &core.locomotives {
        let here = format!("{at} › locomotiva `{}`", locomotive.id);
        if let Some(reward) = &locomotive.unlocked_by
            && !rewards.contains(reward.as_str())
        {
            report.push(&here, format!("a recompensa `{reward}` não existe"));
        }
        validate_sources(&mut report, &here, &locomotive.sources);
        if locomotive.kind == ContentKind::Real {
            if locomotive.builder.is_none()
                || locomotive.year.is_none()
                || locomotive.wheel_arrangement.is_none()
            {
                report.push(
                    &here,
                    "locomotiva real precisa de fabricante, ano e arranjo de rodas",
                );
            }
            if locomotive.sources.is_empty() {
                report.push(&here, "locomotiva real precisa de ao menos uma fonte");
            }
        }
        if locomotive.year.is_some_and(|y| !(1800..=2100).contains(&y)) {
            report.push(&here, "ano fora de 1800–2100");
        }
    }
    report.errors
}

/// Checks one line document. An empty result means the line may join the session.
pub fn validate_line(line: &LineDef) -> Vec<CatalogError> {
    let mut report = Report { errors: Vec::new() };
    let at = format!("linha `{}`", line.id);
    let at = at.as_str();
    if line.line_format != 1 {
        report.push(
            at,
            format!("formato de linha {} não é compreendido", line.line_format),
        );
    }
    if !valid_id(&line.id) {
        report.push(at, "identificador inválido");
    }
    if line.name.trim().is_empty() {
        report.push(at, "falta o nome");
    }
    let playable = line.status == LineStatus::Playable;
    let real = line.kind == ContentKind::Real;
    validate_sources(&mut report, at, &line.sources);
    let sources: BTreeSet<&str> = line.sources.iter().map(|s| s.id.as_str()).collect();
    let stations = report.ids(
        at,
        "estação",
        line.stations
            .iter()
            .map(|s| (s.id.as_str(), &s.aliases[..])),
    );
    report.ids(
        at,
        "trecho",
        line.segments.iter().map(|s| (s.id.as_str(), &[][..])),
    );
    let active: Vec<&StationDef> = line.stations.iter().filter(|s| !s.retired).collect();
    if active.len() < 2 {
        report.push(at, "uma linha precisa de ao menos duas estações ativas");
    }
    if let Some(era) = &line.era
        && (era.from_year > era.to_year || !(1800..=2100).contains(&era.from_year))
    {
        report.push(at, "época com anos invertidos ou fora de 1800–2100");
    }
    if real {
        if line.sources.is_empty() {
            report.push(at, "linha real precisa de ao menos uma fonte");
        }
        if playable && line.era.is_none() {
            report.push(at, "linha real jogável precisa de uma época definida");
        }
        if playable && line.gauge_mm.is_none() {
            report.push(at, "linha real jogável precisa da bitola");
        }
        let derived = line.sources.iter().any(|s| s.license.is_some());
        if derived && (line.data_license.is_none() || line.attribution.is_none()) {
            report.push(
                at,
                "dados derivados de fonte licenciada precisam de `data_license` e `attribution`",
            );
        }
    }
    let mut previous_km: Option<f64> = None;
    for (index, station) in line.stations.iter().enumerate() {
        let here = format!("{at} › estação `{}`", station.id);
        if station.name.trim().is_empty() {
            report.push(&here, "falta o nome");
        }
        for reference in &station.source_refs {
            if !sources.contains(reference.as_str()) {
                report.push(&here, format!("a fonte `{reference}` não existe"));
            }
        }
        if station.retired {
            continue;
        }
        match station.km {
            Some(km) if !km.is_finite() || !(0.0..=20_000.0).contains(&km) => {
                report.push(&here, "`km` deve estar entre 0 e 20000");
            }
            Some(km) => {
                if index == 0 && km != 0.0 {
                    report.push(&here, "a primeira estação fica no km 0");
                }
                if previous_km.is_some_and(|previous| km <= previous) {
                    report.push(&here, "`km` deve crescer na ordem das estações");
                }
                previous_km = Some(km);
            }
            None if real && playable => report.push(&here, "falta o `km`"),
            None => {}
        }
        if station
            .elevation_m
            .is_some_and(|e| !e.is_finite() || !(-500.0..=9_000.0).contains(&e))
        {
            report.push(&here, "altitude fora de −500 a 9000 m");
        }
        if real && playable && (!station.verified || station.source_refs.is_empty()) {
            report.push(
                &here,
                "estação de linha jogável precisa estar verificada e citar a fonte",
            );
        }
    }
    // Connectivity: segments follow the active stations in order, with a way back.
    let expected = match line.layout {
        Layout::Circuit => active.len(),
        Layout::Shuttle => active.len().saturating_sub(1),
    };
    if line.segments.len() != expected {
        report.push(
            at,
            format!(
                "{} estações ativas pedem {expected} trechos; há {}",
                active.len(),
                line.segments.len()
            ),
        );
    }
    for (index, segment) in line.segments.iter().enumerate() {
        let here = format!("{at} › trecho `{}`", segment.id);
        for end in [&segment.from, &segment.to] {
            if !stations.contains(end.as_str()) {
                report.push(&here, format!("a estação `{end}` não existe"));
            } else if line.stations.iter().any(|s| &s.id == end && s.retired) {
                report.push(&here, format!("a estação `{end}` está desativada"));
            }
        }
        if !active.is_empty() {
            let from = active[index % active.len()];
            let to = active[(index + 1) % active.len()];
            if index < expected && (segment.from != from.id || segment.to != to.id) {
                report.push(
                    &here,
                    format!(
                        "fora de ordem: esperado `{}` → `{}`; sem isso há estação inalcançável",
                        from.id, to.id
                    ),
                );
            }
            if let (Some(distance), Some(a), Some(b)) = (segment.distance_km, from.km, to.km)
                && line.layout == Layout::Shuttle
                && (distance - (b - a)).abs() > 0.05
            {
                report.push(
                    &here,
                    format!(
                        "`distance_km` {distance} difere da distância entre as estações ({:.2})",
                        b - a
                    ),
                );
            }
        }
        if !BIOMES.contains(&segment.biome.as_str()) {
            report.push(
                &here,
                format!(
                    "bioma `{}` desconhecido; use {}",
                    segment.biome,
                    BIOMES.join(", ")
                ),
            );
        }
        if segment
            .scenery
            .iter()
            .enumerate()
            .any(|(i, tag)| segment.scenery[..i].contains(tag))
        {
            report.push(&here, "etiqueta de cenário repetida");
        }
        for reference in &segment.source_refs {
            if !sources.contains(reference.as_str()) {
                report.push(&here, format!("a fonte `{reference}` não existe"));
            }
        }
        if segment
            .distance_km
            .is_some_and(|d| !d.is_finite() || d <= 0.0 || d > 5_000.0)
        {
            report.push(&here, "`distance_km` deve ser maior que 0 e até 5000");
        }
        match line.kind {
            ContentKind::Fictional => {
                match segment.base_units {
                    Some(units) if (200..=100_000).contains(&units) => {
                        if segment.variation_units.is_some_and(|v| v >= units) {
                            report.push(&here, "variação maior ou igual ao comprimento");
                        }
                    }
                    Some(_) => report.push(&here, "`base_units` deve estar entre 200 e 100000"),
                    None => report.push(&here, "trecho fictício precisa de `base_units`"),
                }
                if segment.adaptation != Adaptation::Invented {
                    report.push(&here, "trecho fictício deve declarar `invented`");
                }
            }
            ContentKind::Real => {
                if segment.base_units.is_some() || segment.variation_units.is_some() {
                    report.push(&here, "trecho real usa `distance_km`, não `base_units`");
                }
                if playable && segment.distance_km.is_none() {
                    report.push(&here, "falta `distance_km`");
                }
                if playable
                    && segment.adaptation != Adaptation::Invented
                    && segment.source_refs.is_empty()
                {
                    report.push(
                        &here,
                        "trecho fiel ou simplificado precisa citar ao menos uma fonte",
                    );
                }
            }
        }
    }
    if playable && !line.open_questions.is_empty() {
        report.push(at, "linha jogável não pode ter questões em aberto");
    }
    report.errors
}

impl Catalog {
    /// Parses and validates documents. `Err` only when the core cannot be used at all.
    pub fn from_documents(core: &str, lines: &[&str]) -> Result<Self, Vec<CatalogError>> {
        let core: CoreDocument = serde_json::from_str(core).map_err(|e| {
            vec![CatalogError {
                at: "catálogo".into(),
                message: format!("documento ilegível: {e}"),
            }]
        })?;
        let errors = validate_core(&core);
        if !errors.is_empty() {
            return Err(errors);
        }
        let mut accepted: Vec<LineDef> = Vec::new();
        let mut rejected = Vec::new();
        for (index, text) in lines.iter().enumerate() {
            let line: LineDef = match serde_json::from_str(text) {
                Ok(line) => line,
                Err(e) => {
                    rejected.push(CatalogError {
                        at: format!("linha nº {}", index + 1),
                        message: format!("documento ilegível: {e}"),
                    });
                    continue;
                }
            };
            let mut errors = validate_line(&line);
            if accepted.iter().any(|other| other.id == line.id) {
                errors.push(CatalogError {
                    at: format!("linha `{}`", line.id),
                    message: "identificador de linha repetido".into(),
                });
            }
            if errors.is_empty() {
                accepted.push(line);
            } else {
                rejected.append(&mut errors);
            }
        }
        Ok(Self {
            catalog_version: core.catalog_version,
            wagons: core.wagons,
            cargo: core.cargo,
            liveries: core.liveries,
            service_rewards: core.service_rewards,
            locomotives: core.locomotives,
            lines: accepted,
            rejected,
        })
    }

    /// The content shipped inside this build.
    pub fn builtin() -> &'static Result<Catalog, Vec<CatalogError>> {
        static BUILTIN: OnceLock<Result<Catalog, Vec<CatalogError>>> = OnceLock::new();
        BUILTIN.get_or_init(|| Self::from_documents(CORE, &LINES))
    }

    pub fn line(&self, id: &str) -> Option<&LineDef> {
        self.lines.iter().find(|line| line.id == id)
    }

    /// Lines the player may choose. Drafts are never listed.
    pub fn playable_lines(&self) -> impl Iterator<Item = &LineDef> {
        self.lines
            .iter()
            .filter(|line| line.status == LineStatus::Playable)
    }

    pub fn resolve_wagon(&self, id: &str) -> Resolution<'_> {
        resolve(
            id,
            self.wagons
                .iter()
                .map(|w| (w.id.as_str(), &w.aliases[..], w.retired)),
        )
    }
    pub fn resolve_cargo(&self, id: &str) -> Resolution<'_> {
        resolve(
            id,
            self.cargo
                .iter()
                .map(|c| (c.id.as_str(), &c.aliases[..], c.retired)),
        )
    }
    pub fn resolve_livery(&self, id: &str) -> Resolution<'_> {
        resolve(
            id,
            self.liveries
                .iter()
                .map(|l| (l.id.as_str(), &l.aliases[..], l.retired)),
        )
    }
    pub fn resolve_service_reward(&self, id: &str) -> Resolution<'_> {
        resolve(
            id,
            self.service_rewards
                .iter()
                .map(|r| (r.id.as_str(), &r.aliases[..], r.retired)),
        )
    }
    pub fn resolve_locomotive(&self, id: &str) -> Resolution<'_> {
        resolve(
            id,
            self.locomotives
                .iter()
                .map(|l| (l.id.as_str(), &l.aliases[..], l.retired)),
        )
    }
}

impl LineDef {
    pub fn resolve_station(&self, id: &str) -> Resolution<'_> {
        resolve(
            id,
            self.stations
                .iter()
                .map(|s| (s.id.as_str(), &s.aliases[..], s.retired)),
        )
    }

    /// Credits that must accompany the line wherever it is shown.
    pub fn credits(&self) -> Option<&str> {
        self.attribution.as_deref()
    }
}
