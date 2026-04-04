<script lang="ts">

	// APP
	import { invoke } from "@tauri-apps/api/core";

	// COMPONENTS
	import { FolderPlus, ListPlus, FileDown, LayoutGrid, List, ChevronRight, ChevronDown, Folder, FolderOpen, House } from "lucide-svelte";

	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Button } from "$lib/components/ui/button/index.js";

	// CUSTOM COMPONENTS
	import AudioCard from "$lib/components/app-ui/AudioCard.svelte";
	import PlaylistRow from "$lib/components/app-ui/playlist/PlaylistRow.svelte";
	import PlaylistSortBar from "$lib/components/app-ui/playlist/PlaylistSortBar.svelte";
	import CreateNewPlaylist from "$lib/components/dialogs/CreateNewPlaylist.svelte";
	import CreateNewFolder from "$lib/components/dialogs/CreateNewFolder.svelte";
	import FolderContext from "$lib/components/app-ui/context-menus/FolderContext.svelte";
	import PlaylistContext from "$lib/components/app-ui/context-menus/PlaylistContext.svelte";
	import PlaylistFolderCard from "$lib/components/app-ui/playlist/PlaylistFolderCard.svelte";
	import ImportPlaylist from "$lib/components/dialogs/ImportPlaylist.svelte";

	// SCRIPTS
	import { library } from "$lib/ts/library.svelte";
	import { dragState, endDrag, setHoveredPlaylist } from "$lib/ts/app-states/state_drag.svelte";
	import { isDraggingFolderType } from "$lib/ts/drag-n-drop/dragdrop";
	import { startFolderDrag, onFolderDragOver, onFolderDragExit, onBreadcrumbDragOver, onBreadcrumbDragExit, nestFolder, moveFolderToRoot, resetFolderTimers } from "$lib/ts/drag-n-drop/dragdrop_folders";
	import { startPlaylistDrag, dropOnPlaylist, movePlaylists } from "$lib/ts/drag-n-drop/dragdrop_playlists";
	import { addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
	import { folderSelection, navigateTo, breadcrumbs } from "$lib/ts/app/folderSelection.svelte";
	import { playlistOrder, applyCustomOrder } from "$lib/ts/app/playlistOrderStore.svelte";

	import type { Playlist } from "$lib/ts/util/types";
	type PlaylistSortField = "title" | "date_created" | "custom";

	type GridDropTarget = {
		kind: "folder" | "playlist";
		index: number;
		side: "before" | "after";
	} | null;

	// VARIABLES
	const SORT_KEY = "sonodex:view:playlists";
	const _saved = loadSort();
	let sortField = $state<PlaylistSortField>(_saved.field);
	let sortDir   = $state<"asc" | "desc">(_saved.dir);
	let compact   = $state<boolean>(_saved.compact);

	let search          = $state("");
	let expandedFolders = $state<Set<string>>(new Set());

	let importDialogOpen   = $state(false);
	let createDialogOpen   = $state(false);
	let createDialogFolder = $state<string | null>(null);
	let folderDialogOpen   = $state(false);
	let folderDialogParent = $state<string | null>(null);

	let draggingPlaylistUid = $state<string | null>(null);
	let draggingFolderPath  = $state<string | null>(null);

	let gridDropTarget  = $state<GridDropTarget>(null);
	let compactDropIndex = $state<number | null>(null);
	let compactDropSide  = $state<"before" | "after">("before");

	let playlists   = $derived(library.playlists ?? []);
	let currentPath = $derived(folderSelection.currentPath);
	let crumbs      = $derived(breadcrumbs());

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

	$effect(() => {
		try {
			localStorage.setItem(SORT_KEY, JSON.stringify({ sortField, sortDir, compact }));
		} catch {}
	});

	// FUNCTIONS
	function loadSort(): { field: PlaylistSortField; dir: "asc" | "desc"; compact: boolean } {
		try {
			const raw = localStorage.getItem(SORT_KEY);
			if (raw) {
				const s = JSON.parse(raw);
				return {
					field: s.sortField ?? "custom",
					dir: s.sortDir ?? "asc",
					compact: s.compact ?? false,
				};
			}
		} catch {}
		return { field: "custom", dir: "asc", compact: false };
	}

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

	function getSide(e: DragEvent): "before" | "after" {
		const el = e.currentTarget as HTMLElement;
		const rect = el.getBoundingClientRect();
		return (e.clientX - rect.left) < rect.width / 2 ? "before" : "after";
	}

	function clearGridDrop() {
		gridDropTarget = null;
	}

	function handleDragEnd() {
		draggingPlaylistUid = null;
		draggingFolderPath = null;
		gridDropTarget = null;
		compactDropIndex = null;
		resetFolderTimers();
		endDrag();
	}

	function handleFolderDragStart(e: DragEvent, folderPath: string) {
		draggingFolderPath = folderPath;
		startFolderDrag(e, folderPath);
	}

	function handlePlaylistDragStart(e: DragEvent, uid: string) {
		draggingPlaylistUid = uid;
		startPlaylistDrag(e, uid);
	}

	// GRID DRAG OVER

	function onGridFolderDragOver(e: DragEvent, fi: number, folderPath: string) {
		e.preventDefault();
		e.stopPropagation();
		if (draggingFolderPath === folderPath) return;

		const el = e.currentTarget as HTMLElement;
		const rect = el.getBoundingClientRect();
		const edgeZone = rect.width * 0.2;
		const x = e.clientX - rect.left;
		const nearEdge = x < edgeZone || x > rect.width - edgeZone;

		if (isDraggingFolderType(e) && nearEdge) {
			const side = x < rect.width / 2 ? "before" : "after";
			gridDropTarget = { kind: "folder", index: fi, side };
			onFolderDragExit(folderPath);
		} else {
			gridDropTarget = null;
			onFolderDragOver(folderPath, navigateTo);
		}
	}

	function onGridPlaylistDragOver(e: DragEvent, pi: number, uid: string) {
		e.preventDefault();
		e.stopPropagation();
		if (draggingPlaylistUid === uid) return;

		if (isDraggingFolderType(e)) {
			const side = getSide(e);
			gridDropTarget = { kind: "folder", index: visibleChildFolders.length, side };
		} else {
			const side = getSide(e);
			gridDropTarget = { kind: "playlist", index: pi, side };
			setHoveredPlaylist(null);
		}
	}

	function onGridEmptyDragOver(e: DragEvent) {
		e.preventDefault();
		if (isDraggingFolderType(e)) {
			gridDropTarget = { kind: "folder", index: visibleChildFolders.length, side: "after" };
		} else {
			gridDropTarget = { kind: "playlist", index: filteredDirect.length, side: "after" };
		}
	}

	// GRID DROP

	async function onGridFolderDrop(e: DragEvent, fi: number, folderPath: string) {
		e.preventDefault();
		e.stopPropagation();
		onFolderDragExit(folderPath);

		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { clearGridDrop(); endDrag(); return; }
		const val = raw.trim();

		if (gridDropTarget !== null) {
			if (isDraggingFolderType(e)) {
				await doFolderReorder(val, gridDropTarget.index, gridDropTarget.side);
			} else if (val.startsWith("p-")) {
				await doPlaylistReorder(val, gridDropTarget.index, gridDropTarget.side);
			}
		} else {
			if (isDraggingFolderType(e)) {
				await nestFolder(val, folderPath);
			} else if (val.startsWith("p-")) {
				await movePlaylists([val], folderPath);
			} else {
				const first = getDirectPlaylists(folderPath)[0];
				if (first) await addTracksToPlaylist(first.uid, val.split(",").map((u) => u.trim()).filter(Boolean));
			}
		}

		clearGridDrop();
		endDrag();
	}

	async function onGridPlaylistDrop(e: DragEvent, pi: number, uid: string) {
		e.preventDefault();
		e.stopPropagation();
		setHoveredPlaylist(null);

		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { clearGridDrop(); endDrag(); return; }
		const val = raw.trim();

		if (isDraggingFolderType(e) && gridDropTarget !== null) {
			await doFolderReorder(val, gridDropTarget.index, gridDropTarget.side);
		} else if (!isDraggingFolderType(e) && val.startsWith("p-") && gridDropTarget !== null) {
			await doPlaylistReorder(val, gridDropTarget.index, gridDropTarget.side);
		} else if (!isDraggingFolderType(e) && !val.startsWith("p-")) {
			await addTracksToPlaylist(uid, val.split(",").map((u) => u.trim()).filter(Boolean));
		}

		clearGridDrop();
		endDrag();
	}

	async function onGridEmptyDrop(e: DragEvent) {
		e.preventDefault();
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { clearGridDrop(); endDrag(); return; }
		const val = raw.trim();

		if (isDraggingFolderType(e)) {
			await doFolderReorder(val, visibleChildFolders.length, "after");
		} else if (val.startsWith("p-")) {
			await doPlaylistReorder(val, filteredDirect.length, "after");
		}

		clearGridDrop();
		endDrag();
	}

	// REORDER HELPERS

	async function doFolderReorder(draggedPath: string, targetIndex: number, side: "before" | "after") {
		const current = getSortedFolders(currentPath);
		const fromIndex = current.indexOf(draggedPath);
		if (fromIndex === -1) {
			await nestFolder(draggedPath, currentPath ?? draggedPath.split("/").slice(0, -1).join("/"));
			const updated = getSortedFolders(currentPath);
			const newFrom = updated.indexOf(draggedPath);
			if (newFrom === -1) return;
			const insertAt = side === "before" ? targetIndex : targetIndex + 1;
			const without = updated.filter((_, i) => i !== newFrom);
			const adjusted = newFrom < insertAt ? insertAt - 1 : insertAt;
			without.splice(Math.max(0, Math.min(adjusted, without.length)), 0, draggedPath);
			playlistOrder.setFolders(currentPath, without);
			return;
		}
		const insertAt = side === "before" ? targetIndex : targetIndex + 1;
		const without = current.filter((_, i) => i !== fromIndex);
		const adjusted = fromIndex < insertAt ? insertAt - 1 : insertAt;
		without.splice(Math.max(0, Math.min(adjusted, without.length)), 0, draggedPath);
		playlistOrder.setFolders(currentPath, without);
	}

	async function doPlaylistReorder(draggedUid: string, targetIndex: number, side: "before" | "after") {
		const current = getDirectPlaylists(currentPath);
		const fromIndex = current.findIndex((p) => p.uid === draggedUid);
		if (fromIndex === -1) {
			await movePlaylists([draggedUid], currentPath);
			const updated = getDirectPlaylists(currentPath);
			const newFrom = updated.findIndex((p) => p.uid === draggedUid);
			if (newFrom === -1) return;
			const insertAt = side === "before" ? targetIndex : targetIndex + 1;
			const uids = updated.map((p) => p.uid);
			uids.splice(newFrom, 1);
			const adjusted = newFrom < insertAt ? insertAt - 1 : insertAt;
			uids.splice(Math.max(0, Math.min(adjusted, uids.length)), 0, draggedUid);
			playlistOrder.setPlaylists(currentPath, uids);
			return;
		}
		const insertAt = side === "before" ? targetIndex : targetIndex + 1;
		const uids = current.map((p) => p.uid);
		uids.splice(fromIndex, 1);
		const adjusted = fromIndex < insertAt ? insertAt - 1 : insertAt;
		uids.splice(Math.max(0, Math.min(adjusted, uids.length)), 0, draggedUid);
		playlistOrder.setPlaylists(currentPath, uids);
	}

	// ROOT DROP

	async function handleDropOnRoot(e: DragEvent) {
		e.preventDefault();
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { endDrag(); return; }
		const val = raw.trim();
		if (isDraggingFolderType(e)) {
			await moveFolderToRoot(val);
		} else if (val.startsWith("p-")) {
			await movePlaylists([val], null);
		}
		endDrag();
	}

	// COMPACT LIST DRAG

	function handleCompactPlaylistDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		const el = e.currentTarget as HTMLElement;
		const rect = el.getBoundingClientRect();
		compactDropSide = (e.clientY - rect.top) < rect.height / 2 ? "before" : "after";
		compactDropIndex = index;
	}

	async function handleCompactPlaylistDrop(e: DragEvent, toIndex: number) {
		e.preventDefault();
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { compactDropIndex = null; endDrag(); return; }
		const uid = raw.trim();
		if (!uid.startsWith("p-")) { compactDropIndex = null; endDrag(); return; }
		const side = compactDropSide;
		compactDropIndex = null;
		doPlaylistReorder(uid, toIndex, side);
		draggingPlaylistUid = null;
		endDrag();
	}
