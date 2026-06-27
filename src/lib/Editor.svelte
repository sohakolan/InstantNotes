<script lang="ts">
  import { onMount } from "svelte";
  import { EditorState } from "@codemirror/state";
  import {
    EditorView,
    keymap,
    drawSelection,
    placeholder as cmPlaceholder,
  } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { livePreview } from "./livePreview";

  let {
    value = "",
    notePath = null,
    onChange,
  }: {
    value?: string;
    notePath?: string | null;
    onChange?: (v: string) => void;
  } = $props();

  let el: HTMLDivElement;
  let view: EditorView | null = null;
  let lastPath: string | null = null;

  const editorTheme = EditorView.theme({
    "&": {
      height: "100%",
      backgroundColor: "transparent",
      fontSize: "var(--editor-font-size, 15px)",
    },
    ".cm-scroller": {
      fontFamily: "var(--editor-font-family, -apple-system, sans-serif)",
      lineHeight: "1.7",
      padding: "4px 2px",
    },
    // Caret natif masqué : drawSelection fournit un curseur animable.
    ".cm-content": { caretColor: "transparent" },
    "&.cm-focused": { outline: "none" },
    ".cm-line": { padding: "0 2px" },
    ".cm-placeholder": { color: "rgba(120,120,120,0.55)", fontStyle: "italic" },
    // Le style du caret (couleur/épaisseur/glissement) est dans app.css.
    ".cm-selectionBackground": {
      background: "var(--accent-soft) !important",
      borderRadius: "3px",
    },
    "&.cm-focused .cm-selectionBackground": {
      background: "var(--accent-soft) !important",
    },
  });

  function makeState(doc: string) {
    return EditorState.create({
      doc,
      extensions: [
        history(),
        drawSelection(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        syntaxHighlighting(defaultHighlightStyle),
        livePreview(),
        EditorView.lineWrapping,
        editorTheme,
        cmPlaceholder("Écris ta note…  (Markdown)"),
        EditorView.updateListener.of((u) => {
          if (u.docChanged) onChange?.(u.state.doc.toString());
        }),
      ],
    });
  }

  onMount(() => {
    view = new EditorView({ parent: el, state: makeState(value) });
    lastPath = notePath;
    view.focus();
    return () => view?.destroy();
  });

  // Recharge le document uniquement quand on change de note (pas à chaque frappe).
  $effect(() => {
    const p = notePath;
    if (view && p !== lastPath) {
      lastPath = p;
      view.setState(makeState(value));
      view.focus();
    }
  });

  export function focus() {
    view?.focus();
  }
</script>

<div class="editor" bind:this={el}></div>

<style>
  .editor {
    height: 100%;
    overflow: hidden;
  }
</style>
