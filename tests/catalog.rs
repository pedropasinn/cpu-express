//! DATA-01..05: the catalogue is valid, mirrors today's rules and rejects bad content readably.
use cpu_express::{
    rendering::EngineStyle,
    simulation::{
        CargoKind, Route, ServiceReward, ServiceState, ServiceWagon, World,
        catalog::{
            CATALOG_VERSION, Catalog, ContentKind, Counter, Layout, LineStatus, Resolution,
            Traction,
        },
        objectives::Livery,
        service::{CARGO_POSTCARD_GOAL, PASSENGER_POSTCARD_GOAL},
    },
};
use serde_json::{Value, json};

const CORE: &str = include_str!("../data/catalog/core.json");
const AURORA: &str = include_str!("../data/lines/aurora-v1.json");
const PARANAGUA: &str = include_str!("../data/lines/curitiba-paranagua.json");
const VITORIA_MINAS: &str = include_str!("../data/lines/vitoria-minas.json");

fn builtin() -> &'static Catalog {
    Catalog::builtin().as_ref().expect("built-in catalogue")
}

/// The identifier the save format already uses for an enum value.
fn id<T: serde::Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

fn errors_of(core: &Value, lines: &[Value]) -> Vec<String> {
    let lines: Vec<String> = lines.iter().map(Value::to_string).collect();
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    match Catalog::from_documents(&core.to_string(), &lines) {
        Ok(catalog) => catalog.rejected.iter().map(ToString::to_string).collect(),
        Err(errors) => errors.iter().map(ToString::to_string).collect(),
    }
}

fn expect_error(core: &Value, line: &Value, parts: &[&str]) {
    let errors = errors_of(core, std::slice::from_ref(line));
    assert!(
        errors
            .iter()
            .any(|error| parts.iter().all(|part| error.contains(part))),
        "expected an error mentioning {parts:?}, got {errors:#?}"
    );
}

#[test]
fn builtin_catalogue_is_valid_and_loads_identically_every_time() {
    let catalog = builtin();
    assert_eq!(catalog.catalog_version, CATALOG_VERSION);
    assert_eq!(catalog.rejected, Vec::new());
    assert_eq!(catalog.lines.len(), 3);
    let again = Catalog::from_documents(CORE, &[AURORA, PARANAGUA, VITORIA_MINAS]).unwrap();
    assert_eq!(&again, catalog);
}

#[test]
fn wagons_and_cargo_mirror_the_service_rules() {
    let catalog = builtin();
    let wagons = [
        ServiceWagon::Passenger,
        ServiceWagon::Sleeper,
        ServiceWagon::Observation,
        ServiceWagon::Freight,
        ServiceWagon::Tank,
    ];
    assert_eq!(catalog.wagons.len(), wagons.len());
    for wagon in wagons {
        let definition = catalog
            .wagons
            .iter()
            .find(|w| w.id == id(wagon))
            .unwrap_or_else(|| panic!("{wagon:?} missing"));
        assert_eq!((definition.seats, definition.cargo_units), wagon.capacity());
    }
    let kinds = [CargoKind::Parcels, CargoKind::Supplies, CargoKind::Liquid];
    assert_eq!(catalog.cargo.len(), kinds.len());
    for kind in kinds {
        let definition = catalog.cargo.iter().find(|c| c.id == id(kind)).unwrap();
        for wagon in wagons {
            let mut state = ServiceState::migrated(World::default().journey_snapshot());
            state.composition = [wagon; 3];
            let expected = if definition.wagon == id(wagon) {
                wagon.capacity().1 * 3
            } else {
                0
            };
            assert_eq!(
                state.cargo_capacity(kind),
                expected,
                "{kind:?} in {wagon:?}"
            );
        }
    }
}