</script>

<ImportPlaylist bind:open={importDialogOpen} />
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
				onclick={() => (compact = !compact)}
			>
				{#if !compact}
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
					class="hover:text-foreground transition-colors px-1 py-0.5 rounded {dragState.active ? 'bg-primary/20' : ''}"
					class:text-foreground={i === crumbs.length - 1}
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
					ondrop={handleDropOnRoot}
				>
					<House class="size-3" /> Drag here to move to root
				</button>
			{/if}

			{#if !compact}

				<div class="app-music-grid grid gap-2">

					{#each visibleChildFolders as folderPath, fi (folderPath)}
						{@const artUids = getFolderArtworkUids(folderPath)}
						{@const isDraggingThis = draggingFolderPath === folderPath}
						{@const isReorderTarget = gridDropTarget?.kind === "folder" && gridDropTarget.index === fi}
						{@const isMoveTarget = dragState.hoveredFolderPath === folderPath && gridDropTarget === null}

						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full text-left">
								<div
									class="relative rounded-md transition-all"
									class:opacity-40={isDraggingThis}
								>
									{#if isReorderTarget && gridDropTarget?.side === "before" && !isDraggingThis}
										<div class="absolute -left-1 top-0 bottom-0 w-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
									{/if}
									{#if isReorderTarget && gridDropTarget?.side === "after" && !isDraggingThis}
										<div class="absolute -right-1 top-0 bottom-0 w-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
									{/if}
									<button
										class="flex flex-col gap-1 w-full text-left cursor-grab rounded-md"
										class:ring-2={isMoveTarget}
										class:ring-primary={isMoveTarget}
										draggable="true"
										onclick={() => navigateTo(folderPath)}
										ondragstart={(e) => handleFolderDragStart(e, folderPath)}
										ondragend={handleDragEnd}
										ondragover={(e) => onGridFolderDragOver(e, fi, folderPath)}
										ondragleave={() => { clearGridDrop(); onFolderDragExit(folderPath); }}
										ondrop={(e) => onGridFolderDrop(e, fi, folderPath)}
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
						{@const isReorderTarget = gridDropTarget?.kind === "playlist" && gridDropTarget.index === pi}

						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full">
								<div
									class="relative transition-all w-full rounded-md"
									class:opacity-40={isDraggingThis}
								>
									{#if isReorderTarget && gridDropTarget?.side === "before" && !isDraggingThis}
										<div class="absolute -left-1 top-0 bottom-0 w-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
									{/if}
									{#if isReorderTarget && gridDropTarget?.side === "after" && !isDraggingThis}
										<div class="absolute -right-1 top-0 bottom-0 w-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
									{/if}
									<div
										draggable="true"
										class="cursor-grab"
										ondragstart={(e) => handlePlaylistDragStart(e, p.uid)}
										ondragend={handleDragEnd}
										ondragover={(e) => onGridPlaylistDragOver(e, pi, p.uid)}
										ondragleave={() => { clearGridDrop(); setHoveredPlaylist(null); }}
										ondrop={(e) => onGridPlaylistDrop(e, pi, p.uid)}
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

				<div
					class="min-h-16 mt-2 rounded-md transition-colors"
					class:border-2={dragState.active}
					class:border-dashed={dragState.active}
					class:border-muted-foreground={dragState.active}
					ondragover={onGridEmptyDragOver}
					ondragleave={() => clearGridDrop()}
					ondrop={onGridEmptyDrop}
				></div>

			{:else}
				<div class="flex flex-col">

					{#each compactFolderRows(currentPath) as row (row.path)}
						{@const isExpanded = expandedFolders.has(row.path)}
						{@const folderHovered = dragState.hoveredFolderPath === row.path}

						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full">
								<div
									draggable="true"
									class="flex items-center gap-1 py-1.5 rounded-md hover:bg-muted/50 transition-all cursor-grab"
									class:ring-1={folderHovered}
									class:ring-primary={folderHovered}
									class:opacity-40={draggingFolderPath === row.path}
									style="padding-left: {(row.depth + 1) * 16}px; padding-right: 8px;"
									ondragstart={(e) => handleFolderDragStart(e, row.path)}
									ondragend={handleDragEnd}
									ondragover={(e) => {
										e.preventDefault();
										if (draggingFolderPath !== row.path) {
											onFolderDragOver(row.path, navigateTo);
										}
									}}
									ondragleave={() => onFolderDragExit(row.path)}
									ondrop={async (e) => {
										e.preventDefault();
										onFolderDragExit(row.path);
										const raw = e.dataTransfer?.getData("text/plain");
										if (!raw) { endDrag(); return; }
										const val = raw.trim();
										if (isDraggingFolderType(e)) {
											await nestFolder(val, row.path);
										} else if (val.startsWith("p-")) {
											await movePlaylists([val], row.path);
										}
										endDrag();
									}}
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
							{#each getDirectPlaylists(row.path) as p, pi (p.uid)}
								{@const isDropBefore = compactDropIndex === pi && compactDropSide === "before"}
								{@const isDropAfter  = compactDropIndex === pi && compactDropSide === "after"}
								<ContextMenu.Root>
									<ContextMenu.Trigger class="w-full">
										<div
											class="relative"
											class:opacity-40={draggingPlaylistUid === p.uid}
										>
											{#if isDropBefore}
												<div class="absolute left-0 right-0 -top-px h-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
											{/if}
											{#if isDropAfter}
												<div class="absolute left-0 right-0 -bottom-px h-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
											{/if}
											<div
												draggable="true"
												ondragstart={(e) => handlePlaylistDragStart(e, p.uid)}
												ondragend={handleDragEnd}
												ondragover={(e) => handleCompactPlaylistDragOver(e, pi)}
												ondragleave={() => { compactDropIndex = null; }}
												ondrop={(e) => handleCompactPlaylistDrop(e, pi)}
											>
												<PlaylistRow
													playlist={p}
													indent={(row.depth + 2) * 16}
													highlighted={dragState.hoveredPlaylistUid === p.uid && draggingPlaylistUid === null}
													ondragover={(e) => { e.preventDefault(); setHoveredPlaylist(p.uid); }}
													ondragleave={() => setHoveredPlaylist(null)}
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
						{@const isDropBefore = compactDropIndex === pi && compactDropSide === "before"}
						{@const isDropAfter  = compactDropIndex === pi && compactDropSide === "after"}
						<ContextMenu.Root>
							<ContextMenu.Trigger class="w-full">
								<div
									class="relative"
									class:opacity-40={draggingPlaylistUid === p.uid}
								>
									{#if isDropBefore}
										<div class="absolute left-0 right-0 -top-px h-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
									{/if}
									{#if isDropAfter}
										<div class="absolute left-0 right-0 -bottom-px h-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
									{/if}
									<div
										draggable="true"
										ondragstart={(e) => handlePlaylistDragStart(e, p.uid)}
										ondragend={handleDragEnd}
										ondragover={(e) => handleCompactPlaylistDragOver(e, pi)}
										ondragleave={() => { compactDropIndex = null; }}
										ondrop={(e) => handleCompactPlaylistDrop(e, pi)}
									>
										<PlaylistRow
											playlist={p}
											indent={8}
											highlighted={dragState.hoveredPlaylistUid === p.uid && draggingPlaylistUid === null}
											ondragover={(e) => { e.preventDefault(); setHoveredPlaylist(p.uid); }}
											ondragleave={() => setHoveredPlaylist(null)}
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