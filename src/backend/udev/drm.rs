use anyhow::Result;
use smithay::{
    backend::drm::DrmDevice,
    reexports::drm::control::{
        AtomicCommitFlags,
        Device as ControlDevice,
        ResourceHandle,
        atomic::AtomicModeReq,
        crtc,
        connector::{
            self,
            State as ConnectorState,
        },
        dumbbuffer::DumbBuffer,
        property,
    },
};
use std::{
    collections::HashMap,
    os::unix::io::AsRawFd,
};

pub fn display_configuration<A: AsRawFd>(device: &mut DrmDevice<A>) -> Result<HashMap<connector::Handle, crtc::Handle>> {
    let res_handles = device.resource_handles()?;
    let connectors = res_handles.connectors();

    let mut map = HashMap::new();
    let mut cleanup = Vec::new();
    // We expect the previous running drm master (likely the login mananger)
    // to leave the drm device in a sensible state.
    // That means, to reduce flickering, we try to keep an established mapping.
    for conn in connectors
        .iter()
        .flat_map(|conn| device.get_connector(*conn).ok())
    {
        if let Some(enc) = device.get_connector(conn.handle())?.current_encoder() {
            if let Some(crtc) = device.get_encoder(enc)?.crtc() {
                // If is is connected we found a mapping
                if conn.state() == ConnectorState::Connected {
                    map.insert(conn.handle(), crtc);
                // If not, the user just unplugged something,
                // or the drm master did not cleanup?
                // Well, I guess we cleanup after them.
                } else {
                    cleanup.push(crtc);
                }
            }
        }
    }
    // But just in case we try to match all remaining connectors.
    for conn in connectors
        .iter()
        .flat_map(|conn| device.get_connector(*conn).ok())
        .filter(|conn| conn.state() == ConnectorState::Connected)
        .filter(|conn| !map.contains_key(&conn.handle()))
        .collect::<Vec<_>>().iter()
    {
        'outer: for encoder_info in conn
            .encoders()
            .iter()
            .filter_map(|e| *e)
            .flat_map(|encoder_handle| device.get_encoder(encoder_handle))
        {
            for crtc in res_handles.filter_crtcs(encoder_info.possible_crtcs()) {
                if !map.values().any(|v| *v == crtc) {
                    map.insert(conn.handle(), crtc);
                    break 'outer;
                }
            }
        }
    }

    // And then cleanup
    if device.is_atomic() {
        let mut req = AtomicModeReq::new();
        let plane_handles = device.plane_handles()?;

        for conn in connectors
            .iter()
            .flat_map(|conn| device.get_connector(*conn).ok())
            .flat_map(|conn| conn.current_encoder())
            .flat_map(|enc| device.get_encoder(enc).ok())
            .flat_map(|enc| enc.crtc())
            .filter(|c| cleanup.contains(&c))
        {
            let crtc_id = get_prop(device, conn, "CRTC_ID")?;
            req.add_property(conn, crtc_id, property::Value::CRTC(None));
        }

        // We cannot just shortcut and use the legacy api for all cleanups because of this.
        // (Technically a device does not need to be atomic for planes to be used, but nobody does this otherwise.)
        for plane in plane_handles.planes() {
            let info = device.get_plane(*plane)?;
            if let Some(crtc) = info.crtc() {
                if cleanup.contains(&crtc) {
                    let crtc_id = get_prop(device, *plane, "CRTC_ID")?;
                    let fb_id = get_prop(device, *plane, "FB_ID")?;
                    req.add_property(*plane, crtc_id, property::Value::CRTC(None));
                    req.add_property(*plane, fb_id, property::Value::Framebuffer(None));
                }
            }
        }

        for crtc in cleanup {
            let mode_id = get_prop(device, crtc, "MODE_ID")?;
            let active = get_prop(device, crtc, "ACTIVE")?;
            req.add_property(crtc, active, property::Value::Boolean(false));
            req.add_property(crtc, mode_id, property::Value::Unknown(0));
        }

        device.atomic_commit(&[AtomicCommitFlags::AllowModeset], req)?;
    } else {
        for crtc in cleanup {
            #[allow(deprecated)]
            let _ = device.set_cursor(crtc, Option::<&DumbBuffer>::None);
            // null commit (necessary to trigger removal on the kernel side with the legacy api.)
            let _ = device.set_crtc(crtc, None, (0, 0), &[], None);
        }
    }

    Ok(map)
}

pub fn get_prop<A, T>(device: &DrmDevice<A>, handle: T, name: &str) -> Result<property::Handle>
    where
        A: AsRawFd,
        T: ResourceHandle
{
    let props = device.get_properties(handle)?;
    let (prop_handles, _) = props.as_props_and_values();
    for prop in prop_handles {
        let info = device.get_property(*prop)?;
        if Some(name) == info.name().to_str().ok() {
            return Ok(*prop);
        }
    }
    anyhow::bail!("No prop found")
}

