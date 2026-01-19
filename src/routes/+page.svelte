<script lang="ts">
  /**
   * Main Entry View.
   *
   * Serves as the primary mounting point for the Kaomoji Picker component.
   * Standardizes global styles and document metadata.
   *
   * ARCHITECTURE:
   * ┌───────────────┐      ┌───────────────┐      ┌───────────────┐
   * │  Browser Env  │ ───► │ Svelte Shell  │ ───► │ Picker Comp   │
   * └───────────────┘      └───────────────┘      └───────────────┘
   *
   * Author: KleaSCM
   * Email: KleaSCM@gmail.com
   */

  import Picker from "$lib/Components/Picker.svelte";
  import "$lib/Styles.css";
</script>

<svelte:head>
  <title>meh</title>
</svelte:head>

<Picker />
