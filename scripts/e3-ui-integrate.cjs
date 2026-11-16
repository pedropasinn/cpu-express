const fs = require('fs');
// One-time source transformation retained as implementation history; never reapply it.
if (fs.readFileSync('src/app.rs', 'utf8').includes('.service_view()')) {
  console.log('Adaptadores E3 já aplicados. Nenhum arquivo alterado.');
  process.exit(0);
}
const edit = (p, fn) => fs.writeFileSync(p, fn(fs.readFileSync(p, 'utf8')));
for (const p of ['src/app.rs', ...fs.readdirSync('src/app').filter(p => p.endsWith('.rs')).map(p => `src/app/${p}`)]) {
  edit(p, t => t.replaceAll('.service_snapshot()', '.service_view()').replaceAll('.service_state()', '.service_view()'));
}
for (const p of ['src/app/missions.rs', 'src/app/shop.rs', 'src/app/machine.rs']) {
  edit(p, t => t.replaceAll('.journey_snapshot()', '.economic_snapshot()'));
}
edit('src/app/missions.rs', t => t.replaceAll('e.missions(j, s)', 'self.world.missions()'));
edit('src/app/branches.rs', t => t.replaceAll('self.world.journey_snapshot().next_station.name','self.world.travel_view().next_name').replaceAll('self.world.journey_snapshot().segment_progress', 'self.world.travel_view().segment_progress').replace('let branch = self.world.excursion_snapshot();', 'if self.world.active_session().is_some() { ui.label("Ramais disponíveis somente no Circuito Aurora."); return; }\n        let branch = self.world.excursion_snapshot();'));
edit('src/app/scenery.rs', t => t.replace('crate::simulation::route::CIRCUIT_LINE_ID,\n            self.world.journey_snapshot().route_segment.index,\n            true,', 'self.world.active_line_id(),\n            self.world.travel_view().segment,\n            self.world.travel_view().forward,').replace('let snapshot = self.world.journey_snapshot();','let snapshot = self.world.travel_view();'));
edit('src/app.rs', t => t.replace(/SceneContext::from_journey\(\s*&self.world.journey_snapshot\(\),?\s*\)/g,'SceneContext::from_world(&self.world)').replaceAll('self.world.station_wait() / 8.0', 'self.world.station_wait() / self.world.station_dwell()').replaceAll('self.world.journey_snapshot().next_station.name', 'self.world.travel_view().next_name').replaceAll('self.world.journey_snapshot().segment_progress', 'self.world.travel_view().segment_progress').replaceAll('journey.distance_to_next_station_units / 24.0', 'self.world.travel_view().distance_to_next / 24.0'));
edit('src/app/menus.rs', t => t.replace('j.distance_to_next_station_units / 24.0','self.world.travel_view().distance_to_next / 24.0').replace('fn map_page(&mut self, ui: &mut egui::Ui) {','fn map_page(&mut self, ui: &mut egui::Ui) {\n        if self.world.active_session().is_some() { self.line_map_page(ui); return; }'));
for (const p of ['src/app/menus.rs','src/app/journey.rs']) {
 edit(p, t => t.replaceAll('p.destination == station', 'p.destination == crate::simulation::line_session::CatalogStationId::aurora(station)'));
}
edit('src/rendering/stations.rs', t => t.replace('let journey = world.journey_snapshot();','let journey = world.travel_view();').replaceAll('journey.route_segment.length_units','journey.segment_length').replaceAll('journey.route_segment.from.index()','journey.from_index').replaceAll('journey.route_segment.to.index()','journey.to_index').replaceAll('journey.route_segment.biome_hint','journey.biome').replaceAll('journey.next_biome_hint','journey.next_biome').replaceAll('journey.distance_to_next_station_units','journey.distance_to_next').replaceAll('((8.0 - world.station_wait()) / 8.0)','((world.station_dwell() - world.station_wait()) / world.station_dwell())'));
