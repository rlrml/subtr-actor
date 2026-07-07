//! Pure path logic for the persistent save target: sanitizing target names,
//! discovering `.Tem` targets under the game's Training root, resolving a
//! name to an on-disk path, and picking the default save directory.
//!
//! # Directory layout (observed in-game)
//!
//! The game inserts a PER-ACCOUNT directory between the Training root and
//! the listing folders — e.g. Epic under wine uses
//! `...\TAGame\Training\0000000000000000\MyTraining\*.Tem`; other accounts
//! use their online-id. Some setups also have `MyTraining\` directly under
//! the root, so both layouts are scanned:
//!
//! ```text
//! <root>/<account>/MyTraining/*.Tem   (normal)
//! <root>/<account>/Downloaded/*.Tem
//! <root>/MyTraining/*.Tem             (legacy/robustness)
//! <root>/Downloaded/*.Tem
//! ```
//!
//! Account directory names are NOT pattern-restricted (16-digit ids, online
//! ids, anything): every immediate subdirectory of the root other than the
//! listing folders themselves counts.
//!
//! This lives in the Rust core (behind the ABI) rather than the C++ plugin
//! so it is unit-testable; the account-dir bug shipped precisely because the
//! C++ path logic had no test harness.

use std::path::{Path, PathBuf};

/// The subfolders the game lists custom training from.
pub const TARGET_FOLDERS: [&str; 2] = ["MyTraining", "Downloaded"];

/// Canonical case for a listing-folder name, matched case-insensitively.
fn canonical_folder(component: &str) -> Option<&'static str> {
    TARGET_FOLDERS
        .iter()
        .find(|folder| folder.eq_ignore_ascii_case(component))
        .copied()
}

/// Normalizes a user-entered target name:
///
/// * trims whitespace, converts `/` to `\`, strips a trailing `.tem`/`.Tem`,
/// * `... \ <Folder> \ <stem>` with a known folder (`MyTraining` /
///   `Downloaded`, any case) canonicalizes the folder's case and keeps at
///   most one preceding component as the account qualifier — so a pasted
///   full path collapses to `<account>\<Folder>\<stem>`,
/// * anything else (e.g. a bare stem) is returned with empty components
///   dropped.
pub fn sanitize_target_name(value: &str) -> String {
    let mut value = value.trim().replace('/', "\\");
    if value.len() >= 4 && value[value.len() - 4..].eq_ignore_ascii_case(".tem") {
        value.truncate(value.len() - 4);
    }
    let components: Vec<&str> = value.split('\\').filter(|part| !part.is_empty()).collect();
    if components.len() >= 2 {
        if let Some(folder) = canonical_folder(components[components.len() - 2]) {
            let stem = components[components.len() - 1];
            return if components.len() >= 3 {
                let account = components[components.len() - 3];
                format!("{account}\\{folder}\\{stem}")
            } else {
                format!("{folder}\\{stem}")
            };
        }
    }
    components.join("\\")
}

/// Immediate subdirectories of `root` that are candidate account
/// directories: any directory that is not itself a listing folder and
/// contains at least one listing folder. Sorted by name.
fn account_dirs(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| canonical_folder(name).is_none())
        })
        .filter(|path| {
            TARGET_FOLDERS
                .iter()
                .any(|folder| path.join(folder).is_dir())
        })
        .collect();
    dirs.sort();
    dirs
}

/// `.Tem` stems (file names without extension) in `directory`, sorted.
fn tem_stems(directory: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut stems: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("tem"))
        })
        .filter_map(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
        })
        .collect();
    stems.sort();
    stems
}

/// One discovered target location before name qualification.
struct Discovered {
    /// `None` for a root-level listing folder, `Some(account)` otherwise.
    account: Option<String>,
    folder: &'static str,
    stem: String,
}

fn discover_raw(root: &Path) -> Vec<Discovered> {
    let mut found = Vec::new();
    for folder in TARGET_FOLDERS {
        for stem in tem_stems(&root.join(folder)) {
            found.push(Discovered {
                account: None,
                folder,
                stem,
            });
        }
    }
    for account_dir in account_dirs(root) {
        let Some(account) = account_dir
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        for folder in TARGET_FOLDERS {
            for stem in tem_stems(&account_dir.join(folder)) {
                found.push(Discovered {
                    account: Some(account.clone()),
                    folder,
                    stem,
                });
            }
        }
    }
    found
}

