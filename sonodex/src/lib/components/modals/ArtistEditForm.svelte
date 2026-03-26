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
	import { editModal, closeEditModal } from "$lib/ts/app/editModal.svelte";
	import { loadLibrary } from "$lib/library.svelte";

	let { uid } = $props<{ uid: string }>();

	let name = $state("");
	let aka = $state("");
	let about = $state("");
	let genres = $state("");
	let tags = $state("");
	let websites = $state("");
	let members = $state("");
	let profileArtPath = $state<string | null>(null);
	let saving = $state(false);

	onMount(async () => {
		const artist = await invoke<any | null>("get_artist", { uid });
		if (!artist) return;

		name = artist.name ?? "";
		about = artist.about ?? "";
		profileArtPath = artist.profile_art_path ?? null;

		try { aka = (artist.aka ? JSON.parse(artist.aka) : []).join(", "); } catch { aka = ""; }
		try { genres = (artist.genres ? JSON.parse(artist.genres) : []).join(", "); } catch { genres = ""; }
		try { tags = (artist.tags ? JSON.parse(artist.tags) : []).join(", "); } catch { tags = ""; }
		try { websites = (artist.websites ? JSON.parse(artist.websites) : []).join(", "); } catch { websites = ""; }
		try { members = (artist.members ? JSON.parse(artist.members) : []).join(", "); } catch { members = ""; }
	});

	function splitList(val: string) {
		return val.split(",").map((s) => s.trim()).filter(Boolean);
	}

	async function save() {
		saving = true;
		try {
			const update: Record<string, any> = {
				name: name || null,
				about: about || null,
				aka: aka ? JSON.stringify(splitList(aka)) : null,
				genres: genres ? JSON.stringify(splitList(genres)) : null,
				tags: tags ? JSON.stringify(splitList(tags)) : null,
				websites: websites ? JSON.stringify(splitList(websites)) : null,
				members: members ? JSON.stringify(splitList(members)) : null,
				profile_art_path: profileArtPath,
			};

			await invoke("update_artist_entry", { uid, update });
			await loadLibrary();
			closeEditModal();
		} finally {
			saving = false;
		}
	}
</script>

<Tabs.Root value="info">
	<Tabs.List class="w-full">
		<Tabs.Trigger value="info" class="flex-1">Info</Tabs.Trigger>
		<Tabs.Trigger value="details" class="flex-1">Details</Tabs.Trigger>
		<Tabs.Trigger value="artwork" class="flex-1">Artwork</Tabs.Trigger>
	</Tabs.List>

	<Tabs.Content value="info" class="space-y-3 mt-4">
		<div class="space-y-1.5">
			<Label for="artist-name">Name</Label>
			<Input id="artist-name" bind:value={name} />
		</div>
		<div class="space-y-1.5">
			<Label for="artist-aka">Also Known As <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="artist-aka" bind:value={aka} />
		</div>
		<div class="space-y-1.5">
			<Label for="artist-genres">Genres <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="artist-genres" bind:value={genres} />
		</div>
		<div class="space-y-1.5">
			<Label for="artist-members">Members <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="artist-members" bind:value={members} />
		</div>
		<div class="space-y-1.5">
			<Label for="artist-about">Biography</Label>
			<Textarea id="artist-about" bind:value={about} rows={5} />
		</div>
	</Tabs.Content>

	<Tabs.Content value="details" class="space-y-3 mt-4">
		<div class="space-y-1.5">
			<Label for="artist-websites">Websites <span class="text-muted-foreground text-xs">(comma-separated URLs)</span></Label>
			<Input id="artist-websites" bind:value={websites} />
		</div>
		<div class="space-y-1.5">
			<Label for="artist-tags">Tags <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="artist-tags" bind:value={tags} />
		</div>
	</Tabs.Content>

	<Tabs.Content value="artwork" class="mt-4">
		<p class="text-xs text-muted-foreground mb-3">Profile art</p>
		<ArtworkEditor
			entityType="artist"
			entityUid={uid}
			onchange={(path) => { profileArtPath = path; }}
		/>
	</Tabs.Content>
</Tabs.Root>

<Separator class="my-4" />

<div class="flex justify-end gap-2">
	<Button variant="outline" onclick={closeEditModal}>Cancel</Button>
	<Button onclick={save} disabled={saving}>{saving ? "Saving…" : "Save"}</Button>
</div>
