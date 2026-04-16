<script lang="ts">
	import { Pencil, Trash, CloudDownload } from "lucide-svelte";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import { Checkbox } from "$lib/components/ui/checkbox/index.js";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { removeArtist, enrichArtist } from "$lib/ts/app/libraryManager";
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { library } from "$lib/ts/library.svelte";
	import type { Artist } from "$lib/ts/util/types";

	let {
		artist,
		selected = false,
		onToggle,
		onShiftClick,
	}: {
		artist: Artist;
		selected?: boolean;
		onToggle?: () => void;
		onShiftClick?: () => void;
	} = $props();

	const trackCount = $derived(
		library.tracks.filter((t) => {
			try {
				const arr: string[] = t.artists ? JSON.parse(t.artists as string) : [];
				return arr.includes(artist.name);
			} catch { return false; }
		}).length
	);

	const genres = $derived.by<string[]>(() => {
		try {
			return artist.genres ? JSON.parse(artist.genres as string) : [];
		} catch { return []; }
	});

	function handleRowClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (target.closest("a, button")) return;
		if (e.shiftKey) {
			onShiftClick?.();
		} else {
			onToggle?.();
		}
	}
</script>

<div class="flex gap-2 items-center">
	<Checkbox
		checked={selected}
		onCheckedChange={(v) => {
			if (v !== selected) onToggle?.();
		}}
	/>
	<div
		class="flex gap-2 p-2 border-2 rounded-md justify-between items-center flex-1 cursor-pointer"
		class:border-primary={selected}
		onclick={handleRowClick}
		onmousedown={(e) => { if (e.shiftKey) e.preventDefault(); }}
		role="row"
		aria-selected={selected}
	>
		<div class="flex gap-2 items-center min-w-0 flex-1">
			<ArtworkDisplay uid={artist.uid} size={40} type="artist" />
			<div class="min-w-0 grid">
				<button
					onclick={(e) => { e.stopPropagation(); setSelection(artist.uid, "artist"); }}
					class="text-sm truncate cursor-pointer hover:underline text-left"
				>
					{artist.name}
				</button>
			</div>
		</div>

		{#if !selected}
			<div class="flex gap-2 shrink-0">
				<Tooltip.Root>
					<Tooltip.Trigger
						class={buttonVariants({ variant: "destructive", size: "icon" })}
						onclick={(e: MouseEvent) => { e.stopPropagation(); removeArtist(artist.uid); }}
					>
						<Trash />
					</Tooltip.Trigger>
					<Tooltip.Content><p>Delete artist record</p></Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger
						class={buttonVariants({ variant: "outline", size: "icon" })}
						onclick={(e: MouseEvent) => { e.stopPropagation(); enrichArtist(artist.uid); }}
					>
						<CloudDownload />
					</Tooltip.Trigger>
					<Tooltip.Content><p>Fetch metadata via API</p></Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger
						class={buttonVariants({ variant: "outline", size: "icon" })}
						onclick={(e: MouseEvent) => { e.stopPropagation(); openEditModal({ type: "artist", uid: artist.uid }); }}
					>
						<Pencil />
					</Tooltip.Trigger>
					<Tooltip.Content><p>Edit artist</p></Tooltip.Content>
				</Tooltip.Root>
			</div>
		{/if}
	</div>
</div>