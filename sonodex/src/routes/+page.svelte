<script lang="ts">
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { loadLibrary } from "$lib/library.svelte";
	import { selection, scanState } from "$lib/session.svelte";

	import * as Resizable from "$lib/components/ui/resizable/index.js";
	import { Button } from "$lib/components/ui/button/index.js";

	import HomeView from "$lib/components/views/Profile.svelte";
	import MusicView from "$lib/components/views/MusicLibrary.svelte";
	import PlaylistsView from "$lib/components/views/Playlists.svelte";
	import PlayControls from "$lib/components/PlayControls.svelte";
	import NowPlaying from "$lib/components/NowPlaying.svelte";
	import AlbumDisplay from "$lib/components/views/AlbumDisplay.svelte";
	import ArtistDisplay from "$lib/components/views/ArtistDisplay.svelte";
	import PlaylistDisplay from "$lib/components/views/PlaylistDisplay.svelte";
	import TrackDisplay from "$lib/components/views/TrackDisplay.svelte";
	import ProfileSetup from "$lib/components/ProfileSetup.svelte";

	import { House, Music, ListMusic, Search, Tags } from "lucide-svelte";

	const VIEW_TABS = [
		{ value: "home", label: "Profile", icon: House },
		{ value: "search", label: "Explore", icon: Search },
		{ value: "music", label: "Music Library", icon: Music },
		{ value: "playlists", label: "Playlists", icon: ListMusic },
		{ value: "tags", label: "Tags", icon: Tags },
	];

	let activeView = $state("home");
	let needsSetup = $state(false);
	let setupChecked = $state(false);

	let containerWidth = $state(0);
	let minViewWidth = $derived(containerWidth ? (375 / containerWidth) * 100 : 15);
	let minSidebarWidth = $derived(containerWidth ? (230 / containerWidth) * 100 : 15);
	let maxSidebarWidth = $derived(containerWidth ? (480 / containerWidth) * 100 : 40);
	let defaultSidebarWidth = $derived(containerWidth ? (300 / containerWidth) * 100 : 40);

	onMount(async () => {
		needsSetup = await invoke<boolean>("needs_profile_setup");
		setupChecked = true;

		if (!needsSetup) {
			await loadLibrary();
		}

		await listen("library:updated", () => {
			loadLibrary();
		});

		await listen("profile:ready", async () => {
			needsSetup = false;
			await loadLibrary();
		});

		window.addEventListener("keydown", (e) => {
			if (e.key === "F5") {
				e.preventDefault();
				location.reload();
			}
		});
	});

	function onSetupComplete() {
		needsSetup = false;
		loadLibrary();
	}
</script>

{#if setupChecked && needsSetup}
	<ProfileSetup onComplete={onSetupComplete} />
{/if}

<div bind:clientWidth={containerWidth} class="h-full w-full overflow-hidden">
<Resizable.PaneGroup direction="horizontal" class="app-wrapper grid gap-0.5 p-2 overflow-hidden">

	<Resizable.Pane defaultSize={defaultSidebarWidth} minSize={minSidebarWidth} maxSize={maxSidebarWidth} class="app-sidebar grid gap-1">
		<div class="app-sidebar grid gap-1 max-[{maxSidebarWidth}px]">
			<div>
				{#if scanState.loading && scanState.total > 0}
					<div class="space-y-1 px-2 py-1">
						<div class="w-full bg-muted rounded-full h-2">
							<div
								class="bg-primary h-2 rounded-full transition-all"
								style="width: {Math.round((scanState.progress / scanState.total) * 100)}%"
							></div>
						</div>
						<p class="text-xs text-muted-foreground">{scanState.progress} / {scanState.total} files</p>
					</div>
				{/if}
				{#if scanState.enriching && scanState.enrichTotal > 0}
					<div class="space-y-1 px-2 py-1">
						<div class="w-full bg-muted rounded-full h-2">
							<div
								class="bg-primary h-2 rounded-full transition-all"
								style="width: {Math.round((scanState.enrichDone / scanState.enrichTotal) * 100)}%"
							></div>
						</div>
						<p class="text-xs text-muted-foreground">Enriching {scanState.enrichDone} / {scanState.enrichTotal}</p>
					</div>
				{/if}
				<!-- {#if scanState.status}
					<p class="text-xs text-muted-foreground px-2">{scanState.status}</p>
				{/if} -->
			</div>

			<div class="app-nav bg-muted flex flex-col p-2 gap-1 rounded-md">
				{#each VIEW_TABS as tab}
					<Button
						variant="{activeView === tab.value ? 'default' : 'outline'}"
						onclick={() => activeView = tab.value}
						class="justify-start"
					>
						<svelte:component this={tab.icon} />
						<span>{tab.label}</span>
					</Button>
				{/each}
			</div>

			<NowPlaying />
		</div>
	</Resizable.Pane>

	<Resizable.Handle class="opacity-0" />

	<Resizable.Pane class="app-body h-full w-full overflow-hidden grid gap-1">
		<div class="app-body h-full w-full overflow-hidden grid gap-1">
			<Resizable.PaneGroup direction="horizontal" class="app-views h-full w-full overflow-hidden flex gap-0.5">
				<Resizable.Pane minSize={minViewWidth}>
					{#if activeView === "home"}
						<HomeView />
					{:else if activeView === "music"}
						<MusicView />
					{:else if activeView === "playlists"}
						<PlaylistsView />
					{/if}
				</Resizable.Pane>

				{#if selection.type !== "none"}
					<Resizable.Handle class="opacity-0" />

					<Resizable.Pane defaultSize={50} minSize={minViewWidth}>
						{#if selection.type === "album"}
							<AlbumDisplay />
						{/if}
						{#if selection.type === "track"}
							<TrackDisplay />
						{/if}
						{#if selection.type === "playlist"}
							<PlaylistDisplay />
						{/if}
						{#if selection.type === "artist"}
							<ArtistDisplay />
						{/if}
					</Resizable.Pane>
				{/if}
			</Resizable.PaneGroup>

			<PlayControls />
		</div>
	</Resizable.Pane>

</Resizable.PaneGroup>
</div>

<style>
	.app-wrapper {
		grid-template-columns: minmax(18rem, 25rem) minmax(50%, 85%);
		height: 100%;
	}

	.app-sidebar {
		grid-template-rows: auto 1fr auto;
		flex-direction: column;
		min-width: 10rem;
		max-width: 30rem;
	}

	.app-body {
		grid-template-rows: 1fr auto;
		flex-direction: column;
	}
</style>