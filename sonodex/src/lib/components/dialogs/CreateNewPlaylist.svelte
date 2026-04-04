<script lang="ts">
	import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	import ImportPlaylist from "$lib/components/dialogs/ImportPlaylist.svelte";
	
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
		<div class="h-[400px] flex flex-col justify-between">
			<Tabs.Root value="new">
				<Tabs.List class="w-full">
					<Tabs.Trigger value="new" class="flex-1">New</Tabs.Trigger>
					<Tabs.Trigger value="import" class="flex-1">Import</Tabs.Trigger>
					<Tabs.Trigger value="url" class="flex-1">URL</Tabs.Trigger>
				</Tabs.List>
	
				<Tabs.Content value="new">
					<AlertDialog.Header>
						<AlertDialog.Title class="mt-4">New Playlist</AlertDialog.Title>
						<AlertDialog.Description>
							<Input placeholder="Playlist name" bind:value={nameInput} class="w-full mt-2" />
						</AlertDialog.Description>
					</AlertDialog.Header>
				</Tabs.Content>
				
				<Tabs.Content value="import">
					<ImportPlaylist/>
				</Tabs.Content>
	
				<Tabs.Content value="url">
				</Tabs.Content>
			</Tabs.Root>
	
			<AlertDialog.Footer>
				<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
				<AlertDialog.Action onclick={handleCreatePlaylist}>Create</AlertDialog.Action>
			</AlertDialog.Footer>
		</div>
	</AlertDialog.Content>
</AlertDialog.Root>