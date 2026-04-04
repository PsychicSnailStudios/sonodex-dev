<script lang="ts">
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { library, getArtistUidFromName } from "$lib/ts/library.svelte";
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { parseArtists } from "$lib/ts/util/helpers";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import type { Track } from "$lib/ts/util/types";

	type Scrobble = {
		uid: string;
		timestamp: number;
		track_uid: string;
		artist_uid: string;
		duration_played: number;
		did_seek: boolean;
		did_pause: boolean;
	};

	type TopTrack = {
		track: Track;
		plays: number;
	};

	const MIN_DURATION_MS = 30_000;

	let topTracks = $state<TopTrack[]>([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			const scrobbles = await invoke<Scrobble[]>("get_scrobbles");

			const countMap = new Map<string, number>();
			for (const s of scrobbles) {
				if (s.duration_played >= MIN_DURATION_MS) {
					countMap.set(s.track_uid, (countMap.get(s.track_uid) ?? 0) + 1);
				}
			}

			const sorted = [...countMap.entries()]
				.sort((a, b) => b[1] - a[1])
				.slice(0, 5);

			topTracks = sorted.flatMap(([uid, plays]) => {
				const track = library.tracks.find((t) => t.uid === uid);
				if (!track) return [];
				return [{ track, plays }];
			});
		} catch (e) {
			console.error("TopTracks failed to load", e);
		} finally {
			loading = false;
		}
	});
</script>

<div class="rounded-lg border bg-card text-card-foreground shadow-sm p-4 flex flex-col gap-3">
	<div class="flex items-center justify-between">
		<span class="text-sm font-medium">Top Tracks</span>
		<span class="text-xs text-muted-foreground">All time</span>
	</div>

	{#if loading}
		<div class="flex flex-col gap-2">
			{#each Array(5) as _}
				<div class="flex gap-2 p-2 items-center animate-pulse">
					<div class="rounded shrink-0 bg-muted" style="width:30px;height:30px;"></div>
					<div class="flex flex-col gap-1 flex-1 min-w-0">
						<div class="h-3 w-3/4 rounded bg-muted"></div>
						<div class="h-2 w-1/2 rounded bg-muted"></div>
					</div>
				</div>
			{/each}
		</div>
	{:else if topTracks.length === 0}
		<p class="text-xs text-muted-foreground text-center py-4">No plays recorded yet.</p>
	{:else}
		<div class="flex flex-col">
			{#each topTracks as { track, plays }, i}
				<div class="flex items-center gap-2 p-2 rounded-md hover:bg-muted/50 transition-colors">
					<span class="text-xs text-muted-foreground w-4 shrink-0 text-right">{i + 1}</span>
					<ArtworkDisplay uid={track.uid} size={30} type="track" />
					<div class="min-w-0 grid flex-1">
						<span
							role="button"
							tabindex="0"
							onclick={() => setSelection(track.uid, "track")}
							onkeydown={(e) => { if (e.key === "Enter") setSelection(track.uid, "track"); }}
							class="text-sm truncate cursor-pointer hover:underline"
						>
							{track.title}
						</span>
						<span
							role="button"
							tabindex="0"
							onclick={() => setSelection(getArtistUidFromName(track.album_artist!), "artist")}
							onkeydown={(e) => { if (e.key === "Enter") setSelection(getArtistUidFromName(track.album_artist!), "artist"); }}
							class="text-xs text-muted-foreground truncate cursor-pointer hover:underline"
						>
							{parseArtists(track.artists)}
						</span>
					</div>
					<span class="text-xs text-muted-foreground shrink-0 tabular-nums">
						{plays} {plays === 1 ? "play" : "plays"}
					</span>
				</div>
			{/each}
		</div>
	{/if}
</div>