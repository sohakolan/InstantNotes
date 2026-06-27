<script lang="ts">
  import Fuse from "fuse.js";
  import type { TreeNode } from "./ipc";

  let {
    nodes,
    recents = [],
    onOpen,
    onCreateNote,
    onCreateFolder,
    onChangeVault,
    onClose,
  }: {
    nodes: TreeNode[];
    recents?: string[];
    onOpen: (path: string) => void;
    onCreateNote: (path: string) => void;
    onCreateFolder: (path: string) => void;
    onChangeVault: () => void;
    onClose: () => void;
  } = $props();

  let query = $state("");
  let selected = $state(0);
  let input: HTMLInputElement;

  // Aplatit l'arbre en liste de notes (chemins relatifs).
  function flatten(list: TreeNode[], acc: { name: string; path: string }[] = []) {
    for (const n of list) {
      if (n.is_dir) flatten(n.children ?? [], acc);
      else acc.push({ name: n.name, path: n.path });
    }
    return acc;
  }

  let notes = $derived(flatten(nodes));
  let fuse = $derived(new Fuse(notes, { keys: ["name", "path"], threshold: 0.4 }));

  type Item =
    | { kind: "note"; label: string; sub: string; path: string; hint?: string }
    | { kind: "create-note"; label: string }
    | { kind: "create-folder"; label: string }
    | { kind: "change-vault"; label: string };

  let byPath = $derived(new Map(notes.map((n) => [n.path, n])));

  let items = $derived.by<Item[]>(() => {
    const q = query.trim();
    const result: Item[] = [];

    if (!q) {
      // Vide : on propose d'abord les notes récentes, puis le reste.
      const seen = new Set<string>();
      for (const path of recents) {
        const n = byPath.get(path);
        if (n) {
          seen.add(path);
          result.push({ kind: "note", label: n.name, sub: n.path, path, hint: "récent" });
        }
      }
      for (const n of notes) {
        if (!seen.has(n.path)) result.push({ kind: "note", label: n.name, sub: n.path, path: n.path });
      }
    } else {
      for (const m of fuse.search(q).map((r) => r.item).slice(0, 50)) {
        result.push({ kind: "note", label: m.name, sub: m.path, path: m.path });
      }
      const exact = notes.some((n) => n.name.toLowerCase() === q.toLowerCase());
      if (!exact) result.push({ kind: "create-note", label: `Créer la note « ${q} »` });
      result.push({ kind: "create-folder", label: `Créer le dossier « ${q} »` });
    }
    result.push({ kind: "change-vault", label: "Changer de vault…" });
    return result;
  });

  // Garde la sélection dans les bornes quand la liste change.
  $effect(() => {
    if (selected >= items.length) selected = Math.max(0, items.length - 1);
  });

  function choose(item: Item) {
    const q = query.trim();
    switch (item.kind) {
      case "note":
        onOpen(item.path);
        break;
      case "create-note":
        onCreateNote(q);
        break;
      case "create-folder":
        onCreateFolder(q);
        break;
      case "change-vault":
        onChangeVault();
        break;
    }
    onClose();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selected = Math.min(selected + 1, items.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selected = Math.max(selected - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (items[selected]) choose(items[selected]);
    }
  }

  $effect(() => {
    input?.focus();
  });
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => e.target === e.currentTarget && onClose()}
>
  <div class="palette" role="dialog" tabindex="-1">
    <input
      bind:this={input}
      bind:value={query}
      class="search"
      placeholder="Rechercher ou créer une note…"
      onkeydown={onKey}
      spellcheck="false"
    />
    <ul class="results">
      {#each items as item, i (item.kind + (item.kind === "note" ? item.path : i))}
        <li>
          <button
            class="item"
            class:sel={i === selected}
            class:action={item.kind !== "note"}
            onmousemove={() => (selected = i)}
            onclick={() => choose(item)}
          >
            <span class="itext">
              <span class="ilabel">{item.label}</span>
              {#if item.kind === "note"}<span class="isub">{item.sub}</span>{/if}
            </span>
            {#if item.kind === "note" && item.hint}<span class="badge">{item.hint}</span>{/if}
            {#if item.kind === "create-note"}<span class="badge new">nouveau</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.28);
    display: flex;
    justify-content: center;
    padding-top: 11vh;
    z-index: 100;
    animation: overlay-in 0.14s ease;
  }
  .palette {
    width: 90%;
    max-height: 64vh;
    background: var(--elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: pop-in 0.16s cubic-bezier(0.22, 1, 0.36, 1);
  }
  .search {
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 15px;
    padding: 14px 16px;
    outline: none;
    border-bottom: 1px solid var(--border);
  }
  .results {
    list-style: none;
    margin: 0;
    padding: 6px;
    overflow-y: auto;
  }
  .item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    padding: 8px 10px;
    border-radius: 8px;
    cursor: pointer;
  }
  .itext {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .item.sel {
    background: var(--accent-soft);
  }
  .item.sel .ilabel {
    color: var(--accent);
  }
  .item.action .ilabel {
    color: var(--accent);
    font-weight: 600;
  }
  .ilabel {
    font-size: 13.5px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .isub {
    font-size: 11px;
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .badge {
    flex-shrink: 0;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-dim);
    background: var(--hover);
    padding: 2px 6px;
    border-radius: 5px;
  }
  .badge.new {
    color: var(--accent);
    background: var(--accent-soft);
  }
</style>
