//! Store and collection presentation. Only the domain draws and grants items.
use super::menus::{pager, rarity_style, tabs};
use super::*;
use crate::rendering::EngineStyle;
use crate::simulation::{
    economy::{CrateKind, EconomyAction, EconomyState, ItemId, Rarity},
    objectives::Livery,
};

#[derive(Default)]
pub(super) struct ShopState {
    tab: usize,
    category: usize,
    page: usize,
    collection_category: usize,
    collection_page: usize,
    info: Option<CrateKind>,
    info_page: usize,
    reveal: Option<(u64, f64)>,
    direct_result: Option<ItemId>,
    seen_transaction: u64,
    earned_at_visit: Option<u64>,
    pub visit_open: bool,
    pub visit_summary: Option<String>,
    gain: Option<(u64, Instant)>,
}

fn crate_sprite(kind: CrateKind) -> game::Sprite {
    match kind {
        CrateKind::GeneralYard => game::Sprite::CaixaPatio,
        CrateKind::CargoWorkshop => game::Sprite::CaixaOficina,
        CrateKind::PlatformComfort => game::Sprite::CaixaPlataforma,
        CrateKind::EngineGarage => game::Sprite::CaixaGaragem,
        CrateKind::PaintAtelier => game::Sprite::CaixaAtelie,
    }
}

fn category_of(item: &ItemId) -> usize {
    match item {
        ItemId::Wagon(_) => 0,
        ItemId::Locomotive(_) => 1,
        ItemId::Livery(_) => 2,
    }
}

fn item_name(item: &ItemId) -> &'static str {
    match item {
        ItemId::Wagon(id) => CoachStyle::ALL
            .iter()
            .find(|v| service_wagon(**v).id() == id)
            .map(|v| v.name()),
        ItemId::Locomotive(id) => EngineStyle::ALL
            .iter()
            .find(|v| v.id() == id)
            .map(|v| v.name()),
        ItemId::Livery(id) => Livery::ALL.iter().find(|v| v.id() == id).map(|v| v.name()),
    }
    .unwrap_or("Peça desconhecida")
}

