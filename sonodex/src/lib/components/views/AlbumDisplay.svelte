<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	import { selection } from "$lib/session.svelte";
	import { library } from "$lib/library.svelte";
	import type { Album, Track } from "$lib/types";

	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app/TrackTable.svelte";

	let album: Album | null = $state(null);

	let tracks: Track[] = $derived.by(() => {
		if (!album) return [];
		const albumTrackIds: number[] = JSON.parse(album.tracks ?? "[]").map((t: { id: number }) => t.id);
		return library.tracks.filter(t => albumTrackIds.includes(t.id));
	});

	function formatDuration(ms: number): string {
		const totalSeconds = Math.floor(ms / 1000);
		const minutes = Math.floor(totalSeconds / 60);
		const seconds = totalSeconds % 60;
		return `${minutes}:${seconds.toString().padStart(2, "0")}`;
	}

	function totalDuration(tracks: Track[]): string {
		const total = tracks.reduce((sum, t) => sum + (t.duration_ms ?? 0), 0);
		return formatDuration(total);
	}

	$effect(() => {
		const id = selection.id;
		album = null;
		invoke("get_album", { id }).then((a) => {
			album = a as Album;
		});
	});
</script>

<div class="flex flex-col gap-4 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	{#if album}
		<div class="flex gap-4 items-end">
			<ArtworkDisplay id={album.id} size={160} type="album" />

			<div class="flex flex-col gap-1">
				{#if album.format}
					<span class="text-xs text-muted-foreground">{album.format}</span>
				{/if}
				<h2 class="text-2xl font-bold">{album.title}</h2>
				<div class="flex gap-3 text-sm text-muted-foreground">
					{#if album.album_artist}
						<span>{album.album_artist}</span>
					{/if}
					{#if album.release_date}
						<span>{album.release_date}</span>
					{/if}
					<span>{tracks.length} songs</span>
					{#if tracks.length > 0}
						<span>{totalDuration(tracks)}</span>
					{/if}
				</div>
			</div>
		</div>

		<TrackTable type="album" tracks={tracks} />
	{:else}
		<span class="text-muted-foreground text-sm">Loading...</span>
	{/if}

</div>