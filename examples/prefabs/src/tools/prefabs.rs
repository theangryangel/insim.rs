use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use insim::{core::object::ObjectCoordinate, insim::ObjectInfo};

#[derive(Debug)]
pub struct Prefabs {
    pub dir: PathBuf,
    pub entries: Vec<PrefabEntry>,
}

#[derive(Debug, Clone)]
pub struct PrefabEntry {
    pub name: String,
    pub path: PathBuf,
}

impl Prefabs {
    pub fn load(dir: PathBuf) -> Result<Self> {
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| anyhow!("failed to create directory '{}': {e}", dir.display()))?;
            return Ok(Self {
                dir,
                entries: Vec::new(),
            });
        }

        let mut entries: Vec<PrefabEntry> = fs::read_dir(&dir)
            .map_err(|e| anyhow!("failed to read directory '{}': {e}", dir.display()))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()? != "yaml" {
                    return None;
                }
                let stem = path.file_stem()?.to_str()?.to_string();
                let name = stem.replace('_', " ");
                Some(PrefabEntry { name, path })
            })
            .collect();

        entries.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(Self { dir, entries })
    }

    pub fn load_prefab(&self, idx: usize) -> Result<Prefab> {
        let entry = self
            .entries
            .get(idx)
            .ok_or_else(|| anyhow!("prefab index {idx} out of range"))?;

        let raw = fs::read_to_string(&entry.path)
            .map_err(|e| anyhow!("failed to read '{}': {e}", entry.path.display()))?;

        let objects: Vec<ObjectInfo> = serde_norway::from_str(&raw)
            .map_err(|e| anyhow!("failed to parse '{}': {e}", entry.path.display()))?;

        Ok(Prefab { objects })
    }

    pub fn add_and_save_selection(
        &mut self,
        name: &str,
        selection: &[ObjectInfo],
    ) -> Result<String> {
        if selection.is_empty() {
            return Err(anyhow!("cannot save prefab: selection is empty"));
        }

        let name = if name.trim().is_empty() {
            "prefab"
        } else {
            name.trim()
        };

        let safe = to_safe_filename(name);
        let path = unique_path(&self.dir, &safe);

        let relative = to_relative(selection);
        let yaml = serde_norway::to_string(&relative)
            .map_err(|e| anyhow!("failed to serialize prefab yaml: {e}"))?;
        fs::write(&path, yaml)
            .map_err(|e| anyhow!("failed to write '{}': {e}", path.display()))?;

        let display_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
            .replace('_', " ");

        self.entries.push(PrefabEntry {
            name: display_name.clone(),
            path,
        });
        self.entries.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(display_name)
    }
}

#[derive(Debug, Clone)]
pub struct Prefab {
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

fn to_safe_filename(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    s.trim_matches('_').to_string()
}

fn unique_path(dir: &PathBuf, stem: &str) -> PathBuf {
    let path = dir.join(format!("{stem}.yaml"));
    if !path.exists() {
        return path;
    }
    let mut n = 2_u32;
    loop {
        let path = dir.join(format!("{stem}-{n}.yaml"));
        if !path.exists() {
            return path;
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
