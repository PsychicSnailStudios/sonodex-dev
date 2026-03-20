<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	import ArtworkEditor from "./ArtworkEditor.svelte";
	import { Label } from "$lib/components/ui/label";
	import { Input } from "$lib/components/ui/input";
	import { Textarea } from "$lib/components/ui/textarea";
	import { Button } from "$lib/components/ui/button";
	import { Separator } from "$lib/components/ui/separator";
	import * as Tabs from "$lib/components/ui/tabs";
	import { editModal, closeEditModal } from "$lib/editModal.svelte";
	import { loadLibrary } from "$lib/library.svelte";

	let { id } = $props<{ id: number }>();

	let title = $state("");
	let description = $state("");
	let owner = $state("");
	let artworkPath = $state<string | null>(null);
	let saving = $state(false);

	onMount(async () => {
		const playlist = await invoke<any | null>("get_playlist", { id });
		if (!playlist) return;

		title = playlist.title ?? "";
		description = playlist.description ?? "";
		owner = playlist.owner ?? "";
		artworkPath = playlist.artwork_path ?? null;
	});

	async function save() {
		saving = true;
		try {
			const update: Record<string, any> = {
				title: title || null,
				description: description || null,
				owner: owner || null,
				artwork_path: artworkPath,
			};

			await invoke("update_playlist_entry", { id, update });
			await loadLibrary();
			closeEditModal();
		} finally {
			saving = false;
		}
	}

	async function deletePlaylist() {
		await invoke("delete_playlist_entry", { id });
		await loadLibrary();
		closeEditModal();
	}
</script>

<Tabs.Root value="info">
	<Tabs.List class="w-full">
		<Tabs.Trigger value="info" class="flex-1">Info</Tabs.Trigger>
		<Tabs.Trigger value="artwork" class="flex-1">Artwork</Tabs.Trigger>
	</Tabs.List>

	<Tabs.Content value="info" class="space-y-3 mt-4">
		<div class="space-y-1.5">
			<Label for="playlist-title">Title</Label>
			<Input id="playlist-title" bind:value={title} />
		</div>
		<div class="space-y-1.5">
			<Label for="playlist-owner">Owner</Label>
			<Input id="playlist-owner" bind:value={owner} />
		</div>
		<div class="space-y-1.5">
			<Label for="playlist-desc">Description</Label>
			<Textarea id="playlist-desc" bind:value={description} rows={4} />
		</div>
	</Tabs.Content>

	<Tabs.Content value="artwork" class="mt-4">
		<ArtworkEditor
			entityType="playlist"
			entityId={id}
			onchange={(path) => { artworkPath = path; }}
		/>
	</Tabs.Content>
</Tabs.Root>

<Separator class="my-4" />

<div class="flex justify-end gap-2">
	<Button variant="destructive" onclick={deletePlaylist}>Delete</Button>
	<Button variant="outline" onclick={closeEditModal}>Cancel</Button>
	<Button onclick={() => save()} disabled={saving}>{saving ? "Saving…" : "Save"}</Button>
</div>