pub fn get_manufacturer(vendor: &[char; 3]) -> &'static str {
	match vendor {
	    ['A', 'A', 'A'] => "Avolites Ltd",
	    ['A', 'C', 'I'] => "Ancor Communications Inc",
	    ['A', 'C', 'R'] => "Acer Technologies",
	    ['A', 'D', 'A'] => "Addi-Data GmbH",
	    ['A', 'P', 'P'] => "Apple Computer Inc",
	    ['A', 'S', 'K'] => "Ask A/S",
	    ['A', 'V', 'T'] => "Avtek (Electronics) Pty Ltd",
	    ['B', 'N', 'O'] => "Bang & Olufsen",
	    ['B', 'N', 'Q'] => "BenQ Corporation",
	    ['C', 'M', 'N'] => "Chimei Innolux Corporation",
	    ['C', 'M', 'O'] => "Chi Mei Optoelectronics corp.",
	    ['C', 'R', 'O'] => "Extraordinary Technologies PTY Limited",
	    ['D', 'E', 'L'] => "Dell Inc.",
	    ['D', 'G', 'C'] => "Data General Corporation",
	    ['D', 'O', 'N'] => "DENON, Ltd.",
	    ['E', 'N', 'C'] => "Eizo Nanao Corporation",
	    ['E', 'P', 'H'] => "Epiphan Systems Inc.",
	    ['E', 'X', 'P'] => "Data Export Corporation",
	    ['F', 'N', 'I'] => "Funai Electric Co., Ltd.",
	    ['F', 'U', 'S'] => "Fujitsu Siemens Computers GmbH",
	    ['G', 'S', 'M'] => "Goldstar Company Ltd",
	    ['H', 'I', 'Q'] => "Kaohsiung Opto Electronics Americas, Inc.",
	    ['H', 'S', 'D'] => "HannStar Display Corp",
	    ['H', 'T', 'C'] => "Hitachi Ltd",
	    ['H', 'W', 'P'] => "Hewlett Packard",
	    ['I', 'N', 'T'] => "Interphase Corporation",
	    ['I', 'N', 'X'] => "Communications Supply Corporation (A division of WESCO)",
	    ['I', 'T', 'E'] => "Integrated Tech Express Inc",
	    ['I', 'V', 'M'] => "Iiyama North America",
	    ['L', 'E', 'N'] => "Lenovo Group Limited",
	    ['M', 'A', 'X'] => "Rogen Tech Distribution Inc",
	    ['M', 'E', 'G'] => "Abeam Tech Ltd",
	    ['M', 'E', 'I'] => "Panasonic Industry Company",
	    ['M', 'T', 'C'] => "Mars-Tech Corporation",
	    ['M', 'T', 'X'] => "Matrox",
	    ['N', 'E', 'C'] => "NEC Corporation",
	    ['N', 'E', 'X'] => "Nexgen Mediatech Inc.",
	    ['O', 'N', 'K'] => "ONKYO Corporation",
	    ['O', 'R', 'N'] => "ORION ELECTRIC CO., LTD.",
	    ['O', 'T', 'M'] => "Optoma Corporation",
	    ['O', 'V', 'R'] => "Oculus VR, Inc.",
	    ['P', 'H', 'L'] => "Philips Consumer Electronics Company",
	    ['P', 'I', 'O'] => "Pioneer Electronic Corporation",
	    ['P', 'N', 'R'] => "Planar Systems, Inc.",
	    ['Q', 'D', 'S'] => "Quanta Display Inc.",
	    ['R', 'A', 'T'] => "Rent-A-Tech",
	    ['R', 'E', 'N'] => "Renesas Technology Corp.",
	    ['S', 'A', 'M'] => "Samsung Electric Company",
	    ['S', 'A', 'N'] => "Sanyo Electric Co., Ltd.",
	    ['S', 'E', 'C'] => "Seiko Epson Corporation",
	    ['S', 'H', 'P'] => "Sharp Corporation",
	    ['S', 'I', 'I'] => "Silicon Image, Inc.",
	    ['S', 'N', 'Y'] => "Sony",
	    ['S', 'T', 'D'] => "STD Computer Inc",
	    ['S', 'V', 'S'] => "SVSI",
	    ['S', 'Y', 'N'] => "Synaptics Inc",
	    ['T', 'C', 'L'] => "Technical Concepts Ltd",
	    ['T', 'O', 'P'] => "Orion Communications Co., Ltd.",
	    ['T', 'S', 'B'] => "Toshiba America Info Systems Inc",
	    ['T', 'S', 'T'] => "Transtream Inc",
	    ['U', 'N', 'K'] => "Unknown",
	    ['V', 'E', 'S'] => "Vestel Elektronik Sanayi ve Ticaret A. S.",
	    ['V', 'I', 'T'] => "Visitech AS",
	    ['V', 'I', 'Z'] => "VIZIO, Inc",
	    ['V', 'S', 'C'] => "ViewSonic Corporation",
	    ['Y', 'M', 'H'] => "Yamaha Corporation",
	    _ => "Unknown",
    }
}