/// Scans the Training root for `.Tem` targets — both `<root>/<Folder>` and
/// `<root>/<account>/<Folder>` for every account directory — and returns
/// their names, sorted. A name is the short `<Folder>\<stem>` form when that
/// folder+stem pair exists in only one location; duplicates found under
/// account directories are qualified as `<account>\<Folder>\<stem>` so they
/// stay unambiguous.
pub fn discover_targets(root: &Path) -> Vec<String> {
    let found = discover_raw(root);
    let mut names: Vec<String> = found
        .iter()
        .map(|target| {
            let duplicates = found
                .iter()
                .filter(|other| other.folder == target.folder && other.stem == target.stem)
                .count();
            match (&target.account, duplicates > 1) {
                (Some(account), true) => {
                    format!("{account}\\{}\\{}", target.folder, target.stem)
                }
                _ => format!("{}\\{}", target.folder, target.stem),
            }
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

/// A resolved target name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedTarget {
    /// The single on-disk path this name binds to (the file may not exist
    /// yet — the first save creates it).
    Path(PathBuf),
    /// The unqualified name matched existing files in several locations; the
    /// caller should warn and require one of these qualified names.
    Ambiguous(Vec<String>),
}

/// Resolves a (raw or sanitized) target name against the Training root:
///
/// * `<account>\<Folder>\<stem>` — binds directly to
///   `<root>/<account>/<Folder>/<stem>.Tem`,
/// * `<Folder>\<stem>` (or a bare `<stem>`, which defaults into
///   `MyTraining`) — searched across the root-level folder and every
///   account directory: exactly one existing file wins; several existing
///   files are [`ResolvedTarget::Ambiguous`]; when none exist yet the name
///   binds into the sole account directory when there is exactly one
///   (matching where the game will list it), else the root-level folder.
///
/// `Err` for names that are empty after sanitizing.
pub fn resolve_target_path(root: &Path, name: &str) -> Result<ResolvedTarget, String> {
    let sanitized = sanitize_target_name(name);
    if sanitized.is_empty() {
        return Err("target name is empty after sanitizing".to_string());
    }
    let components: Vec<&str> = sanitized.split('\\').collect();
    let (account, folder, stem) = match components.as_slice() {
        [account, folder, stem] => (Some(*account), canonical_folder(folder), *stem),
        [folder, stem] => (None, canonical_folder(folder), *stem),
        [stem] => (None, Some(TARGET_FOLDERS[0]), *stem),
        _ => (None, None, ""),
    };
    let Some(folder) = folder else {
        // Not a recognized shape; treat the sanitized name as a path
        // relative to the root.
        let mut path = root.to_path_buf();
        for component in &components {
            path.push(component);
        }
        path.set_extension("Tem");
        return Ok(ResolvedTarget::Path(path));
    };
    let file_name = format!("{stem}.Tem");

    if let Some(account) = account {
        return Ok(ResolvedTarget::Path(
            root.join(account).join(folder).join(file_name),
        ));
    }

    // Unqualified: search the root-level folder and every account dir.
    let mut existing: Vec<(Option<String>, PathBuf)> = Vec::new();
    let root_candidate = root.join(folder).join(&file_name);
    if root_candidate.is_file() {
        existing.push((None, root_candidate));
    }
    let accounts = account_dirs(root);
    for account_dir in &accounts {
        let candidate = account_dir.join(folder).join(&file_name);
        if candidate.is_file() {
            let account = account_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            existing.push((Some(account), candidate));
        }
    }
    match existing.len() {
        1 => Ok(ResolvedTarget::Path(existing.remove(0).1)),
        0 => {
            // Nothing on disk yet: bind where the game will list it.
            if accounts.len() == 1 {
                Ok(ResolvedTarget::Path(
                    accounts[0].join(folder).join(file_name),
                ))
            } else {
                Ok(ResolvedTarget::Path(root.join(folder).join(file_name)))
            }
        }
        _ => Ok(ResolvedTarget::Ambiguous(
            existing
                .into_iter()
                .map(|(account, _)| match account {
                    Some(account) => format!("{account}\\{folder}\\{stem}"),
                    None => format!("{folder}\\{stem}"),
                })
                .collect(),
        )),
    }
}

/// The directory untargeted (auto-GUID) saves land in: the sole account
/// directory's `MyTraining\` when exactly one account directory exists (so
/// the pack is visible under Training > Custom Training), otherwise the
/// Training root itself, matching the previous behavior.
pub fn default_save_dir(root: &Path) -> PathBuf {
    let accounts = account_dirs(root);
    if accounts.len() == 1 {
        accounts[0].join(TARGET_FOLDERS[0])
    } else {
        root.to_path_buf()
    }
}

#[cfg(test)]
#[path = "targets_tests.rs"]
mod tests;
