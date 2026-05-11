<script lang="ts">
	import { Pencil } from "lucide-svelte";

	import { ScrollArea } from "$shadcn/scroll-area/index.js";
	import Button from "$shadcn/button/button.svelte";

	import TrackTableSettings from "$lib/components/custom/track-table/TrackTableSettings.svelte";
	import ArtworkDisplay from "$lib/components/custom/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/custom/track-table/TrackTable.svelte";
	import AudioCard from "$lib/components/custom/cards/AudioCard.svelte";
	import NavButtons from "$lib/components/custom/NavButtons.svelte";
	import TagList from "$lib/components/custom/tags/TagList.svelte";
	import DownloadButton from "$lib/components/custom/DownloadButton.svelte";

	import { selection, setSelection } from "$ts/store/session.svelte";
	import { getAlbum, getTrackArrayFromUID, library } from "$ts/store/library.svelte";
	import { queueTracksByObject } from "$ts/audio/audioManager.svelte";
	import { openEditModal } from "$ts/ui/editModal.svelte";
	import { getArtworkColor, totalDuration } from "$ts/util/helpers";
	import { createPersistedViewState } from "$ts/store/session.svelte";
	import { artworkCache } from "$ts/library/artworkLoader";

	import type { Album, Track } from "$ts/util/types";
    import { parseTags } from "$ts/util/parsers";

	const view = createPersistedViewState("album", {
		sortField: "number",
		sortDir: "asc",
		colPreset: "album",
	});

	let album = $derived(getAlbum(selection.uid) ?? null);
	let genres = $derived(album?.genres ? parseTags(album.genres) : null);
	let tags = $derived(album?.tags ? parseTags(album.tags) : null);

	let color = $state("rgb(30, 30, 30)");

	let tracks: Track[] = $derived.by(() => {
		if (!album) return [];
		return getTrackArrayFromUID(album.uid, view.sort);
	});

	let artistAlbums: Album[] = $derived.by(() => {
		if (!album?.album_artist) return [];
		return library.albums.filter(a =>
			a.album_artist?.name.toLowerCase() === album?.album_artist?.name.toLowerCase()
		);
	});

	$effect(() => {
		const uid = selection.uid;
		if (!uid) return;
		color = "var(--muted)";
		const cached = artworkCache.get(`album:${uid}`);
		if (cached) {
			fetch(cached).then(r => r.arrayBuffer()).then(buf => {
				getArtworkColor(Array.from(new Uint8Array(buf)), 0.3).then(c => color = c);
			}).catch(() => {});
		}
	});

	function trackIsGhost(t: Track): boolean {
		const hasLocal = t.path && t.path !== "" && t.path !== t.uid;
		const hasRemote = (t as any).remote_path && (t as any).remote_path.length > 0;
		return !hasLocal && !hasRemote;
	}

	let allGhosts = $derived(
		tracks.length > 0 && tracks.every(t => trackIsGhost(t))
	);
</script>

<div class="flex flex-col gap-4 p-4 border-2 h-full w-full overflow-hidden rounded-md" style="background: linear-gradient(180deg, {color} 0%, transparent 80%)">

	<ScrollArea class="min-h-0 min-w-0 h-full">
	<div class="flex flex-col gap-4 pb-4 pr-4">
		<NavButtons />
			
		{#if album}
		<div class="flex gap-4 items-end">
			<ArtworkDisplay uid={album.uid} size={160} type="album" />

			<div class="flex flex-col gap-1">
				<span class="text-xs text-muted-foreground">{album.format ? album.format : "Album"}</span>
				<h2 class="text-2xl font-bold">{album.title}</h2>
				<div class="flex gap-3 text-sm text-muted-foreground flex-wrap">
					{#if album.album_artist}
						<span role="button" tabindex="0" onclick={() => setSelection(album.album_artist!.uid.toString())} onkeydown={(e) => { if (e.key === 'Enter') setSelection(album.album_artist!.uid.toString()); }} class="text-sm truncate cursor-pointer hover:underline">
							{album.album_artist.name}
						</span>
					{/if}
					{#if album.release_date}
						<span>|</span>
						<span>{album.release_date}</span>
					{/if}
					<span>|</span>
					<span>{tracks.length} {tracks.length === 1 ? "song" : "songs"}</span>
					{#if tracks.length > 0}
						<span>|</span>
						<span>{totalDuration(tracks)}</span>
					{/if}
				</div>

				<TagList uid={album.uid} tags={genres!} canEdit={false} />
				<TagList uid={album.uid} tags={tags!} canEdit={true} />
			</div>
		</div>

		<div class="flex gap-2 justify-between items-center flex-wrap p-2 rounded-md"
			  style="background: {color};">
			<div>
				<Button variant="default" disabled={allGhosts} onclick={() => queueTracksByObject(tracks, true)}>{ tracks.length === 1 ? "Play" : "Play All"}</Button>
				{#if tracks.length > 1}
					<Button variant="outline" disabled={allGhosts} onclick={() => queueTracksByObject(tracks, true, true)}>Shuffle</Button>
				{/if}
			</div>
			<div class="flex gap-1 items-center">
				<DownloadButton uid={album.uid} variant="ghost" />
				<Button variant="ghost" onclick={() => openEditModal({ type: "album", uid: album!.uid })}><Pencil /></Button>
				<TrackTableSettings cols={view.cols} sort={view.sort} compact={view.compact} onCompactChange={(v) => view.compact = v} />
			</div>
		</div>

		<TrackTable tracks={tracks} columns={view.cols} sort={view.sort} compact={view.compact} albumUid={album.uid} emulateType={album.emulate_type} />

		{#if album.label}
			<span class="text-muted-foreground text-sm">{album.label}</span>
		{/if}

		<div class="flex flex-col gap-2 w-full pt-4">
			<h4>More by {album.album_artist?.name}</h4>
			<ScrollArea orientation="horizontal" class="min-h-0 min-w-0">
				<div class="grid gap-2 pb-4" style="grid-auto-columns: 150px; grid-auto-flow: column;">
					{#each artistAlbums as album}
						<AudioCard title={album.title} subTitle={album.album_artist?.name.toString() ?? ""} artworkUid={album.uid} type="album" />
					{/each}
				</div>
			</ScrollArea>
		</div>
		{:else}
			<span class="text-muted-foreground text-sm">Loading...</span>
		{/if}

	</div>
	</ScrollArea>

</div>