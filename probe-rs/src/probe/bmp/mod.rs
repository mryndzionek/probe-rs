use crate::probe::{DebugProbeInfo, DebugProbeType};
use rusb::{Device, UsbContext};
use std::time::Duration;

fn get_bmp_info(device: &Device<rusb::Context>) -> Option<DebugProbeInfo> {
    let timeout = Duration::from_millis(100);
    let d_desc = device.device_descriptor().ok()?;

    if d_desc.vendor_id() != 0x1d50 {
        None
    } else {
        let handle = device.open().ok()?;
        let language = handle.read_languages(timeout).ok()?.get(0).cloned()?;
        let prod_str = handle
            .read_product_string(language, &d_desc, timeout)
            .ok()?;
        let sn_str = handle
            .read_serial_number_string(language, &d_desc, timeout)
            .ok();

        if prod_str.starts_with("Black Magic Probe") {
            Some(DebugProbeInfo {
                identifier: prod_str,
                vendor_id: d_desc.vendor_id(),
                product_id: d_desc.product_id(),
                serial_number: sn_str,
                probe_type: DebugProbeType::BMP,
            })
        } else {
            None
        }
    }
}

pub(crate) fn list_bmp_devices() -> Vec<DebugProbeInfo> {
    match rusb::Context::new().and_then(|ctx| ctx.devices()) {
        Ok(devices) => devices
            .iter()
            .filter_map(|device| get_bmp_info(&device))
            .collect(),
        Err(_) => vec![],
    }
}
