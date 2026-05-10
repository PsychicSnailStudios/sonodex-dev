<script lang="ts">
   import { setSelection } from "$ts/store/session.svelte";
   import { getArtistUidFromName } from "$ts/store/library.svelte";
   import { parseArtists } from "$ts/util/helpers";

	// PROPS
	let { artists } = $props<{ artists: string }>();

	// VARIABLES
	let artistList = JSON.parse(artists);

</script>

{#if artists}
		{#each artistList as artist, i}
			<button onclick={() => setSelection(getArtistUidFromName(artist), "artist")} class="text-sm truncate cursor-pointer hover:underline">
				<p class="text-sm leading-relaxed">{artist}{i < artistList.length - 1 ? ", " : ""}</p>
			</button>
		{/each}
{/if}