fn chances(kind: CrateKind, misses: u32) -> String {
    let def = kind.definition().expect("validated built-in crate");
    let weights = def.effective_weights(misses);
    let sum: u32 = weights.iter().sum();
    Rarity::ALL
        .iter()
        .zip(weights)
        .filter(|(_, weight)| *weight > 0)
        .map(|(rarity, weight)| {
            format!(
                "{} {:.0}%",
                rarity_style(*rarity).0,
                f64::from(weight) * 100.0 / f64::from(sum)
            )
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

impl CpuExpressApp {
    /// Deterministic headless review fixtures, isolated from every saved profile.
    pub fn prepare_demo_store(&mut self, page: &str, index: usize) {
        if !self.demo_mode || self.progress_path.is_some() {
            return;
        }
        self.world = World::default();
        self.shop = ShopState::default();
        self.menu = menus::MenuState::default();
        self.show_controls = true;
        self.show_help = false;
        self.welcome = false;
        self.panel_tab = PanelTab::Shop;
        self.config.reduce_menu_motion = true;
        if page != "loja-inicial" {
            self.world
                .transact_economy(
                    |e| {
                        e.record("Saldo de demonstração", 5000, 0)?;
                        Ok(())
                    },
                    |_| Ok(()),
                )
                .unwrap();
        }
        match page {
            "chances" => {
                self.shop.info = Some(CrateKind::ALL[index % 5]);
            }
            "garantia" => {
                self.shop.info = Some(CrateKind::EngineGarage);
                self.world
                    .transact_economy(
                        |e| {
                            e.crate_misses.insert(CrateKind::EngineGarage, 5);
                            Ok(())
                        },
                        |_| Ok(()),
                    )
                    .unwrap();
            }
            "direta" | "colecao" => {
                self.shop.category = (index / 5).min(2);
                self.shop.page = index % 5;
                self.shop.tab = 1;
                if page == "colecao" {
                    self.panel_tab = PanelTab::Collection;
                    self.shop.collection_category = (index / 5).min(3);
                    self.shop.collection_page = index % 5;
                    self.menu.postcards = index % 5;
                }
            }
            "museu" | "repetida" => {
                self.world
                    .transact_economy(
                        |e| {
                            e.inventory.extend(EconomyState::collection_items());
                            Ok(())
                        },
                        |_| Ok(()),
                    )
                    .unwrap();
                if page == "museu" {
                    self.world
                        .economy_action(EconomyAction::BuyMuseumStamp, |_| Ok(()))
                        .unwrap();
                    self.shop.tab = 2;
                } else {
                    self.world
                        .economy_action(
                            EconomyAction::BuyCrate {
                                kind: CrateKind::ALL[index % 5],
                                free: false,
                            },
                            |_| Ok(()),
                        )
                        .unwrap();
                }
            }
            "revelacao" | "abertura" => {
                let base = self.world.clone();
                for seed in 1..=1000 {
                    let mut candidate = base.clone();
                    candidate
                        .transact_economy(
                            |e| {
                                e.random_state = seed;
                                Ok(())
                            },
                            |_| Ok(()),
                        )
                        .unwrap();
                    candidate
                        .economy_action(
                            EconomyAction::BuyCrate {
                                kind: CrateKind::GeneralYard,
                                free: true,
                            },
                            |_| Ok(()),
                        )
                        .unwrap();
                    if candidate
                        .economy()
                        .pending_purchase
                        .as_ref()
                        .unwrap()
                        .rarity
                        == Rarity::ALL[index % 4]
                    {
                        self.world = candidate;
                        break;
                    }
                }
                if page == "abertura" {
                    self.config.reduce_menu_motion = false;
                    if index == 1 {
                        self.shop.reveal = Some((
                            self.world
                                .economy()
                                .pending_purchase
                                .as_ref()
                                .unwrap()
                                .transaction_id,
                            -0.3,
                        ));
                    }
                }
            }
            "compra-concluida" => {
                let item = ItemId::Locomotive("rack_electric".into());
                self.world
                    .economy_action(EconomyAction::BuyDirect(item.clone()), |_| Ok(()))
                    .unwrap();
                self.shop.direct_result = Some(item);
            }
            "moedas" => {
                self.panel_tab = PanelTab::Drive;
                self.observe_economy();
                self.shop.earned_at_visit =
                    Some(self.world.economy().total_earned.saturating_sub(84));
                self.shop.gain = Some((4, Instant::now()));
            }
            "maquina" => {
                self.panel_tab = PanelTab::Drive;
                self.menu.drive = 5;
            }
            _ => {}
        }
    }

    pub(super) fn observe_economy(&mut self) {
        let economy = self.world.economy();
        if self.shop.earned_at_visit.is_none() {
            self.shop.earned_at_visit = Some(economy.total_earned);
            self.shop.seen_transaction = economy.next_transaction_id.saturating_sub(1);
        }
        let gain: u64 = economy
            .transactions
            .iter()
            .filter(|t| t.id > self.shop.seen_transaction && t.reason == "Chegada e serviços")
            .map(|t| t.credited)
            .sum();
        self.shop.seen_transaction = economy.next_transaction_id.saturating_sub(1);
        if gain > 0 {
            self.shop.gain = Some((gain, Instant::now()));
        }
        if !self.show_controls && !self.show_help {
            self.shop.visit_open = false;
        }
    }

    pub(super) fn begin_economy_visit(&mut self) {
        self.observe_economy();
        let economy = self.world.economy();
        if !self.shop.visit_open {
            let gain = economy
                .total_earned
                .saturating_sub(self.shop.earned_at_visit.unwrap_or(economy.total_earned));
            let ready = economy
                .daily_orders
                .iter()
                .filter(|o| {
                    o.claimable(
                        economy,
                        self.world.economic_snapshot(),
                        self.world.service_view(),
                    )
                })
                .count();
            self.shop.visit_summary = (gain > 0 || ready > 0).then(|| {
                format!("Desde a última visita nesta sessão: +{gain} réis · {ready} ordens prontas")
            });
            if economy.pending_purchase.is_some() && !self.welcome {
                self.panel_tab = PanelTab::Shop;
                self.show_help = false;
            }
            self.shop.visit_open = true;
        }
        self.shop.earned_at_visit = Some(economy.total_earned);
    }

    pub(super) fn paint_coin_gain(&self, ui: &egui::Ui, rect: egui::Rect, logical_width: f32) {
        if self.config.quiet_mode {
            return;
        }
        let Some((coins, started)) = self.shop.gain else {
            return;
        };
        let elapsed = started.elapsed().as_secs_f32();
        if elapsed >= 2.0 {
            return;
        }
        let rise = if self.config.reduce_menu_motion {
            0.0
        } else {
            elapsed * 8.0
        };
        // Match the renderer's left-anchored train and its centering below 320 columns.
        let scale = rect.width() / logical_width;
        let train_x = 210.0 + ((logical_width - 320.0) / 2.0).min(0.0);
        let center = rect.min + egui::vec2(train_x * scale, 18.0 * scale - rise);
        let painter = ui.painter_at(rect);
        let label = format!("+{coins}");
        let text = painter.layout_no_wrap(label, egui::FontId::proportional(17.0), game::CREAM);
        let badge = egui::Rect::from_center_size(center, text.size() + egui::vec2(28.0, 6.0));
        painter.rect_filled(badge, 3, game::RIBBON);
        game::sprite(
            &painter,
            egui::Rect::from_min_size(badge.min + egui::vec2(3.0, 3.0), egui::Vec2::splat(16.0)),
            game::Sprite::Moeda,
        );
        painter.galley(badge.min + egui::vec2(23.0, 3.0), text, game::CREAM);
        ui.ctx().request_repaint_after(Duration::from_millis(33));
    }

    fn item_texture(&self, item: &ItemId) -> &egui::TextureHandle {
        let gallery = self
            .stock_gallery
            .as_ref()
            .expect("gallery initialized on page entry");
        match item {
            ItemId::Wagon(id) => gallery.coach(
                *CoachStyle::ALL
                    .iter()
                    .find(|v| service_wagon(**v).id() == id)
                    .expect("catalogued wagon"),
            ),
            ItemId::Locomotive(id) => gallery.engine(
                *EngineStyle::ALL
                    .iter()
                    .find(|v| v.id() == id)
                    .expect("catalogued engine"),
            ),
            ItemId::Livery(id) => gallery.livery(
                *Livery::ALL
                    .iter()
                    .find(|v| v.id() == id)
                    .expect("catalogued paint"),
            ),
        }
    }

    pub(super) fn shop_page(&mut self, ui: &mut egui::Ui) {
        if self.stock_gallery.is_none() {
            self.stock_gallery = Some(StockGallery::new(ui.ctx()));
        }
        if self.world.economy().pending_purchase.is_some() {
            self.reveal_page(ui);
            return;
        }
        if let Some(item) = self.shop.direct_result.clone() {
            game::heading(ui, "Comprado!");
            self.result_item(ui, &item, "COMPRADO!");
            self.result_actions(ui, &item, None);
            return;
        }
        tabs(
            ui,
            "shop",
            &mut self.shop.tab,
            &["Remessas", "Compra direta", "Museu"],
        );
        if self.shop.tab == 1 {
            self.catalog_page(ui, true);
            return;
        }
        if self.shop.tab == 2 {
            self.museum_page(ui);
            return;
        }
        if let Some(kind) = self.shop.info {
            self.crate_info(ui, kind);
            return;
        }
        let mut action = None;
        for row in CrateKind::ALL.chunks(3) {
            ui.horizontal(|ui| {
                let width = (ui.available_width() - 20.0) / 3.0;
                if row.len() == 2 {
                    ui.add_space((width + 10.0) / 2.0);
                }
                ui.spacing_mut().item_spacing.x = 10.0;
                for &kind in row {
                    let def = kind.definition().expect("built-in crate");
                    let economy = self.world.economy();
                    let free = kind == CrateKind::GeneralYard && economy.welcome_crate_available;
                    let price = if free { 0 } else { def.price };
                    let misses = economy.crate_misses.get(&kind).copied().unwrap_or(0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, 212.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let card = egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(width, 212.0),
                            );
                            ui.painter().rect_filled(card, 3, game::CARD);
                            ui.painter().rect_stroke(
                                card,
                                3,
                                egui::Stroke::new(2.0, game::GOLD),
                                egui::StrokeKind::Inside,
                            );
                            let title = match kind {
                                CrateKind::GeneralYard => "Pátio geral",
                                CrateKind::CargoWorkshop => "Cargas e oficina",
                                CrateKind::PlatformComfort => "Plataforma",
                                CrateKind::EngineGarage => "Locomotivas",
                                CrateKind::PaintAtelier => "Ateliê",
                            };
                            game::pixel_label(ui, title, 1.5, game::INK).on_hover_text(kind.name());
                            let (r, response) = ui
                                .allocate_exact_size(egui::vec2(80.0, 80.0), egui::Sense::hover());
                            ui.painter().circle_filled(r.center(), 40.0, game::PAPER);
                            game::sprite(ui.painter(), r, crate_sprite(kind));
                            response.on_hover_text(kind.name());
                            let (bar, response) = ui
                                .allocate_exact_size(egui::vec2(180.0, 8.0), egui::Sense::hover());
                            let weights = def.effective_weights(misses);
                            let total = weights.iter().sum::<u32>() as f32;
                            let mut x = bar.left();
                            for (rarity, weight) in Rarity::ALL.into_iter().zip(weights) {
                                let w = bar.width() * weight as f32 / total;
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(
                                        egui::pos2(x, bar.top()),
                                        egui::vec2(w, 8.0),
                                    ),
                                    0,
                                    rarity_style(rarity).1,
                                );
                                x += w;
                            }
                            response.on_hover_text(chances(kind, misses));
                            ui.horizontal(|ui| {
                                ui.add_space(14.0);
                                ui.spacing_mut().item_spacing.x = 3.0;
                                for n in 0..def.guarantee {
                                    let (r, _) = ui.allocate_exact_size(
                                        egui::vec2(8.0, 10.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        r,
                                        1,
                                        if n < misses { game::GOLD } else { game::PAPER },
                                    );
                                }
                                if game::button(
                                    ui,
                                    ("info", kind as u8),
                                    Icon::Help,
                                    "i",
                                    false,
                                    egui::vec2(28.0, 28.0),
                                )
                                .on_hover_text(chances(kind, misses))
                                .clicked()
                                {
                                    self.shop.info = Some(kind);
                                    self.shop.info_page = 0;
                                }
                            });
                            let label = if free {
                                "Grátis".into()
                            } else {
                                format!("{price} réis")
                            };
                            if ui
                                .add_enabled_ui(
                                    economy.balance >= price && self.progress_path.is_some(),
                                    |ui| {
                                        game::button(
                                            ui,
                                            ("buy", kind as u8),
                                            Icon::Cargo,
                                            &label,
                                            free,
                                            egui::vec2(180.0, 44.0),
                                        )
                                    },
                                )
                                .inner
                                .on_hover_text(format!(
                                    "{}; raro garantido em {} aberturas. {}",
                                    kind.name(),
                                    def.guarantee.saturating_sub(misses),
                                    chances(kind, misses)
                                ))
                                .clicked()
                            {
                                action = Some(EconomyAction::BuyCrate { kind, free });
                            }
                        },
                    );
                }
            });
        }
        if let Some(action) = action {
            self.perform_economy_action(action);
        }
    }

    fn crate_info(&mut self, ui: &mut egui::Ui, kind: CrateKind) {
        let def = kind.definition().expect("built-in crate");
        ui.horizontal(|ui| {
            if game::action(ui, "< Remessas").clicked() {
                self.shop.info = None;
            }
            game::heading(ui, kind.name());
        });
        let misses = self
            .world
            .economy()
            .crate_misses
            .get(&kind)
            .copied()
            .unwrap_or(0);
        ui.label(format!("Próxima abertura: {}", chances(kind, misses)));
        ui.label(format!(
            "Raro{} garantido: faltam {}.",
            if def.weights[3] == 0 {
                ""
            } else {
                " ou épico"
            },
            def.guarantee.saturating_sub(misses)
        ));
        game::hint(
            ui,
            "Sorteia a raridade, depois uma peça ainda não possuída dessa faixa, se houver. Todas as peças da faixa têm chances iguais.",
        );
        ui.label(format!(
            "Repetida: devolve 20% ({} réis). A grátis não devolve réis.",
            def.price / 5
        ));
        game::hint(
            ui,
            "Peças da mesma raridade têm chances iguais. Raridade não dá bônus.",
        );
        let range = pager(
            ui,
            "crate-content",
            &mut self.shop.info_page,
            def.items.len(),
            6,
        );
        for item in &def.items[range] {
            ui.horizontal(|ui| {
                let texture = self.item_texture(item);
                ui.add(egui::Image::new(texture).fit_to_exact_size(texture.size_vec2()));
                ui.label(item_name(item));
                ui.colored_label(
                    rarity_style(item.rarity().unwrap_or_default()).1,
                    rarity_style(item.rarity().unwrap_or_default()).0,
                );
                if self.world.economy().owns(item) {
                    ui.small("Possuído");
                }
            });
        }
    }

    pub(super) fn collection_page(&mut self, ui: &mut egui::Ui) {
        if self.stock_gallery.is_none() {
            self.stock_gallery = Some(StockGallery::new(ui.ctx()));
        }
        tabs(
            ui,
            "collection",
            &mut self.shop.collection_category,
            &["Vagões", "Motores", "Pinturas", "Postais"],
        );
        if self.shop.collection_category == 3 {
            self.postcards_page(ui);
        } else {
            self.catalog_page(ui, false);
        }
    }

    fn catalog_page(&mut self, ui: &mut egui::Ui, buying: bool) {
        if buying {
            tabs(
                ui,
                "direct-category",
                &mut self.shop.category,
                &["Vagões", "Motores", "Pinturas"],
            );
        }
        let category = if buying {
            self.shop.category
        } else {
            self.shop.collection_category
        };
        let items: Vec<_> = EconomyState::collection_items()
            .into_iter()
            .filter(|i| category_of(i) == category)
            .collect();
        let economy = self.world.economy();
        game::pixel_label(
            ui,
            &format!(
                "{}/{} modelos",
                items.iter().filter(|i| economy.owns(i)).count(),
                items.len()
            ),
            1.5,
            game::INK,
        );
        let page = if buying {
            &mut self.shop.page
        } else {
            &mut self.shop.collection_page
        };
        let range = pager(ui, "catalog-page", page, items.len(), 6);
        let mut chosen = None;
        for row in items[range].chunks(3) {
            ui.columns(3, |cols| {
                for (col, item) in cols.iter_mut().zip(row) {
                    let owned = self.world.economy().owns(item);
                    let rarity = item.rarity().unwrap_or_default();
                    let (name, color) = rarity_style(rarity);
                    egui::Frame::new()
                        .fill(game::CARD)
                        .stroke(egui::Stroke::new(4.0, color))
                        .inner_margin(3)
                        .show(col, |ui| {
                            ui.spacing_mut().item_spacing.y = 3.0;
                            ui.set_min_width(ui.available_width());
                            egui::Frame::new()
                                .fill(color)
                                .inner_margin(2)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    game::pixel_label(ui, name, 1.5, game::CREAM);
                                });
                            ui.vertical_centered(|ui| {
                                let texture = self.item_texture(item);
                                let size = texture.size_vec2();
                                let scale = (160.0 / size.x).min(70.0 / size.y).floor();
                                ui.add(
                                    egui::Image::new(texture)
                                        .fit_to_exact_size(size * scale)
                                        .tint(if owned || buying {
                                            egui::Color32::WHITE
                                        } else {
                                            egui::Color32::from_gray(80)
                                        }),
                                );
                            });
                            ui.add(egui::Label::new(item_name(item)).truncate())
                                .on_hover_text(item_name(item));
                            let label = if buying && owned {
                                "Possuído".into()
                            } else if buying {
                                format!("{} réis", rarity.direct_price())
                            } else if owned {
                                "Ver no trem".into()
                            } else {
                                format!("{} réis", rarity.direct_price())
                            };
                            if ui
                                .add_enabled_ui(
                                    !buying
                                        || (!owned
                                            && self.world.economy().balance
                                                >= rarity.direct_price()
                                            && self.progress_path.is_some()),
                                    |ui| {
                                        game::button(
                                            ui,
                                            ("catalog-item", item_name(item)),
                                            if owned { Icon::Check } else { Icon::Lock },
                                            &label,
                                            false,
                                            egui::vec2(ui.available_width(), 36.0),
                                        )
                                    },
                                )
                                .inner
                                .clicked()
                            {
                                chosen = Some(item.clone());
                            }
                        });
                }
            });
        }
        if let Some(item) = chosen {
            if buying {
                if self.perform_economy_action(EconomyAction::BuyDirect(item.clone())) {
                    self.shop.direct_result = Some(item);
                }
            } else if self.world.economy().owns(&item) {
                self.panel_tab = PanelTab::Drive;
                self.menu.drive = category + 1;
                self.menu.stock[category] = stock_index(&item) / 6;
            } else {
                self.panel_tab = PanelTab::Shop;
                self.shop.tab = 1;
                self.shop.category = category;
                self.shop.page = items.iter().position(|i| i == &item).unwrap_or(0) / 6;
            }
        }
    }

    fn museum_page(&mut self, ui: &mut egui::Ui) {
        game::heading(ui, "Selos do museu");
        ui.horizontal(|ui| {
            for sprite in [
                game::Sprite::Moeda,
                game::Sprite::PilhaMoedas,
                game::Sprite::SacoMoedas,
            ] {
                let (r, _) = ui.allocate_exact_size(egui::Vec2::splat(64.0), egui::Sense::hover());
                game::sprite(ui.painter(), r, sprite);
            }
        });
        let e = self.world.economy();
        ui.label(format!(
            "Coleção de modelos: {} / {}",
            e.inventory.len(),
            EconomyState::collection_items().len()
        ));
        ui.label(if e.museum_stamps == 0 {
            "Nenhum selo ainda".into()
        } else {
            format!("Selos possuídos: nº 1 a nº {}", e.museum_stamps)
        });
        ui.label("Complete os 39 modelos para colecionar selos numerados.");
        game::hint(
            ui,
            "Selos de coleção: 120 réis cada, sem bônus ou manutenção.",
        );
        if ui
            .add_enabled(
                e.collection_complete() && e.balance >= 120 && self.progress_path.is_some(),
                game::ActionButton::new(format!(
                    "Comprar selo nº {} · 120 réis",
                    e.museum_stamps.saturating_add(1)
                )),
            )
            .clicked()
        {
            self.perform_economy_action(EconomyAction::BuyMuseumStamp);
        }
    }

    fn reveal_page(&mut self, ui: &mut egui::Ui) {
        let pending = self
            .world
            .economy()
            .pending_purchase
            .clone()
            .expect("pending on entry");
        let now = ui.input(|i| i.time);
        let start = match self.shop.reveal {
            Some((id, start)) if id == pending.transaction_id => start,
            _ => {
                self.shop.reveal = Some((pending.transaction_id, now));
                now
            }
        };
        let elapsed = if self.config.reduce_menu_motion {
            2.0
        } else {
            (now - start).max(0.0)
        };
        if elapsed < 1.2 {
            let (area, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 410.0),
                egui::Sense::hover(),
            );
            let painter = ui.painter_at(area);
            painter.rect_filled(area, 4, game::WOOD_DARK);
            let title = pending.crate_kind.name();
            game::display_text(
                &painter,
                egui::pos2(
                    area.center().x - game::display_text_size(title, 2.0).x / 2.0,
                    area.top() + 24.0,
                ),
                title,
                2.0,
                game::CREAM,
                game::WOOD_DARK,
            );
            let offset = if elapsed < 0.4 {
                (elapsed * 65.0).sin() as f32 * 3.0
            } else {
                0.0
            };
            let r = egui::Rect::from_center_size(
                area.center() + egui::vec2(offset, 12.0),
                egui::Vec2::splat(192.0),
            );
            if elapsed < 0.4 {
                game::sprite(&painter, r, crate_sprite(pending.crate_kind));
            } else {
                let split = r.top() + 64.0;
                let lift = ((elapsed - 0.4) / 0.2).clamp(0.0, 1.0) as f32 * 40.0;
                let color = rarity_style(pending.rarity).1;
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        r.center() - egui::vec2(0.0, 40.0),
                        egui::vec2(48.0, 180.0),
                    ),
                    0,
                    color.gamma_multiply(0.7),
                );
                let lid = painter.with_clip_rect(egui::Rect::from_min_max(
                    area.min,
                    egui::pos2(area.right(), split - lift),
                ));
                game::sprite(
                    &lid,
                    r.translate(egui::vec2(0.0, -lift)),
                    crate_sprite(pending.crate_kind),
                );
                let base = painter.with_clip_rect(egui::Rect::from_min_max(
                    egui::pos2(area.left(), split),
                    area.max,
                ));
                game::sprite(&base, r, crate_sprite(pending.crate_kind));
            }
            ui.horizontal(|ui| {
                ui.add_space(((ui.available_width() - 220.0) / 2.0).max(0.0));
                if game::button(
                    ui,
                    "open-now",
                    Icon::Cargo,
                    "Abrir!",
                    true,
                    egui::vec2(220.0, 56.0),
                )
                .clicked()
                {
                    self.shop.reveal = Some((pending.transaction_id, now - 1.2));
                }
            });
            ui.ctx().request_repaint_after(Duration::from_millis(33));
            return;
        }
        let state = if pending.duplicate {
            format!("Repetida · +{} réis", pending.refund)
        } else {
            "Nova!".into()
        };
        self.result_item(ui, &pending.item, &state);
        self.result_actions(ui, &pending.item, Some(pending.transaction_id));
    }

    fn result_item(&self, ui: &mut egui::Ui, item: &ItemId, state: &str) {
        let (rarity, color) = rarity_style(item.rarity().unwrap_or_default());
        let (area, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 332.0),
            egui::Sense::hover(),
        );
        let card = egui::Rect::from_center_size(area.center(), egui::vec2(360.0, 320.0));
        let painter = ui.painter_at(area);
        painter.rect_filled(area, 4, game::WOOD_DARK);
        painter.rect_filled(card, 4, game::CARD);
        painter.rect_stroke(
            card,
            4,
            egui::Stroke::new(6.0, color),
            egui::StrokeKind::Inside,
        );
        painter.rect_filled(
            egui::Rect::from_min_size(card.min, egui::vec2(360.0, 34.0)),
            2,
            color,
        );
        game::display_text(
            &painter,
            egui::pos2(
                card.center().x - game::display_text_size(rarity, 2.0).x / 2.0,
                card.top() + 8.0,
            ),
            rarity,
            2.0,
            game::CREAM,
            game::WOOD_DARK,
        );
        let texture = self.item_texture(item);
        let bounds =
            egui::Rect::from_min_size(card.min + egui::vec2(36.0, 60.0), egui::vec2(288.0, 144.0));
        let scale = (bounds.width() / texture.size_vec2().x)
            .min(bounds.height() / texture.size_vec2().y)
            .floor();
        painter.rect_filled(bounds.expand(8.0), 2, game::PAPER);
        painter.image(
            texture.id(),
            egui::Rect::from_center_size(bounds.center(), texture.size_vec2() * scale),
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        let name = item_name(item);
        let cell = (328.0 / game::pixel_text_size(name, 1.0).x)
            .min(2.0)
            .floor()
            .max(1.0);
        game::pixel_text(
            &painter,
            egui::pos2(
                card.center().x - game::pixel_text_size(name, cell).x / 2.0,
                card.top() + 224.0,
            ),
            name,
            cell,
            game::INK,
        );
        let duplicate = state.starts_with("Repetida");
        let badge = if duplicate { "REPETIDA" } else { state };
        game::display_text(
            &painter,
            egui::pos2(
                card.center().x - game::display_text_size(badge, 2.0).x / 2.0,
                card.top() + 258.0,
            ),
            badge,
            2.0,
            color,
            game::WOOD_DARK,
        );
        if duplicate {
            let pending = self
                .world
                .economy()
                .pending_purchase
                .as_ref()
                .expect("duplicate result");
            game::sprite(
                &painter,
                egui::Rect::from_min_size(
                    card.min + egui::vec2(120.0, 280.0),
                    egui::Vec2::splat(28.0),
                ),
                game::Sprite::Moeda,
            );
            game::display_text(
                &painter,
                card.min + egui::vec2(156.0, 286.0),
                &format!("+{}", pending.refund),
                2.0,
                game::INK,
                game::WOOD_DARK,
            );
        }
        painter.text(
            card.center(),
            egui::Align2::CENTER_CENTER,
            state,
            egui::FontId::proportional(12.0),
            egui::Color32::TRANSPARENT,
        );
    }

    pub(super) fn wagon_slots(&mut self, ui: &mut egui::Ui) {
        if self.stock_gallery.is_none() {
            self.stock_gallery = Some(StockGallery::new(ui.ctx()));
        }
        ui.horizontal(|ui| {
            ui.add_space(((ui.available_width() - 316.0) / 2.0).max(0.0));
            for slot in 0..3 {
                let response = game::button(
                    ui,
                    ("wagon-slot", slot),
                    Icon::Train,
                    "",
                    self.coach_slot == slot,
                    egui::vec2(100.0, 50.0),
                );
                let texture = self
                    .stock_gallery
                    .as_ref()
                    .unwrap()
                    .coach(self.config.visual.coaches[slot]);
                let image =
                    egui::Rect::from_center_size(response.rect.center(), texture.size_vec2() * 1.4);
                ui.painter().image(
                    texture.id(),
                    image,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::Role::Button,
                        ui.is_enabled(),
                        self.coach_slot == slot,
                        format!("Vagão {}", slot + 1),
                    )
                });
                if response
                    .on_hover_text(format!("Equipar no vagão {}", slot + 1))
                    .clicked()
                {
                    self.coach_slot = slot;
                }
            }
        });
    }

    fn result_actions(&mut self, ui: &mut egui::Ui, item: &ItemId, transaction: Option<u64>) {
        if matches!(item, ItemId::Wagon(_)) {
            self.wagon_slots(ui);
        } else {
            ui.add_space(50.0);
        }
        ui.add_space(11.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            ui.add_space(((ui.available_width() - 396.0) / 2.0).max(0.0));
            if game::button(
                ui,
                "result-equip",
                Icon::Train,
                "Equipar",
                true,
                egui::vec2(190.0, 48.0),
            )
            .on_hover_text("A ocupação atual precisa caber na nova composição")
            .clicked()
            {
                self.equip_store_item(item);
            }
            if game::button(
                ui,
                "result-keep",
                Icon::Save,
                "Guardar",
                false,
                egui::vec2(190.0, 48.0),
            )
            .clicked()
            {
                let confirmed = transaction.is_none_or(|transaction_id| {
                    self.perform_economy_action(EconomyAction::Reveal { transaction_id })
                });
                if confirmed {
                    self.shop.reveal = None;
                    self.shop.direct_result = None;
                }
            }
        });
    }

    fn equip_store_item(&mut self, item: &ItemId) {
        if !self.world.economy().owns(item) {
            return;
        }
        let index = stock_index(item);
        if let ItemId::Wagon(_) = item {
            let mut coaches = self.config.visual.coaches;
            coaches[self.coach_slot] = CoachStyle::ALL[index];
            let mut candidate = self.world.clone();
            let result = candidate
                .set_service_composition(coaches.map(service_wagon))
                .and_then(|()| {
                    let path = self
                        .progress_path
                        .as_ref()
                        .ok_or("Abra um perfil salvo para equipar.")?;
                    crate::progress::save_checkpoint(path, &candidate)
                });
            match result {
                Ok(warning) => {
                    self.world = candidate;
                    self.config.visual.coaches = coaches;
                    self.progress_notice = warning;
                }
                Err(error) => {
                    self.progress_notice = Some(error);
                    return;
                }
            }
        } else if matches!(item, ItemId::Locomotive(_)) {
            self.config.visual.engine = EngineStyle::ALL[index];
        } else {
            self.config.visual.livery = Livery::ALL[index];
        }
        self.preferences_changed();
        self.shop.visit_summary = None;
        self.notification = Some(format!("{} equipado.", item_name(item)));
        self.play_cue(crate::audio::AudioCue::Equip);
    }
}

