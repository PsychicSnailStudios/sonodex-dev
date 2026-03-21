<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";

	import { library } from "$lib/library.svelte";
	import type { Track, Lyrics } from "$lib/types";
	import { selection } from "$lib/session.svelte";
	import { openEditModal } from "$lib/editModal.svelte";

	import { formatDuration, formatRating, parseAlbum, parseArtists } from '$lib/helpers';

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";

	let track = $derived(library.tracks.find(t => t.uid === selection.uid) ?? null);
	let lyrics: Lyrics | null = $state(null);

	onMount(async () => {
		lyrics = await invoke("get_track_lyrics", { uid: selection.uid });
	});
</script>

<div class="flex flex-col gap-4 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	{#if track}
		<div class="flex gap-4 items-center">
			<ArtworkDisplay uid={track.uid} size={160} type="track" />

			<div class="flex flex-col gap-1">
				<h2 class="text-2xl font-bold">{track.title ?? "Unknown Title"}</h2>
				<div class="flex gap-2 text-sm text-muted-foreground">
					<span>{parseArtists(track.artists)}</span>
					<span>|</span>
					<span>{parseAlbum(track.albums)}</span>
					<span>|</span>
					<span>{track.year ?? "—"}</span>
					<span>|</span>
					<span>{formatDuration(track.duration_ms)}</span>
				</div>
				<div class="text-sm">{formatRating(track.rating)}</div>
				<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "track", uid: track!.uid })}>...</Button>
			</div>
		</div>

		<Tabs.Root value="lyrics" class="flex flex-col min-h-0 flex-1">
			<Tabs.List>
				<Tabs.Trigger value="lyrics">Lyrics</Tabs.Trigger>
				<Tabs.Trigger value="credits">Credits</Tabs.Trigger>
				<Tabs.Trigger value="explore">Explore</Tabs.Trigger>
			</Tabs.List>

			<Tabs.Content value="lyrics" class="flex-1 overflow-y-auto mt-2">
				{#if lyrics?.instrumental}
					<p class="text-muted-foreground text-sm">This track is instrumental.</p>
				{:else if lyrics?.plain}
					<pre class="text-sm whitespace-pre-wrap font-sans leading-relaxed">{lyrics.plain}</pre>
				{:else}
					<p class="text-muted-foreground text-sm">No lyrics available.</p>
				{/if}
			</Tabs.Content>

			<Tabs.Content value="credits" class="flex-1 overflow-y-auto mt-2">
				{#if track.credits}
					<pre class="text-sm whitespace-pre-wrap font-sans">{track.credits}</pre>
				{:else}
					<p class="text-muted-foreground text-sm">No credits available.</p>
				{/if}
			</Tabs.Content>

			<Tabs.Content value="explore" class="flex-1 overflow-y-auto mt-2">
				<p class="text-muted-foreground text-sm">Nothing here yet.</p>
			</Tabs.Content>
		</Tabs.Root>
	{:else}
		<span class="text-muted-foreground text-sm">Loading...</span>
	{/if}

</div>
