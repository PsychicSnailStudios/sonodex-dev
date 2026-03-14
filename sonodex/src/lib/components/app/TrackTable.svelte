<script lang="ts">
	import type { Track } from '$lib/types';
	import { setSelection } from '$lib/session.svelte';

	import { Button } from "$lib/components/ui/button/index.js";
	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";

	import { playTrack } from "$lib/player.svelte";

	let { tracks, type } = $props<{ tracks: Track[]; type: string }>();

	function parseArtists(artists: string | null): string {
		if (!artists) return "Unknown Artist";
		try {
			const parsed = JSON.parse(artists);
			return Array.isArray(parsed) ? parsed.join(", ") : "Unknown Artist";
		} catch {
			return "Unknown Artist";
		}
	}

	function parseAlbum(albums: string | null): string {
		if (!albums) return "—";
		try {
			const parsed = JSON.parse(albums);
			return Array.isArray(parsed) && parsed.length > 0 ? parsed[0].name : "—";
		} catch {
			return "—";
		}
	}

	function formatRating(rating: number | null): string {
		if (rating === null) return "—";
		return rating.toFixed(1);
	}

	function formatDuration(ms: number | null): string {
		if (ms === null) return "—";
		const totalSeconds = Math.floor(ms / 1000);
		const minutes = Math.floor(totalSeconds / 60);
		const seconds = totalSeconds % 60;
		return `${minutes}:${seconds.toString().padStart(2, "0")}`;
	}
</script>

<div class="flex flex-col h-full overflow-hidden">
	{#if type === "library"}

	<div class="grid text-xs font-medium text-muted-foreground px-3 py-2 border-b" style="grid-template-columns: 40px 1fr 1fr 60px 80px 60px 40px;">
		<span></span>
		<span>Title</span>
		<span>Album</span>
		<span>Year</span>
		<span>Rating</span>
		<span>Duration</span>
		<span></span>
	</div>
	<div class="flex-1 overflow-y-auto">
		{#each tracks as track (track.id)}
			<div class="grid items-center px-3 border-b hover:bg-muted/50" style="grid-template-columns: 40px 1fr 1fr 60px 80px 60px 40px; height: 56px;"
				  ondblclick={() => playTrack(track)}
				  role="row">
				<ArtworkDisplay id={track.id} size={30} />
				<div class="flex flex-col min-w-0">
					<button onclick={() => setSelection(track.id, "track")} class="text-sm truncate">{track.title ?? "Unknown Title"}</button>
					<button onclick={() => setSelection(track.id, "artist")} class="text-xs text-muted-foreground truncate">{parseArtists(track.artists)}</button>
				</div>
				<button onclick={() => setSelection(track.id, "album")} class="text-sm truncate pr-4">{parseAlbum(track.albums)}</button>
				<span class="text-sm">{track.year ?? "—"}</span>
				<span class="text-sm font-mono">{formatRating(track.rating)}</span>
				<span class="text-sm font-mono">{formatDuration(track.duration_ms)}</span>
				<Button variant="ghost" size="icon">⋯</Button>
			</div>
		{/each}
	</div>

	{:else if type === "album"}
	
	<div class="grid text-xs font-medium text-muted-foreground px-3 py-2 border-b" style="grid-template-columns: 40px 1fr 1fr 60px 80px 60px 40px;">
		<span></span>
		<span>Title</span>
		<span>Rating</span>
		<span>Duration</span>
		<span></span>
	</div>
	<div class="flex-1 overflow-y-auto">
		{#each tracks as track (track.id)}
			<div class="grid items-center px-3 border-b hover:bg-muted/50" style="grid-template-columns: 40px 1fr 1fr 60px 80px 60px 40px; height: 56px;">
				<ArtworkDisplay id={track.id} size={30} />
				<div class="flex flex-col min-w-0">
					<span class="text-sm truncate">{track.title ?? "Unknown Title"}</span>
					<span class="text-xs text-muted-foreground truncate">{parseArtists(track.artists)}</span>
				</div>
				<span class="text-sm font-mono">{formatRating(track.rating)}</span>
				<span class="text-sm font-mono">{formatDuration(track.duration_ms)}</span>
				<Button variant="ghost" size="icon">⋯</Button>
			</div>
		{/each}
	</div>
	
	{/if}
</div>