<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "$lib/ipc";
  import { app, loadSettings, saveSettings, shiftVariant, pushRecent } from "$lib/state.svelte";
  import Editor from "$lib/Editor.svelte";
  import FileTree from "$lib/FileTree.svelte";
  import CommandPalette from "$lib/CommandPalette.svelte";
  import Settings from "$lib/Settings.svelte";

  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let gitTimer: ReturnType<typeof setTimeout> | null = null;
  const GIT_IDLE_MS = 120_000; // commit après 2 min d'inactivité

  let rootEl = $state<HTMLDivElement>();

  onMount(async () => {
    listen("open-settings", () => (app.settingsOpen = true));
    listen("panel-shown", () => triggerSlide());
    listen("new-note", () => createInstantNote());

    await loadSettings();
    try {
      await api.setShortcut(
        app.settings.shortcut,
        shiftVariant(app.settings.shortcut),
        app.settings.panelWidthPct,
      );
    } catch (e) {
      console.error("Raccourci:", e);
    }
    if (app.settings.vaultPath) {
      try {
        await api.setVault(app.settings.vaultPath);
        await refreshTree();
        await ensureGit();
        if (app.settings.lastNote) await openNote(app.settings.lastNote);
      } catch (e) {
        console.error(e);
        app.settings.vaultPath = null;
      }
    }
    app.ready = true;
    triggerSlide();
  });

  // Rejoue l'animation de glissement (au démarrage et à chaque ouverture du panneau).
  function triggerSlide() {
    const el = rootEl;
    if (!el) return;
    el.classList.remove("slide");
    void el.offsetWidth; // force le reflow pour relancer la keyframe
    el.classList.add("slide");
  }

  async function refreshTree() {
    app.tree = await api.readTree();
  }

  async function ensureGit() {
    if (!app.settings.gitEnabled) return;
    try {
      await api.gitInit();
    } catch (e) {
      console.error("git init:", e);
    }
  }

  async function commitNow() {
    if (gitTimer) {
      clearTimeout(gitTimer);
      gitTimer = null;
    }
    if (!app.settings.gitEnabled || !app.settings.vaultPath) return;
    try {
      await api.gitCommit();
    } catch (e) {
      console.error("git commit:", e);
    }
  }

  function scheduleCommit() {
    if (!app.settings.gitEnabled) return;
    if (gitTimer) clearTimeout(gitTimer);
    gitTimer = setTimeout(commitNow, GIT_IDLE_MS);
  }

  async function chooseVault() {
    const dir = await open({ directory: true, multiple: false, title: "Choisir un vault" });
    if (typeof dir !== "string") return;
    await api.setVault(dir);
    app.settings.vaultPath = dir;
    app.currentNote = null;
    app.content = "";
    await saveSettings();
    await refreshTree();
    await ensureGit();
  }

  async function openNote(path: string) {
    await flushSave();
    app.content = await api.readNote(path);
    app.currentNote = path;
    app.settings.lastNote = path;
    pushRecent(path);
    saveSettings();
  }

  // Crée une note horodatée et l'ouvre immédiatement (raccourci +Maj).
  async function createInstantNote() {
    if (!app.settings.vaultPath) return;
    const d = new Date();
    const p = (n: number) => String(n).padStart(2, "0");
    const name = `Note ${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}.${p(d.getMinutes())}.${p(d.getSeconds())}`;
    try {
      const rel = await api.createNote(name);
      await refreshTree();
      await openNote(rel);
    } catch (e) {
      console.error("note instantanée:", e);
    }
  }

  function backToList() {
    flushSave();
    app.currentNote = null;
  }

  function onEditorChange(v: string) {
    app.content = v;
    app.dirty = true;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(flushSave, 500);
    scheduleCommit();
  }

  async function flushSave() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    if (!app.dirty || !app.currentNote) return;
    app.saving = true;
    try {
      await api.writeNote(app.currentNote, app.content);
      app.dirty = false;
    } finally {
      app.saving = false;
    }
  }

  async function createNote(name: string) {
    const rel = await api.createNote(name);
    await refreshTree();
    await openNote(rel);
  }

  async function createFolder(name: string) {
    await api.createFolder(name);
    await refreshTree();
  }

  function onWindowKey(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "p") {
      e.preventDefault();
      if (app.tree.length || app.settings.vaultPath) app.paletteOpen = !app.paletteOpen;
    } else if (e.key === "Escape" && !app.paletteOpen && !app.settingsOpen) {
      e.preventDefault();
      flushSave();
      commitNow();
      api.hidePanel();
    }
  }

  let vaultName = $derived(app.settings.vaultPath?.split("/").pop() ?? "");
  let noteTitle = $derived(app.currentNote?.split("/").pop()?.replace(/\.md$/, "") ?? "");
</script>

<svelte:window onkeydown={onWindowKey} />

