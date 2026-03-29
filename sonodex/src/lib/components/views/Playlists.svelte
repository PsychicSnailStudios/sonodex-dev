<script lang="ts">
	import { library } from "$lib/ts/library.svelte";
	import {
		dragState,
		setHoveredPlaylist,
		endDrag,
		onFolderDragOver,
		onFolderDragExit,
		onBreadcrumbDragOver,
		onBreadcrumbDragExit,
		dropOnPlaylist,
		movePlaylists,
        startDrag,
	} from "$lib/ts/app/dragState.svelte";
	import {
		folderSelection,
		navigateTo,
		breadcrumbs,
	} from "$lib/ts/app/folderSelection.svelte";
	import { addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
	import type { Playlist } from "$lib/ts/util/types";

	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import AudioCard from "$lib/components/app-ui/AudioCard.svelte";
	import PlaylistRow from "$lib/components/app-ui/PlaylistRow.svelte";
	import {
		FolderPlus, ListPlus, FileDown,
		LayoutGrid, List,	ChevronRight, ChevronDown,
		Folder, FolderOpen, House,
	} from "lucide-svelte";
   import CreateNewPlaylist from "$lib/components/dialogs/CreateNewPlaylist.svelte";
   import CreateNewFolder from "$lib/components/dialogs/CreateNewFolder.svelte";
   import FolderContext from "$lib/components/app-ui/context-menus/FolderContext.svelte";
   import PlaylistContext from "$lib/components/app-ui/context-menus/PlaylistContext.svelte";
   import PlaylistFolderCard from "$lib/components/app-ui/PlaylistFolderCard.svelte";

	type ViewMode = "tiled" | "compact";

	let search           = $state("");
	let viewMode         = $state<ViewMode>("tiled");
	let expandedFolders  = $state<Set<string>>(new Set());

	let createDialogOpen   = $state(false);
	let createDialogFolder = $state<string | null>(null);
	let folderDialogOpen   = $state(false);
	let folderDialogParent = $state<string | null>(null);
	let nameInput          = $state("");
	let folderNameInput    = $state("");

	let playlists   = $derived(library.playlists ?? []);
	let currentPath = $derived(folderSelection.currentPath);
	let crumbs      = $derived(breadcrumbs());

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

		for (const p of playlists) {
			const pf = folderOf(p);
			if (!pf) continue;
			const child = immediateChild(parentPath, pf);
			if (child) seen.add(parentPath === null ? child : `${parentPath}/${child}`);
		}

		for (const known of folderSelection.knownFolders) {
			const child = immediateChild(parentPath, known);
			if (child) seen.add(parentPath === null ? child : `${parentPath}/${child}`);
		}

		return [...seen].sort();
	}

	function getDirectPlaylists(folderPath: string | null): Playlist[] {
		return playlists.filter((p) => folderOf(p) === folderPath);
	}

	function getFolderArtworkUids(folderPath: string): string[] {
		return playlists
			.filter((p) => {
				const pf = folderOf(p);
				return pf === folderPath || (pf?.startsWith(folderPath + "/") ?? false);
			})
			.map((p) => p.uid)
			.slice(0, 4);
	}

	function folderLabel(fullPath: string): string {
		return fullPath.split("/").at(-1) ?? fullPath;
	}

	function compactFolderRows(
		parentPath: string | null,
		depth: number = 0
	): { path: string; depth: number }[] {
		const result: { path: string; depth: number }[] = [];
		for (const fp of getChildFolders(parentPath)) {
			result.push({ path: fp, depth });
			if (expandedFolders.has(fp)) {
				result.push(...compactFolderRows(fp, depth + 1));
			}
		}
		return result;
	}

	const filteredDirect = $derived(
		search.trim() === ""
			? getDirectPlaylists(currentPath)
			: playlists.filter((p) => {
				const q = search.toLowerCase();
				return (
					p.title.toLowerCase().includes(q) ||
					((p as any).description?.toLowerCase() ?? "").includes(q)
				);
			})
	);

	const visibleChildFolders = $derived(
		search.trim() === "" ? getChildFolders(currentPath) : []
	);

	const allFolderPaths = $derived(
		[...new Set([
			...playlists.map((p) => folderOf(p)).filter((f): f is string => f !== null),
			...folderSelection.knownFolders,
		])].sort()
	);

	function openCreatePlaylistIn(folder: string | null) {
		createDialogFolder = folder;
		nameInput = "";
		createDialogOpen = true;
	}

	function openCreateFolderIn(parent: string | null) {
		folderDialogParent = parent;
		folderNameInput = "";
		folderDialogOpen = true;
	}

	function toggleExpand(folderPath: string) {
		const next = new Set(expandedFolders);
		next.has(folderPath) ? next.delete(folderPath) : next.add(folderPath);
		expandedFolders = next;
	}

	function handlePlaylistDragOver(e: DragEvent, playlistUid: string) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
		setHoveredPlaylist(playlistUid);
	}

	function handlePlaylistDragLeave(playlistUid: string) {
		if (dragState.hoveredPlaylistUid === playlistUid) setHoveredPlaylist(null);
	}

	async function handleFolderDrop(e: DragEvent, folderPath: string) {
		e.preventDefault();
		onFolderDragExit(folderPath);
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { endDrag(); return; }
		const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
		if (uids.every((u) => u.startsWith("p-"))) {
			await movePlaylists(uids, folderPath);
		} else {
			const first = getDirectPlaylists(folderPath)[0];
			if (first) await addTracksToPlaylist(first.uid, uids);
		}
		endDrag();
	}

	async function handleRootDrop(e: DragEvent) {
		e.preventDefault();
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { endDrag(); return; }
		const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
		if (uids.every((u) => u.startsWith("p-"))) {
			await movePlaylists(uids, null);
		}
		endDrag();
	}

	function startPlaylistDrag(e: DragEvent, playlistUid: string) {
		if (e.dataTransfer) {
			e.dataTransfer.setData("text/plain", playlistUid);
			e.dataTransfer.effectAllowed = "move";
		}
		startDrag({ type: "tracks", uids: [playlistUid], sourcePlaylistUid: null });
	}