#[test]
fn rewards_liveries_and_locomotives_mirror_the_game() {
    let catalog = builtin();
    assert_eq!(catalog.liveries.len(), Livery::ALL.len());
    for livery in Livery::ALL {
        let definition = catalog
            .liveries
            .iter()
            .find(|l| l.id == id(livery))
            .unwrap();
        assert_eq!(definition.required_stops, livery.required_stops());
        assert_eq!(definition.name, livery.name());
    }
    for (reward, counter, goal) in [
        (
            ServiceReward::PassengerPostcard,
            Counter::PassengersDelivered,
            PASSENGER_POSTCARD_GOAL,
        ),
        (
            ServiceReward::CargoPostcard,
            Counter::CargoDelivered,
            CARGO_POSTCARD_GOAL,
        ),
    ] {
        let definition = catalog
            .service_rewards
            .iter()
            .find(|r| r.id == id(reward))
            .unwrap();
        assert_eq!((definition.counter, definition.goal), (counter, goal));
    }
    assert_eq!(catalog.service_rewards.len(), 2);
    assert_eq!(catalog.locomotives.len(), EngineStyle::ALL.len());
    for (engine, traction, unlock) in [
        (EngineStyle::Steam, Traction::Steam, None),
        (
            EngineStyle::Diesel,
            Traction::Diesel,
            Some(ServiceReward::CargoPostcard),
        ),
        (
            EngineStyle::Electric,
            Traction::Electric,
            Some(ServiceReward::PassengerPostcard),
        ),
    ] {
        let definition = catalog
            .locomotives
            .iter()
            .find(|l| l.id == id(engine))
            .unwrap();
        assert_eq!(definition.traction, traction);
        assert_eq!(definition.kind, ContentKind::Fictional);
        assert_eq!(definition.unlocked_by, unlock.map(id));
        assert_eq!(definition.name, engine.name());
    }
}

#[test]
fn aurora_line_mirrors_the_running_route_for_any_seed() {
    let line = builtin().line("aurora-v1").unwrap();
    assert_eq!(line.status, LineStatus::Playable);
    assert_eq!(line.layout, Layout::Circuit);
    assert_eq!(line.kind, ContentKind::Fictional);
    for seed in [0, 1, 0x4350_5520_4558_5052, u64::MAX, 987_654_321] {
        let route = Route::from_seed(seed);
        assert_eq!(line.name, route.name());
        assert_eq!(line.stations.len(), route.stations().len());
        for (definition, station) in line.stations.iter().zip(route.stations()) {
            assert_eq!(definition.id, id(station.id));
            assert_eq!(definition.name, station.name);
        }
        for (definition, segment) in line.segments.iter().zip(route.segments()) {
            assert_eq!(definition.from, id(segment.from));
            assert_eq!(definition.to, id(segment.to));
            assert_eq!(
                definition.biome,
                format!("{:?}", segment.biome_hint).to_lowercase()
            );
            let base = f64::from(definition.base_units.unwrap());
            let variation = f64::from(definition.variation_units.unwrap());
            assert!(
                (segment.length_units - base).abs() <= variation,
                "seed {seed}"
            );
        }
    }
}

#[test]
fn draft_line_is_checked_but_never_offered_to_the_player() {
    let catalog = builtin();
    let draft = catalog.line("curitiba-paranagua").unwrap();
    assert_eq!(draft.status, LineStatus::Draft);
    assert_eq!(draft.layout, Layout::Shuttle);
    assert_eq!(draft.segments.len(), draft.stations.len() - 1);
    assert!(!draft.open_questions.is_empty());
    let playable: Vec<&str> = catalog.playable_lines().map(|l| l.id.as_str()).collect();
    assert_eq!(playable, ["aurora-v1"]);

    // The researched data is complete: only the recorded open questions keep it a draft.
    let core: Value = serde_json::from_str(CORE).unwrap();
    let mut promoted: Value = serde_json::from_str(PARANAGUA).unwrap();
    promoted["status"] = json!("playable");
    let errors = errors_of(&core, std::slice::from_ref(&promoted));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(errors[0].contains("questões em aberto"));
    promoted["open_questions"] = json!([]);
    assert_eq!(
        errors_of(&core, std::slice::from_ref(&promoted)),
        Vec::<String>::new()
    );
    // Without its researched facts the same line could not be promoted.
    let mut stripped = promoted.clone();
    stripped["era"] = Value::Null;
    stripped["gauge_mm"] = Value::Null;
    stripped["stations"][3]["km"] = Value::Null;
    stripped["stations"][3]["verified"] = json!(false);
    stripped["segments"][0]["distance_km"] = Value::Null;
    for parts in [
        &["curitiba-paranagua", "época"][..],
        &["curitiba-paranagua", "bitola"],
        &["estação `morretes`", "km"],
        &["estação `morretes`", "verificada"],
        &["trecho `paranagua-alexandra`", "distance_km"],
    ] {
        expect_error(&core, &stripped, parts);
    }
}

