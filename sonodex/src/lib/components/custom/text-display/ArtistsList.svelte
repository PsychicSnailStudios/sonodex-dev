<script lang="ts">
   import { setSelection } from "$ts/store/session.svelte";
    import type { ArtistEntry } from "$ts/util/types";

	// PROPS
	let { artists } = $props<{ artists: ArtistEntry[] }>();

</script>

{#if artists}
		{#each artists as artist, i}
			<button onclick={() => setSelection(artist.uid)} class="text-sm truncate cursor-pointer hover:underline">
				<p class="text-sm leading-relaxed">{artist.name}{i < artists.length - 1 ? ", " : ""}</p>
			</button>
		{/each}
{/if}