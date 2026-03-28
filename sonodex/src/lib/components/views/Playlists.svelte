<script lang="ts">
	import { library } from "$lib/library.svelte";
	import { dragState, setHoveredPlaylist, endDrag } from "$lib/dragState.svelte";
	import { addTracksToPlaylist, createPlaylist } from "$lib/playlistManager.svelte";
	import { profileState } from "$lib/profiles.svelte";

	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import AudioCard from "$lib/components/app/AudioCard.svelte";
	import { FolderPlus, ListPlus, FileDown } from "lucide-svelte";

	let search = $state("");
	let dialogOpen = $state(false);
	let createDialogOpen = $state(false);
	let nameInput = $state("");

	let playlists = $derived(library.playlists ?? []);

	const filteredPlaylists = $derived(
		search.trim() === ""
			? playlists
			: playlists.filter((a) => {
				const q = search.toLowerCase();
				return a.title.toLowerCase().includes(q) || (a.description?.toLowerCase() ?? "").includes(q);
			})
	);

	async function handleCreatePlaylist() {
		if (!nameInput.trim()) return;
		const ownerName = profileState.active?.name ?? null
		await createPlaylist(nameInput.trim(), ownerName)
		nameInput = "";
		createDialogOpen = false;
	}

	function handleCardDragLeave(e: DragEvent, playlistUid: string) {
		if (dragState.hoveredPlaylistUid === playlistUid) {
			setHoveredPlaylist(null);
		}
	}

	function handleCardDragOver(e: DragEvent, playlistUid: string) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
		setHoveredPlaylist(playlistUid);
	}

	async function handleCardDrop(e: DragEvent, playlistUid: string) {
		e.preventDefault();
		setHoveredPlaylist(null);
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) return;
		const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
		await addTracksToPlaylist(playlistUid, uids);
		endDrag();
	}
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Playlists</h1>

		<Input
			placeholder="Search..."
			bind:value={search}
			class="w-48"
		/>
	</div>

	<div class="flex flex-col h-full w-full overflow-hidden">
		<div class="flex justify-between">
			<span>{playlists.length} playlists</span>
			<div class="flex gap-1">
				<AlertDialog.Root bind:open={createDialogOpen}>
					<AlertDialog.Trigger class={buttonVariants({ variant: "outline" })}>
						<ListPlus />
					</AlertDialog.Trigger>
					<AlertDialog.Content>
						<AlertDialog.Header>
							<AlertDialog.Title>New Playlist</AlertDialog.Title>
							<AlertDialog.Description>
								<Input placeholder="Playlist name" bind:value={nameInput} class="w-full mt-2" />
							</AlertDialog.Description>
						</AlertDialog.Header>
						<AlertDialog.Footer>
							<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
							<AlertDialog.Action onclick={handleCreatePlaylist}>Create</AlertDialog.Action>
						</AlertDialog.Footer>
					</AlertDialog.Content>
				</AlertDialog.Root>

				<AlertDialog.Root bind:open={dialogOpen}>
					<AlertDialog.Trigger class={buttonVariants({ variant: "outline" })}>
						<FolderPlus />
					</AlertDialog.Trigger>
					<AlertDialog.Content>
						<AlertDialog.Header>
							<AlertDialog.Title>New Folder</AlertDialog.Title>
							<AlertDialog.Description>
								<Input placeholder="Folder name" bind:value={nameInput} class="w-48" />
							</AlertDialog.Description>
						</AlertDialog.Header>
						<AlertDialog.Footer>
							<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
							<AlertDialog.Action>Create</AlertDialog.Action>
						</AlertDialog.Footer>
					</AlertDialog.Content>
				</AlertDialog.Root>

				<AlertDialog.Root bind:open={dialogOpen}>
					<AlertDialog.Trigger class={buttonVariants({ variant: "outline" })}>
						<FileDown />
					</AlertDialog.Trigger>
					<AlertDialog.Content>
						<AlertDialog.Header>
							<AlertDialog.Title>New Folder</AlertDialog.Title>
							<AlertDialog.Description>
								<Input placeholder="Folder name" bind:value={nameInput} class="w-48" />
							</AlertDialog.Description>
						</AlertDialog.Header>
						<AlertDialog.Footer>
							<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
							<AlertDialog.Action>Create</AlertDialog.Action>
						</AlertDialog.Footer>
					</AlertDialog.Content>
				</AlertDialog.Root>
			</div>
		</div>

		<ScrollArea class="min-h-0 min-w-0">
			<div class="app-music-grid grid gap-2 pt-2">
				{#each filteredPlaylists as p (p.uid)}
				<div
						class="relative transition-all w-full"
						class:ring-2={dragState.hoveredPlaylistUid === p.uid}
						class:ring-primary={dragState.hoveredPlaylistUid === p.uid}
						class:rounded-md={dragState.hoveredPlaylistUid === p.uid}
						ondragover={(e) => handleCardDragOver(e, p.uid)}
						ondragleave={(e) => handleCardDragLeave(e, p.uid)}
						ondrop={(e) => handleCardDrop(e, p.uid)}
						role="region"
						aria-label="Playlist drop target"
					>
						<AudioCard title={p.title} subTitle={p.owner} artworkUid={p.uid} type="playlist" />
					</div>
				{/each}
			</div>
		</ScrollArea>
	</div>

</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>