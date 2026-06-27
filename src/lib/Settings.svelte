<script lang="ts">
  import {
    app,
    saveSettings,
    applyAppearance,
    ACCENT_PRESETS,
    FONTS,
    shiftVariant,
    type Theme,
    type PushFrequency,
  } from "./state.svelte";
  import * as api from "./ipc";

  let {
    onClose,
    onChangeVault,
    onPushNow,
  }: { onClose: () => void; onChangeVault: () => void; onPushNow: () => void } = $props();

  const THEMES: { id: Theme; label: string }[] = [
    { id: "system", label: "Système" },
    { id: "light", label: "Clair" },
    { id: "dark", label: "Sombre" },
  ];

  const FREQS: { id: PushFrequency; label: string }[] = [
    { id: "manual", label: "Manuel" },
    { id: "hourly", label: "Chaque heure" },
    { id: "daily", label: "Quotidien" },
    { id: "onsave", label: "À chaque sauvegarde" },
  ];

  // ── Synchronisation git distante ──
  let remoteInput = $state(app.settings.gitRemote ?? "");
  let linking = $state(false);
  let linkError = $state("");

  async function linkRemote() {
    const url = remoteInput.trim();
    if (!url) return;
    linking = true;
    linkError = "";
    try {
      await api.gitSetRemote(url);
      app.settings.gitRemote = url;
      app.settings.gitEnabled = true; // le push s'appuie sur les commits git
      await saveSettings();
    } catch (e) {
      linkError = String(e);
    } finally {
      linking = false;
    }
  }

  async function unlinkRemote() {
    app.settings.gitRemote = null;
    app.settings.pushFrequency = "manual";
    remoteInput = "";
    await saveSettings();
  }

  function setFreq(f: PushFrequency) {
    app.settings.pushFrequency = f;
    saveSettings();
  }

  function setPushTime(v: string) {
    app.settings.pushTime = v;
    saveSettings();
  }

  function lastPushLabel(): string {
    const t = app.settings.lastPushAt;
    if (!t) return "Jamais poussé";
    const d = new Date(t);
    const p = (n: number) => String(n).padStart(2, "0");
    return `Dernier envoi : ${p(d.getDate())}/${p(d.getMonth() + 1)} à ${p(d.getHours())}:${p(d.getMinutes())}`;
  }

  // Applique immédiatement + persiste les changements d'apparence.
  function updateAppearance(patch: Partial<typeof app.settings>) {
    Object.assign(app.settings, patch);
    applyAppearance(app.settings);
    saveSettings();
  }

  let recording = $state(false);
  let error = $state("");
  let shortcut = $state(app.settings.shortcut);
  let widthPct = $state(app.settings.panelWidthPct);

  // Affiche l'accélérateur avec les symboles macOS (⌘⌥⌃⇧).
  function pretty(acc: string): string {
    return acc
      .split("+")
      .map((p) => {
        const k = p.toLowerCase();
        if (k === "cmdorctrl" || k === "cmd" || k === "command" || k === "super" || k === "meta")
          return "⌘";
        if (k === "ctrl" || k === "control") return "⌃";
        if (k === "alt" || k === "option") return "⌥";
        if (k === "shift") return "⇧";
        return p.toUpperCase();
      })
      .join(" ");
  }

  // Convertit un KeyboardEvent en accélérateur Tauri (ex. "CmdOrCtrl+Alt+Ctrl+T").
  function toAccelerator(e: KeyboardEvent): string | null {
    const mods: string[] = [];
    if (e.metaKey) mods.push("CmdOrCtrl");
    if (e.ctrlKey) mods.push("Ctrl");
    if (e.altKey) mods.push("Alt");
    if (e.shiftKey) mods.push("Shift");

    const key = e.key;
    // Ignore les touches de modification seules.
    if (["Meta", "Control", "Alt", "Shift"].includes(key)) return null;

    let main: string;
    if (key === " ") main = "Space";
    else if (/^[a-z]$/i.test(key)) main = key.toUpperCase();
    else if (/^F\d{1,2}$/.test(key)) main = key;
    else if (key.length === 1) main = key.toUpperCase();
    else main = key; // ArrowUp, Enter, etc.

    if (mods.length === 0) return null; // exige au moins un modificateur
    return [...mods, main].join("+");
  }

  function onRecordKey(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    const acc = toAccelerator(e);
    if (acc) {
      shortcut = acc;
      recording = false;
    }
  }

  async function apply() {
    error = "";
    try {
      await api.setShortcut(shortcut, shiftVariant(shortcut), widthPct);
      app.settings.shortcut = shortcut;
      app.settings.panelWidthPct = widthPct;
      await saveSettings();
    } catch (e) {
      error = String(e);
    }
  }

  async function toggleGit() {
    app.settings.gitEnabled = !app.settings.gitEnabled;
    await saveSettings();
  }

  function changeVault() {
    onClose();
    onChangeVault();
  }