</script>

<CreateNewPlaylist bind:open={createDialogOpen} folder={createDialogFolder} />
<CreateNewFolder bind:open={folderDialogOpen} parent={folderDialogParent} />

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Playlists</h1>
		<div class="flex items-center gap-2">
			<Input placeholder="Search..." bind:value={search} class="w-48" />
			<Button
				variant="outline"
				size="icon"
				onclick={() => (viewMode = viewMode === "tiled" ? "compact" : "tiled")}
			>
				{#if viewMode === "tiled"}
					<List class="size-4" />
				{:else}
					<LayoutGrid class="size-4" />
				{/if}
			</Button>
		</div>
	</div>

	<div class="flex items-center justify-between gap-2">
		<div class="flex items-center gap-1 text-sm text-muted-foreground flex-wrap">
			{#each crumbs as crumb, i}
				{#if i > 0}
					<ChevronRight class="size-3 shrink-0" />
				{/if}
				<button
					class="hover:text-foreground transition-colors px-1 py-0.5 rounded"
					class:text-foreground={i === crumbs.length - 1}
					class:bg-primary={dragState.active}
					onclick={() => navigateTo(crumb.path)}
					ondragover={(e) => { e.preventDefault(); onBreadcrumbDragOver(crumb.path, navigateTo); }}
					ondragleave={onBreadcrumbDragExit}
					ondrop={(e) => { e.preventDefault(); navigateTo(crumb.path); endDrag(); }}
				>
					{#if i === 0}<House class="size-3 inline mr-1" />{/if}
					{crumb.label}
				</button>
			{/each}
		</div>

		<div class="flex gap-1 shrink-0">
			<Button variant="outline" size="icon" onclick={() => openCreatePlaylistIn(currentPath)}>
				<ListPlus class="size-4" />
			</Button>
			<Button variant="outline" size="icon" onclick={() => openCreateFolderIn(currentPath)}>
				<FolderPlus class="size-4" />
			</Button>
			<Button variant="outline" size="icon">
				<FileDown class="size-4" />
			</Button>
		</div>
	</div>

	<div class="flex flex-col h-full w-full overflow-hidden">
		<span class="text-sm text-muted-foreground mb-2">{playlists.length} playlists</span>

		<ScrollArea class="min-h-0 min-w-0">

			{#if currentPath !== null && dragState.active}
				<button
					class="w-full mb-2 py-1.5 px-3 rounded-md border border-dashed border-muted-foreground/30 text-xs text-muted-foreground hover:border-primary hover:text-primary transition-colors flex items-center gap-2"
					ondragover={(e) => { e.preventDefault(); onBreadcrumbDragOver(null, navigateTo); }}
					ondragleave={onBreadcrumbDragExit}
					ondrop={handleRootDrop}
				>
					<House class="size-3" /> Drag here to move to root
				</button>
			{/if}

			{#if viewMode === "tiled"}
				<div class="app-music-grid grid gap-2">

					{#each visibleChildFolders as folderPath (folderPath)}
						{@const artUids = getFolderArtworkUids(folderPath)}
						{@const folderHovered = dragState.hoveredFolderPath === folderPath}

						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full text-left">
								<button
									class="flex flex-col gap-1 w-full text-left cursor-pointer rounded-md transition-all"
									class:ring-2={folderHovered}
									class:ring-primary={folderHovered}
									onclick={() => navigateTo(folderPath)}
									ondragover={(e) => { e.preventDefault(); onFolderDragOver(folderPath, navigateTo); }}
									ondragleave={() => onFolderDragExit(folderPath)}
									ondrop={(e) => handleFolderDrop(e, folderPath)}
								>
									<PlaylistFolderCard folderPath={folderLabel(folderPath)} artUids={artUids} />
								</button>
							</ContextMenu.Trigger>
							<FolderContext path={folderPath} onCreatePlaylist={openCreatePlaylistIn} onCreateFolder={openCreateFolderIn} />
						</ContextMenu.Root>
					{/each}

					{#each filteredDirect as p (p.uid)}
						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full">
								<div
									class="relative transition-all w-full"
									class:ring-2={dragState.hoveredPlaylistUid === p.uid}
									class:ring-primary={dragState.hoveredPlaylistUid === p.uid}
									class:rounded-md={dragState.hoveredPlaylistUid === p.uid}
									draggable="true"
									ondragstart={(e) => startPlaylistDrag(e, p.uid)}
									ondragover={(e) => handlePlaylistDragOver(e, p.uid)}
									ondragleave={() => handlePlaylistDragLeave(p.uid)}
									ondragend={endDrag}
									ondrop={(e) => dropOnPlaylist(e, p.uid)}
									role="region"
									aria-label="Playlist"
								>
									<AudioCard title={p.title} subTitle={(p as any).owner ?? ""} artworkUid={p.uid} type="playlist" />
								</div>
							</ContextMenu.Trigger>
							<PlaylistContext folderPaths={allFolderPaths} uid={p.uid} />
						</ContextMenu.Root>
					{/each}

				</div>

			{:else}
				<div class="flex flex-col">

					{#each compactFolderRows(currentPath) as row (row.path)}
						{@const isExpanded = expandedFolders.has(row.path)}
						{@const folderHovered = dragState.hoveredFolderPath === row.path}

						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full">
								<div
									class="flex items-center gap-1 py-1.5 rounded-md hover:bg-muted/50 transition-all"
									class:ring-1={folderHovered}
									class:ring-primary={folderHovered}
									style="padding-left: {(row.depth + 1) * 16}px; padding-right: 8px;"
									ondragover={(e) => { e.preventDefault(); onFolderDragOver(row.path, navigateTo); }}
									ondragleave={() => onFolderDragExit(row.path)}
									ondrop={(e) => handleFolderDrop(e, row.path)}
								>
									<button
										class="shrink-0 text-muted-foreground hover:text-foreground transition-colors"
										onclick={() => toggleExpand(row.path)}
										aria-label={isExpanded ? "Collapse" : "Expand"}
									>
										{#if isExpanded}
											<ChevronDown class="size-4" />
										{:else}
											<ChevronRight class="size-4" />
										{/if}
									</button>
									<button
										class="flex items-center gap-2 flex-1 min-w-0 text-left"
										onclick={() => navigateTo(row.path)}
									>
										{#if isExpanded}
											<FolderOpen class="size-4 shrink-0 text-muted-foreground" />
										{:else}
											<Folder class="size-4 shrink-0 text-muted-foreground" />
										{/if}
										<span class="text-sm truncate">{folderLabel(row.path)}</span>
									</button>
								</div>
							</ContextMenu.Trigger>
							<FolderContext path={row.path} onCreatePlaylist={openCreatePlaylistIn} onCreateFolder={openCreateFolderIn} />
						</ContextMenu.Root>

						{#if isExpanded}
							{#each getDirectPlaylists(row.path) as p (p.uid)}
								<ContextMenu.Root>
									<ContextMenu.Trigger class="w-full">
										<div draggable="true" ondragstart={(e) => startPlaylistDrag(e, p.uid)} ondragend={endDrag} role="region" aria-label="Playlist Row">
											<PlaylistRow
												playlist={p}
												indent={(row.depth + 2) * 16}
												highlighted={dragState.hoveredPlaylistUid === p.uid}
												ondragover={(e) => handlePlaylistDragOver(e, p.uid)}
												ondragleave={() => handlePlaylistDragLeave(p.uid)}
												ondrop={(e) => dropOnPlaylist(e, p.uid)}
											/>
										</div>
									</ContextMenu.Trigger>
									<PlaylistContext folderPaths={allFolderPaths} uid={p.uid} />
								</ContextMenu.Root>
							{/each}
						{/if}
					{/each}

					{#each filteredDirect as p (p.uid)}
						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full">
								<div draggable="true" ondragstart={(e) => startPlaylistDrag(e, p.uid)} ondragend={endDrag}>
									<PlaylistRow
										playlist={p}
										indent={8}
										highlighted={dragState.hoveredPlaylistUid === p.uid}
										ondragover={(e) => handlePlaylistDragOver(e, p.uid)}
										ondragleave={() => handlePlaylistDragLeave(p.uid)}
										ondrop={(e) => dropOnPlaylist(e, p.uid)}
									/>
								</div>
							</ContextMenu.Trigger>
							<PlaylistContext folderPaths={allFolderPaths} uid={p.uid} />
						</ContextMenu.Root>
					{/each}

				</div>
			{/if}

		</ScrollArea>
	</div>

</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>