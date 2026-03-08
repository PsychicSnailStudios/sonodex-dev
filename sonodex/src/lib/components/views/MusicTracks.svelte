<script lang="ts">
	import VirtualList from "svelte-virtual-list";
	import { Button } from "$lib/components/ui/button/index.js";
  import TrackArtwork from "$lib/components/TrackArtwork.svelte";

	type Track = {
		id: number;
		path: string;
		last_modified: number;
		title: string | null;
		artists: string | null;
		album_artist: string | null;
		albums: string | null;
		genres: string | null;
		year: string | null;
		rating: number | null;
		tags: string | null;
		duration_ms: number | null;
		bpm: number | null;
		key: string | null;
	};

	let { tracks }: { tracks: Track[] } = $props();

	function parseArtists(artists: string | null): string {
		if (!artists) return "Unknown Artist";
		try {
			const parsed: string[] = JSON.parse(artists);
			return parsed.join(", ");
		} catch {
			return artists;
		}
	}

	function parseAlbum(albums: string | null): string {
		if (!albums) return "—";
		try {
			const parsed: { name: string; track_number: number | null }[] = JSON.parse(albums);
			return parsed[0]?.name ?? "—";
		} catch {
			return "—";
		}
	}

	function formatDuration(ms: number | null): string {
		if (!ms) return "--:--";
		const totalSeconds = Math.floor(ms / 1000);
		const minutes = Math.floor(totalSeconds / 60);
		const seconds = totalSeconds % 60;
		return `${minutes}:${seconds.toString().padStart(2, "0")}`;
	}

	function formatRating(rating: number | null): string {
		if (rating === null) return "—";
		const stars = Math.round(rating / 2);
		return "★".repeat(stars) + "☆".repeat(5 - stars);
	}
</script>

<div class="flex flex-col h-full">
	<div class="grid text-xs font-medium text-muted-foreground px-3 py-2 border-b" style="grid-template-columns: 40px 1fr 1fr 60px 120px 60px 40px;">
		<span></span>
		<span>Title</span>
		<span>Album</span>
		<span>Year</span>
		<span>Rating</span>
		<span>Duration</span>
		<span></span>
	</div>

	<div class="flex-1 overflow-hidden">
		<VirtualList items={tracks} itemHeight={56} let:item={track}>
			<div class="grid items-center px-3 border-b hover:bg-muted/50" style="grid-template-columns: 40px 1fr 1fr 60px 120px 60px 40px; height: 56px;">
        <TrackArtwork id={track.id} />
        <div class="flex flex-col min-w-0">
          <span class="text-sm truncate">{track.title ?? "Unknown Title"}</span>
          <span class="text-xs text-muted-foreground truncate">{parseArtists(track.artists)}</span>
        </div>
        <span class="text-sm truncate pr-4">{parseAlbum(track.albums)}</span>
        <span class="text-sm">{track.year ?? "—"}</span>
        <span class="text-sm font-mono">{formatRating(track.rating)}</span>
        <span class="text-sm font-mono">{formatDuration(track.duration_ms)}</span>
        <Button variant="ghost" size="icon">⋯</Button>
      </div>
		</VirtualList>
	</div>
</div>