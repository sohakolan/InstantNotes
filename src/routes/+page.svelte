<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Panel from "$lib/Panel.svelte";
  import Manager from "$lib/Manager.svelte";

  // Une seule page : la fenêtre « manager » affiche l'organisateur, sinon le panneau.
  const isManager = getCurrentWindow().label === "manager";
</script>

{#if isManager}
  <Manager />
{:else}
  <Panel />
{/if}
