<script lang="ts">

	// COMPONENTS
	import { FolderPlus, ListPlus, FileDown, LayoutGrid, List, ChevronRight, ChevronDown, Folder, FolderOpen, House } from "lucide-svelte";
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Button } from "$lib/components/ui/button/index.js";

	// CUSTOM COMPONENTS
	import PlaylistSortBar from "$lib/components/app-ui/playlist/PlaylistSortBar.svelte";
	import PlaylistFolderCard from "$lib/components/app-ui/playlist/PlaylistFolderCard.svelte";
	import PlaylistGridCard from "$lib/components/app-ui/playlist/PlaylistGridCard.svelte";
	import PlaylistCompactRow from "$lib/components/app-ui/playlist/PlaylistCompactRow.svelte";
	import FolderContext from "$lib/components/app-ui/context-menus/FolderContext.svelte";
	import CreateNewPlaylist from "$lib/components/dialogs/playlists/CreateNewPlaylist.svelte";
	import CreateNewFolder from "$lib/components/dialogs/playlists/CreateNewFolder.svelte";
	import ImportPlaylist from "$lib/components/dialogs/playlists/ImportPlaylist.svelte";
	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";
	import MediaGrid from "$lib/layouts/MediaGrid.svelte";
   import PlaylistViewContext from "$lib/components/app-ui/context-menus/PlaylistViewContext.svelte";

	// SCRIPTS
	import Fuse from "fuse.js";
	import { library } from "$lib/ts/library.svelte";
	import { dragState, endDrag, setHoveredPlaylist } from "$lib/ts/app-states/state_drag.svelte";
	import { isDraggingFolderType } from "$lib/ts/drag-n-drop/dragdrop";
	import { startFolderDrag, onFolderDragOver, onFolderDragExit, onBreadcrumbDragOver, onBreadcrumbDragExit, nestFolder, moveFolderToRoot, resetFolderTimers, doFolderReorder, doCompactFolderReorder } from "$lib/ts/drag-n-drop/dragdrop_folders";
	import { startPlaylistDrag, dropOnPlaylist, movePlaylists, doPlaylistReorder } from "$lib/ts/drag-n-drop/dragdrop_playlists";
	import { addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
	import { folderSelection, navigateTo, breadcrumbs } from "$lib/ts/app/folderSelection.svelte";
	import { playlistOrder } from "$lib/ts/app/playlistOrderStore.svelte";
	import { folderOf, folderLabel, getSortedFolders, getDirectPlaylists, getFolderArtworkUids, compactRows, compactFolderRows } from "$lib/ts/app/playlistLibrary.svelte";
	import type { PlaylistSortField, CompactRow } from "$lib/ts/app/playlistLibrary.svelte";
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";

	type GridDropTarget = {
		kind: "folder" | "playlist";
		index: number;
		side: "before" | "after";
	} | null;

	type CompactDropTarget = { folderPath: string | null; index: number; side: "before" | "after" } | null;
	type CompactFolderDrop = { index: number; side: "before" | "after" } | null;

	// VARIABLES
	const SORT_KEY = "sonodex:view:playlists";
	const _saved = loadSort();
	let sortField = $state<PlaylistSortField>(_saved.field);
	let sortDir   = $state<"asc" | "desc">(_saved.dir);
	let compact   = $state<boolean>(_saved.compact);

	let playlistHoverTimer: ReturnType<typeof setTimeout> | null = null;

	let search          = $state("");
	let expandedFolders = $state<Set<string>>(new Set());

	let importDialogOpen   = $state(false);
	let createDialogOpen   = $state(false);
	let createDialogFolder = $state<string | null>(null);
	let folderDialogOpen   = $state(false);
	let folderDialogParent = $state<string | null>(null);

	let draggingPlaylistUid = $state<string | null>(null);
	let draggingFolderPath  = $state<string | null>(null);

	let gridDropTarget   = $state<GridDropTarget>(null);
	let compactDrop      = $state<CompactDropTarget>(null);
	let compactFolderDrop = $state<CompactFolderDrop>(null);

	let playlists   = $derived(library.playlists ?? []);
	let currentPath = $derived(folderSelection.currentPath);
	let crumbs      = $derived(breadcrumbs());

	const visibleChildFolders = $derived(
		search.trim() === "" ? getSortedFolders(playlists, currentPath, sortField, sortDir) : []
	);

	let fuseInstance: Fuse<(typeof library.playlists)[0]> | null = $state(null);
	let lastPlaylistsRef: typeof library.playlists | null = null;

	function getFuse() {
		if (fuseInstance && lastPlaylistsRef === library.playlists) return fuseInstance;
		lastPlaylistsRef = library.playlists;
		fuseInstance = new Fuse(library.playlists, {
			keys: [
				{ name: "title", weight: 0.5,  getFn: (t) => t.title ?? "" },
				{ name: "owner", weight: 0.25, getFn: (t) => t.owner ?? "" },
			],
			threshold: 0.35,
			ignoreLocation: true,
			includeScore: false,
			useExtendedSearch: false,
			minMatchCharLength: 2,
		});
		return fuseInstance;
	}

	const filteredDirect = $derived(
		search.trim() === ""
			? getDirectPlaylists(playlists, currentPath, sortField, sortDir)
			: search.trim().length < 2 ? library.playlists : getFuse().search(search).map((r) => r.item)
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

	function clearGridDrop() { gridDropTarget = null; }

	function handleDragEnd() {
		draggingPlaylistUid = null;
		draggingFolderPath = null;
		gridDropTarget = null;
		compactDrop = null;
		compactFolderDrop = null;
		if (playlistHoverTimer !== null) { clearTimeout(playlistHoverTimer); playlistHoverTimer = null; }
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
			gridDropTarget = { kind: "folder", index: fi, side: x < rect.width / 2 ? "before" : "after" };
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
			gridDropTarget = { kind: "folder", index: visibleChildFolders.length, side: getSide(e) };
		} else if (draggingPlaylistUid !== null) {
			gridDropTarget = { kind: "playlist", index: pi, side: getSide(e) };
			setHoveredPlaylist(null);
			if (playlistHoverTimer !== null) { clearTimeout(playlistHoverTimer); playlistHoverTimer = null; }
		} else {
			gridDropTarget = null;
			setHoveredPlaylist(uid);
			if (playlistHoverTimer === null) {
				playlistHoverTimer = setTimeout(() => {
					setSelection(uid, "playlist");
					playlistHoverTimer = null;
				}, 700);
			}
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
				await doFolderReorder(val, gridDropTarget.index, gridDropTarget.side, playlists, currentPath, sortField, sortDir);
			} else if (val.startsWith("p-")) {
				await doPlaylistReorder(val, gridDropTarget.index, gridDropTarget.side, playlists, sortField, sortDir, currentPath);
			}
		} else {
			if (isDraggingFolderType(e)) {
				await nestFolder(val, folderPath);
			} else if (val.startsWith("p-")) {
				await movePlaylists([val], folderPath);
			} else {
				const first = getDirectPlaylists(playlists, folderPath, sortField, sortDir)[0];
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
		console.log(val)
		if (isDraggingFolderType(e) && gridDropTarget !== null) {
			await doFolderReorder(val, gridDropTarget.index, gridDropTarget.side, playlists, currentPath, sortField, sortDir);
		} else if (val.startsWith("p-") && gridDropTarget !== null) {
			await doPlaylistReorder(val, gridDropTarget.index, gridDropTarget.side, playlists, sortField, sortDir, currentPath);
		} else if (!val.startsWith("p-") && !isDraggingFolderType(e)) {
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
			await doFolderReorder(val, visibleChildFolders.length, "after", playlists, currentPath, sortField, sortDir);
		} else if (val.startsWith("p-")) {
			await doPlaylistReorder(val, filteredDirect.length, "after", playlists, sortField, sortDir, currentPath);
		}
		clearGridDrop();
		endDrag();
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

	function handleCompactPlaylistDragOver(e: DragEvent, index: number, folderPath: string | null) {
		e.preventDefault();
		const el = e.currentTarget as HTMLElement;
		const rect = el.getBoundingClientRect();
		compactDrop = {
			folderPath,
			index,
			side: (e.clientY - rect.top) < rect.height / 2 ? "before" : "after",
		};
	}

	async function handleCompactPlaylistDrop(e: DragEvent, toIndex: number, folderPath: string | null) {
		e.preventDefault();
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { compactDrop = null; endDrag(); return; }
		const uid = raw.trim();
		if (!uid.startsWith("p-")) { compactDrop = null; endDrag(); return; }
		const side = compactDrop?.side ?? "after";
		compactDrop = null;
		await doPlaylistReorder(uid, toIndex, side, playlists, sortField, sortDir, folderPath);
		draggingPlaylistUid = null;
		endDrag();
	}

	function handleCompactFolderDragOver(e: DragEvent, ri: number, folderPath: string) {
		e.preventDefault();
		if (draggingFolderPath === folderPath) return;
		const el = e.currentTarget as HTMLElement;
		const rect = el.getBoundingClientRect();
		const edgeZone = rect.height * 0.2;
		const y = e.clientY - rect.top;
		const nearEdge = y < edgeZone || y > rect.height - edgeZone;

		if (isDraggingFolderType(e) && nearEdge) {
			compactFolderDrop = { index: ri, side: y < rect.height / 2 ? "before" : "after" };
			onFolderDragExit(folderPath);
		} else if (isDraggingFolderType(e)) {
			compactFolderDrop = null;
			onFolderDragOver(folderPath, navigateTo);
		} else {
			// playlist dragged over folder — track insertion point as "after last item"
			compactFolderDrop = null;
			compactDrop = {
				folderPath,
				index: getDirectPlaylists(playlists, folderPath, sortField, sortDir).length,
				side: "after",
			};
			onFolderDragOver(folderPath, navigateTo);
		}
	}

	async function handleCompactFolderDrop(e: DragEvent, ri: number, folderPath: string) {
		e.preventDefault();
		onFolderDragExit(folderPath);
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) { compactFolderDrop = null; compactDrop = null; endDrag(); return; }
		const val = raw.trim();

		if (compactFolderDrop !== null && isDraggingFolderType(e)) {
			await doCompactFolderReorder(val, compactFolderDrop.index, compactFolderDrop.side, playlists, currentPath, sortField, sortDir, expandedFolders);
		} else if (isDraggingFolderType(e)) {
			await nestFolder(val, folderPath);
		} else if (val.startsWith("p-")) {
			const drop = compactDrop;
			if (drop !== null) {
				await doPlaylistReorder(val, drop.index, drop.side, playlists, sortField, sortDir, folderPath);
			} else {
				await movePlaylists([val], folderPath);
			}
		}
		compactFolderDrop = null;
		compactDrop = null;
		endDrag();
	}
</script>

<CreateNewPlaylist bind:open={createDialogOpen} folder={createDialogFolder} />
<CreateNewFolder bind:open={folderDialogOpen} parent={folderDialogParent} />

<ContextMenu.Root>
	<ContextMenu.Trigger class="h-full w-full overflow-hidden">
		<div class="flex flex-col gap-2 border-2 h-full w-full overflow-hidden rounded-md">
		
			<div class="flex justify-between items-center gap-2 p-4">
				<h1 class="h1">Playlists</h1>
				<div class="flex items-center gap-1 shrink-0">
					<SearchBar bind:search searchCount={filteredDirect.length} />
					
					<Button variant="outline" size="icon" onclick={() => openCreatePlaylistIn(currentPath)}>
						<ListPlus class="size-4" />
					</Button>
					<Button variant="outline" size="icon" onclick={() => openCreateFolderIn(currentPath)}>
						<FolderPlus class="size-4" />
					</Button>
				</div>
			</div>
		
			<div class="flex justify-between items-center pl-4 pr-4">
				<span class="text-sm text-muted-foreground">{playlists.length} playlists</span>
				
				<div class="flex items-center gap-2">
					<PlaylistSortBar bind:field={sortField} bind:direction={sortDir} />
					<Button variant="ghost" size="icon" onclick={() => (compact = !compact)}>
						{#if !compact}
						<LayoutGrid class="size-4" />
						{:else}
						<List class="size-4" />
						{/if}
					</Button>
				</div>
			</div>
		
			<div class="flex items-center justify-between gap-2 pl-4 pr-4">
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
			</div>

			<div class="flex flex-col h-full w-full overflow-hidden pr-0.5">
				<ScrollArea class="min-h-0 min-w-0 mt-0 pt-0">
		
					{@const showRootDrop = currentPath !== null && dragState.active}
					<button
						class="w-full mx-4 px-3 rounded-md border border-dashed border-muted-foreground/30 text-xs text-muted-foreground flex items-center gap-2 transition-all duration-150 overflow-hidden"
						class:opacity-0={!showRootDrop}
						class:max-h-0={!showRootDrop}
						class:max-h-10={showRootDrop}
						class:py-2={showRootDrop}
						class:mb-4={showRootDrop}
						class:pointer-events-none={!showRootDrop}
						ondragover={(e) => { e.preventDefault(); onBreadcrumbDragOver(null, navigateTo); }}
						ondragleave={onBreadcrumbDragExit}
						ondrop={handleDropOnRoot}
					>
						<House class="size-3" /> Drag here to move to root
					</button>
		
					{#if !compact}
		
						<MediaGrid>
		
							{#each visibleChildFolders as folderPath, fi (folderPath)}
								{@const artUids = getFolderArtworkUids(playlists, folderPath)}
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
												<div class="absolute -left-2 top-0 bottom-0 w-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
											{/if}
											{#if isReorderTarget && gridDropTarget?.side === "after" && !isDraggingThis}
												<div class="absolute -right-2 top-0 bottom-0 w-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
											{/if}
											<button
												class="flex flex-col gap-1 w-full text-left rounded-md bg-muted"
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
									<FolderContext path={folderPath} folderPaths={allFolderPaths} sortField={sortField} sortDir={sortDir} onCreatePlaylist={openCreatePlaylistIn} onCreateFolder={openCreateFolderIn} />
								</ContextMenu.Root>
							{/each}
		
							{#each filteredDirect as p, pi (p.uid)}
								<PlaylistGridCard
									playlist={p}
									isDraggingThis={draggingPlaylistUid === p.uid}
									isHovered={dragState.hoveredPlaylistUid === p.uid}
									isReorderBefore={gridDropTarget?.kind === "playlist" && gridDropTarget.index === pi && gridDropTarget.side === "before"}
									isReorderAfter={gridDropTarget?.kind === "playlist" && gridDropTarget.index === pi && gridDropTarget.side === "after"}
									{allFolderPaths}
									ondragstart={(e) => handlePlaylistDragStart(e, p.uid)}
									ondragend={handleDragEnd}
									ondragover={(e) => onGridPlaylistDragOver(e, pi, p.uid)}
									ondragleave={() => { clearGridDrop(); setHoveredPlaylist(null); if (playlistHoverTimer !== null) { clearTimeout(playlistHoverTimer); playlistHoverTimer = null; } }}
									ondrop={(e) => onGridPlaylistDrop(e, pi, p.uid)}
								/>
							{/each}
		
							<div
								class="min-h-16 mt-2 rounded-md transition-colors"
								class:border-2={dragState.active}
								class:border-dashed={dragState.active}
								class:border-muted-foreground={dragState.active}
								ondragover={onGridEmptyDragOver}
								ondragleave={() => clearGridDrop()}
								ondrop={onGridEmptyDrop}
							></div>
						</MediaGrid>
		
		
					{:else}
						<div class="flex flex-col pr-4 pl-4 pb-4">
		
							{#each compactRows(playlists, currentPath, sortField, sortDir, expandedFolders) as row, ri}
								{#if row.kind === "folder"}
									{@const isExpanded = expandedFolders.has(row.path)}
									{@const folderHovered = dragState.hoveredFolderPath === row.path && compactFolderDrop === null}
									{@const isReorderTarget = compactFolderDrop?.index === ri}
		
									<ContextMenu.Root>
										<ContextMenu.Trigger class="w-full">
											<div class="relative">
												{#if isReorderTarget && compactFolderDrop?.side === "before" && draggingFolderPath !== row.path}
													<div class="absolute left-0 right-0 -top-px h-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
												{/if}
												{#if isReorderTarget && compactFolderDrop?.side === "after" && draggingFolderPath !== row.path}
													<div class="absolute left-0 right-0 -bottom-px h-0.5 bg-primary rounded-full z-10 pointer-events-none"></div>
												{/if}
												<div
													draggable="true"
													class="flex items-center gap-1 py-1.5 rounded-md hover:bg-muted/50 transition-all cursor-grab"
													class:ring-1={folderHovered}
													class:ring-primary={folderHovered}
													class:opacity-40={draggingFolderPath === row.path}
													style="padding-left: {(row.depth + 1) * 16}px; padding-right: 8px;"
													ondragstart={(e) => handleFolderDragStart(e, row.path)}
													ondragend={handleDragEnd}
													ondragover={(e) => handleCompactFolderDragOver(e, ri, row.path)}
													ondragleave={(e) => {
														if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) {
															compactFolderDrop = null;
															onFolderDragExit(row.path);
														}
													}}
													ondrop={(e) => handleCompactFolderDrop(e, ri, row.path)}
												>
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
												</div>
											</div>
										</ContextMenu.Trigger>
										<FolderContext path={row.path} folderPaths={allFolderPaths} sortField={sortField} sortDir={sortDir}  onCreatePlaylist={openCreatePlaylistIn} onCreateFolder={openCreateFolderIn} />
									</ContextMenu.Root>
		
								{:else}
									{@const pi = getDirectPlaylists(playlists, row.folderPath, sortField, sortDir).findIndex((p) => p.uid === row.playlist.uid)}
									<PlaylistCompactRow
										playlist={row.playlist}
										indent={(row.depth + 1) * 16}
										isDraggingThis={draggingPlaylistUid === row.playlist.uid}
										isDropBefore={compactDrop?.folderPath === row.folderPath && compactDrop.index === pi && compactDrop.side === "before"}
										isDropAfter={compactDrop?.folderPath === row.folderPath && compactDrop.index === pi && compactDrop.side === "after"}
										{allFolderPaths}
										{draggingPlaylistUid}
										ondragstart={(e) => handlePlaylistDragStart(e, row.playlist.uid)}
										ondragend={handleDragEnd}
										ondragover={(e) => handleCompactPlaylistDragOver(e, pi, row.folderPath)}
										ondragleave={() => { compactDrop = null; }}
										ondrop={(e) => handleCompactPlaylistDrop(e, pi, row.folderPath)}
										onrowdragover={(e) => { e.preventDefault(); setHoveredPlaylist(row.playlist.uid); }}
										onrowdragleave={() => setHoveredPlaylist(null)}
										onrowdrop={(e) => dropOnPlaylist(e, row.playlist.uid)}
									/>
								{/if}
							{/each}
		
							{#each filteredDirect as p, pi (p.uid)}
								<PlaylistCompactRow
									playlist={p}
									indent={8}
									isDraggingThis={draggingPlaylistUid === p.uid}
									isDropBefore={compactDrop?.folderPath === currentPath && compactDrop.index === pi && compactDrop.side === "before"}
									isDropAfter={compactDrop?.folderPath === currentPath && compactDrop.index === pi && compactDrop.side === "after"}
									{allFolderPaths}
									{draggingPlaylistUid}
									ondragstart={(e) => handlePlaylistDragStart(e, p.uid)}
									ondragend={handleDragEnd}
									ondragover={(e) => handleCompactPlaylistDragOver(e, pi, currentPath)}
									ondragleave={() => { compactDrop = null; }}
									ondrop={(e) => handleCompactPlaylistDrop(e, pi, currentPath)}
									onrowdragover={(e) => { e.preventDefault(); setHoveredPlaylist(p.uid); }}
									onrowdragleave={() => setHoveredPlaylist(null)}
									onrowdrop={(e) => dropOnPlaylist(e, p.uid)}
								/>
							{/each}
		
						</div>
					{/if}
		
				</ScrollArea>
			</div>
		</div>
	</ContextMenu.Trigger>
	<PlaylistViewContext	path={currentPath} onCreatePlaylist={openCreatePlaylistIn} onCreateFolder={openCreateFolderIn} />
</ContextMenu.Root>
