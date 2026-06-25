use std::collections::HashMap;
use std::sync::RwLock;

use serde::Serialize;
use uuid::Uuid;

use super::{
    plan_target, validate_pack, AutomationError, AutomationResult, MacroInventory, MacroPack,
    MacroPlan, MacroTarget,
};

#[derive(Debug, Default)]
pub struct MacroRegistry {
    packs: RwLock<HashMap<String, LoadedPack>>,
}

impl MacroRegistry {
    pub fn load_json(&self, input: &str) -> AutomationResult<MacroLoadRecord> {
        let pack: MacroPack = serde_json::from_str(input)?;
        self.load_pack(pack)
    }

    pub fn load_pack(&self, pack: MacroPack) -> AutomationResult<MacroLoadRecord> {
        let inventory = validate_pack(&pack)?;
        let pack_id = format!("{}-{}", inventory.name, Uuid::new_v4());
        let loaded = LoadedPack {
            inventory: inventory.clone(),
            pack,
        };
        self.packs
            .write()
            .map_err(|_| AutomationError::Transport("macro registry lock failed".to_string()))?
            .insert(pack_id.clone(), loaded);
        Ok(MacroLoadRecord { pack_id, inventory })
    }

    pub fn list(&self, pack_id: Option<&str>) -> AutomationResult<MacroList> {
        let packs = self
            .packs
            .read()
            .map_err(|_| AutomationError::Transport("macro registry lock failed".to_string()))?;
        let mut loaded = Vec::new();

        if let Some(id) = pack_id {
            let pack = packs
                .get(id)
                .ok_or_else(|| AutomationError::UnknownMacro(id.to_string()))?;
            loaded.push(LoadedMacroPack {
                pack_id: id.to_string(),
                inventory: pack.inventory.clone(),
            });
            return Ok(MacroList { packs: loaded });
        }

        for (id, pack) in packs.iter() {
            loaded.push(LoadedMacroPack {
                pack_id: id.clone(),
                inventory: pack.inventory.clone(),
            });
        }
        loaded.sort_by(|left, right| left.pack_id.cmp(&right.pack_id));
        Ok(MacroList { packs: loaded })
    }

    pub fn unload(&self, pack_id: &str) -> AutomationResult<bool> {
        let removed = self
            .packs
            .write()
            .map_err(|_| AutomationError::Transport("macro registry lock failed".to_string()))?
            .remove(pack_id)
            .is_some();
        Ok(removed)
    }

    pub fn plan(&self, pack_id: &str, target: MacroTarget) -> AutomationResult<MacroPlan> {
        let pack = self.pack(pack_id)?;
        plan_target(&pack, target)
    }

    pub fn pack(&self, pack_id: &str) -> AutomationResult<MacroPack> {
        let packs = self
            .packs
            .read()
            .map_err(|_| AutomationError::Transport("macro registry lock failed".to_string()))?;
        packs
            .get(pack_id)
            .map(|loaded| loaded.pack.clone())
            .ok_or_else(|| AutomationError::UnknownMacro(pack_id.to_string()))
    }
}

#[derive(Debug, Clone)]
struct LoadedPack {
    inventory: MacroInventory,
    pack: MacroPack,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroLoadRecord {
    pub pack_id: String,
    pub inventory: MacroInventory,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroList {
    pub packs: Vec<LoadedMacroPack>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadedMacroPack {
    pub pack_id: String,
    pub inventory: MacroInventory,
}
