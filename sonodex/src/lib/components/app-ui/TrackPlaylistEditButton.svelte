<script lang="ts">
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import Button from "../ui/button/button.svelte";
	import Input from "../ui/input/input.svelte";
	import ScrollArea from "../ui/scroll-area/scroll-area.svelte";

	import { BadgePlus, ChevronRight, House, Folder } from "lucide-svelte";
	import { CirclePlus, CircleCheck } from "lucide-svelte";

	import { library } from "$lib/ts/library.svelte";
	import type { Playlist, Track } from "$lib/ts/util/types";
	import { addTrackToPlaylist, removeTrackFromPlaylist } from "$lib/ts/audio/playlistManager.svelte";

	let { track, isButton = true } = $props<{ track: Track; isButton?: boolean }>();

	let search = $state("");
	let browsePath = $state<string | null>(null);

	function folderOf(p: Playlist): string | null {
		const f = (p as any).folder;
		return f && f !== "" ? f : null;
	}

	function immediateChild(parentPath: string | null, folderPath: string): string | null {
		if (parentPath === null) return folderPath.split("/")[0];
		if (!folderPath.startsWith(parentPath + "/")) return null;
		return folderPath.slice(parentPath.length + 1).split("/")[0];
	}

	function getChildFolders(parentPath: string | null): string[] {
		const seen = new Set<string>();
		for (const p of library.playlists) {
			const pf = folderOf(p);
			if (!pf) continue;
			const child = immediateChild(parentPath, pf);
			if (child) seen.add(parentPath === null ? child : `${parentPath}/${child}`);
		}
		return [...seen].sort();
	}

	function getDirectPlaylists(folderPath: string | null): Playlist[] {
		return library.playlists.filter((p) => folderOf(p) === folderPath);
	}

	function folderLabel(fullPath: string): string {
		return fullPath.split("/").at(-1) ?? fullPath;
	}

	function breadcrumbs(): { label: string; path: string | null }[] {
		const crumbs: { label: string; path: string | null }[] = [{ label: "All", path: null }];
		if (!browsePath) return crumbs;
		let acc = "";
		for (const part of browsePath.split("/")) {
			acc = acc ? `${acc}/${part}` : part;
			crumbs.push({ label: part, path: acc });
		}
		return crumbs;
	}

	const searchActive = $derived(search.trim() !== "");

	const filteredPlaylists = $derived(
		searchActive
			? library.playlists.filter((p) => {
				const q = search.toLowerCase();
				return p.title.toLowerCase().includes(q) || ((p as any).description?.toLowerCase() ?? "").includes(q);
			})
			: getDirectPlaylists(browsePath)
	);

	const visibleFolders = $derived(searchActive ? [] : getChildFolders(browsePath));

	function hasTrack(playlistUid: string): boolean {
		const playlist = library.playlists.find((p) => p.uid === playlistUid);
		if (!playlist) return false;
		const tracks: { uid: string }[] = JSON.parse((playlist as any).tracks ?? "[]");
		return tracks.some((t) => t.uid === track.uid);
	}

	function updateTrack(playlist: Playlist) {
		if (hasTrack(playlist.uid)) {
			removeTrackFromPlaylist(playlist, track);
		} else {
			addTrackToPlaylist(playlist, track);
		}
	}

	function onOpen() {
		browsePath = null;
		search = "";
	}
</script>

<DropdownMenu.Root onOpenChange={(open) => { if (open) onOpen(); }}>
	{#if isButton}
		<DropdownMenu.Trigger>
			{#snippet child({ props }: { props: Record<string, unknown> })}
				<Button {...props} variant="ghost" size="icon"><BadgePlus /></Button>
			{/snippet}
		</DropdownMenu.Trigger>
	{:else}
		<DropdownMenu.Trigger>
			<span class="text-sm">Add to playlist</span>
		</DropdownMenu.Trigger>
	{/if}

	<DropdownMenu.Content class="w-64">
		<DropdownMenu.Group>
			<DropdownMenu.Label>Add to playlist</DropdownMenu.Label>
			<div class="px-2 pb-1">
				<Input placeholder="Search" bind:value={search} />
			</div>

			{#if !searchActive && breadcrumbs().length > 1}
				<div class="flex items-center gap-1 px-2 py-1 text-xs text-muted-foreground flex-wrap">
					{#each breadcrumbs() as crumb, i}
						{#if i > 0}<ChevronRight class="size-3 shrink-0" />{/if}
						<button
							class="hover:text-foreground transition-colors"
							class:text-foreground={i === breadcrumbs().length - 1}
							onclick={() => (browsePath = crumb.path)}
						>
							{#if i === 0}<House class="size-3 inline mr-0.5" />{/if}
							{crumb.label}
						</button>
					{/each}
				</div>
			{/if}

			<DropdownMenu.Separator />

			<ScrollArea class="h-[250px]">
				<div class="flex flex-col">
					{#each visibleFolders as fp}
						<button
							class="flex items-center gap-2 px-2 py-1.5 text-sm hover:bg-muted/50 rounded-sm text-left w-full"
							onclick={() => (browsePath = fp)}
						>
							<Folder class="size-4 shrink-0 text-muted-foreground" />
							<span class="truncate flex-1">{folderLabel(fp)}</span>
							<ChevronRight class="size-3 shrink-0 text-muted-foreground" />
						</button>
					{/each}

					{#each filteredPlaylists as playlist}
						<div class="flex gap-2 px-2 py-1 items-center justify-between">
							<span class="text-sm truncate flex-1">{playlist.title}</span>
							<Button variant="ghost" size="icon" onclick={() => updateTrack(playlist)}>
								{#if hasTrack(playlist.uid)}
									<CircleCheck />
								{:else}
									<CirclePlus />
								{/if}
							</Button>
						</div>
					{/each}
				</div>
			</ScrollArea>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>