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
	import { reloadLibrary } from "$lib/ts/library.svelte";
	import { syncArtists, pruneArtists } from "$lib/ts/dbManager";
	import { enrichAlbum } from "$lib/ts/app/enrichment";

	// PROPS
	let { uid } = $props<{ uid: string }>();

	// VARIABLES
	let title = $state("");
	let albumArtist = $state("");
	let artists = $state("");
	let releaseDate = $state("");
	let format = $state("");
	let genres = $state("");
	let label = $state("");
	let rating = $state<string>("");
	let credits = $state("");
	let tags = $state("");
	let artworkPath = $state<string | null>(null);
	let saving = $state(false);

	// Track originals for pruning on save
	let originalArtists = $state<string[]>([]);
	let originalAlbumArtist = $state("");

	// APP FUNCTIONS
	onMount(async () => {
		const album = await invoke<any | null>("get_album", { uid });
		if (!album) return;

		title = album.title ?? "";
		albumArtist = album.album_artist ?? "";
		originalAlbumArtist = albumArtist;
		releaseDate = album.release_date ?? "";
		format = album.format ?? "";
		label = album.label ?? "";
		credits = album.credits ?? "";
		rating = album.rating != null ? String(album.rating) : "";
		artworkPath = album.artwork_path ?? null;

		try {
			const arr = album.artists ? JSON.parse(album.artists) : [];
			artists = arr.join(", ");
			originalArtists = arr;
		} catch { artists = ""; originalArtists = []; }

		try {
			const arr = album.genres ? JSON.parse(album.genres) : [];
			genres = arr.join(", ");
		} catch { genres = ""; }

		try {
			const arr = album.tags ? JSON.parse(album.tags) : [];
			tags = arr.join(", ");
		} catch { tags = ""; }
	});

	// FUNCTIONS
	async function save() {
		saving = true;
		try {
			const artistArr = artists.split(",").map((s) => s.trim()).filter(Boolean);
			const allNewNames = [...artistArr, ...(albumArtist ? [albumArtist] : [])];

			// Create any newly added artist records
			await syncArtists(allNewNames);

			// Delete artist records that are no longer referenced anywhere
			const removedArtists = [
				...originalArtists.filter((n) => !artistArr.map((x) => x.toLowerCase()).includes(n.toLowerCase())),
				...(originalAlbumArtist && originalAlbumArtist !== albumArtist ? [originalAlbumArtist] : []),
			];
			await pruneArtists(removedArtists);

			const update: Record<string, any> = {
				title: title || null,
				album_artist: albumArtist || null,
				artists: artistArr.length ? JSON.stringify(artistArr) : null,
				release_date: releaseDate || null,
				format: format || null,
				genres: genres ? JSON.stringify(genres.split(",").map((s) => s.trim()).filter(Boolean)) : null,
				label: label || null,
				rating: rating ? Number(rating) : null,
				credits: credits || null,
				tags: tags ? JSON.stringify(tags.split(",").map((s) => s.trim()).filter(Boolean)) : null,
				artwork_path: artworkPath,
			};

			await invoke("update_album_entry", { uid, update });
			await reloadLibrary("albums");
			await reloadLibrary("artists");
		} finally {
			saving = false;
			closeEditModal();
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
			<Label for="album-title">Title</Label>
			<Input id="album-title" bind:value={title} />
		</div>
		<div class="space-y-1.5">
			<Label for="album-artist">Album Artist</Label>
			<Input id="album-artist" bind:value={albumArtist} />
		</div>
		<div class="space-y-1.5">
			<Label for="album-artists">Artists <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="album-artists" bind:value={artists} />
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div class="space-y-1.5">
				<Label for="album-release">Release Date</Label>
				<Input id="album-release" bind:value={releaseDate} />
			</div>
			<div class="space-y-1.5">
				<Label for="album-format">Format</Label>
				<Input id="album-format" placeholder="LP, EP, Single…" bind:value={format} />
			</div>
		</div>
		<div class="space-y-1.5">
			<Label for="album-genres">Genres <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="album-genres" bind:value={genres} />
		</div>
	</Tabs.Content>

	<Tabs.Content value="details" class="space-y-3 mt-4">
		<div class="grid grid-cols-2 gap-3">
			<div class="space-y-1.5">
				<Label for="album-label">Label</Label>
				<Input id="album-label" bind:value={label} />
			</div>
			<div class="space-y-1.5">
				<Label for="album-rating">Rating <span class="text-muted-foreground text-xs">(0–10)</span></Label>
				<Input id="album-rating" type="number" min="0" max="10" step="0.1" bind:value={rating} />
			</div>
		</div>
		<div class="space-y-1.5">
			<Label for="album-tags">Tags <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="album-tags" bind:value={tags} />
		</div>
		<div class="space-y-1.5">
			<Label for="album-credits">Credits</Label>
			<Textarea id="album-credits" bind:value={credits} rows={4} />
		</div>
	</Tabs.Content>

	<Tabs.Content value="artwork" class="mt-4">
		<ArtworkEditor
			entityType="album"
			entityUid={uid}
			onchange={(path) => { artworkPath = path; }}
		/>
	</Tabs.Content>
</Tabs.Root>

<Separator class="my-4" />

<div class="flex justify-end gap-2">
	<Button variant="outline" onclick={() => enrichAlbum(uid)}>Enrich</Button>
	<Button variant="outline" onclick={closeEditModal}>Cancel</Button>
	<Button onclick={save} disabled={saving}>{saving ? "Saving…" : "Save"}</Button>
</div>
