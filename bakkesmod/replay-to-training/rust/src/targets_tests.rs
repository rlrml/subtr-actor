use super::*;

/// A scratch Training-root layout builder. Directories/files are created
/// under a unique temp dir per test and cleaned up on drop.
struct Layout {
    root: PathBuf,
}

impl Layout {
    fn new(name: &str) -> Layout {
        let root = std::env::temp_dir()
            .join(format!("replay-to-training-targets-{}", std::process::id()))
            .join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Layout { root }
    }

    /// Creates an (empty) `.Tem` file at `relative` (components separated by
    /// `/`), creating parent directories.
    fn tem(&self, relative: &str) -> &Layout {
        let path = self.root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"").unwrap();
        self
    }

    fn dir(&self, relative: &str) -> &Layout {
        std::fs::create_dir_all(self.root.join(relative)).unwrap();
        self
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

// --- sanitize ---

#[test]
fn sanitize_normalizes_slashes_case_and_extension() {
    assert_eq!(
        sanitize_target_name("mytraining/Shots.tem"),
        "MyTraining\\Shots"
    );
    assert_eq!(
        sanitize_target_name(" downloaded\\Pack.TEM "),
        "Downloaded\\Pack"
    );
    assert_eq!(
        sanitize_target_name("MyTraining\\Shots"),
        "MyTraining\\Shots"
    );
}

#[test]
fn sanitize_keeps_one_account_qualifier() {
    assert_eq!(
        sanitize_target_name("0000000000000000\\mytraining\\Shots.tem"),
        "0000000000000000\\MyTraining\\Shots"
    );
    // A pasted full path collapses to <account>\<Folder>\<stem>.
    assert_eq!(
        sanitize_target_name(
            "C:\\users\\steamuser\\Documents\\My Games\\Rocket League\\TAGame\\Training\\0000000000000000\\MyTraining\\Shots.Tem"
        ),
        "0000000000000000\\MyTraining\\Shots"
    );
}

#[test]
fn sanitize_passes_through_bare_stems_and_unknown_shapes() {
    assert_eq!(sanitize_target_name("Shots"), "Shots");
    assert_eq!(sanitize_target_name("a\\b"), "a\\b");
    assert_eq!(sanitize_target_name("  "), "");
}

// --- discovery ---

#[test]
fn discover_finds_account_dir_and_root_level_targets() {
    let layout = Layout::new("discover-basic");
    layout
        .tem("0000000000000000/MyTraining/Alpha.Tem")
        .tem("0000000000000000/Downloaded/Beta.Tem")
        .tem("MyTraining/Gamma.Tem")
        // Noise that must be ignored: non-.Tem files and the game's other
        // root-level files/dirs that hold no listing folders.
        .tem("0000000000000000/MyTraining/notes.txt")
        .dir("SomeOtherDir");
    assert_eq!(
        discover_targets(&layout.root),
        vec![
            "Downloaded\\Beta".to_string(),
            "MyTraining\\Alpha".to_string(),
            "MyTraining\\Gamma".to_string(),
        ]
    );
}

#[test]
fn discover_qualifies_duplicate_stems_across_accounts() {
    let layout = Layout::new("discover-dup");
    layout
        .tem("AccountA/MyTraining/Shots.Tem")
        .tem("AccountB/MyTraining/Shots.Tem");
    assert_eq!(
        discover_targets(&layout.root),
        vec![
            "AccountA\\MyTraining\\Shots".to_string(),
            "AccountB\\MyTraining\\Shots".to_string(),
        ]
    );
}

#[test]
fn discover_handles_missing_root_gracefully() {
    let missing = std::env::temp_dir().join("replay-to-training-definitely-missing-root");
    assert!(discover_targets(&missing).is_empty());
}

// --- resolution ---

#[test]
fn resolve_unqualified_finds_the_single_existing_file_in_an_account_dir() {
    let layout = Layout::new("resolve-single");
    layout.tem("0000000000000000/MyTraining/Shots.Tem");
    assert_eq!(
        resolve_target_path(&layout.root, "MyTraining\\Shots").unwrap(),
        ResolvedTarget::Path(layout.path("0000000000000000/MyTraining/Shots.Tem"))
    );
    // A bare stem defaults into MyTraining and finds the same file.
    assert_eq!(
        resolve_target_path(&layout.root, "Shots").unwrap(),
        ResolvedTarget::Path(layout.path("0000000000000000/MyTraining/Shots.Tem"))
    );
}

#[test]
fn resolve_ambiguous_when_stem_exists_in_multiple_accounts() {
    let layout = Layout::new("resolve-ambiguous");
    layout
        .tem("AccountA/MyTraining/Shots.Tem")
        .tem("AccountB/MyTraining/Shots.Tem");
    assert_eq!(
        resolve_target_path(&layout.root, "MyTraining\\Shots").unwrap(),
        ResolvedTarget::Ambiguous(vec![
            "AccountA\\MyTraining\\Shots".to_string(),
            "AccountB\\MyTraining\\Shots".to_string(),
        ])
    );
    // The qualified form binds directly and is never ambiguous.
    assert_eq!(
        resolve_target_path(&layout.root, "AccountB\\MyTraining\\Shots").unwrap(),
        ResolvedTarget::Path(layout.path("AccountB/MyTraining/Shots.Tem"))
    );
}

#[test]
fn resolve_new_name_binds_into_the_sole_account_dir() {
    let layout = Layout::new("resolve-new-sole");
    // The account dir exists (with a listing folder) but the target does not.
    layout.dir("0000000000000000/MyTraining");
    assert_eq!(
        resolve_target_path(&layout.root, "MyTraining\\Fresh").unwrap(),
        ResolvedTarget::Path(layout.path("0000000000000000/MyTraining/Fresh.Tem"))
    );
}

#[test]
fn resolve_new_name_falls_back_to_root_without_a_sole_account() {
    let no_accounts = Layout::new("resolve-new-root");
    assert_eq!(
        resolve_target_path(&no_accounts.root, "MyTraining\\Fresh").unwrap(),
        ResolvedTarget::Path(no_accounts.path("MyTraining/Fresh.Tem"))
    );

    let two_accounts = Layout::new("resolve-new-two");
    two_accounts
        .dir("AccountA/MyTraining")
        .dir("AccountB/MyTraining");
    assert_eq!(
        resolve_target_path(&two_accounts.root, "MyTraining\\Fresh").unwrap(),
        ResolvedTarget::Path(two_accounts.path("MyTraining/Fresh.Tem"))
    );
}

#[test]
fn resolve_rejects_empty_names() {
    let layout = Layout::new("resolve-empty");
    assert!(resolve_target_path(&layout.root, "  ").is_err());
}

// --- default save dir ---

#[test]
fn default_save_dir_prefers_the_sole_account_mytraining() {
    let layout = Layout::new("default-sole");
    layout.dir("0000000000000000/MyTraining");
    assert_eq!(
        default_save_dir(&layout.root),
        layout.path("0000000000000000/MyTraining")
    );
}

#[test]
fn default_save_dir_falls_back_to_root_otherwise() {
    let none = Layout::new("default-none");
    assert_eq!(default_save_dir(&none.root), none.root);

    let two = Layout::new("default-two");
    two.dir("AccountA/MyTraining").dir("AccountB/Downloaded");
    assert_eq!(default_save_dir(&two.root), two.root);
}
