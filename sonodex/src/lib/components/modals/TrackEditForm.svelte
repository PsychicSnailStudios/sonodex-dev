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
	let artists = $state("");
	let albumArtist = $state("");
	let album = $state("");
	let trackNumber = $state<string>("");
	let year = $state("");
	let genres = $state("");
	let bpm = $state<string>("");
	let key = $state("");
	let rating = $state<string>("");
	let label = $state("");
	let credits = $state("");
	let tags = $state("");
	let artworkPath = $state<string | null>(null);
	let saving = $state(false);
	let writeToFile = $state(false);

	onMount(async () => {
		const tracks = await invoke<any[]>("get_tracks");
		const track = tracks.find((t) => t.id === id);
		if (!track) return;

		title = track.title ?? "";
		albumArtist = track.album_artist ?? "";
		year = track.year ?? "";
		bpm = track.bpm != null ? String(track.bpm) : "";
		key = track.key ?? "";
		rating = track.rating != null ? String(track.rating) : "";
		label = track.label ?? "";
		credits = track.credits ?? "";
		artworkPath = track.artwork_path ?? null;

		try {
			const artistArr = track.artists ? JSON.parse(track.artists) : [];
			artists = artistArr.join(", ");
		} catch { artists = ""; }

		try {
			const albumArr = track.albums ? JSON.parse(track.albums) : [];
			if (albumArr.length > 0) {
				album = albumArr[0].name ?? "";
				trackNumber = albumArr[0].track_number != null ? String(albumArr[0].track_number) : "";
			}
		} catch { album = ""; }

		try {
			const genreArr = track.genres ? JSON.parse(track.genres) : [];
			genres = genreArr.join(", ");
		} catch { genres = ""; }

		try {
			const tagArr = track.tags ? JSON.parse(track.tags) : [];
			tags = tagArr.join(", ");
		} catch { tags = ""; }
	});

	async function save() {
		saving = true;
		try {
			const artistArr = artists.split(",").map((s) => s.trim()).filter(Boolean);
			const genreArr = genres.split(",").map((s) => s.trim()).filter(Boolean);
			const tagArr = tags.split(",").map((s) => s.trim()).filter(Boolean);
			const albumArr = album ? [{ name: album, track_number: trackNumber ? Number(trackNumber) : null }] : [];

			const update: Record<string, any> = {
				title: title || null,
				artists: artistArr.length ? JSON.stringify(artistArr) : null,
				album_artist: albumArtist || null,
				albums: albumArr.length ? JSON.stringify(albumArr) : null,
				year: year || null,
				genres: genreArr.length ? JSON.stringify(genreArr) : null,
				bpm: bpm ? Number(bpm) : null,
				key: key || null,
				rating: rating ? Number(rating) : null,
				label: label || null,
				credits: credits || null,
				tags: tagArr.length ? JSON.stringify(tagArr) : null,
				artwork_path: artworkPath,
			};

			if (writeToFile) {
				const tracks = await invoke<any[]>("get_tracks");
				const track = tracks.find((t) => t.id === id);
				if (track) await invoke("write_track_tags", { id, path: track.path, update });
			} else {
				await invoke("update_track_metadata", { id, update });
			}

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
			<Label for="track-title">Title</Label>
			<Input id="track-title" bind:value={title} />
		</div>
		<div class="space-y-1.5">
			<Label for="track-artists">Artists <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="track-artists" bind:value={artists} />
		</div>
		<div class="space-y-1.5">
			<Label for="track-album-artist">Album Artist</Label>
			<Input id="track-album-artist" bind:value={albumArtist} />
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div class="space-y-1.5">
				<Label for="track-album">Album</Label>
				<Input id="track-album" bind:value={album} />
			</div>
			<div class="space-y-1.5">
				<Label for="track-number">Track #</Label>
				<Input id="track-number" type="number" bind:value={trackNumber} />
			</div>
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div class="space-y-1.5">
				<Label for="track-year">Year</Label>
				<Input id="track-year" bind:value={year} />
			</div>
			<div class="space-y-1.5">
				<Label for="track-genres">Genres <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
				<Input id="track-genres" bind:value={genres} />
			</div>
		</div>
	</Tabs.Content>

	<Tabs.Content value="details" class="space-y-3 mt-4">
		<div class="grid grid-cols-3 gap-3">
			<div class="space-y-1.5">
				<Label for="track-bpm">BPM</Label>
				<Input id="track-bpm" type="number" bind:value={bpm} />
			</div>
			<div class="space-y-1.5">
				<Label for="track-key">Key</Label>
				<Input id="track-key" placeholder="e.g. Am" bind:value={key} />
			</div>
			<div class="space-y-1.5">
				<Label for="track-rating">Rating <span class="text-muted-foreground text-xs">(0–10)</span></Label>
				<Input id="track-rating" type="number" min="0" max="10" step="0.1" bind:value={rating} />
			</div>
		</div>
		<div class="space-y-1.5">
			<Label for="track-label">Label</Label>
			<Input id="track-label" bind:value={label} />
		</div>
		<div class="space-y-1.5">
			<Label for="track-tags">Tags <span class="text-muted-foreground text-xs">(comma-separated)</span></Label>
			<Input id="track-tags" bind:value={tags} />
		</div>
		<div class="space-y-1.5">
			<Label for="track-credits">Credits</Label>
			<Textarea id="track-credits" bind:value={credits} rows={3} />
		</div>
	</Tabs.Content>

	<Tabs.Content value="artwork" class="mt-4">
		<ArtworkEditor
			entityType="track"
			entityId={id}
			onchange={(path) => { artworkPath = path; }}
		/>
	</Tabs.Content>
</Tabs.Root>

<Separator class="my-4" />

<div class="flex items-center justify-between">
	<label class="flex items-center gap-2 text-sm cursor-pointer select-none">
		<input type="checkbox" bind:checked={writeToFile} class="rounded" />
		Write tags to file
	</label>
	<div class="flex gap-2">
		<Button variant="outline" onclick={closeEditModal}>Cancel</Button>
		<Button onclick={save} disabled={saving}>
			{saving ? "Saving…" : "Save"}
		</Button>
	</div>
</div>