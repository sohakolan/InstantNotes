use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

/// État global : chemin du vault courant.
#[derive(Default)]
pub struct VaultState(pub Mutex<Option<PathBuf>>);

/// Un nœud de l'arborescence du vault (note `.md` ou dossier).
#[derive(Serialize)]
pub struct TreeNode {
    /// Nom affiché (sans extension pour les notes).
    pub name: String,
    /// Chemin relatif au vault, séparateurs `/`.
    pub path: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TreeNode>,
}

fn vault_root(state: &VaultState) -> Result<PathBuf, String> {
    state
        .0
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Aucun vault ouvert".to_string())
}

/// Résout un chemin relatif en absolu et garantit qu'il reste dans le vault.
fn resolve(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let candidate = root.join(rel);
    // On nettoie les `..` en normalisant composant par composant.
    let mut normalized = root.to_path_buf();
    for comp in Path::new(rel).components() {
        use std::path::Component::*;
        match comp {
            Normal(c) => normalized.push(c),
            ParentDir => {
                return Err("Chemin invalide".into());
            }
            CurDir | RootDir | Prefix(_) => {}
        }
    }
    let _ = candidate; // candidate gardé pour lisibilité
    if !normalized.starts_with(root) {
        return Err("Chemin hors du vault".into());
    }
    Ok(normalized)
}

fn read_dir_recursive(dir: &Path, root: &Path) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return nodes,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        // Ignore les fichiers cachés et le dossier git.
        if file_name.starts_with('.') {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            nodes.push(TreeNode {
                name: file_name,
                path: rel,
                is_dir: true,
                children: read_dir_recursive(&path, root),
            });
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(file_name);
            nodes.push(TreeNode {
                name: stem,
                path: rel,
                is_dir: false,
                children: Vec::new(),
            });
        }
    }
    // Dossiers d'abord, puis tri alphabétique.
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    nodes
}

/// Instructions destinées à une IA pour ranger le vault sans dénaturer les notes.
const PROMPT_MD: &str = r#"# Comment ranger ce vault de notes

Ce dossier est un *vault* de notes markdown (un fichier `.md` = une note).
Si on te demande de **ranger / organiser** ces notes, suis ces règles strictement.

## Règles d'or

1. **Ne modifie JAMAIS le contenu d'une note.** Tu peux la déplacer, jamais la réécrire,
   la résumer, la corriger ou la tronquer. Le texte des notes est sacré.
2. **Organise par dossiers thématiques.** Crée des dossiers clairs et sémantiques
   (ex. `Projets/`, `Idées/`, `Personnel/`, `Travail/`, `Références/`, `Quotidien/`).
3. **Range en déplaçant les fichiers** dans le bon dossier, sans renommer le contenu.
   Tu peux renommer un fichier si son titre est vide ou non signifiant (ex. `Note 2026-...`),
   en t'appuyant uniquement sur ce que la note contient déjà.
4. **Pas de doublons, pas de suppression.** Ne supprime aucune note. En cas d'ambiguïté,
   place la note dans le dossier le plus probable plutôt que d'en créer un trop spécifique.
5. **Hiérarchie peu profonde.** Évite les dossiers à plus de 2–3 niveaux.

## Méthode suggérée

1. Lis le titre (nom de fichier) et les premières lignes de chaque note.
2. Déduis 5 à 10 catégories qui couvrent l'ensemble.
3. Crée les dossiers correspondants, puis déplace chaque note dans sa catégorie.
4. Laisse à la racine uniquement ce qui n'entre dans aucune catégorie.

Ne touche pas à ce fichier `prompt.md`.
"#;

fn write_scaffold(root: &Path) {
    let prompt = root.join("prompt.md");
    if !prompt.exists() {
        let _ = fs::write(prompt, PROMPT_MD);
    }
}

#[tauri::command]
pub fn set_vault(path: String, state: State<VaultState>) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.is_dir() {
        return Err("Le dossier sélectionné n'existe pas".into());
    }
    write_scaffold(&p);
    *state.0.lock().unwrap() = Some(p);
    Ok(())
}

#[tauri::command]
pub fn read_tree(state: State<VaultState>) -> Result<Vec<TreeNode>, String> {
    let root = vault_root(&state)?;
    Ok(read_dir_recursive(&root, &root))
}

#[tauri::command]
pub fn read_note(path: String, state: State<VaultState>) -> Result<String, String> {
    let root = vault_root(&state)?;
    let full = resolve(&root, &path)?;
    fs::read_to_string(&full).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_note(path: String, content: String, state: State<VaultState>) -> Result<(), String> {
    let root = vault_root(&state)?;
    let full = resolve(&root, &path)?;
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&full, content).map_err(|e| e.to_string())
}

/// Crée une note vide. Si `path` ne finit pas par `.md`, l'extension est ajoutée.
/// Retourne le chemin relatif final créé.
#[tauri::command]
pub fn create_note(path: String, state: State<VaultState>) -> Result<String, String> {
    let root = vault_root(&state)?;
    let mut rel = path;
    if !rel.ends_with(".md") {
        rel.push_str(".md");
    }
    let full = resolve(&root, &rel)?;
    if full.exists() {
        return Err("Une note porte déjà ce nom".into());
    }
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&full, "").map_err(|e| e.to_string())?;
    Ok(rel)
}

#[tauri::command]
pub fn create_folder(path: String, state: State<VaultState>) -> Result<(), String> {
    let root = vault_root(&state)?;
    let full = resolve(&root, &path)?;
    fs::create_dir_all(&full).map_err(|e| e.to_string())
}

/// Déplace (ou renomme) une note/un dossier vers `to` (chemin relatif).
#[tauri::command]
pub fn move_path(from: String, to: String, state: State<VaultState>) -> Result<(), String> {
    let root = vault_root(&state)?;
    let src = resolve(&root, &from)?;
    let dst = resolve(&root, &to)?;
    if !src.exists() {
        return Err("Source introuvable".into());
    }
    if dst.exists() {
        return Err("Une note/dossier porte déjà ce nom à destination".into());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::rename(&src, &dst).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_path(path: String, state: State<VaultState>) -> Result<(), String> {
    let root = vault_root(&state)?;
    let full = resolve(&root, &path)?;
    if full == root {
        return Err("Impossible de supprimer la racine".into());
    }
    if full.is_dir() {
        fs::remove_dir_all(&full).map_err(|e| e.to_string())
    } else {
        fs::remove_file(&full).map_err(|e| e.to_string())
    }
}
