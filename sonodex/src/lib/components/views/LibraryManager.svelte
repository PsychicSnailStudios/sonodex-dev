<script lang="ts">

	// APP
	import { onMount } from "svelte";

	// COMPONENTS
	import { Pencil, Trash, FolderInput, Paperclip, CloudDownload } from "lucide-svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Button } from "$lib/components/ui/button/index.js";

	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import ArtistsList from "$lib/components/app-ui/ArtistsList.svelte";
	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";
	
	// SCRIPTS
   import { getAlbumUidFromName, library } from "$lib/ts/library.svelte";
	import { parseAlbumEntries } from "$lib/ts/util/helpers";
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	
	// VARIABLES
	let search = $state("");
	let ghosts = $state(false);
	
	const filteredTracks = $derived(
		search.trim() === ""
			? library.tracks.filter((t) => {
					if (!ghosts) return true;
					return t.path == "";
			  })
			: library.tracks.filter((t) => {
					const q = search.toLowerCase();
					const title = t.title?.toLowerCase() ?? "";
					const artists = t.artists?.toLowerCase() ?? "";
					const album_artist = t.album_artist?.toLowerCase() ?? "";
					const albums = t.albums?.toLowerCase() ?? "";
					return (title.includes(q) || artists.includes(q) || album_artist.includes(q) || albums.includes(q)) && t.path == "";
			  })
	);

	// APP FUNCTIONS

	// FUNCTIONS


</script>

<div class="flex flex-col gap-2 p-2 border-2 rounded-md h-full w-full overflow-hidden">
	<h2 class="h2">Library Manager</h2>

	<ScrollArea class="h-full w-full min-h-0 min-w-0">
		<div class="flex flex-col gap-4 p-2 pr-4">
			<Tabs.Root value="tracks" class="flex flex-col min-h-0 flex-1">
				<Tabs.List class="w-full">
					<Tabs.Trigger value="tracks" class="flex-1">Tracks</Tabs.Trigger>
					<Tabs.Trigger value="albums" class="flex-1">Albums</Tabs.Trigger>
					<Tabs.Trigger value="artists" class="flex-1">Artists</Tabs.Trigger>
					<Tabs.Trigger value="tags" class="flex-1">Tags</Tabs.Trigger>
					<Tabs.Trigger value="duplicates" class="flex-1">Duplicates</Tabs.Trigger>
				</Tabs.List>

				<Tabs.Content value="tracks">
					<div class="flex flex-col gap-2 pt-2">
						<SearchBar bind:search searchCount={filteredTracks.length} />

						<div class="flex flex-col gap-2">
							{#each filteredTracks as track}
								<div class="flex gap-2 p-2 border-2 rounded-md justify-between items-center">
									<div class="flex gap-2">
										<ArtworkDisplay uid={track.uid} size={40} type="track" />
										<div class="min-w-0 grid">
											{#if track.path != ""}
												<span class="text-sm truncate">{track.title}</span>
											{:else}
												<span class="text-sm text-muted-foreground truncate">{track.title}</span>
											{/if}
											<div class="text-xs text-muted-foreground truncate">
												<ArtistsList artists={track.artists} />
												{#if track.albums}
												{@const albumList = parseAlbumEntries(track.albums)}
												{" : "}
												{#each albumList as album, i}
													<button onclick={() => setSelection(getAlbumUidFromName(album.name), "album")} class="text-sm truncate cursor-pointer hover:underline">
														{album.name}{i < albumList.length - 1 ? "," : ""}
													</button>
												{/each}
												{/if}
											</div>
										</div>
									</div>

									<div class="flex gap-2">
										<Button size="icon" variant="destructive"><Trash/></Button>
										<Button size="icon" variant="outline"><CloudDownload/></Button>
										<Button size="icon" variant="outline"><Paperclip/></Button>
										{#if track.path != ""}
											<Button size="icon" variant="outline"><FolderInput/></Button>
											{/if}
										<Button size="icon" variant="outline" onclick={() => openEditModal({ type: "track", uid: track!.uid })}><Pencil/></Button>
									</div>
								</div>
							{/each}
						</div>
					</div>
				</Tabs.Content>

				<Tabs.Content value="albums">
					<p>Albums</p>
				</Tabs.Content>
				<Tabs.Content value="artists">
					<p>Artists</p>
				</Tabs.Content>
				<Tabs.Content value="tags">
					<p>Tags</p>
				</Tabs.Content>
				<Tabs.Content value="duplicates">
					<p>Duplicates</p>
				</Tabs.Content>
			</Tabs.Root>
		</div>
	</ScrollArea>
</div>