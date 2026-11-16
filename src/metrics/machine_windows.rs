//! Aggregate Windows counters. Fixed storage; no process, file or key inspection.
use crate::simulation::machine::MachineSignals;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows_sys::{
    Win32::{
        System::{
            Performance::*,
            Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS},
            SystemInformation::GetTickCount,
        },
        UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    },
    core::w,
};

pub(super) struct MachineCollector {
    query: PDH_HQUERY,
    disk: PDH_HCOUNTER,
    network: PDH_HCOUNTER,
    buffer: [u64; 8192],
    input_enabled: Arc<AtomicBool>,
    last_input: Option<u32>,
}

// SAFETY: PDH handles are not thread-affine. Ownership moves as a unit; all calls
// require exclusive access and Drop closes the sole owned query. No Sync is exposed.
unsafe impl Send for MachineCollector {}

impl MachineCollector {
    pub fn new(input_enabled: Arc<AtomicBool>) -> Self {
        let mut collector = Self {
            query: std::ptr::null_mut(),
            disk: std::ptr::null_mut(),
            network: std::ptr::null_mut(),
            buffer: [0; 8192],
            input_enabled,
            last_input: None,
        };
        // SAFETY: output handles are valid writable storage; constant paths are NUL terminated.
        unsafe {
            if PdhOpenQueryW(std::ptr::null(), 0, &mut collector.query) == 0 {
                PdhAddEnglishCounterW(
                    collector.query,
                    w!("\\PhysicalDisk(_Total)\\Disk Bytes/sec"),
                    0,
                    &mut collector.disk,
                );
                PdhAddEnglishCounterW(
                    collector.query,
                    w!("\\Network Interface(*)\\Bytes Total/sec"),
                    0,
                    &mut collector.network,
                );
            }
        }
        collector
    }

    pub fn sample(&mut self) -> MachineSignals {
        let mut signals = MachineSignals::default();
        // SAFETY: query is owned by this collector and used only on its worker.
        if !self.query.is_null() && unsafe { PdhCollectQueryData(self.query) } == 0 {
            let mut value = PDH_FMT_COUNTERVALUE::default();
            // SAFETY: counter belongs to the live query; output storage is correctly sized.
            if !self.disk.is_null()
                // SAFETY: live owned counter and valid output storage, as described above.
                && unsafe {
                    PdhGetFormattedCounterValue(
                        self.disk,
                        PDH_FMT_DOUBLE,
                        std::ptr::null_mut(),
                        &mut value,
                    )
                } == 0
            {
                signals.disk_bytes_per_second = counter_value(value);
            }
            let mut bytes = std::mem::size_of_val(&self.buffer) as u32;
            let mut count = 0;
            let items = self
                .buffer
                .as_mut_ptr()
                .cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
            // SAFETY: u64 buffer has sufficient alignment and bytes advertises its actual capacity.
            // On insufficient capacity we return None; no resizing or names are read.
            if !self.network.is_null()
                // SAFETY: live owned counter and valid output storage, as described above.
                && unsafe {
                    PdhGetFormattedCounterArrayW(
                        self.network,
                        PDH_FMT_DOUBLE,
                        &mut bytes,
                        &mut count,
                        items,
                    )
                } == 0
                && count > 0
                && count as usize
                    <= std::mem::size_of_val(&self.buffer)
                        / std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>()
            {
                let mut total = Some(0.0);
                for i in 0..count as usize {
                    // SAFETY: successful PDH call initialized count items within the checked buffer.
                    let value = counter_value(unsafe { (*items.add(i)).FmtValue });
                    total = total.zip(value).map(|(a, b)| a + b);
                }
                signals.network_bytes_per_second = total;
            }
        }
        let mut power = SYSTEM_POWER_STATUS::default();
        // SAFETY: power points to writable storage of the required C layout.
        if unsafe { GetSystemPowerStatus(&mut power) } != 0 {
            signals.on_ac_power = match power.ACLineStatus {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            };
            if power.BatteryFlag != 255
                && power.BatteryFlag & 128 == 0
                && power.BatteryLifePercent <= 100
            {
                signals.battery_percent = Some(f32::from(power.BatteryLifePercent));
            }
        }
        if self.input_enabled.load(Ordering::Relaxed) {
            let mut input = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };
            // SAFETY: initialized structure has the documented size. No keyboard hooks are installed.
            if unsafe { GetLastInputInfo(&mut input) } != 0 {
                signals.input_active = self.last_input.map(|previous| previous != input.dwTime);
                self.last_input = Some(input.dwTime);
                // SAFETY: GetTickCount takes no pointers; wrapping subtraction handles its rollover.
                signals.input_idle_seconds =
                    Some(unsafe { GetTickCount() }.wrapping_sub(input.dwTime) / 1000);
            } else {
                self.last_input = None;
            }
        } else {
            self.last_input = None;
        }
        signals.validated()
    }
}

fn counter_value(value: PDH_FMT_COUNTERVALUE) -> Option<f32> {
    if value.CStatus != PDH_CSTATUS_VALID_DATA && value.CStatus != PDH_CSTATUS_NEW_DATA {
        return None;
    }
    // SAFETY: callers requested PDH_FMT_DOUBLE and checked the API return and CStatus.
    let number = unsafe { value.Anonymous.doubleValue } as f32;
    (number.is_finite() && number >= 0.0).then_some(number)
}

impl Drop for MachineCollector {
    fn drop(&mut self) {
        if !self.query.is_null() {
            // SAFETY: this is the sole owner; closing a query releases both counters.
            unsafe {
                PdhCloseQuery(self.query);
            }
        }
    }
}
