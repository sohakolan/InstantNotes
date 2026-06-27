import { syntaxTree } from "@codemirror/language";
import { RangeSet, type Range } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";

/** Décoration qui masque (collapse) une plage : utilisée pour la syntaxe markdown. */
const hide = Decoration.replace({});

/** Nœuds de contenu auxquels on applique un style inline. */
const STYLE_CLASS: Record<string, string> = {
  StrongEmphasis: "cm-md-strong",
  Emphasis: "cm-md-em",
  InlineCode: "cm-md-code",
  Strikethrough: "cm-md-strike",
  Link: "cm-md-link",
};

/** Lignes de titre → classe (taille). */
const HEADING_CLASS: Record<string, string> = {
  ATXHeading1: "cm-md-h1",
  ATXHeading2: "cm-md-h2",
  ATXHeading3: "cm-md-h3",
  ATXHeading4: "cm-md-h4",
  ATXHeading5: "cm-md-h5",
  ATXHeading6: "cm-md-h6",
};

/** Marqueurs de syntaxe à masquer hors de la ligne active. */
const MARK_NODES = new Set([
  "HeaderMark",
  "EmphasisMark",
  "CodeMark",
  "StrikethroughMark",
  "QuoteMark",
  "LinkMark",
  "URL",
]);

function buildDecorations(view: EditorView): DecorationSet {
  const ranges: Range<Decoration>[] = [];
  const doc = view.state.doc;

  // Lignes couvertes par une sélection/curseur : on y montre la syntaxe brute.
  const activeLines = new Set<number>();
  for (const r of view.state.selection.ranges) {
    const start = doc.lineAt(r.from).number;
    const end = doc.lineAt(r.to).number;
    for (let l = start; l <= end; l++) activeLines.add(l);
  }

  for (const { from, to } of view.visibleRanges) {
    syntaxTree(view.state).iterate({
      from,
      to,
      enter: (node) => {
        const name = node.name;

        if (STYLE_CLASS[name]) {
          ranges.push(Decoration.mark({ class: STYLE_CLASS[name] }).range(node.from, node.to));
        } else if (HEADING_CLASS[name]) {
          ranges.push(Decoration.mark({ class: HEADING_CLASS[name] }).range(node.from, node.to));
        }

        if (MARK_NODES.has(name)) {
          const lineNo = doc.lineAt(node.from).number;
          if (!activeLines.has(lineNo)) {
            let end = node.to;
            // Avale l'espace après `#` ou `>` pour ne pas laisser d'indentation.
            if (
              (name === "HeaderMark" || name === "QuoteMark") &&
              doc.sliceString(end, end + 1) === " "
            ) {
              end += 1;
            }
            if (end > node.from) ranges.push(hide.range(node.from, end));
          }
        }
      },
    });
  }

  ranges.sort((a, b) => a.from - b.from || a.value.startSide - b.value.startSide);
  return RangeSet.of(ranges, true);
}

const livePreviewPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = buildDecorations(view);
    }
    update(u: ViewUpdate) {
      if (u.docChanged || u.viewportChanged || u.selectionSet) {
        this.decorations = buildDecorations(u.view);
      }
    }
  },
  { decorations: (v) => v.decorations },
);

const livePreviewTheme = EditorView.baseTheme({
  ".cm-md-h1": { fontSize: "1.7em", fontWeight: "700", lineHeight: "1.3" },
  ".cm-md-h2": { fontSize: "1.45em", fontWeight: "700", lineHeight: "1.3" },
  ".cm-md-h3": { fontSize: "1.25em", fontWeight: "600" },
  ".cm-md-h4": { fontSize: "1.1em", fontWeight: "600" },
  ".cm-md-h5": { fontSize: "1em", fontWeight: "600" },
  ".cm-md-h6": { fontSize: "1em", fontWeight: "600", opacity: "0.8" },
  ".cm-md-strong": { fontWeight: "700" },
  ".cm-md-em": { fontStyle: "italic" },
  ".cm-md-strike": { textDecoration: "line-through", opacity: "0.7" },
  ".cm-md-code": {
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
    background: "rgba(135,131,120,0.18)",
    borderRadius: "4px",
    padding: "0.1em 0.3em",
  },
  ".cm-md-link": { color: "#3b82f6", textDecoration: "underline", cursor: "pointer" },
});

/** Extension CodeMirror : aperçu live hybride façon Obsidian. */
export function livePreview() {
  return [livePreviewPlugin, livePreviewTheme];
}
