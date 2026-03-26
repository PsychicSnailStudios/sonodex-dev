<script lang="ts">
	import { onMount } from "svelte";
	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import ScrollArea from "../ui/scroll-area/scroll-area.svelte";
	import Settings from "$lib/components/views/Settings.svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";

	import {
		profileState,
		loadProfiles,
		switchProfile,
		createProfile,
		deleteProfile,
		getProfileAvatar,
	} from "$lib/profiles.svelte";

	import { User } from "lucide-svelte";

	let addOpen = $state(false);
	let swichOpen = $state(false);
	let newName = $state("");
	let creating = $state(false);

	let avatarUrls = $state<Record<string, string>>({});

	onMount(async () => {
		await loadProfiles();
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

	async function handleCreate() {
		if (!newName.trim()) return;
		creating = true;
		try {
			await createProfile(newName.trim(), null, null);
			await loadAvatars();
			newName = "";
			addOpen = false;
		} finally {
			creating = false;
		}
	}

	async function handleSwitch(uid: string) {
		if (uid === profileState.active?.uid) return;
		await switchProfile(uid);
	}

	async function handleDelete(uid: string) {
		await deleteProfile(uid);
		await loadAvatars();
	}
</script>

<div class="flex flex-col gap-2 p-2 border-2 rounded-md h-full w-full overflow-hidden">

	<div class="flex items-center justify-start gap-2">
		<h1 class="h1 text-2xl text-primary">{profileState.active?.name}</h1>
		<Button variant="outline" size="sm" onclick={() => swichOpen = true}>
			Swich Profile
		</Button>
	</div>

	<Tabs.Root value="statistics">
		<Tabs.List class="w-full">
			<Tabs.Trigger value="statistics" class="flex-1">Stats</Tabs.Trigger>
			<Tabs.Trigger value="settings" class="flex-1">Settings</Tabs.Trigger>
		</Tabs.List>

		<Tabs.Content value="statistics" class="space-y-3 mt-4">
			<ScrollArea class="min-h-0 min-w-0">
			</ScrollArea>
		</Tabs.Content>

		<Tabs.Content value="settings" class="space-y-3 mt-4">
			<div class="h-full w-full overflow-hidden">
				<Settings />
			</div>
		</Tabs.Content>

	</Tabs.Root>

</div>

<Dialog.Root bind:open={swichOpen}>
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
			<Button variant="outline" onclick={() => addOpen = true}>Add Profile</Button>
			<Button variant="outline" onclick={() => swichOpen = false}>Close</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={addOpen}>
	<Dialog.Content class="max-w-sm w-full">
		<Dialog.Header>
			<Dialog.Title>New Profile</Dialog.Title>
		</Dialog.Header>
		<div class="space-y-3 py-2">
			<div class="space-y-1.5">
				<Label for="new-profile-name">Name</Label>
				<Input
					id="new-profile-name"
					bind:value={newName}
					placeholder="Enter a name"
					onkeydown={(e) => { if (e.key === "Enter") handleCreate(); }}
				/>
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => addOpen = false}>Cancel</Button>
			<Button onclick={handleCreate} disabled={creating || !newName.trim()}>
				{creating ? "Creating…" : "Create"}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
