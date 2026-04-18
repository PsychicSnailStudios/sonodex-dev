<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import {
		spotifyIsConnected,
		getSpotifyPlaylists,
		importSpotifyPlaylist,
		type SpotifyPlaylistSummary,
	} from "$lib/ts/connections/spotify";
	import { reloadLibrary } from "$lib/ts/library.svelte";
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { onMount } from "svelte";
   import Fuse from "fuse.js";
    import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";

	let { folder = null, onClose }: { folder?: string | null; onClose: () => void } = $props();

	let connected = $state(false);
	let playlists = $state<SpotifyPlaylistSummary[]>([]);
	let loading = $state(false);
	let importing = $state<string | null>(null);

	let search = $state("");

	const fuse = $derived(
		new Fuse(playlists, {
			keys: [
					{ name: "title",        weight: 0.5,  getFn: (t) => t.name ?? ""                        },
			],
			threshold:          0.35,  // 0 = exact only, 1 = match anything
			ignoreLocation:     true,  // don't penalise matches deep in a string
			includeScore:       false,
			useExtendedSearch:  false,
			minMatchCharLength: 2,     // ignore single-character queries
		})
	);

	const filteredPlaylists = $derived(
		search.trim().length < 2
			? playlists
			: fuse.search(search).map((r) => r.item)
	);

	onMount(async () => {
		connected = await spotifyIsConnected();
		if (connected) await loadPlaylists();
	});

	async function loadPlaylists() {
		loading = true;
		try {
			playlists = await getSpotifyPlaylists();
		} catch (e) {
			console.error("Failed to load Spotify playlists:", e);
		} finally {
			loading = false;
		}
	}

	async function handleImport(pl: SpotifyPlaylistSummary) {
		importing = pl.id;
		try {
			const uid = await importSpotifyPlaylist(pl.id, pl.name, pl.owner);
			await reloadLibrary("playlists");
			setSelection(uid, "playlist");
			onClose();
		} catch (e) {
			console.error("Failed to import playlist:", e);
		} finally {
			importing = null;
		}
	}
</script>

{#if !connected}
	<div class="flex flex-col items-center justify-center gap-3 py-8 text-center">
		<p class="text-sm text-muted-foreground">Connect your Spotify account in Settings to import playlists.</p>
	</div>
{:else if loading}
	<div class="flex items-center justify-center py-8">
		<p class="text-sm text-muted-foreground">Loading playlists...</p>
	</div>
{:else if playlists.length === 0}
	<div class="flex flex-col items-center justify-center gap-3 py-8 text-center">
		<p class="text-sm text-muted-foreground">No playlists found.</p>
		<Button variant="outline" onclick={loadPlaylists}>Refresh</Button>
	</div>
{:else}
	<div class="pt-3 overflow-hidden">
		<div class="flex items-center justify-center mb-2">
			<SearchBar bind:search searchCount={playlists.length} />
		</div>
		<ScrollArea class="h-[360px]">
			<div class="flex flex-col gap-1 pr-3">
				{#each filteredPlaylists as pl}
					<div class="flex items-center justify-between border rounded px-3 py-2 text-sm w-full">
						<div class="flex flex-col min-w-0">
							<span class="font-medium truncate">{pl.name}</span>
							<span class="text-xs text-muted-foreground">{pl.track_count} tracks · {pl.owner}</span>
						</div>
						<Button
							variant="outline"
							class="shrink-0 ml-2"
							onclick={() => handleImport(pl)}
							disabled={importing === pl.id}
						>
							{importing === pl.id ? "Importing..." : "Import"}
						</Button>
					</div>
				{/each}
			</div>
		</ScrollArea>
	</div>
{/if}