#[test]
fn paranagua_line_matches_the_collected_sources() {
    let line = builtin().line("curitiba-paranagua").unwrap();
    assert_eq!(line.stations.len(), 14);
    assert_eq!(line.gauge_mm, Some(1000));
    assert_eq!(line.credits(), Some("© OpenStreetMap contributors"));
    assert_eq!(line.data_license.as_deref(), Some("ODbL-1.0"));
    let first = line.stations.first().unwrap();
    let last = line.stations.last().unwrap();
    assert_eq!((first.id.as_str(), first.km), ("paranagua", Some(0.0)));
    assert_eq!(last.id, "curitiba");
    // Independent sources give 110 km for the whole railway.
    let length = last.km.unwrap();
    assert!((108.0..=111.0).contains(&length), "{length}");
    let total: f64 = line.segments.iter().map(|s| s.distance_km.unwrap()).sum();
    assert!((total - length).abs() < 0.2, "{total} vs {length}");
    // The climb of the Serra do Mar: sea level at the port, the summit near Roça Nova.
    let elevation = |id: &str| {
        line.stations
            .iter()
            .find(|s| s.id == id)
            .unwrap()
            .elevation_m
            .unwrap()
    };
    assert!(elevation("paranagua") < 30.0 && elevation("morretes") < 30.0);
    assert!(elevation("roca-nova") > 900.0);
    assert!(line.stations.iter().all(|s| s.verified));
    // The viaduct recorded at km 62.21 by the photographic archive lies after Marumbi.
    let serra = line
        .segments
        .iter()
        .find(|s| s.id == "marumbi-cadeado")
        .unwrap();
    assert!(
        serra
            .scenery
            .contains(&cpu_express::simulation::catalog::SceneryTag::Viaduct)
    );
}

#[test]
fn invalid_core_is_refused_with_the_offending_identifier() {
    let aurora: Value = serde_json::from_str(AURORA).unwrap();
    let valid: Value = serde_json::from_str(CORE).unwrap();
    type Change = fn(&mut Value);
    let cases: [(Change, &[&str]); 8] = [
        (
            |c| c["wagons"][1]["id"] = json!("passenger"),
            &["vagão `passenger`", "mais de uma vez"],
        ),
        (
            |c| c["cargo"][2]["wagon"] = json!("hopper"),
            &["carga `liquid`", "`hopper` não existe"],
        ),
        (
            |c| c["cargo"][0]["wagon"] = json!("sleeper"),
            &["carga `parcels`", "não transporta carga"],
        ),
        (
            |c| c["wagons"][0]["seats"] = json!(0),
            &["vagão `passenger`", "sem lugares"],
        ),
        (
            |c| c["locomotives"][1]["unlocked_by"] = json!("gold_medal"),
            &["locomotiva `diesel`", "`gold_medal` não existe"],
        ),
        (
            |c| c["locomotives"][0]["kind"] = json!("real"),
            &["locomotiva `steam`", "fabricante, ano e arranjo de rodas"],
        ),
        (
            |c| c["catalog_version"] = json!(99),
            &["versão 99", "não é compreendida"],
        ),
        (
            |c| c["liveries"][0]["id"] = json!("Verde Aurora"),
            &["pintura `Verde Aurora`", "letras minúsculas"],
        ),
    ];
    for (change, parts) in cases {
        let mut core = valid.clone();
        change(&mut core);
        let lines = [aurora.to_string()];
        let result = Catalog::from_documents(&core.to_string(), &[&lines[0]]);
        let errors: Vec<String> = result
            .expect_err("invalid core must not load")
            .iter()
            .map(ToString::to_string)
            .collect();
        assert!(
            errors.iter().any(|e| parts.iter().all(|p| e.contains(p))),
            "{parts:?} not in {errors:#?}"
        );
    }
    assert!(Catalog::from_documents("{ not json", &[]).is_err());
}

