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
	import { loadLibrary } from "$lib/ts/library.svelte";
	import { X, Plus, ChevronUp, ChevronDown } from "lucide-svelte";

	let { uid } = $props<{ uid: string }>();

	type AlbumEntry = {
		uid: string;
		name: string;
		track_number: number | null;
	};

	let title = $state("");
	let artists = $state("");
	let albumArtist = $state("");
	let albums = $state<AlbumEntry[]>([]);
	let originalAlbumUids = $state<string[]>([]);
	let year = $state("");
	let genres = $state("");
	let bpm = $state<string>("");
	let key = $state("");
	let rating = $state<string>("");
	let label = $state("");
	let credits = $state("");
	let tags = $state("");
	let path = $state("");
	let artworkPath = $state<string | null>(null);
	let saving = $state(false);
	let writeToFile = $state(false);

	onMount(async () => {
		const tracks = await invoke<any[]>("get_tracks");
		const track = tracks.find((t) => t.uid === uid);
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
		path = track.path ?? "";

		try {
			const artistArr = track.artists ? JSON.parse(track.artists) : [];
			artists = artistArr.join(", ");
		} catch { artists = ""; }

		try {
			const parsed = track.albums ? JSON.parse(track.albums) : [];
			albums = parsed;
			originalAlbumUids = parsed.map((a: AlbumEntry) => a.uid).filter(Boolean);
		} catch { albums = []; }

		try {
			const genreArr = track.genres ? JSON.parse(track.genres) : [];
			genres = genreArr.join(", ");
		} catch { genres = ""; }

		try {
			const tagArr = track.tags ? JSON.parse(track.tags) : [];
			tags = tagArr.join(", ");
		} catch { tags = ""; }
	});

	function addAlbum() {
		albums = [...albums, { uid: "", name: "", track_number: null }];
	}

	function removeAlbum(index: number) {
		albums = albums.filter((_, i) => i !== index);
	}

	function moveUp(index: number) {
		if (index === 0) return;
		const next = [...albums];
		[next[index - 1], next[index]] = [next[index], next[index - 1]];
		albums = next;
	}

	function moveDown(index: number) {
		if (index === albums.length - 1) return;
		const next = [...albums];
		[next[index], next[index + 1]] = [next[index + 1], next[index]];
		albums = next;
	}

	async function syncAlbums(cleanedAlbums: AlbumEntry[], trackTitle: string, trackAlbumArtist: string): Promise<AlbumEntry[]> {
		const allAlbums = await invoke<any[]>("get_albums");
		const finalAlbumEntries: AlbumEntry[] = [];

		for (const entry of cleanedAlbums) {
			const nameLower = entry.name.toLowerCase();
			const artistLower = trackAlbumArtist.toLowerCase();

			const match = allAlbums.find((a) => {
				const titleMatch = (a.title ?? "").toLowerCase() === nameLower;
				const artistMatch = (a.album_artist ?? "").toLowerCase() === artistLower;
				return titleMatch && artistMatch;
			});

			if (match) {
				let existingTracks: any[] = [];
				try {
					existingTracks = match.tracks ? JSON.parse(match.tracks) : [];
				} catch { existingTracks = []; }

				const alreadyIn = existingTracks.some((t: any) => t.uid === uid);
				if (alreadyIn) {
					const updated = existingTracks.map((t: any) =>
						t.uid === uid ? { ...t, track_number: entry.track_number ?? null } : t
					);
					await invoke("update_album_entry", {
						uid: match.uid,
						update: { tracks: JSON.stringify(updated) },
					});
				} else {
					existingTracks.push({ uid, name: trackTitle, track_number: entry.track_number ?? null });
					await invoke("update_album_entry", {
						uid: match.uid,
						update: { tracks: JSON.stringify(existingTracks) },
					});
				}

				finalAlbumEntries.push({ uid: match.uid, name: entry.name, track_number: entry.track_number });
			} else {
				const newAlbum = {
					uid: "",
					title: entry.name,
					album_artist: trackAlbumArtist || null,
					tracks: JSON.stringify([{ uid, name: trackTitle, track_number: entry.track_number ?? null }]),
					artists: null,
					format: null,
					rating: null,
					release_date: null,
					tags: JSON.stringify([]),
					genres: JSON.stringify([]),
					credits: null,
					label: null,
					artwork_blob: null,
					artwork_path: null,
				};

				await invoke("create_album_entry", { album: newAlbum });

				const refreshed = await invoke<any[]>("get_albums");
				const created = refreshed.find((a) => {
					const titleMatch = (a.title ?? "").toLowerCase() === entry.name.toLowerCase();
					const artistMatch = (a.album_artist ?? "").toLowerCase() === trackAlbumArtist.toLowerCase();
					return titleMatch && artistMatch;
				});

				finalAlbumEntries.push({
					uid: created?.uid ?? "",
					name: entry.name,
					track_number: entry.track_number,
				});
			}
		}

		const currentUids = new Set(finalAlbumEntries.map((e) => e.uid).filter(Boolean));
		const removedUids = originalAlbumUids.filter((u) => !currentUids.has(u));

		for (const removedUid of removedUids) {
			const albumRecord = allAlbums.find((a) => a.uid === removedUid);
			if (!albumRecord) continue;

			let existingTracks: any[] = [];
			try {
				existingTracks = albumRecord.tracks ? JSON.parse(albumRecord.tracks) : [];
			} catch { existingTracks = []; }

			const filtered = existingTracks.filter((t: any) => t.uid !== uid);
			await invoke("update_album_entry", {
				uid: removedUid,
				update: { tracks: JSON.stringify(filtered) },
			});
		}

		return finalAlbumEntries;
	}

	async function save() {
		saving = true;
		try {
			const artistArr = artists.split(",").map((s) => s.trim()).filter(Boolean);
			const genreArr = genres.split(",").map((s) => s.trim()).filter(Boolean);
			const tagArr = tags.split(",").map((s) => s.trim()).filter(Boolean);

			const cleanedAlbums = albums
				.filter((a) => a.name.trim())
				.map((a) => ({
					uid: a.uid ?? "",
					name: a.name.trim(),
					track_number: a.track_number != null && !isNaN(Number(a.track_number)) ? Number(a.track_number) : null,
				}));

			const finalAlbumEntries = await syncAlbums(cleanedAlbums, title, albumArtist);

			const update: Record<string, any> = {
				title: title || null,
				artists: artistArr.length ? JSON.stringify(artistArr) : null,
				album_artist: albumArtist || null,
				albums: finalAlbumEntries.length ? JSON.stringify(finalAlbumEntries) : null,
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
				await invoke("write_track_tags", { uid, path, update });
			} else {
				await invoke("update_track_metadata", { uid, update });
			}

			await invoke("enrich_track", { uid });

			await loadLibrary();
			closeEditModal();
		} finally {
			saving = false;
		}
	}

	async function enrichTrack() {
		await invoke("enrich_track", { uid });
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

		<div class="space-y-1.5">
			<div class="flex items-center justify-between">
				<Label>Albums</Label>
				<Button variant="ghost" size="sm" onclick={addAlbum} class="h-7 px-2 text-xs gap-1">
					<Plus class="size-3" />
					Add
				</Button>
			</div>
			<div class="space-y-2">
				{#each albums as album, i}
					<div class="flex gap-1 items-center">
						<div class="flex flex-col">
							<Button
								variant="ghost"
								size="icon"
								onclick={() => moveUp(i)}
								disabled={i === 0}
								class="size-6 text-muted-foreground"
							>
								<ChevronUp class="size-3" />
							</Button>
							<Button
								variant="ghost"
								size="icon"
								onclick={() => moveDown(i)}
								disabled={i === albums.length - 1}
								class="size-6 text-muted-foreground"
							>
								<ChevronDown class="size-3" />
							</Button>
						</div>
						<Input
							placeholder="Album name"
							bind:value={album.name}
							class="flex-1"
						/>
						<Input
							placeholder="Track #"
							type="number"
							bind:value={album.track_number}
							class="w-24"
						/>
						<Button
							variant="ghost"
							size="icon"
							onclick={() => removeAlbum(i)}
							class="size-8 shrink-0 text-muted-foreground hover:text-destructive"
						>
							<X class="size-4" />
						</Button>
					</div>
				{/each}
				{#if albums.length === 0}
					<p class="text-xs text-muted-foreground">No albums — click Add to link one.</p>
				{/if}
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
			entityUid={uid}
			onchange={(p) => { artworkPath = p; }}
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
		<Button variant="outline" onclick={enrichTrack}>Enrich</Button>
		<Button variant="outline" onclick={closeEditModal}>Cancel</Button>
		<Button onclick={save} disabled={saving}>
			{saving ? "Saving…" : "Save"}
		</Button>
	</div>
</div>