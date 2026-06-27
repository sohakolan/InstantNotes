import { load, type Store } from "@tauri-apps/plugin-store";
import type { TreeNode } from "./ipc";

export type Theme = "system" | "light" | "dark";
export type EditorFont =
  | "system"
  | "newyork"
  | "georgia"
  | "avenir"
  | "helvetica"
  | "menlo"
  | "sfmono"
  | "monaco"
  | "courier";
export type CaretColor = "accent" | string;
export type PushFrequency = "manual" | "hourly" | "daily" | "onsave";

export interface Settings {
  vaultPath: string | null;
  shortcut: string;
  panelWidthPct: number;
  gitEnabled: boolean;
  lastNote: string | null;
  recents: string[];
  // Synchronisation git distante
  gitRemote: string | null;
  pushFrequency: PushFrequency;
  pushTime: string; // "HH:MM" pour la fréquence quotidienne
  lastPushAt: number | null;
  // Apparence
  theme: Theme;
  accent: string;
  editorFontSize: number;
  editorFont: EditorFont;
  // Curseur
  caretWidth: number;
  caretColor: CaretColor;
  caretSmooth: boolean;
  caretBlink: boolean;
}

const DEFAULTS: Settings = {
  vaultPath: null,
  shortcut: "CmdOrCtrl+Alt+Ctrl+J",
  panelWidthPct: 20,
  gitEnabled: true,
  lastNote: null,
  recents: [],
  gitRemote: null,
  pushFrequency: "manual",
  pushTime: "19:00",
  lastPushAt: null,
  theme: "system",
  accent: "#4f7cff",
  editorFontSize: 15,
  editorFont: "system",
  caretWidth: 2,
  caretColor: "accent",
  caretSmooth: true,
  caretBlink: true,
};

export const ACCENT_PRESETS = [
  "#4f7cff",
  "#7c5cff",
  "#e0529c",
  "#ef6b4d",
  "#f0a93b",
  "#34c759",
  "#26b1c4",
];

export const FONTS: { id: EditorFont; label: string }[] = [
  { id: "system", label: "Système" },
  { id: "newyork", label: "New York" },
  { id: "georgia", label: "Georgia" },
  { id: "avenir", label: "Avenir Next" },
  { id: "helvetica", label: "Helvetica Neue" },
  { id: "menlo", label: "Menlo" },
  { id: "sfmono", label: "SF Mono" },
  { id: "monaco", label: "Monaco" },
  { id: "courier", label: "Courier" },
];

const FONT_STACKS: Record<EditorFont, string> = {
  system: "-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif",
  newyork: "'New York', ui-serif, 'Iowan Old Style', Georgia, serif",
  georgia: "Georgia, 'Times New Roman', serif",
  avenir: "'Avenir Next', Avenir, sans-serif",
  helvetica: "'Helvetica Neue', Helvetica, Arial, sans-serif",
  menlo: "Menlo, ui-monospace, monospace",
  sfmono: "'SF Mono', ui-monospace, SFMono-Regular, monospace",
  monaco: "Monaco, ui-monospace, monospace",
  courier: "'Courier New', Courier, monospace",
};

// Migration des anciennes valeurs de police (Sans/Serif/Mono).
const FONT_MIGRATION: Record<string, EditorFont> = {
  sans: "system",
  serif: "newyork",
  mono: "sfmono",
};

/** Applique thème, accent, typo et style de curseur via des variables CSS. */
export function applyAppearance(s: Settings) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.setAttribute("data-theme", s.theme);
  root.setAttribute("data-caret-smooth", String(s.caretSmooth));
  root.setAttribute("data-caret-blink", String(s.caretBlink));
  root.style.setProperty("--accent", s.accent);
  root.style.setProperty("--editor-font-size", `${s.editorFontSize}px`);
  root.style.setProperty("--editor-font-family", FONT_STACKS[s.editorFont] ?? FONT_STACKS.system);
  root.style.setProperty("--caret-width", `${s.caretWidth}px`);
  root.style.setProperty(
    "--caret-color",
    s.caretColor === "accent" ? "var(--accent)" : s.caretColor,
  );
}

/** Renvoie la variante « +Maj » d'un accélérateur (ou "" si Maj déjà présent). */
export function shiftVariant(acc: string): string {
  const parts = acc.split("+");
  if (parts.some((p) => p.toLowerCase() === "shift")) return "";
  const key = parts.pop()!;
  return [...parts, "Shift", key].join("+");
}

/** Ajoute un chemin en tête des récents (dédupliqué, plafonné). */
export function pushRecent(path: string) {
  const r = app.settings.recents.filter((p) => p !== path);
  r.unshift(path);
  app.settings.recents = r.slice(0, 8);
}

// Anciens raccourcis par défaut : migrés automatiquement vers le nouveau.
const LEGACY_SHORTCUTS = ["CmdOrCtrl+Alt+Ctrl+T", "CmdOrCtrl+Alt+Ctrl+M"];

/** État réactif global de l'app. */
export const app = $state({
  settings: { ...DEFAULTS },
  tree: [] as TreeNode[],
  currentNote: null as string | null,
  content: "",
  dirty: false,
  saving: false,
  pushing: false,
  pushError: "",
  paletteOpen: false,
  settingsOpen: false,
  ready: false,
});

let store: Store | null = null;

/** Charge les préférences persistées depuis le disque. */
export async function loadSettings(): Promise<Settings> {
  store = await load("settings.json", { autoSave: false, defaults: {} });
  const saved = (await store.get<Partial<Settings>>("settings")) ?? {};
  app.settings = { ...DEFAULTS, ...saved };
  // Migration des anciens raccourcis par défaut.
  if (LEGACY_SHORTCUTS.includes(app.settings.shortcut)) {
    app.settings.shortcut = DEFAULTS.shortcut;
    await saveSettings();
  }
  // Migration des anciennes polices (sans/serif/mono).
  const mig = FONT_MIGRATION[app.settings.editorFont as string];
  if (mig) {
    app.settings.editorFont = mig;
    await saveSettings();
  }
  applyAppearance(app.settings);
  return app.settings;
}

/** Persiste les préférences courantes. */
export async function saveSettings(): Promise<void> {
  if (!store) store = await load("settings.json", { autoSave: false, defaults: {} });
  await store.set("settings", $state.snapshot(app.settings));
  await store.save();
}
