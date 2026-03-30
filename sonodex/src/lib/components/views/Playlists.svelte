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
		registerFolder,
	} from "$lib/ts/app/folderSelection.svelte";
	import { addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
	import { createPersistedViewState } from "$lib/ts/session.svelte";
	import { playlistOrder, applyCustomOrder, reorder } from "$lib/ts/app/playlistOrderStore.svelte";
	import type { Playlist } from "$lib/ts/util/types";
	import type { PlaylistSortField } from "$lib/components/app-ui/PlaylistSortBar.svelte";

	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import AudioCard from "$lib/components/app-ui/AudioCard.svelte";
	import PlaylistRow from "$lib/components/app-ui/PlaylistRow.svelte";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import PlaylistSortBar from "$lib/components/app-ui/PlaylistSortBar.svelte";
	import CreateNewPlaylist from "$lib/components/dialogs/CreateNewPlaylist.svelte";
	import CreateNewFolder from "$lib/components/dialogs/CreateNewFolder.svelte";
	import FolderContext from "$lib/components/app-ui/context-menus/FolderContext.svelte";
	import PlaylistContext from "$lib/components/app-ui/context-menus/PlaylistContext.svelte";
	import PlaylistFolderCard from "$lib/components/app-ui/PlaylistFolderCard.svelte";
	import {
		FolderPlus, ListPlus, FileDown,
		LayoutGrid, List,
		ChevronRight, ChevronDown,
		Folder, FolderOpen,
		House,
	} from "lucide-svelte";
	import { invoke } from "@tauri-apps/api/core";
    import ImportPlaylist from "../dialogs/ImportPlaylist.svelte";

	type ViewMode = "tiled" | "compact";

	const viewState = createPersistedViewState("playlists", { sortField: null, sortDir: "asc" });

	let sortField = $state<PlaylistSortField>("custom");
	let sortDir   = $state<"asc" | "desc">("asc");

	let search          = $state("");
	let viewMode        = $state<ViewMode>("tiled");
	let expandedFolders = $state<Set<string>>(new Set());

	let importDialogOpen   = $state(false);
	let createDialogOpen   = $state(false);
	let createDialogFolder = $state<string | null>(null);
	let folderDialogOpen   = $state(false);
	let folderDialogParent = $state<string | null>(null);

	let dragOverIndex    = $state<number | null>(null);
	let dragOverFolderIndex = $state<number | null>(null);
	let draggingPlaylistUid = $state<string | null>(null);
	let draggingFolderPath  = $state<string | null>(null);

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

	function getChildFoldersRaw(parentPath: string | null): string[] {
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
		return [...seen];
	}

	function getSortedFolders(parentPath: string | null): string[] {
		const raw = getChildFoldersRaw(parentPath);
		if (sortField === "custom") {
			return applyCustomOrder(raw, playlistOrder.getFolders(parentPath), (f) => f);
		}
		return [...raw].sort((a, b) => {
			const la = a.split("/").at(-1) ?? a;
			const lb = b.split("/").at(-1) ?? b;
			return sortDir === "asc" ? la.localeCompare(lb) : lb.localeCompare(la);
		});
	}

	function getDirectPlaylists(folderPath: string | null): Playlist[] {
		const direct = playlists.filter((p) => folderOf(p) === folderPath);
		if (sortField === "custom") {
			return applyCustomOrder(direct, playlistOrder.getPlaylists(folderPath), (p) => p.uid);
		}
		return [...direct].sort((a, b) => {
			if (sortField === "title") {
				return sortDir === "asc"
					? a.title.localeCompare(b.title)
					: b.title.localeCompare(a.title);
			}
			if (sortField === "date_created") {
				const ai = (a as any).id ?? 0;
				const bi = (b as any).id ?? 0;
				return sortDir === "asc" ? ai - bi : bi - ai;
			}
			return 0;
		});
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
		for (const fp of getSortedFolders(parentPath)) {
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
		search.trim() === "" ? getSortedFolders(currentPath) : []
	);

	const allFolderPaths = $derived(
		[...new Set([
			...playlists.map((p) => folderOf(p)).filter((f): f is string => f !== null),
			...folderSelection.knownFolders,
		])].sort()
	);

	function openCreatePlaylistIn(folder: string | null) {
		createDialogFolder = folder;
		createDialogOpen = true;
	}

	function openCreateFolderIn(parent: string | null) {
		folderDialogParent = parent;
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
		dragOverIndex = null;
		dragOverFolderIndex = null;
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

	function startPlaylistDrag(e: DragEvent, uid: string) {
		if (e.dataTransfer) {
			e.dataTransfer.setData("text/plain", uid);
			e.dataTransfer.effectAllowed = "move";
		}
		draggingPlaylistUid = uid;
		startDrag({ type: "tracks", uids: [uid], sourcePlaylistUid: null });
	}

	function startFolderDrag(e: DragEvent, path: string) {
		if (e.dataTransfer) {
			e.dataTransfer.setData("text/plain", path);
			e.dataTransfer.setData("application/x-sonodex-folder", path);
			e.dataTransfer.effectAllowed = "move";
		}
		draggingFolderPath = path;
		startDrag({ type: "tracks", uids: [path], sourcePlaylistUid: null });
	}

	function handleGridItemDragOver(e: DragEvent, index: number, isFolder: boolean) {
		e.preventDefault();
		e.stopPropagation();
		if (isFolder) {
			dragOverFolderIndex = index;
			dragOverIndex = null;
		} else {
			dragOverIndex = index;
			dragOverFolderIndex = null;
		}
	}

	function handleGridItemDragLeave() {
		dragOverIndex = null;
		dragOverFolderIndex = null;
	}

	function handleGridPlaylistDrop(e: DragEvent, toIndex: number) {
		e.preventDefault();
		dragOverIndex = null;
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { endDrag(); return; }
		const uid = raw.trim();
		if (!uid.startsWith("p-")) { endDrag(); return; }

		const current = getDirectPlaylists(currentPath);
		const fromIndex = current.findIndex((p) => p.uid === uid);
		if (fromIndex === -1) { endDrag(); return; }

		const reordered = reorder(current, fromIndex, toIndex);
		playlistOrder.setPlaylists(currentPath, reordered.map((p) => p.uid));
		draggingPlaylistUid = null;
		endDrag();
	}

	function handleGridFolderDrop(e: DragEvent, toIndex: number) {
		e.preventDefault();
		dragOverFolderIndex = null;
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { endDrag(); return; }
		const draggedPath = raw.trim();

		const current = getSortedFolders(currentPath);
		const fromIndex = current.indexOf(draggedPath);
		if (fromIndex === -1) {
			handleFolderDrop(e, current[toIndex]);
			return;
		}

		const reordered = reorder(current, fromIndex, toIndex);
		playlistOrder.setFolders(currentPath, reordered);
		draggingFolderPath = null;
		endDrag();
	}

	function handleDragEnd() {
		draggingPlaylistUid = null;
		draggingFolderPath = null;
		dragOverIndex = null;
		dragOverFolderIndex = null;
		endDrag();
	}

	function getCompactPlaylistOrder(folderPath: string | null): Playlist[] {
		return getDirectPlaylists(folderPath);
	}

	function handleCompactPlaylistDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		dragOverIndex = index;
	}

	function handleCompactPlaylistDrop(e: DragEvent, toIndex: number) {
		e.preventDefault();
		dragOverIndex = null;
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { endDrag(); return; }
		const uid = raw.trim();
		if (!uid.startsWith("p-")) { endDrag(); return; }

		const current = getDirectPlaylists(currentPath);
		const fromIndex = current.findIndex((p) => p.uid === uid);
		if (fromIndex === -1) { endDrag(); return; }

		const reordered = reorder(current, fromIndex, toIndex);
		playlistOrder.setPlaylists(currentPath, reordered.map((p) => p.uid));
		draggingPlaylistUid = null;
		endDrag();
	}
</script>

<CreateNewPlaylist bind:open={createDialogOpen} folder={createDialogFolder} />
<CreateNewFolder bind:open={folderDialogOpen} parent={folderDialogParent} />
<ImportPlaylist bind:open={importDialogOpen} />

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

		<div class="flex items-center gap-1 shrink-0">
			<PlaylistSortBar bind:field={sortField} bind:direction={sortDir} />
			<Button variant="outline" size="icon" onclick={() => openCreatePlaylistIn(currentPath)}>
				<ListPlus class="size-4" />
			</Button>
			<Button variant="outline" size="icon" onclick={() => openCreateFolderIn(currentPath)}>
				<FolderPlus class="size-4" />
			</Button>
			<Button variant="outline" size="icon" onclick={() => importDialogOpen = true}>
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

					{#each visibleChildFolders as folderPath, fi (folderPath)}
						{@const artUids = getFolderArtworkUids(folderPath)}
						{@const folderHovered = dragState.hoveredFolderPath === folderPath}
						{@const isDraggingThis = draggingFolderPath === folderPath}
						{@const isDropTarget = dragOverFolderIndex === fi}

						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full text-left">
								<div
									class="relative rounded-md transition-all"
									class:opacity-40={isDraggingThis}
									class:ring-2={isDropTarget}
									class:ring-primary={isDropTarget}
								>
									{#if isDropTarget && !isDraggingThis}
										<div class="absolute -left-1 top-0 bottom-0 w-0.5 bg-primary rounded-full z-10"></div>
									{/if}
									<button
										class="flex flex-col gap-1 w-full text-left cursor-grab rounded-md"
										class:ring-2={folderHovered && !isDropTarget}
										class:ring-primary={folderHovered && !isDropTarget}
										draggable="true"
										onclick={() => navigateTo(folderPath)}
										ondragstart={(e) => startFolderDrag(e, folderPath)}
										ondragend={handleDragEnd}
										ondragover={(e) => {
											if (draggingFolderPath !== null) {
												handleGridItemDragOver(e, fi, true);
											} else {
												e.preventDefault();
												onFolderDragOver(folderPath, navigateTo);
											}
										}}
										ondragleave={() => {
											handleGridItemDragLeave();
											onFolderDragExit(folderPath);
										}}
										ondrop={(e) => {
											if (draggingFolderPath !== null) {
												handleGridFolderDrop(e, fi);
											} else {
												handleFolderDrop(e, folderPath);
											}
										}}
									>
										<PlaylistFolderCard folderPath={folderLabel(folderPath)} artUids={artUids} />
									</button>
								</div>
							</ContextMenu.Trigger>
							<FolderContext path={folderPath} onCreatePlaylist={openCreatePlaylistIn} onCreateFolder={openCreateFolderIn} />
						</ContextMenu.Root>
					{/each}

					{#each filteredDirect as p, pi (p.uid)}
						{@const isDraggingThis = draggingPlaylistUid === p.uid}
						{@const isDropTarget = dragOverIndex === pi}

						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full">
								<div
									class="relative transition-all w-full rounded-md"
									class:opacity-40={isDraggingThis}
									class:ring-2={dragState.hoveredPlaylistUid === p.uid && !isDropTarget}
									class:ring-primary={dragState.hoveredPlaylistUid === p.uid && !isDropTarget}
								>
									{#if isDropTarget && !isDraggingThis}
										<div class="absolute -left-1 top-0 bottom-0 w-0.5 bg-primary rounded-full z-10"></div>
									{/if}
									<div
										draggable="true"
										class="cursor-grab"
										ondragstart={(e) => startPlaylistDrag(e, p.uid)}
										ondragend={handleDragEnd}
										ondragover={(e) => {
											if (draggingPlaylistUid !== null) {
												handleGridItemDragOver(e, pi, false);
											} else {
												handlePlaylistDragOver(e, p.uid);
											}
										}}
										ondragleave={() => {
											handleGridItemDragLeave();
											handlePlaylistDragLeave(p.uid);
										}}
										ondrop={(e) => {
											if (draggingPlaylistUid !== null) {
												handleGridPlaylistDrop(e, pi);
											} else {
												dropOnPlaylist(e, p.uid);
											}
										}}
										role="region"
										aria-label="Playlist"
									>
										<AudioCard title={p.title} subTitle={(p as any).owner ?? ""} artworkUid={p.uid} type="playlist" />
									</div>
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
									role="region"
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
							{#each getCompactPlaylistOrder(row.path) as p, pi (p.uid)}
								{@const isDropTarget = dragOverIndex === pi && draggingPlaylistUid !== null}
								<ContextMenu.Root>
									<ContextMenu.Trigger class="w-full">
										<div
											class="relative"
											class:opacity-40={draggingPlaylistUid === p.uid}
										>
											{#if isDropTarget}
												<div class="absolute left-0 right-0 -top-px h-0.5 bg-primary rounded-full z-10"></div>
											{/if}
											<div
												draggable="true"
												role="region"
												ondragstart={(e) => startPlaylistDrag(e, p.uid)}
												ondragend={handleDragEnd}
												ondragover={(e) => handleCompactPlaylistDragOver(e, pi)}
												ondrop={(e) => handleCompactPlaylistDrop(e, pi)}
											>
												<PlaylistRow
													playlist={p}
													indent={(row.depth + 2) * 16}
													highlighted={dragState.hoveredPlaylistUid === p.uid}
													ondragover={(e) => handlePlaylistDragOver(e, p.uid)}
													ondragleave={() => handlePlaylistDragLeave(p.uid)}
													ondrop={(e) => dropOnPlaylist(e, p.uid)}
												/>
											</div>
										</div>
									</ContextMenu.Trigger>
									<PlaylistContext folderPaths={allFolderPaths} uid={p.uid} />
								</ContextMenu.Root>
							{/each}
						{/if}
					{/each}

					{#each filteredDirect as p, pi (p.uid)}
						{@const isDropTarget = dragOverIndex === pi && draggingPlaylistUid !== null}
						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full">
								<div
									class="relative"
									class:opacity-40={draggingPlaylistUid === p.uid}
								>
									{#if isDropTarget}
										<div class="absolute left-0 right-0 -top-px h-0.5 bg-primary rounded-full z-10"></div>
									{/if}
									<div
										draggable="true"
										role="region"
										ondragstart={(e) => startPlaylistDrag(e, p.uid)}
										ondragend={handleDragEnd}
										ondragover={(e) => handleCompactPlaylistDragOver(e, pi)}
										ondrop={(e) => handleCompactPlaylistDrop(e, pi)}
									>
										<PlaylistRow
											playlist={p}
											indent={8}
											highlighted={dragState.hoveredPlaylistUid === p.uid && draggingPlaylistUid === null}
											ondragover={(e) => handlePlaylistDragOver(e, p.uid)}
											ondragleave={() => handlePlaylistDragLeave(p.uid)}
											ondrop={(e) => dropOnPlaylist(e, p.uid)}
										/>
									</div>
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