<div class="app-root" bind:this={rootEl}>
  {#if !app.ready}
    <div class="splash"><span class="spinner"></span></div>
  {:else if !app.settings.vaultPath}
    <div class="welcome">
      <div class="logo">⌘</div>
      <h1>InstantNotes</h1>
      <p>Choisis un dossier qui contiendra tes notes markdown.</p>
      <button class="primary" onclick={chooseVault}>Choisir un vault…</button>
    </div>
  {:else}
    <div class="panel">
      <header class="bar">
        {#if app.currentNote}
          <button class="icon" title="Retour aux notes" onclick={backToList}>‹</button>
        {:else}
          <button class="icon" title="Notes (⌘P)" onclick={() => (app.paletteOpen = true)}>⌘P</button>
        {/if}
        <span class="title">{noteTitle || vaultName}</span>
        <span class="dot" class:saving={app.saving} class:dirty={app.dirty}></span>
        <button class="icon" title="Organiser les notes" onclick={() => api.openManager()}>⊞</button>
        <button class="icon" title="Réglages" onclick={() => (app.settingsOpen = true)}>⚙</button>
      </header>

      <main class="body">
        {#if app.currentNote}
          <div class="editor-wrap">
            <Editor value={app.content} notePath={app.currentNote} onChange={onEditorChange} />
          </div>
        {:else if app.tree.length}
          <FileTree nodes={app.tree} current={app.currentNote} onOpen={openNote} />
        {:else}
          <div class="empty">
            <p>Aucune note pour l'instant.</p>
            <button class="ghost" onclick={() => (app.paletteOpen = true)}>
              Créer une note (⌘P)
            </button>
          </div>
        {/if}
      </main>

      <footer class="statusbar">
        <span class="vault" title={app.settings.vaultPath}>{vaultName}</span>
        <span class="state">
          {#if app.saving}Enregistrement…{:else if app.dirty}Modifié{:else}Enregistré{/if}
          {#if app.settings.gitEnabled}<span class="git" title="Git activé">⎇</span>{/if}
        </span>
      </footer>
    </div>
  {/if}
</div>

{#if app.paletteOpen}
  <CommandPalette
    nodes={app.tree}
    recents={app.settings.recents}
    onOpen={openNote}
    onCreateNote={createNote}
    onCreateFolder={createFolder}
    onChangeVault={chooseVault}
    onClose={() => (app.paletteOpen = false)}
  />
{/if}

{#if app.settingsOpen}
  <Settings onClose={() => (app.settingsOpen = false)} onChangeVault={chooseVault} />
{/if}

<style>
  .app-root {
    height: 100vh;
  }
  .app-root.slide {
    animation: slide-in-right 0.26s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .panel {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  /* Barre supérieure */
  .bar {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 40px;
    padding: 0 8px;
    flex-shrink: 0;
    -webkit-app-region: drag;
    border-bottom: 1px solid var(--border);
  }
  .bar :global(button),
  .bar .icon {
    -webkit-app-region: no-drag;
  }
  .title {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.01em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .icon {
    border: none;
    background: transparent;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    padding: 5px 8px;
    border-radius: 8px;
    opacity: 0.7;
    transition: background 0.12s, opacity 0.12s;
  }
  .icon:hover {
    background: var(--hover);
    opacity: 1;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: transparent;
    transition: background 0.2s;
  }
  .dot.dirty {
    background: var(--text-dim);
  }
  .dot.saving {
    background: var(--accent);
  }

  /* Corps */
  .body {
    flex: 1;
    overflow-y: auto;
    padding: 6px 8px;
  }
  .editor-wrap {
    height: 100%;
    max-width: 720px;
    margin: 0 auto;
    padding: 4px 8px;
  }
  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-dim);
    text-align: center;
  }

  /* Barre de statut */
  .statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    padding: 0 12px;
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    font-size: 11px;
    color: var(--text-dim);
  }
  .statusbar .vault {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 55%;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .git {
    opacity: 0.7;
  }

  /* Accueil & splash */
  .welcome,
  .splash {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    text-align: center;
    padding: 28px;
  }
  .logo {
    width: 56px;
    height: 56px;
    display: grid;
    place-items: center;
    font-size: 26px;
    color: white;
    background: linear-gradient(135deg, var(--accent), var(--accent-strong));
    border-radius: 16px;
    box-shadow: 0 10px 30px var(--accent-soft);
  }
  .welcome h1 {
    font-size: 20px;
    margin: 4px 0 0;
    letter-spacing: -0.02em;
  }
  .welcome p {
    color: var(--text-dim);
    margin: 0 0 6px;
    max-width: 260px;
    line-height: 1.5;
  }
  .primary {
    border: none;
    background: var(--accent);
    color: white;
    font-size: 13px;
    font-weight: 600;
    padding: 10px 18px;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background 0.12s, transform 0.08s;
  }
  .primary:hover {
    background: var(--accent-strong);
  }
  .primary:active {
    transform: scale(0.97);
  }
  .ghost {
    border: 1px solid var(--border-strong);
    background: transparent;
    font-size: 12px;
    padding: 7px 14px;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.12s;
  }
  .ghost:hover {
    background: var(--hover);
  }

  .spinner {
    width: 22px;
    height: 22px;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
