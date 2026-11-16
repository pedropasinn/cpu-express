//! Local availability probe, without application, weather or persisted profile.
use cpu_express::metrics::{MetricsProvider, SysinfoProvider};
use std::time::Duration;

fn main() {
    let mut provider = SysinfoProvider::new().expect("Inicializar métricas");
    for _ in 0..4 {
        let sample = provider.sample();
        println!(
            "CPU={:?} RAM={:?} disco_Bps={:?} rede_Bps={:?} tomada={:?} bateria={:?} entrada_disponivel={}",
            sample.cpu_percent,
            sample.ram_percent,
            sample.machine.disk_bytes_per_second,
            sample.machine.network_bytes_per_second,
            sample.machine.on_ac_power,
            sample.machine.battery_percent,
            sample.machine.input_active.is_some()
        );
        std::thread::sleep(Duration::from_secs(1));
    }
}