</script>

<svelte:window onkeydown={onRecordKey} />

<div class="overlay" role="presentation" onclick={(e) => e.target === e.currentTarget && onClose()}>
  <div class="sheet" role="dialog" tabindex="-1">
    <header>
      <h2>Réglages</h2>
      <button class="x" onclick={onClose}>✕</button>
    </header>

    <section>
      <label for="sc">Raccourci global</label>
      <div class="row">
        <button
          id="sc"
          class="recorder"
          class:recording
          onclick={() => (recording = !recording)}
        >
          {recording ? "Appuie sur la combinaison…" : pretty(shortcut)}
        </button>
      </div>
      <p class="hint">Au moins un modificateur (⌘, ⌃, ⌥, ⇧) + une touche.</p>
    </section>

    <section>
      <label for="w">Largeur du panneau : {widthPct}%</label>
      <input id="w" type="range" min="10" max="50" bind:value={widthPct} />
    </section>

    <div class="divider"></div>

    <section>
      <span class="seclabel">Thème</span>
      <div class="seg">
        {#each THEMES as t}
          <button
            class:active={app.settings.theme === t.id}
            onclick={() => updateAppearance({ theme: t.id })}
          >
            {t.label}
          </button>
        {/each}
      </div>
    </section>

    <section>
      <span class="seclabel">Couleur d'accent</span>
      <div class="swatches">
        {#each ACCENT_PRESETS as c}
          <button
            class="swatch"
            class:active={app.settings.accent.toLowerCase() === c.toLowerCase()}
            style="--c: {c}"
            aria-label={c}
            onclick={() => updateAppearance({ accent: c })}
          ></button>
        {/each}
        <label class="swatch custom" style="--c: {app.settings.accent}" title="Couleur libre">
          <input
            type="color"
            value={app.settings.accent}
            oninput={(e) => updateAppearance({ accent: e.currentTarget.value })}
          />
        </label>
      </div>
    </section>

    <section>
      <label for="font">Police de l'éditeur</label>
      <select
        id="font"
        class="select"
        value={app.settings.editorFont}
        onchange={(e) => updateAppearance({ editorFont: e.currentTarget.value as any })}
      >
        {#each FONTS as f}
          <option value={f.id}>{f.label}</option>
        {/each}
      </select>
    </section>

    <section>
      <label for="fs">Taille du texte : {app.settings.editorFontSize}px</label>
      <input
        id="fs"
        type="range"
        min="12"
        max="22"
        value={app.settings.editorFontSize}
        oninput={(e) => updateAppearance({ editorFontSize: +e.currentTarget.value })}
      />
    </section>

    <div class="divider"></div>

    <section>
      <span class="seclabel">Curseur</span>
      <div class="caret-row">
        <span class="sub">Couleur</span>
        <div class="swatches">
          <button
            class="swatch"
            class:active={app.settings.caretColor === "accent"}
            style="--c: var(--accent)"
            aria-label="Accent"
            onclick={() => updateAppearance({ caretColor: "accent" })}
          ></button>
          {#each ["#1d1d20", "#e8e8ea", "#ef6b4d", "#34c759"] as c}
            <button
              class="swatch"
              class:active={app.settings.caretColor === c}
              style="--c: {c}"
              aria-label={c}
              onclick={() => updateAppearance({ caretColor: c })}
            ></button>
          {/each}
        </div>
      </div>
      <div class="caret-row">
        <label for="cw" class="sub">Épaisseur : {app.settings.caretWidth}px</label>
        <input
          id="cw"
          class="mini-range"
          type="range"
          min="1"
          max="4"
          value={app.settings.caretWidth}
          oninput={(e) => updateAppearance({ caretWidth: +e.currentTarget.value })}
        />
      </div>
      <div class="caret-row">
        <span class="sub">Glissement fluide</span>
        <button
          class="switch"
          class:on={app.settings.caretSmooth}
          role="switch"
          aria-checked={app.settings.caretSmooth}
          aria-label="Glissement fluide"
          onclick={() => updateAppearance({ caretSmooth: !app.settings.caretSmooth })}
        >
          <span class="knob"></span>
        </button>
      </div>
      <div class="caret-row">
        <span class="sub">Clignotement</span>
        <button
          class="switch"
          class:on={app.settings.caretBlink}
          role="switch"
          aria-checked={app.settings.caretBlink}
          aria-label="Clignotement"
          onclick={() => updateAppearance({ caretBlink: !app.settings.caretBlink })}
        >
          <span class="knob"></span>
        </button>
      </div>
    </section>

    <div class="divider"></div>

    <section class="inline">
      <span>Sauvegarde git automatique</span>
      <button
        class="switch"
        class:on={app.settings.gitEnabled}
        role="switch"
        aria-checked={app.settings.gitEnabled}
        aria-label="Sauvegarde git automatique"
        onclick={toggleGit}
      >
        <span class="knob"></span>
      </button>
    </section>

    <div class="divider"></div>

    <section>
      <span class="seclabel">Synchronisation (push git)</span>
      {#if !app.settings.gitRemote}
        <input
          class="text-input"
          type="text"
          placeholder="git@github.com:vous/notes.git"
          bind:value={remoteInput}
          onkeydown={(e) => e.key === "Enter" && linkRemote()}
        />
        <button
          class="primary sm"
          onclick={linkRemote}
          disabled={linking || !remoteInput.trim()}
        >
          {linking ? "Liaison…" : "Lier le dépôt"}
        </button>
        {#if linkError}<p class="error">{linkError}</p>{/if}
        <p class="hint">Utilise le <code>git</code> du système (clés SSH / identifiants déjà configurés).</p>
      {:else}
        <div class="remote">
          <span class="remote-url" title={app.settings.gitRemote}>{app.settings.gitRemote}</span>
          <button class="ghost sm" onclick={unlinkRemote}>Délier</button>
        </div>

        <span class="sub-label">Fréquence d'envoi</span>
        <div class="freq">
          {#each FREQS as f}
            <button
              class:active={app.settings.pushFrequency === f.id}
              onclick={() => setFreq(f.id)}
            >
              {f.label}
            </button>
          {/each}
        </div>

        {#if app.settings.pushFrequency === "daily"}
          <div class="caret-row">
            <label for="pt" class="sub">Heure quotidienne</label>
            <input
              id="pt"
              class="time-input"
              type="time"
              value={app.settings.pushTime}
              onchange={(e) => setPushTime(e.currentTarget.value)}
            />
          </div>
        {/if}

        <div class="push-row">
          <span class="push-status" class:err={!!app.pushError}>
            {#if app.pushing}Envoi en cours…{:else if app.pushError}Échec : {app.pushError}{:else}{lastPushLabel()}{/if}
          </span>
          <button class="primary sm" onclick={onPushNow} disabled={app.pushing}>
            Pousser
          </button>
        </div>
      {/if}
    </section>

    <div class="divider"></div>

    <section class="inline">
      <span class="vault" title={app.settings.vaultPath ?? ""}>
        Vault : {app.settings.vaultPath?.split("/").pop() ?? "—"}
      </span>
      <button class="ghost" onclick={changeVault}>Changer…</button>
    </section>

    {#if error}<p class="error">{error}</p>{/if}

    <footer>
      <button class="primary" onclick={apply}>Appliquer</button>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.32);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
    animation: overlay-in 0.14s ease;
  }
  .sheet {
    width: 92%;
    max-width: 380px;
    max-height: 86vh;
    overflow-y: auto;
    background: var(--elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    padding: 16px;
    box-shadow: var(--shadow);
    animation: pop-in 0.16s cubic-bezier(0.22, 1, 0.36, 1);
  }
  .divider {
    height: 1px;
    background: var(--border);
    margin: 14px -16px;
  }
  .seg {
    display: flex;
    gap: 4px;
    background: var(--hover);
    padding: 3px;
    border-radius: 9px;
  }
  .seg button {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 12px;
    font-weight: 500;
    padding: 6px 8px;
    border-radius: 7px;
    cursor: pointer;
    transition: background 0.12s, box-shadow 0.12s;
  }
  .seg button.active {
    background: var(--elevated);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
    font-weight: 600;
  }
  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .swatch {
    width: 24px;
    height: 24px;
    padding: 0;
    border-radius: 50%;
    border: 2px solid transparent;
    background: var(--c);
    cursor: pointer;
    transition: transform 0.1s;
    position: relative;
  }
  .swatch:hover {
    transform: scale(1.12);
  }
  .swatch.active {
    border-color: var(--text);
    box-shadow: 0 0 0 2px var(--elevated) inset;
  }
  .swatch.custom {
    display: grid;
    place-items: center;
    background:
      conic-gradient(from 0deg, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);
    overflow: hidden;
  }
  .swatch.custom input {
    opacity: 0;
    width: 100%;
    height: 100%;
    cursor: pointer;
  }
  .select {
    width: 100%;
    font-size: 13px;
    font-family: inherit;
    color: var(--text);
    background: var(--hover);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 10px;
    cursor: pointer;
    appearance: none;
  }
  .caret-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .caret-row .sub {
    font-size: 12px;
    font-weight: 500;
    opacity: 0.8;
  }
  .mini-range {
    width: 120px;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  h2 {
    font-size: 15px;
    margin: 0;
  }
  section {
    margin: 14px 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  section.inline {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
  label,
  .seclabel,
  .vault,
  section.inline span {
    font-size: 13px;
    font-weight: 600;
    opacity: 0.85;
  }
  .recorder {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 14px;
    padding: 10px 12px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--hover);
    color: var(--text);
    cursor: pointer;
  }
  .recorder.recording {
    border-color: var(--accent);
    color: var(--accent);
  }
  .hint {
    font-size: 11px;
    opacity: 0.55;
    margin: 0;
  }
  input[type="range"] {
    width: 100%;
    accent-color: var(--accent);
  }
  .ghost {
    border: 1px solid var(--border);
    background: var(--hover);
    color: var(--text);
    font-size: 12px;
    padding: 6px 12px;
    border-radius: 8px;
    cursor: pointer;
  }

  /* Interrupteur façon iOS */
  .switch {
    flex-shrink: 0;
    width: 40px;
    height: 24px;
    padding: 2px;
    border: none;
    border-radius: 999px;
    background: var(--border-strong);
    cursor: pointer;
    transition: background 0.18s ease;
  }
  .switch .knob {
    display: block;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    transition: transform 0.18s cubic-bezier(0.22, 1, 0.36, 1);
  }
  .switch.on {
    background: var(--accent);
  }
  .switch.on .knob {
    transform: translateX(16px);
  }

  /* Synchronisation git */
  .text-input,
  .time-input {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--text);
    background: var(--hover);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 9px 10px;
  }
  .text-input {
    width: 100%;
  }
  .text-input:focus,
  .time-input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .remote {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .remote-url {
    flex: 1;
    font-family: var(--mono);
    font-size: 12px;
    font-weight: 500;
    opacity: 0.85;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub-label {
    font-size: 12px;
    font-weight: 600;
    opacity: 0.7;
    margin-top: 2px;
  }
  .freq {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
    background: var(--hover);
    padding: 3px;
    border-radius: 9px;
  }
  .freq button {
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 12px;
    font-weight: 500;
    padding: 7px 8px;
    border-radius: 7px;
    cursor: pointer;
    transition: background 0.12s, box-shadow 0.12s;
  }
  .freq button.active {
    background: var(--elevated);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
    font-weight: 600;
  }
  .push-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-top: 2px;
  }
  .push-status {
    font-size: 11px;
    opacity: 0.6;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .push-status.err {
    color: #ef4444;
    opacity: 1;
  }
  .primary.sm {
    padding: 7px 14px;
    font-size: 12px;
    border-radius: 8px;
  }
  .primary.sm:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .ghost.sm {
    padding: 5px 10px;
    font-size: 11px;
  }
  .x {
    border: none;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    opacity: 0.6;
  }
  .error {
    color: #ef4444;
    font-size: 12px;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    margin-top: 12px;
  }
  .primary {
    border: none;
    background: var(--accent);
    color: white;
    font-weight: 600;
    font-size: 13px;
    padding: 9px 18px;
    border-radius: 9px;
    cursor: pointer;
  }
</style>
