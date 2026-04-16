<script lang="ts">
	// APP
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";

	// COMPONENTS
	import * as Resizable from "$lib/components/ui/resizable/index.js";

	// CUSTOM COMPONENTS
	import PlayControls from "$lib/components/app/PlayControls.svelte";
	import NowPlaying from "$lib/components/app/now-playing/NowPlaying.svelte";
   import AppNavigation from "$lib/components/app/AppNavigation.svelte";

	import HomeView from "$lib/components/views/Profile.svelte";
	import PlaylistsView from "$lib/components/views/PlaylistsLibrary.svelte";
	import MusicView from "$lib/components/views/MusicLibrary.svelte";
	import TracksView from "$lib/components/views/TrackLibrary.svelte";
	import AlbumsView from "$lib/components/views/AlbumLibrary.svelte";
	import ArtistsView from "$lib/components/views/ArtistLibrary.svelte";
	import Settings from "$lib/components/views/Settings.svelte";
	import LibraryManager from "$lib/components/views/LibraryManager.svelte";

	import AlbumDisplay from "$lib/components/displays/AlbumDisplay.svelte";
	import ArtistDisplay from "$lib/components/displays/ArtistDisplay.svelte";
	import PlaylistDisplay from "$lib/components/displays/PlaylistDisplay.svelte";
	import TrackDisplay from "$lib/components/displays/TrackDisplay.svelte";

	import ProfileSetup from "$lib/components/dialogs/profile/ProfileSetup.svelte";
   import UpdateDialog from "$lib/components/dialogs/UpdateDialog.svelte";

	// SCRIPTS
	import { loadLibrary } from "$lib/ts/library.svelte";
	import { selection, scanState, activeView, setView, loadSessionState, saveSessionState } from "$lib/ts/app-states/state_session.svelte";
	import { dragState } from "$lib/ts/app-states/state_drag.svelte";
	import { togglePlay, skipBack, skipNext, loadPlayerState, savePlayerState } from "$lib/ts/audio/audioManager.svelte";
	import { checkForUpdate } from "$lib/updater.svelte";

	// VARIABLES
	let needsSetup = $state(false);
	let setupChecked = $state(false);

	let update = $state<import("@tauri-apps/plugin-updater").Update | null>(null);
	let showUpdateDialog = $state(false);

	let containerWidth = $state(0);
	let minViewWidth = $derived(containerWidth ? (375 / containerWidth) * 100 : 15);
	let minSidebarWidth = $derived(containerWidth ? (230 / containerWidth) * 100 : 15);
	let maxSidebarWidth = $derived(containerWidth ? (480 / containerWidth) * 100 : 40);
	let defaultSidebarWidth = $derived(containerWidth ? (300 / containerWidth) * 100 : 40);

	// APP FUNCTIONS
	onMount(() => {
		loadPlayerState();
		loadSessionState();

		window.addEventListener("beforeunload", savePlayerState);
		window.addEventListener("beforeunload", saveSessionState);
		return () => {
			window.removeEventListener("beforeunload", savePlayerState);
			window.removeEventListener("beforeunload", saveSessionState);
		}
	});

	onMount(async () => {
		needsSetup = await invoke<boolean>("needs_profile_setup");
		setupChecked = true;

		const found = await checkForUpdate(true);
		if (found) {
			update = found;
			showUpdateDialog = true;
		}

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
			keydown(e);
		});
	});

	// FUNCTIONS
	function onSetupComplete() {
		needsSetup = false;
		loadLibrary();
	}

	function keydown(e: KeyboardEvent) {
		switch (e.key) {
			case 'F5':
				location.reload();
				break;
			case 'MediaPlayPause':
				togglePlay();
				break;
			case 'MediaTrackNext':
				skipNext();
				break;
			case 'MediaTrackPrevious':
				skipBack();
				break;
		}
	}

	function handleWindowDragOver(e: DragEvent) {
		if (dragState.active) e.preventDefault();
	}
</script>

<svelte:window ondragover={handleWindowDragOver} />

{#if setupChecked && needsSetup}
	<ProfileSetup onComplete={onSetupComplete} />
{/if}

<UpdateDialog bind:open={showUpdateDialog} {update} />

<div bind:clientWidth={containerWidth} class="h-full w-full overflow-hidden">
<Resizable.PaneGroup direction="horizontal" class="app-wrapper grid gap-0.5 pl-2 pr-2 pb-2 overflow-hidden">

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
			</div>

			<AppNavigation />

			<NowPlaying />
		</div>
	</Resizable.Pane>

	<Resizable.Handle class="opacity-0" />

	<Resizable.Pane class="app-body h-full w-full overflow-hidden grid gap-1">
		<div class="app-body h-full w-full overflow-hidden grid gap-1">
			<Resizable.PaneGroup direction="horizontal" class="app-views h-full w-full overflow-hidden flex gap-0.5">
				<Resizable.Pane minSize={minViewWidth}>
					{#if activeView.id === "home"}
						<HomeView />
					{:else if activeView.id === "music"}
						<MusicView />
					{:else if activeView.id === "tracks"}
						<TracksView />
					{:else if activeView.id === "albums"}
						<AlbumsView />
					{:else if activeView.id === "artists"}
						<ArtistsView />
					{:else if activeView.id === "playlists"}
						<PlaylistsView />
					{:else if activeView.id === "settings"}
						<Settings />
					{:else if activeView.id === "manage"}
						<LibraryManager />
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