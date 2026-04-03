<script lang="ts">
	import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	
	import { createPlaylist } from "$lib/ts/audio/playlistManager.svelte";
	import { profileState } from "$lib/ts/profiles.svelte";

	let { open = $bindable(false), folder = null } = $props<{ open: boolean; folder?: string | null }>();

	let nameInput = $state("");

	async function handleCreatePlaylist() {
		if (!nameInput.trim()) return;
		await createPlaylist(nameInput.trim(), profileState.active?.name ?? null, folder);
		nameInput = "";
		open = false;
	}
</script>

<AlertDialog.Root bind:open={open}>
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