fn stock_index(item: &ItemId) -> usize {
    match item {
        ItemId::Wagon(id) => CoachStyle::ALL
            .iter()
            .position(|v| service_wagon(*v).id() == id),
        ItemId::Locomotive(id) => EngineStyle::ALL.iter().position(|v| v.id() == id),
        ItemId::Livery(id) => Livery::ALL.iter().position(|v| v.id() == id),
    }
    .expect("validated built-in item")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(ctx: &egui::Context) -> CpuExpressApp {
        CpuExpressApp::new_demo(
            ctx,
            LaunchOptions {
                config: Config::default(),
                config_path: None,
                warning: None,
                cpu_override: Some(50.0),
                exit_after: None,
                diagnostics: None,
            },
        )
    }
    fn directory() -> PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ce-e6-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }
    fn frame(
        app: &mut CpuExpressApp,
        ctx: &egui::Context,
        time: f64,
        events: Vec<egui::Event>,
    ) -> Vec<egui::epaint::ClippedShape> {
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(704.0, 676.0),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| app.central_menu(ui),
        );
        let shapes = output.shapes.clone();
        output.drop_without_applying_deltas();
        shapes
    }
    fn text_rect(shape: &egui::Shape, label: &str) -> Option<egui::Rect> {
        match shape {
            egui::Shape::Text(t) if t.galley.job.text == label => Some(t.visual_bounding_rect()),
            egui::Shape::Vec(v) => v.iter().find_map(|s| text_rect(s, label)),
            _ => None,
        }
    }
    fn click(app: &mut CpuExpressApp, ctx: &egui::Context, label: &str) {
        click_nth(app, ctx, label, 0);
    }

    fn click_nth(app: &mut CpuExpressApp, ctx: &egui::Context, label: &str, index: usize) {
        let mut pos = None;
        let time = ctx.input(|i| i.time) + 0.01;
        for _ in 0..3 {
            pos = frame(app, ctx, time, vec![])
                .iter()
                .filter_map(|s| text_rect(&s.shape, label))
                .nth(index)
                .map(|r| r.center())
                .or(pos);
        }
        let pos = pos.unwrap_or_else(|| panic!("controle não encontrado: {label}"));
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                time,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }

    #[test]
    fn ganhar_reis_comprar_caixa_equipar_salvar_e_reabrir_preserva_tudo() {
        use eframe::App;
        for (kind, occurrence) in [
            (CrateKind::PlatformComfort, 1),
            (CrateKind::EngineGarage, 0),
            (CrateKind::PaintAtelier, 0),
        ] {
            let ctx = egui::Context::default();
            let mut app = app(&ctx);
            let dir = directory();
            let config_path = dir.join("config.json");
            let journey_path = crate::progress::path_for(&config_path);
            app.config_path = Some(config_path.clone());
            app.progress_path = Some(journey_path.clone());
            app.welcome = false;
            app.config.reduce_menu_motion = true;
            let price = kind.definition().unwrap().price;
            let initial = app.world.economy().balance;
            // Earn through the same arrival/checkpoint adapter used by the app.
            // No wallet injection, mission fixture or free crate funds this purchase.
            for _ in 0..40_000 {
                crate::progress::advance_checkpoint(
                    &journey_path,
                    &mut app.world,
                    &mut app.train,
                    50.0,
                    1.0,
                    &app.config,
                )
                .unwrap();
                if app.world.economy().balance >= price {
                    break;
                }
            }
            assert!(app.world.economy().balance >= price && app.world.economy().balance > initial);
            // Finish existing services so every category can be equipped without discarding cargo.
            app.world
                .set_boarding_closed(true, |candidate| {
                    crate::progress::save_checkpoint(&journey_path, candidate).map(|_| ())
                })
                .unwrap();
            for _ in 0..2_000 {
                if app.world.can_leave_line() {
                    break;
                }
                crate::progress::advance_checkpoint(
                    &journey_path,
                    &mut app.world,
                    &mut app.train,
                    50.0,
                    1.0,
                    &app.config,
                )
                .unwrap();
            }
            assert!(app.world.can_leave_line());
            let earned = app.world.economy().balance;
            app.panel_tab = PanelTab::Shop;
            click_nth(&mut app, &ctx, &format!("{price} réis"), occurrence);
            let pending = app
                .world
                .economy()
                .pending_purchase
                .clone()
                .expect("compra pela UI");
            assert_eq!(pending.crate_kind, kind);
            assert_eq!(app.world.economy().balance, earned - price);
            assert!(!pending.duplicate);
            match kind {
                CrateKind::PlatformComfort => assert!(matches!(pending.item, ItemId::Wagon(_))),
                CrateKind::EngineGarage => assert!(matches!(pending.item, ItemId::Locomotive(_))),
                CrateKind::PaintAtelier => assert!(matches!(pending.item, ItemId::Livery(_))),
                _ => unreachable!(),
            }
            click(&mut app, &ctx, "Equipar");
            assert!(app.progress_notice.is_none(), "{:?}", app.progress_notice);
            match &pending.item {
                ItemId::Wagon(_) => assert_eq!(
                    app.config.visual.coaches[app.coach_slot],
                    CoachStyle::ALL[stock_index(&pending.item)]
                ),
                ItemId::Locomotive(_) => assert_eq!(
                    app.config.visual.engine,
                    EngineStyle::ALL[stock_index(&pending.item)]
                ),
                ItemId::Livery(_) => assert_eq!(
                    app.config.visual.livery,
                    Livery::ALL[stock_index(&pending.item)]
                ),
            }
            click(&mut app, &ctx, "Guardar");
            app.on_exit(None);
            assert!(
                app.preferences_notice.is_none(),
                "{:?}",
                app.preferences_notice
            );
            assert!(app.progress_notice.is_none(), "{:?}", app.progress_notice);
            let economy = app.world.economy().clone();
            let service = app.world.service_state().clone();
            let visual = app.config.visual.clone();
            let lines = app.world.line_progress().clone();
            let distance = app.world.journey_snapshot().total_distance_units;
            let checkpoint = std::fs::read(&journey_path).unwrap();
            drop(app);

            // Reconstruct the application via its production loader and both real files.
            let (config, warning) = config::load(Some(&config_path));
            assert!(warning.is_none());
            let reopened_ctx = egui::Context::default();
            let mut reopened = CpuExpressApp::new(
                &reopened_ctx,
                LaunchOptions {
                    config,
                    config_path: Some(config_path.clone()),
                    warning: None,
                    cpu_override: Some(50.0),
                    exit_after: None,
                    diagnostics: None,
                },
            );
            assert!(reopened.progress_notice.is_none());
            assert_eq!(reopened.world.economy(), &economy);
            assert_eq!(reopened.world.service_state(), &service);
            assert_eq!(reopened.config.visual, visual);
            assert_eq!(reopened.world.line_progress(), lines);
            assert!(
                (reopened.world.journey_snapshot().total_distance_units - distance).abs() < 1e-9
            );
            assert!(reopened.world.boarding_closed());
            assert!(reopened.world.economy().pending_purchase.is_none());
            for time in [0.0, 0.1, 0.2] {
                frame(&mut reopened, &reopened_ctx, time, vec![]);
            }
            assert_eq!(reopened.world.economy(), &economy);
            assert_eq!(std::fs::read(&journey_path).unwrap(), checkpoint);
            reopened.on_exit(None);
            assert_eq!(
                crate::progress::load_world(&journey_path)
                    .unwrap()
                    .unwrap()
                    .economy(),
                &economy
            );
        }
    }

    #[test]
    fn store_button_commits_before_reveal_and_recovers_exact_result() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.prepare_demo_store("loja-inicial", 0);
        let dir = directory();
        let path = dir.join("journey.json");
        app.progress_path = Some(path.clone());
        let original = app.world.economy().clone();
        // A failed writer must not consume the free box or the random draw.
        app.progress_path = Some(dir.clone());
        click(&mut app, &ctx, "Grátis");
        assert_eq!(app.world.economy(), &original);
        assert!(app.progress_notice.is_some());
        app.progress_path = Some(path.clone());
        click(&mut app, &ctx, "Grátis");
        let purchased = app.world.economy().clone();
        let pending = purchased.pending_purchase.clone().unwrap();
        assert_eq!(purchased.balance, original.balance);
        assert!(!purchased.welcome_crate_available);
        app.world = crate::progress::load_world(&path).unwrap().unwrap();
        app.shop = ShopState::default();
        app.panel_tab = PanelTab::Drive;
        app.config.reduce_menu_motion = false;
        frame(&mut app, &ctx, 10.0, vec![]);
        assert_eq!(app.panel_tab, PanelTab::Shop);
        assert_eq!(app.world.economy(), &purchased);
        frame(&mut app, &ctx, 10.9, vec![]);
        let output = frame(&mut app, &ctx, 11.3, vec![]);
        assert!(
            output
                .iter()
                .any(|s| text_rect(&s.shape, "Guardar").is_some())
        );
        assert_eq!(app.world.economy(), &purchased);
        app.config.reduce_menu_motion = true;
        app.progress_path = Some(dir);
        click(&mut app, &ctx, "Guardar");
        assert_eq!(app.world.economy(), &purchased);
        app.progress_path = Some(path.clone());
        click(&mut app, &ctx, "Guardar");
        assert!(app.world.economy().pending_purchase.is_none());
        assert_eq!(app.world.economy().balance, purchased.balance);
        assert!(app.world.economy().owns(&pending.item));
        assert_eq!(
            crate::progress::load_world(&path)
                .unwrap()
                .unwrap()
                .economy(),
            app.world.economy()
        );
    }

    #[test]
    fn direct_purchase_and_equipment_buttons_keep_owned_model_after_reopening() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.prepare_demo_store("direta", 0);
        let path = directory().join("journey.json");
        app.progress_path = Some(path.clone());
        let before = app.world.economy().balance;
        click(&mut app, &ctx, "240 réis");
        let item = app.shop.direct_result.clone().unwrap();
        assert_eq!(app.world.economy().balance, before - 240);
        assert!(app.world.economy().owns(&item));
        let original_composition = app.world.service_view().composition;
        let original_visual = app.config.visual.coaches;
        app.progress_path = Some(path.parent().unwrap().to_owned());
        click(&mut app, &ctx, "Equipar");
        assert_eq!(app.world.service_view().composition, original_composition);
        assert_eq!(app.config.visual.coaches, original_visual);
        app.progress_path = Some(path.clone());
        click(&mut app, &ctx, "Equipar");
        let reopened = crate::progress::load_world(&path).unwrap().unwrap();
        assert_eq!(
            reopened.service_view().composition[0],
            service_wagon(CoachStyle::ALL[stock_index(&item)])
        );
        assert_eq!(reopened.economy(), app.world.economy());
        click(&mut app, &ctx, "Guardar");
        assert!(app.shop.direct_result.is_none());
        assert!(!app.perform_economy_action(EconomyAction::BuyDirect(item)));
        assert_eq!(app.world.economy().balance, before - 240);
    }

    #[test]
    fn every_store_and_collection_page_fits_and_keeps_exit_visible() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        for (page, pages) in [
            ("loja-inicial", 1),
            ("loja", 1),
            ("chances", 5),
            ("garantia", 1),
            ("direta", 15),
            ("colecao", 17),
            ("museu", 1),
            ("revelacao", 4),
            ("repetida", 5),
            ("abertura", 2),
            ("compra-concluida", 1),
            ("moedas", 1),
            ("maquina", 1),
        ] {
            for index in 0..pages {
                app.prepare_demo_store(page, index);
                for time in [0.0, 0.4, 0.9, 1.3] {
                    let output = frame(&mut app, &ctx, time, vec![]);
                    let close = output
                        .iter()
                        .find_map(|s| text_rect(&s.shape, "Fechar"))
                        .unwrap();
                    assert!(close.bottom() < 676.0, "{page}/{index}: {close:?}");
                    let h = ctx
                        .data(|d| d.get_temp::<f32>(egui::Id::unique("central-content-height")))
                        .unwrap();
                    let (w, limit) = ctx
                        .data(|d| {
                            d.get_temp::<(f32, f32)>(egui::Id::unique("central-content-width"))
                        })
                        .unwrap();
                    assert!(h <= 492.1, "{page}/{index}: {h}");
                    assert!(w <= limit + 0.1, "{page}/{index}: {w}>{limit}");
                    if page == "loja-inicial" {
                        let free = output
                            .iter()
                            .find_map(|s| text_rect(&s.shape, "Grátis"))
                            .expect("remessa gratuita visível");
                        assert!(free.bottom() < 590.0 && free.left() > 0.0);
                        for price in ["225 réis", "450 réis", "188 réis"] {
                            assert!(
                                output.iter().any(|s| text_rect(&s.shape, price).is_some()),
                                "{price}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn guarantee_screen_uses_normalized_actual_weights_and_no_imaginary_epic() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.prepare_demo_store("garantia", 0);
        let output = frame(&mut app, &ctx, 0.0, vec![]);
        assert!(output.iter().any(|s| {
            text_rect(&s.shape, "Próxima abertura: Raro 87% · Épico 13%").is_some()
        }));
        for (kind, rare) in [
            (CrateKind::PlatformComfort, "11"),
            (CrateKind::PaintAtelier, "10"),
        ] {
            assert!(chances(kind, 0).contains(&format!("Raro {rare}%")));
            assert!(chances(kind, 7).contains("Raro 100%"));
        }
    }

    #[test]
    fn duplicate_refund_and_museum_button_do_not_repeat_on_repaint() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.prepare_demo_store("repetida", 4);
        let path = directory().join("journey.json");
        app.progress_path = Some(path.clone());
        crate::progress::save_checkpoint(&path, &app.world).unwrap();
        let before = app.world.economy().clone();
        assert_eq!(before.pending_purchase.as_ref().unwrap().refund, 37);
        for time in [0.0, 0.1, 0.2] {
            let output = frame(&mut app, &ctx, time, vec![]);
            assert!(
                output
                    .iter()
                    .any(|s| text_rect(&s.shape, "Repetida · +37 réis").is_some())
            );
            assert_eq!(app.world.economy(), &before);
        }
        click(&mut app, &ctx, "Guardar");
        app.shop.tab = 2;
        let balance = app.world.economy().balance;
        app.progress_path = Some(path.parent().unwrap().to_owned());
        click(&mut app, &ctx, "Comprar selo nº 1 · 120 réis");
        assert_eq!(app.world.economy().museum_stamps, 0);
        assert_eq!(app.world.economy().balance, balance);
        app.progress_path = Some(path.clone());
        click(&mut app, &ctx, "Comprar selo nº 1 · 120 réis");
        assert_eq!(app.world.economy().museum_stamps, 1);
        assert_eq!(app.world.economy().balance, balance - 120);
        assert_eq!(
            crate::progress::load_world(&path)
                .unwrap()
                .unwrap()
                .economy(),
            app.world.economy()
        );
    }

    #[test]
    fn arrival_notice_uses_committed_income_and_visits_do_not_regrant_rewards() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.show_controls = false;
        app.observe_economy();
        let before = app.world.economy().total_earned;
        for _ in 0..300 {
            app.world
                .advance_expedition(&mut app.train, 50.0, 1.0, &app.config);
        }
        app.observe_economy();
        assert!(app.shop.gain.is_some_and(|(v, _)| v > 0));
        let economy = app.world.economy().clone();
        app.show_controls = true;
        app.begin_economy_visit();
        assert!(
            app.shop
                .visit_summary
                .as_ref()
                .unwrap()
                .contains(&format!("+{} réis", economy.total_earned - before))
        );
        app.observe_economy();
        app.begin_economy_visit();
        assert_eq!(app.world.economy(), &economy);
        app.dispatch(AppAction::ClosePanel, &ctx);
        app.dispatch(AppAction::ToggleControls, &ctx);
        app.begin_economy_visit();
        assert!(app.shop.visit_summary.is_none());
    }
}
