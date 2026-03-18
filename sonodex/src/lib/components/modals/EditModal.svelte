<script lang="ts">
	import { editModal, closeEditModal } from "$lib/editModal.svelte";
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import TrackEditForm from "$lib/components/modals/TrackEditForm.svelte";
	import AlbumEditForm from "$lib/components/modals/AlbumEditForm.svelte";
	import ArtistEditForm from "$lib/components/modals/ArtistEditForm.svelte";
	import PlaylistEditForm from "$lib/components/modals/PlaylistEditForm.svelte";

	const titles: Record<string, string> = {
		track: "Edit Track",
		album: "Edit Album",
		artist: "Edit Artist",
		playlist: "Edit Playlist",
	};
</script>

<Dialog.Root open={editModal.open} onOpenChange={(v) => { if (!v) closeEditModal(); }}>
	<Dialog.Content class="max-w-lg w-full max-h-[90vh] overflow-y-auto">
		<Dialog.Header>
			<Dialog.Title>
				{editModal.target ? titles[editModal.target.type] : "Edit"}
			</Dialog.Title>
		</Dialog.Header>

		{#if editModal.target}
			{#if editModal.target.type === "track"}
				<TrackEditForm id={editModal.target.id} />
			{:else if editModal.target.type === "album"}
				<AlbumEditForm id={editModal.target.id} />
			{:else if editModal.target.type === "artist"}
				<ArtistEditForm id={editModal.target.id} />
			{:else if editModal.target.type === "playlist"}
				<PlaylistEditForm id={editModal.target.id} />
			{/if}
		{/if}
	</Dialog.Content>
</Dialog.Root>
