# InstantNotes

Pense-bête markdown ultra-rapide pour macOS. Un raccourci global fait glisser un
panneau sur le bord droit de l'écran : on écrit/colle, on referme (`Échap`), tout est
sauvegardé automatiquement en `.md` dans un *vault* façon Obsidian, avec historique git
local.

## Fonctionnalités

- **Vault de fichiers `.md`** : choisis un dossier au 1er lancement, 100 % compatible Obsidian.
- **Raccourci global personnalisable** : par défaut `⌘⌥⌃T`, ré-enregistrable dans les réglages
  (n'importe quelle combinaison avec modificateurs).
- **Panneau latéral droit** : largeur réglable (par défaut 20 % de l'écran), pleine hauteur,
  toujours au-dessus, visible sur tous les espaces (même par-dessus une app en plein écran).
- **Éditeur markdown hybride** : la syntaxe se rend en live (façon Obsidian Live Preview),
  la ligne active montre le markdown brut.
- **Autosave** débouncé (~500 ms).
- **Palette `⌘P`** : recherche floue, ouvrir/créer une note, créer un dossier, changer de vault.
- **Git local automatique** : `git init` + commits auto (après 2 min d'inactivité et au masquage).
- **App de barre de menus** : icône dans la barre, aucune icône dans le dock.

## Pile technique

Tauri 2 (Rust) · SvelteKit 5 (statique) · CodeMirror 6 · git2 (libgit2) ·
plugins Tauri `global-shortcut`, `store`, `dialog`.

## Développement

```bash
npm install
npm run tauri dev     # lance l'app en mode dev (HMR)
```

## Build de production

```bash
npm run tauri build   # produit InstantNotes.app + .dmg dans src-tauri/target/release/bundle
```

> Pour distribuer hors App Store sans alerte Gatekeeper, signer et notariser le `.app`
> avec un compte Apple Developer (`codesign` + `notarytool`). Non requis en local.

## Architecture

```
src-tauri/src/
  lib.rs       — setup, plugins, tray (barre de menus), activation policy
  vault.rs     — I/O markdown, arborescence (commandes Tauri)
  panel.rs     — positionnement bord droit + show/hide/toggle
  shortcut.rs  — (ré)enregistrement du raccourci global
  git.rs       — init + commit auto (git2, local uniquement)
src/
  routes/+page.svelte  — orchestration (vault, autosave, palette, git)
  lib/Editor.svelte    — CodeMirror 6 + autosave
  lib/livePreview.ts   — décorations live-preview (cœur du rendu hybride)
  lib/CommandPalette.svelte, FileTree.svelte, Settings.svelte
  lib/state.svelte.ts  — état + persistance des préférences
  lib/ipc.ts           — wrappers vers les commandes Rust
```

## Notes & évolutions possibles

- **Panneau non-activant** : actuellement le panneau prend le focus à l'ouverture (pratique
  pour écrire immédiatement). Pour qu'il n'enlève jamais le focus à l'app active, intégrer
  [`tauri-nspanel`](https://github.com/ahkohd/tauri-nspanel) (conversion en `NSPanel`).
- **Push git distant** : aujourd'hui les commits sont locaux ; ajouter un remote + `git push`
  (nécessite de réactiver les features https/ssh de `git2`).
- **Animation de glissement** à l'ouverture du panneau.
