<script lang="ts">

	// APP
	import { onMount } from "svelte";
	
	// COMPONENTS
	import { Pencil, User, Settings } from "lucide-svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	
	// CUSTOM COMPONENTS
	import TopTracks from "$lib/components/app-ui/TopTracks.svelte";
	import AddProfile from "$lib/components/dialogs/profile/AddProfile.svelte";
	import EditProfile from "$lib/components/dialogs/profile/EditProfile.svelte";
	import SwichProfile from "$lib/components/dialogs/profile/SwichProfile.svelte";
	
	// SCRIPTS
	import { profileState, loadProfiles, getProfileAvatar, } from "$lib/ts/profiles.svelte";
   import { setView } from "$lib/ts/app-states/state_session.svelte";

	// VARIABLES
	let swichOpen = $state(false);
	let editOpen = $state(false);
	let newOpen = $state(false);

	let avatarUrls = $state<Record<string, string>>({});

	// APP FUNCTIONS
	onMount(async () => {
		// await loadProfiles();
		loadAvatars();
	});

	async function loadAvatars() {
		for (const p of profileState.all) {
			const bytes = await getProfileAvatar(p.uid);
			if (bytes) {
				const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
				avatarUrls[p.uid] = URL.createObjectURL(blob);
			}
		}
	}

</script>

<EditProfile bind:open={editOpen} />
<SwichProfile bind:open={swichOpen} bind:openAdd={newOpen} />
<AddProfile bind:open={newOpen} />

<div class="flex flex-col gap-4 p-2 border-2 rounded-md h-full w-full overflow-hidden">

	<div class="flex items-center justify-between">
		<div class="flex gap-2 items-center">
			<div class="w-8 h-8 rounded-full bg-muted overflow-hidden flex items-center justify-center flex-shrink-0">
				{#if avatarUrls[profileState.active?.uid]}
					<img src={avatarUrls[profileState.active?.uid]} alt="" class="w-full h-full object-cover" />
				{:else}
					<User class="w-5 h-5 text-muted-foreground" />
				{/if}
			</div>
			<h1 class="h1 text-2xl text-primary">{profileState.active?.name}</h1>
		</div>

		<div class="flex items-center gap-2">
			<Button variant="outline" size="sm" onclick={() => swichOpen = true}>
				Swich Profile
			</Button>
			<Button variant="ghost" size="sm" onclick={() => editOpen = true}>
				<Pencil />
			</Button>
			<Button variant="ghost" size="sm" onclick={() => setView("settings")}>
				<Settings />
			</Button>
		</div>
	</div>

	<ScrollArea class="min-h-0 min-w-0">
		<TopTracks />
	</ScrollArea>

	<!-- <Tabs.Root value="statistics" class="flex flex-col min-h-0 flex-1">
		<Tabs.List class="w-full">
			<Tabs.Trigger value="statistics" class="flex-1">Stats</Tabs.Trigger>
			<Tabs.Trigger value="feed" class="flex-1">Feed</Tabs.Trigger>
			<Tabs.Trigger value="manager" class="flex-1">Library</Tabs.Trigger>
			<Tabs.Trigger value="settings" class="flex-1">Settings</Tabs.Trigger>
		</Tabs.List>
	</Tabs.Root> -->

</div>