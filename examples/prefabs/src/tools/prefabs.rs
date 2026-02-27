use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use insim::{core::object::ObjectCoordinate, insim::ObjectInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Prefabs {
    pub path: PathBuf,
    pub data: Vec<Prefab>,
}

impl Prefabs {
    pub fn load(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                path,
                data: Vec::new(),
            });
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| anyhow!("failed to read '{}': {e}", path.display()))?;
        if raw.trim().is_empty() {
            return Ok(Self {
                path,
                data: Vec::new(),
            });
        }

        let data: Vec<Prefab> = serde_norway::from_str(&raw)
            .map_err(|e| anyhow!("failed to parse '{}': {e}", path.display()))?;

        Ok(Self { path: path, data })
    }

    pub fn save(&self) -> Result<()> {
        let yaml = serde_norway::to_string(&self.data)
            .map_err(|e| anyhow!("failed to serialize prefab yaml: {e}"))?;
        fs::write(&self.path, yaml)
            .map_err(|e| anyhow!("failed to write '{}': {e}", self.path.display()))
    }

    pub fn add_and_save_selection(
        &mut self,
        name: &str,
        selection: &[ObjectInfo],
    ) -> Result<String> {
        if selection.is_empty() {
            return Err(anyhow!("cannot save prefab: selection is empty"));
        }

        let name = unique_prefab_name(&self.data, name);
        let relative = to_relative(selection);
        self.data.push(Prefab {
            name: name.clone(),
            objects: relative,
        });
        self.save()?;

        Ok(name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefab {
    pub name: String,
    pub objects: Vec<ObjectInfo>,
}

impl Prefab {
    pub fn place_at_anchor(&self, anchor: ObjectCoordinate) -> Vec<ObjectInfo> {
        self.objects
            .iter()
            .cloned()
            .map(|mut obj| {
                let pos = obj.position_mut();
                pos.x = crate::clamp_i16(i32::from(anchor.x) + i32::from(pos.x));
                pos.y = crate::clamp_i16(i32::from(anchor.y) + i32::from(pos.y));
                pos.z = crate::clamp_u8(i32::from(anchor.z) + i32::from(pos.z));
                obj
            })
            .collect()
    }
}

fn unique_prefab_name(existing: &[Prefab], requested: &str) -> String {
    let trimmed = requested.trim();
    let base = if trimmed.is_empty() {
        "prefab".to_string()
    } else {
        trimmed.to_string()
    };

    if !existing.iter().any(|p| p.name == base) {
        return base;
    }

    let mut n = 2_u32;
    loop {
        let candidate = format!("{}-{n}", base);
        if !existing.iter().any(|p| p.name == candidate) {
            return candidate;
        }
        n = n.saturating_add(1);
    }
}

pub fn to_relative(selection: &[ObjectInfo]) -> Vec<ObjectInfo> {
    if selection.is_empty() {
        return Vec::new();
    }

    let anchor = *selection[0].position();
    selection
        .iter()
        .cloned()
        .map(|mut obj| {
            let pos = obj.position_mut();
            pos.x = crate::clamp_i16(i32::from(pos.x) - i32::from(anchor.x));
            pos.y = crate::clamp_i16(i32::from(pos.y) - i32::from(anchor.y));
            pos.z = crate::clamp_u8(i32::from(pos.z) - i32::from(anchor.z));
            obj
        })
        .collect()
}
