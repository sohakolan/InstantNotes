<script lang="ts">
  import { onMount } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import * as api from "$lib/ipc";
  import type { TreeNode } from "$lib/ipc";
  import { loadSettings } from "$lib/state.svelte";
  import Editor from "$lib/Editor.svelte";
  import FileTree from "$lib/FileTree.svelte";

  let tree = $state<TreeNode[]>([]);
  let currentNote = $state<string | null>(null);
  let content = $state("");
  let dirty = $state(false);
  let saving = $state(false);
  let ready = $state(false);

  // Mini-formulaire de création (note/dossier).
  let prompting = $state<null | "note" | "folder">(null);
  let promptValue = $state("");
  let promptInput = $state<HTMLInputElement>();

  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    const s = await loadSettings();
    if (s.vaultPath) {
      await api.setVault(s.vaultPath);
      await refresh();
    }
    ready = true;
  });

  async function refresh() {
    tree = await api.readTree();
  }

  async function openNote(path: string) {
    await flushSave();
    content = await api.readNote(path);
    currentNote = path;
  }

  function onChange(v: string) {
    content = v;
    dirty = true;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(flushSave, 500);
  }

  async function flushSave() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    if (!dirty || !currentNote) return;
    saving = true;
    try {
      await api.writeNote(currentNote, content);
      dirty = false;
    } finally {
      saving = false;
    }
  }

  async function onMove(from: string, to: string) {
    try {
      await api.movePath(from, to);
      if (currentNote === from) currentNote = to;
      await refresh();
    } catch (e) {
      console.error(e);
    }
  }

  async function onDelete(path: string) {
    const ok = await ask(`Supprimer « ${path} » ?`, {
      title: "Confirmer",
      kind: "warning",
    });
    if (!ok) return;
    await api.deletePath(path);
    if (currentNote === path || currentNote?.startsWith(path + "/")) {
      currentNote = null;
      content = "";
    }
    await refresh();
  }

  function startPrompt(kind: "note" | "folder") {
    prompting = kind;
    promptValue = "";
    setTimeout(() => promptInput?.focus(), 0);
  }

  async function confirmPrompt() {
    const name = promptValue.trim();
    if (!name || !prompting) return (prompting = null);
    try {
      if (prompting === "note") {
        const rel = await api.createNote(name);
        await refresh();
        await openNote(rel);
      } else {
        await api.createFolder(name);
        await refresh();
      }
    } catch (e) {
      console.error(e);
    }
    prompting = null;
  }

  // Zone de dépôt racine.
  function onRootDrop(e: DragEvent) {
    e.preventDefault();
    const from = e.dataTransfer?.getData("text/plain");
    if (!from) return;
    const base = from.split("/").pop()!;
    if (from === base) return; // déjà à la racine
    onMove(from, base);
  }
</script>

<div class="mgr">
  <aside class="side">
    <div class="toolbar">
      <strong>Organisation</strong>
      <div class="actions">
        <button class="tbtn" onclick={() => startPrompt("note")} title="Nouvelle note">＋ Note</button>
        <button class="tbtn" onclick={() => startPrompt("folder")} title="Nouveau dossier">
          ＋ Dossier
        </button>
      </div>
    </div>

    {#if prompting}
      <div class="prompt">
        <input
          bind:this={promptInput}
          bind:value={promptValue}
          placeholder={prompting === "note" ? "Nom de la note (ou dossier/nom)" : "Nom du dossier"}
          onkeydown={(e) => {
            if (e.key === "Enter") confirmPrompt();
            else if (e.key === "Escape") prompting = null;
          }}
        />
        <button class="tbtn primary" onclick={confirmPrompt}>OK</button>
      </div>
    {/if}

    <div
      class="treezone"
      role="tree"
      tabindex="-1"
      ondragover={(e) => e.preventDefault()}
      ondrop={onRootDrop}
    >
      {#if tree.length}
        <FileTree
          nodes={tree}
          current={currentNote}
          draggable
          {onOpen}
          {onMove}
          {onDelete}
        />
      {:else if ready}
        <p class="hint">Vault vide. Crée une note ou un dossier.</p>
      {/if}
    </div>
    <p class="tip">Glisse une note sur un dossier pour la ranger. Le contenu n'est jamais modifié.</p>
  </aside>

  <main class="pane">
    {#if currentNote}
      <div class="pane-head">
        <span class="pt">{currentNote}</span>
        <span class="st">{saving ? "Enregistrement…" : dirty ? "Modifié" : "Enregistré"}</span>
      </div>
      <div class="ed">
        <Editor value={content} notePath={currentNote} {onChange} />
      </div>
    {:else}
      <div class="placeholder">
        <p>Sélectionne une note à gauche pour la lire ou l'éditer.</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .mgr {
    display: grid;
    grid-template-columns: 280px 1fr;
    height: 100vh;
  }
  .side {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
    min-width: 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 12px 12px 8px;
  }
  .toolbar strong {
    font-size: 13px;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .tbtn {
    border: 1px solid var(--border-strong);
    background: var(--hover);
    color: var(--text);
    font-size: 11px;
    font-weight: 600;
    padding: 5px 8px;
    border-radius: 7px;
    cursor: pointer;
    white-space: nowrap;
  }
  .tbtn:hover {
    background: var(--accent-soft);
  }
  .tbtn.primary {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  .prompt {
    display: flex;
    gap: 6px;
    padding: 0 12px 8px;
  }
  .prompt input {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    padding: 7px 9px;
    border-radius: 7px;
    border: 1px solid var(--border-strong);
    background: var(--elevated);
    color: var(--text);
    outline: none;
  }
  .treezone {
    flex: 1;
    overflow-y: auto;
    padding: 4px 8px;
  }
  .hint {
    opacity: 0.5;
    font-size: 12px;
    padding: 12px;
  }
  .tip {
    font-size: 10.5px;
    color: var(--text-dim);
    padding: 8px 12px;
    border-top: 1px solid var(--border);
    margin: 0;
    line-height: 1.4;
  }
  .pane {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .pane-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
  }
  .pt {
    font-size: 12px;
    font-weight: 600;
    opacity: 0.8;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .st {
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
  }
  .ed {
    flex: 1;
    overflow-y: auto;
    padding: 12px 20px;
    max-width: 820px;
    width: 100%;
    margin: 0 auto;
  }
  .placeholder {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--text-dim);
    font-size: 13px;
  }
</style>
