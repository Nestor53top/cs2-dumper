use std::collections::BTreeMap;
use anyhow::Result;
use log::{debug, error};
use memflow::prelude::v1::*;
use pelite::pattern::save_len;
use pelite::pe64::{Pe, PeView, Rva};

use super::patterns::all_patterns;

pub type OffsetMap = BTreeMap<String, BTreeMap<String, Rva>>;

pub fn offsets<P: Process + MemoryView>(process: &mut P) -> Result<OffsetMap> {
    let mut map = BTreeMap::new();

    for pattern_set in all_patterns() {
        let module = match process.module_by_name(pattern_set.name) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let buf = process
            .read_raw(module.base, module.size as _)
            .data_part()?;

        let view = PeView::from_bytes(&buf)?;

        let mut module_offsets = BTreeMap::new();

        for (name, pattern) in pattern_set.patterns.entries() {
            let mut save = vec![0; save_len(pattern)];

            if !view.scanner().finds_code(pattern, &mut save) {
                error!("outdated pattern: {}", name);
                continue;
            }

            let rva = save[1];

            module_offsets.insert(name.to_string(), rva);

            debug!(
                "found \"{}\" at {:#X} ({} + {:#X})",
                name,
                rva as u64 + view.optional_header().ImageBase,
                pattern_set.name,
                rva
            );
        }

        map.insert(pattern_set.name.to_string(), module_offsets);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Once;
    use serde_json::Value;
    use simplelog::*;
    use super::*;

    #[test]
    fn build_number() -> Result<()> {
        let mut process = setup()?;
        let engine_base = process.module_by_name("engine2.dll")?.base;
        let offset = read_offset("engine2.dll", "dwBuildNumber").unwrap();
        let build_number: u32 = process.read(engine_base + offset).data_part()?;
        debug!("build number: {}", build_number);
        Ok(())
    }

    #[test]
    fn global_vars() -> Result<()> {
        let mut process = setup()?;
        let client_base = process.module_by_name("client.dll")?.base;
        let offset = read_offset("client.dll", "dwGlobalVars").unwrap();
        let global_vars: u64 = process.read(client_base + offset).data_part()?;
        let map_name_addr = process
            .read_addr64((global_vars + 0x180).into())
            .data_part()?;
        let map_name = process.read_utf8(map_name_addr, 128).data_part()?;
        debug!("[global vars] map name: \"{}\"", map_name);
        Ok(())
    }

    #[test]
    fn local_controller() -> Result<()> {
        let mut process = setup()?;
        let client_base = process.module_by_name("client.dll")?.base;
        let local_controller_offset = read_offset("client.dll", "dwLocalPlayerController").unwrap();
        let player_name_offset =
            read_class_field("client.dll", "CBasePlayerController", "m_iszPlayerName").unwrap();
        let local_controller: u64 = process
            .read(client_base + local_controller_offset)
            .data_part()?;
        let player_name = process
            .read_utf8((local_controller + player_name_offset).into(), 128)
            .data_part()?;
        debug!("[local controller] name: \"{}\"", player_name);
        Ok(())
    }

    #[test]
    fn local_pawn() -> Result<()> {
        #[derive(Pod)]
        #[repr(C)]
        struct Vector3D {
            x: f32,
            y: f32,
            z: f32,
        }

        let mut process = setup()?;
        let client_base = process.module_by_name("client.dll")?.base;
        let local_player_pawn_offset = read_offset("client.dll", "dwLocalPlayerPawn").unwrap();
        let game_scene_node_offset =
            read_class_field("client.dll", "C_BaseEntity", "m_pGameSceneNode").unwrap();
        let origin_offset =
            read_class_field("client.dll", "CGameSceneNode", "m_vecAbsOrigin").unwrap();

        let local_player_pawn: u64 = process
            .read(client_base + local_player_pawn_offset)
            .data_part()?;
        let game_scene_node: u64 = process
            .read((local_player_pawn + game_scene_node_offset).into())
            .data_part()?;
        let origin: Vector3D = process
            .read((game_scene_node + origin_offset).into())
            .data_part()?;

        debug!(
            "[local pawn] origin: {:.2}, y: {:.2}, z: {:.2}",
            origin.x, origin.y, origin.z
        );
        Ok(())
    }

    #[test]
    fn window_size() -> Result<()> {
        let mut process = setup()?;
        let engine_base = process.module_by_name("engine2.dll")?.base;
        let window_width_offset = read_offset("engine2.dll", "dwWindowWidth").unwrap();
        let window_height_offset = read_offset("engine2.dll", "dwWindowHeight").unwrap();

        let window_width: u32 = process
            .read(engine_base + window_width_offset)
            .data_part()?;
        let window_height: u32 = process
            .read(engine_base + window_height_offset)
            .data_part()?;

        debug!("window size: {}x{}", window_width, window_height);
        Ok(())
    }

    fn setup() -> Result<IntoProcessInstanceArcBox<'static>> {
        static LOGGER: Once = Once::new();
        LOGGER.call_once(|| {
            SimpleLogger::init(LevelFilter::Trace, Config::default()).ok();
        });

        let os = memflow_native::create_os(&OsArgs::default(), LibArc::default())?;
        let process = os.into_process_by_name("cs2.exe")?;
        Ok(process)
    }

    fn read_class_field(module_name: &str, class_name: &str, field_name: &str) -> Option<u64> {
        let content =
            fs::read_to_string(format!("output/{}.json", module_name.replace(".", "_"))).ok()?;
        let value: Value = serde_json::from_str(&content).ok()?;
        value
            .get(module_name)?
            .get("classes")?
            .get(class_name)?
            .get("fields")?
            .get(field_name)?
            .as_u64()
    }

    fn read_offset(module_name: &str, offset_name: &str) -> Option<u64> {
        let content = fs::read_to_string("output/offsets.json").ok()?;
        let value: Value = serde_json::from_str(&content).ok()?;
        let offset = value.get(module_name)?.get(offset_name)?;
        offset.as_u64()
    }
        }
