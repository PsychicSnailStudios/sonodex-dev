<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";

	import type { Playlist } from '$lib/types';
	import { library } from "$lib/library.svelte";

	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";

	import AudioCard from "$lib/components/app/AudioCard.svelte";

	let search = $state("");
	let dialogOpen = $state(false);
	let playlists = $derived(library.playlists ?? null);
	let nameInput = $state("");

	const filteredPlaylists = $derived(
		search.trim() === ""
			? playlists
			: playlists.filter((a) => {
				const q = search.toLowerCase();
				return a.title.toLowerCase().includes(q) || (a.description?.toLowerCase() ?? "").includes(q);
			})
	);

	onMount(async () => {
		playlists = await invoke("get_playlists");
	});

	async function createPlaylist() {
		if (!nameInput.trim()) return;
		await invoke("create_playlist_entry", {
			playlist: {
				uid: crypto.randomUUID(),
				title: nameInput.trim(),
				description: null,
				owner: null,
				tracks: null,
				artwork_path: null,
			}
		});
		playlists = await invoke("get_playlists");
		nameInput = "";
		dialogOpen = false;
	}
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Playlists</h1>

		<AlertDialog.Root bind:open={dialogOpen}>
			<AlertDialog.Trigger class={buttonVariants({ variant: "outline" })}>
				Create New
			</AlertDialog.Trigger>
			<AlertDialog.Content>
				<AlertDialog.Header>
					<AlertDialog.Title>New Playlist</AlertDialog.Title>
					<AlertDialog.Description>
						<Input placeholder="New Playlist" bind:value={nameInput} class="w-48"/>
					</AlertDialog.Description>
				</AlertDialog.Header>
				<AlertDialog.Footer>
					<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
					<AlertDialog.Action onclick={createPlaylist}>Create</AlertDialog.Action>
				</AlertDialog.Footer>
			</AlertDialog.Content>
		</AlertDialog.Root>

		<Input
			placeholder="Search..."
			bind:value={search}
			class="w-48"
		/>

	</div>

	<div class="min-h-0 flex-1 overflow-hidden">

		<div class="flex flex-col h-full w-full overflow-hidden">
			<span>{playlists.length} playlists</span>
			<ScrollArea class="min-h-0 min-w-0">

				<div class="app-music-grid grid gap-2 p-3">
					{#each filteredPlaylists as p}
						<AudioCard title={p.title} subTitle={p.owner} artworkUid={p.uid} type="playlist" />
					{/each}
				</div>

			</ScrollArea>
		</div>

	</div>

</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>
