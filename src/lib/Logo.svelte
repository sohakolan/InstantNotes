<script lang="ts">
  // Logo de marque InstantNotes (note + éclair). Couleurs fixes = cohérence avec l'icône d'app.
  let { size = 56 }: { size?: number } = $props();
  // Identifiants uniques pour éviter les collisions de <defs> si plusieurs logos sont rendus.
  const uid = Math.random().toString(36).slice(2, 8);
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 1024 1024"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  class="logo"
  aria-label="InstantNotes"
  role="img"
>
  <defs>
    <linearGradient id="bg-{uid}" x1="160" y1="80" x2="880" y2="960" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#6ea0ff" />
      <stop offset="0.55" stop-color="#4f7cff" />
      <stop offset="1" stop-color="#6a4bff" />
    </linearGradient>
    <linearGradient id="sheen-{uid}" x1="512" y1="40" x2="512" y2="560" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#ffffff" stop-opacity="0.22" />
      <stop offset="1" stop-color="#ffffff" stop-opacity="0" />
    </linearGradient>
    <linearGradient id="bolt-{uid}" x1="430" y1="300" x2="600" y2="780" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#ffd64a" />
      <stop offset="1" stop-color="#ff9d0a" />
    </linearGradient>
  </defs>

  <rect x="64" y="64" width="896" height="896" rx="220" fill="url(#bg-{uid})" />
  <rect x="64" y="64" width="896" height="896" rx="220" fill="url(#sheen-{uid})" />

  <rect x="286" y="212" width="452" height="600" rx="56" fill="#ffffff" />

  <g fill="#cfd6ec">
    <rect x="350" y="300" width="324" height="34" rx="17" />
    <rect x="350" y="372" width="324" height="34" rx="17" />
    <rect x="350" y="444" width="210" height="34" rx="17" />
  </g>

  <path
    d="M566 268 L356 596 L488 596 L446 812 L690 484 L548 484 Z"
    fill="url(#bolt-{uid})"
    stroke="#ffffff"
    stroke-width="22"
    stroke-linejoin="round"
  />
</svg>

<style>
  .logo {
    border-radius: 24%;
    box-shadow: 0 12px 30px rgba(79, 124, 255, 0.32);
    flex-shrink: 0;
  }
</style>
