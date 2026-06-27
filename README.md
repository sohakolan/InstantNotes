# InstantNotes

Pense-bête markdown ultra-rapide pour macOS. Un raccourci global fait glisser un
panneau sur le bord droit de l'écran : on écrit/colle, on referme (`Échap`), tout est
sauvegardé automatiquement en `.md` dans un *vault* façon Obsidian, avec historique git
local.

## Fonctionnalités

- **Vault de fichiers `.md`** — choisis un dossier au 1er lancement, 100 % compatible Obsidian.
- **Raccourci global** — par défaut `⌘⌥⌃J`, entièrement personnalisable. **Ajoute `⇧`**
  (`⌘⌥⌃⇧J`) pour ouvrir **et créer une note instantanément**.
- **Panneau latéral droit** — largeur réglable, aligné sous la barre de menus (n'empiète pas
  sur l'horloge / Control Center), toujours au-dessus, visible sur tous les espaces.
- **Éditeur markdown hybride** — la syntaxe se rend en live (façon Obsidian Live Preview),
  caret fluide qui glisse vers sa position.
- **Autosave** débouncé (~500 ms).
- **Palette `⌘P`** — notes **récentes** quand le champ est vide, recherche floue, création
  simple (note, dossier, `dossier/nom`), changement de vault.
- **Mode worktree** (`⊞`) — une fenêtre dédiée pour **organiser** : glisser-déposer les notes
  dans des dossiers, créer notes/dossiers, supprimer — sans jamais modifier le contenu.
- **`prompt.md` auto** — créé dans chaque vault : il explique à une IA comment ranger les
  notes (créer des dossiers, ne jamais altérer le contenu).
- **Git local automatique** — `git init` + commits auto (inactivité + au masquage).
- **App de barre de menus** — icône dans la barre, pas d'icône dans le dock.
- **Personnalisation** — thème (système/clair/sombre), couleur d'accent, 9 polices d'éditeur
  + taille, et le curseur (couleur, épaisseur, glissement, clignotement).

## Installation (Homebrew)

```bash
brew install --cask sohakolan/instantnotes/instant-notes
```

> L'app n'est pas signée/notarisée par Apple. Le cask retire automatiquement la mise en
> quarantaine pour éviter l'erreur « InstantNotes est endommagé ». Si elle apparaît malgré
> tout, exécute : `xattr -cr /Applications/InstantNotes.app` puis relance l'app.

## Développement

```bash
npm install
npm run tauri dev      # mode dev (HMR)
npm run tauri build    # produit InstantNotes.app + .dmg dans src-tauri/target/release/bundle
```

## Architecture

```
src-tauri/src/
  lib.rs       — setup, plugins, tray (barre de menus), double raccourci global
  vault.rs     — I/O markdown, arborescence, déplacement, prompt.md (commandes Tauri)
  panel.rs     — positionnement (visibleFrame), show/hide, fenêtre worktree
  shortcut.rs  — (ré)enregistrement des raccourcis (toggle + variante +Maj)
  git.rs       — init + commit auto (git2, local)
src/
  routes/+page.svelte  — routeur (panneau vs fenêtre worktree selon le label)
  lib/Panel.svelte     — application panneau (vault, autosave, palette, git)
  lib/Manager.svelte   — fenêtre worktree (glisser-déposer, création, suppression)
  lib/Editor.svelte    — CodeMirror 6 + autosave
  lib/livePreview.ts   — décorations live-preview (rendu hybride)
  lib/CommandPalette.svelte, FileTree.svelte, Settings.svelte
  lib/state.svelte.ts  — état, préférences, apparence
  lib/ipc.ts           — wrappers vers les commandes Rust
```

## Licence

MIT
