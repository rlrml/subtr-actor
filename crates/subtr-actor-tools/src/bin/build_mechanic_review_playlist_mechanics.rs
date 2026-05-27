use anyhow::bail;

use super::config::Config;
use super::constants::{ALL_MECHANICS, DEFAULT_MECHANICS};

pub(crate) fn resolve_mechanics(config: &Config) -> anyhow::Result<Vec<&'static str>> {
    let requested: Vec<String> = if config.mechanics.is_empty() {
        DEFAULT_MECHANICS
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    } else {
        config.mechanics.clone()
    };

    let mut resolved = Vec::new();
    for raw in requested {
        let normalized = raw.trim().replace('-', "_").to_ascii_lowercase();
        let names: Vec<&str> = match normalized.as_str() {
            "default" => DEFAULT_MECHANICS.to_vec(),
            "all" => ALL_MECHANICS.to_vec(),
            name if ALL_MECHANICS.contains(&name) => vec![ALL_MECHANICS
                .iter()
                .copied()
                .find(|candidate| *candidate == name)
                .expect("mechanic is known")],
            other => bail!(
                "unknown mechanic {other}; supported mechanics are: {}, default, all",
                ALL_MECHANICS.join(", ")
            ),
        };
        for name in names {
            if !resolved.contains(&name) {
                resolved.push(name);
            }
        }
    }
    Ok(resolved)
}

pub(crate) fn graph_node_names_for_mechanics(mechanics: &[&str]) -> Vec<&'static str> {
    let mut names = Vec::new();
    for mechanic in mechanics {
        let node = match *mechanic {
            "flick" => Some("flick"),
            "musty_flick" => Some("musty_flick"),
            "one_timer" => Some("one_timer"),
            "air_dribble" => Some("ball_carry"),
            "ceiling_shot" => Some("ceiling_shot"),
            "double_tap" => Some("double_tap"),
            "speed_flip" => Some("speed_flip"),
            "half_flip" => Some("half_flip"),
            "wavedash" => Some("wavedash"),
            "flip_reset" => Some("dodge_reset"),
            _ => None,
        };
        if let Some(node) = node {
            if !names.contains(&node) {
                names.push(node);
            }
        }
    }
    names
}
