<script lang="ts">

	// APP
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";

	// COMPONENTS
	import * as Tabs from "$lib/components/ui/tabs";
	import { Label } from "$lib/components/ui/label";
	import { Input } from "$lib/components/ui/input";
	import { Textarea } from "$lib/components/ui/textarea";
	import { Button } from "$lib/components/ui/button";
	import { Separator } from "$lib/components/ui/separator";

	// CUSTOM COMPONENTS
	import ArtworkEditor from "./ArtworkEditor.svelte";

	// SCRIPTS
	import { closeEditModal } from "$lib/ts/app/editModal.svelte";
	import { loadLibrary, reloadLibrary } from "$lib/ts/library.svelte";
	import { deletePlaylist } from "$lib/ts/audio/playlistManager.svelte";

	// PROPS
	let { uid } = $props<{ uid: string }>();

	let title = $state("");
	let description = $state("");
	let owner = $state("");
	let artworkPath = $state<string | null>(null);
	let saving = $state(false);

	// APP FUNCTIONS
	onMount(async () => {
		const playlist = await invoke<any | null>("get_playlist", { uid });
		if (!playlist) return;

		title = playlist.title ?? "";
		description = playlist.description ?? "";
		owner = playlist.owner ?? "";
		artworkPath = playlist.artwork_path ?? null;
	});

	// FUNCTIONS
	async function save() {
		saving = true;
		try {
			const update: Record<string, any> = {
				title: title || null,
				description: description || null,
				owner: owner || null,
				artwork_path: artworkPath,
			};

			await invoke("update_playlist_entry", { uid, update });
			await reloadLibrary("playlists");
		} finally {
			saving = false;
			closeEditModal();
		}
	}

	async function deleteThisPlaylist() {
		deletePlaylist(uid);
		closeEditModal();
	}
</script>

<div class="space-y-3 mt-4">
	<ArtworkEditor
		entityType="playlist"
		entityUid={uid}
		onchange={(path) => { artworkPath = path; }}
	/>
	<div class="space-y-1.5 mt-8">
		<Label for="playlist-title">Title</Label>
		<Input id="playlist-title" bind:value={title} />
	</div>
	<div class="space-y-1.5">
		<Label for="playlist-desc">Description</Label>
		<Textarea id="playlist-desc" bind:value={description} rows={4} />
	</div>
	<Separator class="my-4" />
</div>

<div class="flex justify-end gap-2">
	<Button variant="destructive" onclick={deleteThisPlaylist}>Delete</Button>
	<Button variant="outline" onclick={closeEditModal}>Cancel</Button>
	<Button onclick={() => save()} disabled={saving}>{saving ? "Saving…" : "Save"}</Button>
</div>