#[test]
fn invalid_line_stays_out_of_the_session_and_the_rest_still_loads() {
    let core: Value = serde_json::from_str(CORE).unwrap();
    let valid: Value = serde_json::from_str(AURORA).unwrap();
    type Change = fn(&mut Value);
    let cases: [(Change, &[&str]); 9] = [
        (
            |l| l["stations"][2]["id"] = json!("pines"),
            &["linha `aurora-v1`", "estação `pines`", "mais de uma vez"],
        ),
        (
            |l| l["segments"][1]["to"] = json!("atlantis"),
            &["trecho `pines-ridge`", "`atlantis` não existe"],
        ),
        (
            // A circuit that never returns to its origin leaves the train stranded.
            |l| l["segments"][3]["to"] = json!("ridge"),
            &["trecho `harbor-aurora`", "fora de ordem", "inalcançável"],
        ),
        (
            |l| {
                l["segments"].as_array_mut().unwrap().pop();
            },
            &["4 estações ativas pedem 4 trechos; há 3"],
        ),
        (
            |l| l["segments"][0]["base_units"] = json!(5),
            &["trecho `aurora-pines`", "`base_units`"],
        ),
        (
            |l| l["segments"][0]["biome"] = json!("swamp"),
            &["trecho `aurora-pines`", "bioma `swamp`"],
        ),
        (
            |l| l["segments"][2]["source_refs"] = json!(["lost-book"]),
            &["trecho `ridge-harbor`", "fonte `lost-book` não existe"],
        ),
        (
            |l| l["line_format"] = json!(7),
            &["linha `aurora-v1`", "formato de linha 7"],
        ),
        (
            |l| l["stations"][1]["retired"] = json!(true),
            &["trecho `aurora-pines`", "`pines` está desativada"],
        ),
    ];
    for (change, parts) in cases {
        let mut line = valid.clone();
        change(&mut line);
        expect_error(&core, &line, parts);
        // The damaged line is dropped; a healthy one beside it is still available.
        let texts = [line.to_string(), PARANAGUA.to_owned()];
        let catalog = Catalog::from_documents(CORE, &[&texts[0], &texts[1]]).unwrap();
        assert!(catalog.line("aurora-v1").is_none(), "{parts:?}");
        assert!(catalog.line("curitiba-paranagua").is_some());
        assert!(!catalog.rejected.is_empty());
    }
    // Unknown fields and duplicated line identifiers are refused too.
    let mut unknown = valid.clone();
    unknown["speed_bonus"] = json!(3);
    expect_error(&core, &unknown, &["linha nº 1", "documento ilegível"]);
    let errors = errors_of(&core, &[valid.clone(), valid.clone()]);
    assert!(errors.iter().any(|e| e.contains("repetido")), "{errors:#?}");
}

#[test]
fn real_line_rules_cover_distances_sources_and_licence() {
    let core: Value = serde_json::from_str(CORE).unwrap();
    let line = json!({
        "line_format": 1, "id": "test-real", "name": "Linha de teste", "status": "playable",
        "kind": "real", "layout": "shuttle", "gauge_mm": 1000,
        "era": { "label": "Vapor", "from_year": 1885, "to_year": 1940 },
        "sources": [{ "id": "book", "title": "Livro", "accessed": "2026-11-14" }],
        "stations": [
            { "id": "a", "name": "A", "km": 0.0, "verified": true, "source_refs": ["book"] },
            { "id": "b", "name": "B", "km": 12.5, "verified": true, "source_refs": ["book"] },
            { "id": "c", "name": "C", "km": 40.0, "verified": true, "source_refs": ["book"] }
        ],
        "segments": [
            { "id": "a-b", "from": "a", "to": "b", "distance_km": 12.5, "biome": "coast",
              "scenery": ["port"], "adaptation": "faithful", "source_refs": ["book"] },
            { "id": "b-c", "from": "b", "to": "c", "distance_km": 27.5, "biome": "highland",
              "scenery": ["mountain", "tunnel"], "adaptation": "simplified", "source_refs": ["book"] }
        ]
    });
    assert_eq!(
        errors_of(&core, std::slice::from_ref(&line)),
        Vec::<String>::new()
    );

    type Change = fn(&mut Value);
    let cases: [(Change, &[&str]); 8] = [
        (
            |l| l["segments"][1]["distance_km"] = json!(30.0),
            &[
                "trecho `b-c`",
                "difere da distância entre as estações (27.50)",
            ],
        ),
        (
            |l| l["stations"][2]["km"] = json!(10.0),
            &["estação `c`", "`km` deve crescer"],
        ),
        (
            |l| l["stations"][0]["km"] = json!(3.0),
            &["estação `a`", "km 0"],
        ),
        (
            |l| l["sources"] = json!([]),
            &["linha `test-real`", "ao menos uma fonte"],
        ),
        (
            |l| l["segments"][0]["source_refs"] = json!([]),
            &["trecho `a-b`", "citar ao menos uma fonte"],
        ),
        (
            |l| l["era"] = json!({ "label": "x", "from_year": 1950, "to_year": 1900 }),
            &["linha `test-real`", "anos invertidos"],
        ),
        (
            |l| l["sources"][0]["license"] = json!("ODbL-1.0"),
            &["linha `test-real`", "`data_license` e `attribution`"],
        ),
        (
            |l| l["sources"][0]["accessed"] = json!("ontem"),
            &["fonte `book`", "AAAA-MM-DD"],
        ),
    ];
    for (change, parts) in cases {
        let mut changed = line.clone();
        change(&mut changed);
        expect_error(&core, &changed, parts);
    }

    // With licence and credit declared, OSM-derived data is accepted and exposes its credit.
    let mut derived = line.clone();
    derived["sources"][0]["license"] = json!("ODbL-1.0");
    derived["data_license"] = json!("ODbL-1.0");
    derived["attribution"] = json!("© OpenStreetMap contributors");
    let text = derived.to_string();
    let catalog = Catalog::from_documents(CORE, &[&text]).unwrap();
    assert_eq!(catalog.rejected, Vec::new());
    assert_eq!(
        catalog.line("test-real").unwrap().credits(),
        Some("© OpenStreetMap contributors")
    );
}

