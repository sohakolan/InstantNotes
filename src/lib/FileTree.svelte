<script lang="ts">
  import type { TreeNode } from "./ipc";
  import Self from "./FileTree.svelte";

  let {
    nodes,
    current = null,
    depth = 0,
    draggable = false,
    onOpen,
    onMove,
    onDelete,
  }: {
    nodes: TreeNode[];
    current?: string | null;
    depth?: number;
    draggable?: boolean;
    onOpen: (path: string) => void;
    onMove?: (from: string, to: string) => void;
    onDelete?: (path: string) => void;
  } = $props();

  let collapsed = $state<Record<string, boolean>>({});
  let dragOver = $state<string | null>(null);

  const basename = (p: string) => p.split("/").pop() ?? p;

  function onDragStart(e: DragEvent, path: string) {
    e.dataTransfer?.setData("text/plain", path);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }

  function onDrop(e: DragEvent, folderPath: string) {
    e.preventDefault();
    e.stopPropagation();
    dragOver = null;
    const from = e.dataTransfer?.getData("text/plain");
    if (!from || !onMove) return;
    // Interdit de déposer un dossier dans lui-même ou un descendant.
    if (folderPath === from || folderPath.startsWith(from + "/")) return;
    const to = folderPath ? `${folderPath}/${basename(from)}` : basename(from);
    if (from === to) return;
    onMove(from, to);
  }
</script>

<ul class="tree" class:root={depth === 0}>
  {#each nodes as node (node.path)}
    <li>
      {#if node.is_dir}
        <div
          class="rowwrap"
          class:dragover={dragOver === node.path}
          role="treeitem"
          aria-selected="false"
          tabindex="-1"
          ondragover={(e) => {
            if (onMove) {
              e.preventDefault();
              dragOver = node.path;
            }
          }}
          ondragleave={() => (dragOver === node.path ? (dragOver = null) : null)}
          ondrop={(e) => onDrop(e, node.path)}
        >
          <button
            class="row dir"
            style="padding-left: {depth * 12 + 8}px"
            draggable={draggable}
            ondragstart={(e) => onDragStart(e, node.path)}
            onclick={() => (collapsed[node.path] = !collapsed[node.path])}
          >
            <span class="caret">{collapsed[node.path] ? "▸" : "▾"}</span>
            <span class="label">{node.name}</span>
          </button>
          {#if onDelete}
            <button class="del" title="Supprimer" onclick={() => onDelete?.(node.path)}>✕</button>
          {/if}
        </div>
        {#if !collapsed[node.path] && node.children?.length}
          <Self
            nodes={node.children}
            {current}
            depth={depth + 1}
            {draggable}
            {onOpen}
            {onMove}
            {onDelete}
          />
        {/if}
      {:else}
        <div class="rowwrap">
          <button
            class="row note"
            class:active={current === node.path}
            style="padding-left: {depth * 12 + 22}px"
            draggable={draggable}
            ondragstart={(e) => onDragStart(e, node.path)}
            onclick={() => onOpen(node.path)}
          >
            <span class="label">{node.name}</span>
          </button>
          {#if onDelete}
            <button class="del" title="Supprimer" onclick={() => onDelete?.(node.path)}>✕</button>
          {/if}
        </div>
      {/if}
    </li>
  {/each}
</ul>

<style>
  .tree {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .rowwrap {
    display: flex;
    align-items: center;
    border-radius: 6px;
  }
  .rowwrap.dragover {
    background: var(--accent-soft);
    outline: 1px dashed var(--accent);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    text-align: left;
    padding: 5px 8px;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row:hover {
    background: var(--hover);
  }
  .note.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }
  .caret {
    width: 12px;
    opacity: 0.6;
    font-size: 10px;
  }
  .dir .label {
    font-weight: 600;
    opacity: 0.85;
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .del {
    flex-shrink: 0;
    border: none;
    background: transparent;
    color: var(--text-dim);
    font-size: 11px;
    cursor: pointer;
    padding: 4px 8px;
    opacity: 0;
    border-radius: 6px;
  }
  .rowwrap:hover .del {
    opacity: 0.6;
  }
  .del:hover {
    opacity: 1 !important;
    color: #ef4444;
    background: var(--hover);
  }
</style>
