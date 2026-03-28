<script lang="ts">
	import { library } from "$lib/library.svelte";
	import { dragState, setHoveredPlaylist, endDrag } from "$lib/ts/app/dragState.svelte";
	import { addTracksToPlaylist, createPlaylist } from "$lib/playlistManager.svelte";
	import { profileState } from "$lib/profiles.svelte";
	import { invoke } from "@tauri-apps/api/core";
	import type { Playlist } from "$lib/types";

	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
	import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
	import AudioCard from "$lib/components/app/AudioCard.svelte";
	import {
		FolderPlus, ListPlus, FileDown,
		LayoutGrid, List,
		ChevronRight, ChevronDown,
		Folder, FolderOpen,
		House
	} from "lucide-svelte";

	type ViewMode = "tiled" | "compact";

	let search       = $state("");
	let viewMode     = $state<ViewMode>("tiled");
	let currentPath  = $state<string | null>(null);

	let createDialogOpen = $state(false);
	let folderDialogOpen = $state(false);
	let nameInput        = $state("");
	let folderNameInput  = $state("");

	let expandedFolders = $state<Set<string>>(new Set());

	let playlists = $derived(library.playlists ?? []);

	function getFolder(p: Playlist): string | null {
		return (p as any).folder ?? null;
	}

	function isInFolder(p: Playlist, folder: string | null): boolean {
		const pf = getFolder(p);
		if (folder === null) return pf === null || pf === "";
		return pf === folder || (pf?.startsWith(folder + "/") ?? false);
	}

	function immediateChildFolder(parentPath: string | null, folderPath: string): string | null {
		if (parentPath === null) {
			const parts = folderPath.split("/");
			return parts[0];
		}
		if (!folderPath.startsWith(parentPath + "/")) return null;
		const rest = folderPath.slice(parentPath.length + 1);
		return rest.split("/")[0];
	}

	function getChildFolders(parentPath: string | null): string[] {
		const seen = new Set<string>();
		for (const p of playlists) {
			const pf = getFolder(p);
			if (!pf) continue;
			const child = immediateChildFolder(parentPath, pf);
			if (child) {
				const fullPath = parentPath === null ? child : `${parentPath}/${child}`;
				seen.add(fullPath);
			}
		}
		return [...seen].sort();
	}

	function getDirectPlaylists(folderPath: string | null): Playlist[] {
		return playlists.filter((p) => {
			const pf = getFolder(p);
			if (folderPath === null) return pf === null || pf === "";
			return pf === folderPath;
		});
	}

	function getFolderArtworkUids(folderPath: string): string[] {
		const inside = playlists.filter((p) => isInFolder(p, folderPath));
		return inside.map((p) => p.uid).slice(0, 4);
	}

	function breadcrumbs(): { label: string; path: string | null }[] {
		const crumbs: { label: string; path: string | null }[] = [{ label: "Playlists", path: null }];
		if (!currentPath) return crumbs;
		const parts = currentPath.split("/");
		let acc = "";
		for (const part of parts) {
			acc = acc ? `${acc}/${part}` : part;
			crumbs.push({ label: part, path: acc });
		}
		return crumbs;
	}

	const filteredDirect = $derived(
		search.trim() === ""
			? getDirectPlaylists(currentPath)
			: playlists.filter((p) => {
				const q = search.toLowerCase();
				return p.title.toLowerCase().includes(q) || ((p as any).description?.toLowerCase() ?? "").includes(q);
			})
	);

	const visibleChildFolders = $derived(
		search.trim() === "" ? getChildFolders(currentPath) : []
	);

	async function handleCreatePlaylist() {
		if (!nameInput.trim()) return;
		const ownerName = profileState.active?.name ?? null;
		await createPlaylist(nameInput.trim(), ownerName, currentPath ?? undefined);
		nameInput = "";
		createDialogOpen = false;
	}

	async function handleCreateFolder() {
		if (!folderNameInput.trim()) return;
		const newPath = currentPath
			? `${currentPath}/${folderNameInput.trim()}`
			: folderNameInput.trim();
		const ownerName = profileState.active?.name ?? null;
		await createPlaylist("New Playlist", ownerName, newPath);
		folderNameInput = "";
		folderDialogOpen = false;
	}

	function toggleExpand(folderPath: string) {
		const next = new Set(expandedFolders);
		if (next.has(folderPath)) {
			next.delete(folderPath);
		} else {
			next.add(folderPath);
		}
		expandedFolders = next;
	}

	function navigateInto(folderPath: string) {
		currentPath = folderPath;
		search = "";
	}

	function navigateTo(path: string | null) {
		currentPath = path;
		search = "";
	}

	function folderLabel(fullPath: string): string {
		return fullPath.split("/").at(-1) ?? fullPath;
	}

	function handleCardDragLeave(e: DragEvent, playlistUid: string) {
		if (dragState.hoveredPlaylistUid === playlistUid) setHoveredPlaylist(null);
	}

	function handleCardDragOver(e: DragEvent, playlistUid: string) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
		setHoveredPlaylist(playlistUid);
	}

	async function handleCardDrop(e: DragEvent, playlistUid: string) {
		e.preventDefault();
		setHoveredPlaylist(null);
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) return;
		const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
		await addTracksToPlaylist(playlistUid, uids);
		endDrag();
	}

	function compactFolderRows(
		parentPath: string | null,
		depth: number = 0
	): { type: "folder"; path: string; depth: number }[] {
		const result: { type: "folder"; path: string; depth: number }[] = [];
		for (const fp of getChildFolders(parentPath)) {
			result.push({ type: "folder", path: fp, depth });
			if (expandedFolders.has(fp)) {
				result.push(...compactFolderRows(fp, depth + 1));
			}
		}
		return result;
	}
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Playlists</h1>
		<div class="flex items-center gap-2">
			<Input placeholder="Search..." bind:value={search} class="w-48" />
			<Button
				variant="outline"
				size="icon"
				onclick={() => viewMode = viewMode === "tiled" ? "compact" : "tiled"}
			>
				{#if viewMode === "tiled"}
					<List class="size-4" />
				{:else}
					<LayoutGrid class="size-4" />
				{/if}
			</Button>
		</div>
	</div>

	{#if breadcrumbs().length > 1}
		<div class="flex items-center gap-1 text-sm text-muted-foreground flex-wrap">
			{#each breadcrumbs() as crumb, i}
				{#if i > 0}
					<ChevronRight class="size-3 shrink-0" />
				{/if}
				<button
					class="hover:text-foreground transition-colors"
					class:text-foreground={i === breadcrumbs().length - 1}
					onclick={() => navigateTo(crumb.path)}
				>
					{#if i === 0}
						<House class="size-3 inline mr-1" />
					{/if}
					{crumb.label}
				</button>
			{/each}
		</div>
	{/if}

	<div class="flex flex-col h-full w-full overflow-hidden">
		<div class="flex justify-between items-center">
			<span class="text-sm text-muted-foreground">{playlists.length} playlists</span>
			<div class="flex gap-1">
				<AlertDialog.Root bind:open={createDialogOpen}>
					<AlertDialog.Trigger class={buttonVariants({ variant: "outline", size: "icon" })}>
						<ListPlus class="size-4" />
					</AlertDialog.Trigger>
					<AlertDialog.Content>
						<AlertDialog.Header>
							<AlertDialog.Title>New Playlist</AlertDialog.Title>
							<AlertDialog.Description>
								<Input placeholder="Playlist name" bind:value={nameInput} class="w-full mt-2" />
							</AlertDialog.Description>
						</AlertDialog.Header>
						<AlertDialog.Footer>
							<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
							<AlertDialog.Action onclick={handleCreatePlaylist}>Create</AlertDialog.Action>
						</AlertDialog.Footer>
					</AlertDialog.Content>
				</AlertDialog.Root>

				<AlertDialog.Root bind:open={folderDialogOpen}>
					<AlertDialog.Trigger class={buttonVariants({ variant: "outline", size: "icon" })}>
						<FolderPlus class="size-4" />
					</AlertDialog.Trigger>
					<AlertDialog.Content>
						<AlertDialog.Header>
							<AlertDialog.Title>New Folder</AlertDialog.Title>
							<AlertDialog.Description>
								<Input placeholder="Folder name" bind:value={folderNameInput} class="w-full mt-2" />
							</AlertDialog.Description>
						</AlertDialog.Header>
						<AlertDialog.Footer>
							<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
							<AlertDialog.Action onclick={handleCreateFolder}>Create</AlertDialog.Action>
						</AlertDialog.Footer>
					</AlertDialog.Content>
				</AlertDialog.Root>

				<Button variant="outline" size="icon">
					<FileDown class="size-4" />
				</Button>
			</div>
		</div>

		<ScrollArea class="min-h-0 min-w-0 mt-2">

			{#if viewMode === "tiled"}
				<div class="app-music-grid grid gap-2">

					{#each visibleChildFolders as folderPath (folderPath)}
						{@const artUids = getFolderArtworkUids(folderPath)}
						<button
							class="flex flex-col items-center gap-1 w-full text-left cursor-pointer"
							onclick={() => navigateInto(folderPath)}
						>
							<div class="w-full aspect-square rounded-md bg-muted flex items-center justify-center overflow-hidden">
								{#if artUids.length >= 4}
									<div class="grid grid-cols-2 w-full h-full">
										{#each artUids.slice(0, 4) as uid}
											<AudioCard artworkUid={uid} type="playlist" title={folderLabel(folderPath)} subTitle="" />
										{/each}
									</div>
								{:else if artUids.length > 0}
									<div class="grid grid-cols-2 w-full h-full">
										{#each artUids as uid}
											<AudioCard artworkUid={uid} type="playlist" title={folderLabel(folderPath)} subTitle="" />
										{/each}
										{#each Array(4 - artUids.length) as _}
											<div class="bg-muted-foreground/10 flex items-center justify-center">
												<Folder class="size-6 text-muted-foreground/40" />
											</div>
										{/each}
									</div>
								{:else}
									<Folder class="size-12 text-muted-foreground/40" />
								{/if}
							</div>
							<div class="w-full px-1">
								<p class="text-sm font-medium truncate">{folderLabel(folderPath)}</p>
								<p class="text-xs text-muted-foreground truncate">Folder</p>
							</div>
						</button>
					{/each}

					{#each filteredDirect as p (p.uid)}
						<div
							class="relative transition-all w-full"
							class:ring-2={dragState.hoveredPlaylistUid === p.uid}
							class:ring-primary={dragState.hoveredPlaylistUid === p.uid}
							class:rounded-md={dragState.hoveredPlaylistUid === p.uid}
							ondragover={(e) => handleCardDragOver(e, p.uid)}
							ondragleave={(e) => handleCardDragLeave(e, p.uid)}
							ondrop={(e) => handleCardDrop(e, p.uid)}
							role="region"
							aria-label="Playlist drop target"
						>
							<AudioCard title={p.title} subTitle={(p as any).owner} artworkUid={p.uid} type="playlist" />
						</div>
					{/each}

				</div>

			{:else}
				<div class="flex flex-col">

					{#each compactFolderRows(currentPath) as row (row.path)}
						{@const isExpanded = expandedFolders.has(row.path)}
						{@const label = folderLabel(row.path)}
						<div
							class="flex items-center gap-1 py-1.5 px-2 rounded-md hover:bg-muted/50 group"
							style="padding-left: {(row.depth + 1) * 16}px"
						>
							<button
								class="shrink-0 text-muted-foreground hover:text-foreground transition-colors"
								onclick={() => toggleExpand(row.path)}
								aria-label={isExpanded ? "Collapse folder" : "Expand folder"}
							>
								{#if isExpanded}
									<ChevronDown class="size-4" />
								{:else}
									<ChevronRight class="size-4" />
								{/if}
							</button>

							<button
								class="flex items-center gap-2 flex-1 min-w-0 text-left"
								onclick={() => navigateInto(row.path)}
							>
								{#if isExpanded}
									<FolderOpen class="size-4 shrink-0 text-muted-foreground" />
								{:else}
									<Folder class="size-4 shrink-0 text-muted-foreground" />
								{/if}
								<span class="text-sm truncate">{label}</span>
							</button>
						</div>

						{#if isExpanded}
							{#each getDirectPlaylists(row.path) as p (p.uid)}
								<div
									class="flex items-center gap-2 py-1.5 rounded-md hover:bg-muted/50"
									style="padding-left: {(row.depth + 2) * 16}px"
									ondragover={(e) => handleCardDragOver(e, p.uid)}
									ondragleave={(e) => handleCardDragLeave(e, p.uid)}
									ondrop={(e) => handleCardDrop(e, p.uid)}
									role="region"
									aria-label="Playlist drop target"
								>
									<div class="size-6 rounded shrink-0 overflow-hidden bg-muted">
										<AudioCard title={p.title} subTitle={p.owner} artworkUid={p.uid} type="playlist" />
									</div>
									<span class="text-sm truncate flex-1">{p.title}</span>
									{#if (p as any).owner}
										<span class="text-xs text-muted-foreground mr-2 shrink-0">{(p as any).owner}</span>
									{/if}
								</div>
							{/each}
						{/if}
					{/each}

					{#each filteredDirect as p (p.uid)}
						<div
							class="flex items-center gap-2 py-1.5 px-2 rounded-md hover:bg-muted/50"
							class:ring-1={dragState.hoveredPlaylistUid === p.uid}
							class:ring-primary={dragState.hoveredPlaylistUid === p.uid}
							ondragover={(e) => handleCardDragOver(e, p.uid)}
							ondragleave={(e) => handleCardDragLeave(e, p.uid)}
							ondrop={(e) => handleCardDrop(e, p.uid)}
							role="region"
							aria-label="Playlist drop target"
						>
							<div class="size-8 rounded shrink-0 overflow-hidden bg-muted">
								<AudioCard title={p.title} subTitle={p.owner} artworkUid={p.uid} type="playlist" />
							</div>
							<span class="text-sm truncate flex-1">{p.title}</span>
							{#if (p as any).owner}
								<span class="text-xs text-muted-foreground mr-2 shrink-0">{(p as any).owner}</span>
							{/if}
						</div>
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