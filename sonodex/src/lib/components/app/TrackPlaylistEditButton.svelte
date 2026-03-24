<script lang="ts">
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import Button from "../ui/button/button.svelte";
	import Input from "../ui/input/input.svelte";
	import ScrollArea from "../ui/scroll-area/scroll-area.svelte";

	import { BadgePlus, CirclePlus, CircleCheck } from "lucide-svelte";

	import { library } from "$lib/library.svelte";
	import type { Playlist, Track } from "$lib/types";
	import { addTrackToPlaylist, removeTrackFromPlaylist } from "$lib/playlistManager.svelte";

	let search = $state("");

	let { track, isButton = true } = $props<{ track: Track; isButton?: boolean }>()

	const filteredPlaylists = $derived(
		search.trim() === ""
			? library.playlists
			: library.playlists.filter((a) => {
				const q = search.toLowerCase();
				return a.title.toLowerCase().includes(q) || (a.description?.toLowerCase() ?? "").includes(q);
			})
	);

	function hasTrack(playlistUid: string): boolean {
		const playlist = library.playlists.find(p => p.uid === playlistUid);
		if (!playlist) return false;
		const tracks: { uid: string }[] = JSON.parse(playlist.tracks ?? "[]");
		return tracks.some(t => t.uid === track.uid);
	}

	function updateTrack(playlist: Playlist, track: Track) {
		if (hasTrack(playlist.uid)) {
			removeTrackFromPlaylist(playlist, track)
		}
		else {
			addTrackToPlaylist(playlist, track)
		}
	}
</script>

<DropdownMenu.Root>
	{#if isButton}
	<DropdownMenu.Trigger>
		{#snippet child({ props }: { props: Record<string, unknown> })}
			<Button {...props} variant="ghost" size="icon"><BadgePlus /></Button>
		{/snippet}
	</DropdownMenu.Trigger>
	{:else}
	<DropdownMenu.Trigger>
		Add to playlist
	</DropdownMenu.Trigger>
	{/if}

	<DropdownMenu.Content>
		<DropdownMenu.Group>
			<DropdownMenu.Label>Add to playlist</DropdownMenu.Label>
			<Input placeholder="Search" bind:value={search}/>
			<DropdownMenu.Separator />

			<ScrollArea class="h-[250px]">
				<div>
					{#each filteredPlaylists as playlist}
						<div class="flex gap-2 p-2 items-center justify-between">
							<span>{playlist.title}</span>
							<Button variant="ghost" size="icon" onclick={() => updateTrack(playlist, track)}>
								{#if hasTrack(playlist.uid)}
									<CircleCheck />
								{:else}
									<CirclePlus />
								{/if}
							</Button>
						</div>
					{/each}
				</div>
			</ScrollArea>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>