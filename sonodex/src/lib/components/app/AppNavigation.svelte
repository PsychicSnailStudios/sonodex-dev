<script lang="ts">
	// COMPONENTS
	import { House, Music, ListMusic, Search, Tags, DiscAlbum, Disc, SquareUser } from "lucide-svelte";

	import { Button } from "$lib/components/ui/button/index.js";

	// SCRIPTS
	import { dragState } from "$lib/ts/app-states/state_drag.svelte";

	// PROPS
	let {activeView = $bindable("home")} = $props<{ activeView: string }>();

	// VARIABLES
	const VIEW_TABS = [
		{ value: "home", label: "Profile", icon: House },
		// { value: "search", label: "Explore", icon: Search },
		// { value: "music", label: "Music", icon: Music },
		{ value: "tracks", label: "Tracks", icon: Music },
		{ value: "albums", label: "Albums", icon: DiscAlbum },
		{ value: "artists", label: "Artists", icon: SquareUser },
		{ value: "playlists", label: "Playlists", icon: ListMusic },
	];

	let hoverTabTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	function handleTabDragEnter(tabValue: string) {
		if (!dragState.active) return;
		if (tabValue === activeView) return;

		const timeout = setTimeout(() => {
			activeView = tabValue;
			hoverTabTimeouts.delete(tabValue);
		}, 700);

		hoverTabTimeouts.set(tabValue, timeout);
	}
	
	function handleTabDragLeave(tabValue: string) {
		const timeout = hoverTabTimeouts.get(tabValue);
		if (timeout) {
			clearTimeout(timeout);
			hoverTabTimeouts.delete(tabValue);
		}
	}
</script>

<div class="app-nav bg-muted flex flex-col p-2 gap-1 rounded-md">
	{#each VIEW_TABS as tab}
		<Button
			variant="{activeView === tab.value ? 'default' : 'outline'}"
			onclick={() => activeView = tab.value}
			class="justify-start"
			ondragenter={() => handleTabDragEnter(tab.value)}
			ondragleave={() => handleTabDragLeave(tab.value)}
		>
			<svelte:component this={tab.icon} />
			<span>{tab.label}</span>
		</Button>
	{/each}
</div>