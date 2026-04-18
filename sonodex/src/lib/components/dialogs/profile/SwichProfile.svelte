<script lang="ts">

	// APP
	import { onMount } from "svelte";
	
	// COMPONENTS
	import { User } from "lucide-svelte";

	import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	
	// SCRIPTS
	import { profileState, loadProfiles, switchProfile, deleteProfile, getProfileAvatar, } from "$lib/ts/profiles.svelte";
	import { saveSessionState } from "$lib/ts/app-states/state_session.svelte";
	import { savePlayerState } from "$lib/ts/audio/audioManager.svelte";
	
	// VARIABLES
	let { open = $bindable(true), openAdd = $bindable(true) } = $props<{ open: boolean, openAdd: boolean }>();

	let avatarUrls = $state<Record<string, string>>({});

	// APP FUNCTIONS
	onMount(async () => {
		loadAvatars();
	});

	// FUNCTIONS
	async function loadAvatars() {
		for (const p of profileState.all) {
			const bytes = await getProfileAvatar(p.uid);
			if (bytes) {
				const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
				avatarUrls[p.uid] = URL.createObjectURL(blob);
			}
		}
	}

	async function handleSwitch(uid: string) {
		if (uid === profileState.active?.uid) return;
		const activeUid = profileState.active?.uid;
		if (activeUid) {
			saveSessionState(activeUid);
			savePlayerState();
		}
		await switchProfile(uid);
	}

	async function handleDelete(uid: string) {
		await deleteProfile(uid);
		await loadAvatars();
	}

</script>

<Dialog.Root bind:open={open}>
	<Dialog.Content class="max-w-sm w-full">
		<Dialog.Header>
			<Dialog.Title>Switch Profile</Dialog.Title>
		</Dialog.Header>
		
		<div class="flex flex-col gap-3 h-[350px]">
			<ScrollArea class="min-h-0 min-w-0 h-full">
				<div class="flex flex-col gap-2 pr-4">
					{#each profileState.all as profile}
						<div class="flex items-center gap-3 p-2 rounded-md border {profile.uid === profileState.active?.uid ? 'border-primary bg-muted' : 'hover:bg-muted/50'}">

							<div class="w-10 h-10 rounded-full bg-muted overflow-hidden flex items-center justify-center flex-shrink-0">
								{#if avatarUrls[profile.uid]}
									<img src={avatarUrls[profile.uid]} alt="" class="w-full h-full object-cover" />
								{:else}
									<User class="w-5 h-5 text-muted-foreground" />
								{/if}
							</div>

							<div class="flex-1 min-w-0">
								<p class="text-sm font-medium truncate">{profile.name}</p>
								{#if profile.uid === profileState.active?.uid}
									<p class="text-xs text-muted-foreground">Active</p>
								{/if}
							</div>

							<div class="flex gap-1 flex-shrink-0">
								{#if profile.uid !== profileState.active?.uid}
									<Button variant="outline" size="sm" onclick={() => handleSwitch(profile.uid)}>
										Switch
									</Button>
									<AlertDialog.Root>
										<AlertDialog.Trigger>
											<Button variant="ghost" size="sm" class="text-destructive hover:text-destructive">
												Delete
											</Button>
										</AlertDialog.Trigger>
										<AlertDialog.Content>
											<AlertDialog.Header>
												<AlertDialog.Title>Delete profile?</AlertDialog.Title>
												<AlertDialog.Description>
													This will permanently delete {profile.name}'s profile and all their library data.
												</AlertDialog.Description>
											</AlertDialog.Header>
											<AlertDialog.Footer>
												<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
												<AlertDialog.Action onclick={() => handleDelete(profile.uid)}>Delete</AlertDialog.Action>
											</AlertDialog.Footer>
										</AlertDialog.Content>
									</AlertDialog.Root>
								{/if}
							</div>

						</div>
					{/each}
				</div>
			</ScrollArea>
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => openAdd = true}>Add Profile</Button>
			<Button variant="outline" onclick={() => open = false}>Close</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>