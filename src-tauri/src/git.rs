use std::path::Path;

use git2::{IndexAddOption, Repository, Signature};
use tauri::State;

use crate::vault::VaultState;

fn vault_path(state: &VaultState) -> Result<std::path::PathBuf, String> {
    state
        .0
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Aucun vault ouvert".to_string())
}

fn open_or_init(path: &Path) -> Result<Repository, String> {
    match Repository::open(path) {
        Ok(repo) => Ok(repo),
        Err(_) => Repository::init(path).map_err(|e| e.to_string()),
    }
}

fn signature(repo: &Repository) -> Result<Signature<'static>, String> {
    repo.signature()
        .or_else(|_| Signature::now("InstantNotes", "instantnotes@localhost"))
        .map_err(|e| e.to_string())
}

/// Initialise un dépôt git dans le vault s'il n'existe pas.
#[tauri::command]
pub fn git_init(state: State<VaultState>) -> Result<(), String> {
    let path = vault_path(&state)?;
    open_or_init(&path)?;
    Ok(())
}

/// Ajoute tout et crée un commit si des changements existent.
/// Retourne `true` si un commit a été créé, `false` si rien n'a changé.
#[tauri::command]
pub fn git_commit(state: State<VaultState>) -> Result<bool, String> {
    let path = vault_path(&state)?;
    let repo = open_or_init(&path)?;

    let mut index = repo.index().map_err(|e| e.to_string())?;
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())?;

    let tree_id = index.write_tree().map_err(|e| e.to_string())?;
    let tree = repo.find_tree(tree_id).map_err(|e| e.to_string())?;

    let parent = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok());

    // Rien de neuf depuis le dernier commit → on n'écrit pas.
    if let Some(p) = &parent {
        let parent_tree = p.tree().map_err(|e| e.to_string())?;
        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
            .map_err(|e| e.to_string())?;
        if diff.deltas().len() == 0 {
            return Ok(false);
        }
    }

    let sig = signature(&repo)?;
    let parents: Vec<&git2::Commit> = parent.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, "InstantNotes autosave", &tree, &parents)
        .map_err(|e| e.to_string())?;
    Ok(true)
}