#[test]
fn renamed_retired_and_unknown_identifiers_have_distinct_outcomes() {
    let mut core: Value = serde_json::from_str(CORE).unwrap();
    core["wagons"][3]["id"] = json!("boxcar");
    core["wagons"][3]["aliases"] = json!(["freight"]);
    core["cargo"][0]["wagon"] = json!("boxcar");
    core["cargo"][1]["wagon"] = json!("boxcar");
    core["liveries"][3]["retired"] = json!(true);
    let mut line: Value = serde_json::from_str(AURORA).unwrap();
    line["stations"][3]["aliases"] = json!(["porto"]);
    let text = line.to_string();
    let catalog = Catalog::from_documents(&core.to_string(), &[&text]).unwrap();
    assert_eq!(catalog.rejected, Vec::new());

    assert_eq!(catalog.resolve_wagon("tank"), Resolution::Current("tank"));
    assert_eq!(
        catalog.resolve_wagon("freight"),
        Resolution::Renamed("boxcar")
    );
    assert_eq!(catalog.resolve_wagon("hopper"), Resolution::Unknown);
    // A retired livery stays recognisable so a save that owns it is not rejected.
    assert_eq!(
        catalog.resolve_livery("midnight"),
        Resolution::Retired("midnight")
    );
    assert_eq!(
        catalog.resolve_cargo("liquid"),
        Resolution::Current("liquid")
    );
    assert_eq!(
        catalog.resolve_service_reward("cargo_postcard"),
        Resolution::Current("cargo_postcard")
    );
    assert_eq!(catalog.resolve_locomotive("maglev"), Resolution::Unknown);
    let aurora = catalog.line("aurora-v1").unwrap();
    assert_eq!(
        aurora.resolve_station("porto"),
        Resolution::Renamed("harbor")
    );
    assert_eq!(
        aurora.resolve_station("ridge"),
        Resolution::Current("ridge")
    );

    // An old name may never be reused for something else.
    core["wagons"][0]["aliases"] = json!(["tank"]);
    let errors = errors_of(&core, &[]);
    assert!(
        errors
            .iter()
            .any(|e| e.contains("vagão `passenger`") && e.contains("`tank`")),
        "{errors:#?}"
    );
}

#[test]
fn every_identifier_already_stored_in_a_save_resolves_in_the_catalogue() {
    let catalog = builtin();
    let state = ServiceState::migrated(World::default().journey_snapshot());
    for wagon in state.composition {
        assert!(matches!(
            catalog.resolve_wagon(&id(wagon)),
            Resolution::Current(_)
        ));
    }
    for livery in &state.owned_rewards {
        assert!(matches!(
            catalog.resolve_livery(&id(*livery)),
            Resolution::Current(_)
        ));
    }
    let line = catalog.line("aurora-v1").unwrap();
    for station in Route::default().stations() {
        assert!(matches!(
            line.resolve_station(&id(station.id)),
            Resolution::Current(_)
        ));
    }
}

#[test]
fn vitoria_minas_draft_keeps_unmeasured_stretches_open() {
    let line = builtin().line("vitoria-minas").unwrap();
    assert_eq!(line.status, LineStatus::Draft);
    assert_eq!(line.stations.len(), 29);
    assert_eq!(line.stations.first().unwrap().id, "pedro-nolasco");
    assert_eq!(line.stations.last().unwrap().id, "belo-horizonte");
    // Distances are recorded only up to the first gap in the mapped tracks.
    let measured = line.stations.iter().take_while(|s| s.km.is_some()).count();
    assert_eq!(line.stations[measured - 1].id, "ipaba");
    assert!(line.stations[measured..].iter().all(|s| s.km.is_none()));
    assert!(line.stations.iter().all(|s| !s.verified));
    assert!(
        line.open_questions
            .iter()
            .any(|q| q.contains("interrompidos"))
    );
    // A line with open distances can never be offered or turned into an itinerary.
    assert!(
        cpu_express::simulation::itinerary::Itinerary::from_line(line, 480.0)
            .unwrap_err()
            .contains("sem comprimento definido")
    );
}
