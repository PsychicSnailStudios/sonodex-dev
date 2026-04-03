<script lang="ts">

	// APP
	import { onMount } from "svelte";
	
	// COMPONENTS
	import { User } from "lucide-svelte";

	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	
	// SCRIPTS
	import { profileState, loadProfiles, updateProfile, getProfileAvatar, } from "$lib/ts/profiles.svelte";

	// VARIABLES
	let { open = $bindable(true) } = $props<{ open: boolean }>();
	let saving = $state(false);

	let editName = $state("");
	let editAvatarBytes = $state<number[] | null>(null);
	let editAvatarPreview = $state<string | null>(null);

	let avatarUrls = $state<Record<string, string>>({});

	// APP FUNCTIONS
	onMount(async () => {
		
		await loadProfiles();
		loadAvatars();

		editName = profileState.active?.name ?? "";
		editAvatarBytes = null;
		editAvatarPreview = avatarUrls[profileState.active?.uid ?? ""] ?? null;
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

	function handleAvatarFileChange(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		editAvatarPreview = URL.createObjectURL(file);

		const reader = new FileReader();
		reader.onload = () => {
			const arrayBuffer = reader.result as ArrayBuffer;
			editAvatarBytes = Array.from(new Uint8Array(arrayBuffer));
		};
		reader.readAsArrayBuffer(file);
	}

	async function handleSaveEdit() {
		if (!profileState.active || !editName.trim()) return;
		saving = true;
		try {
			await updateProfile(profileState.active.uid, editName.trim(), editAvatarBytes);
			await loadProfiles();
			await loadAvatars();
			open = false;
		} finally {
			saving = false;
		}
	}
</script>

<Dialog.Root bind:open={open}>
	<Dialog.Content class="max-w-sm w-full">
		<Dialog.Header>
			<Dialog.Title>Edit Profile</Dialog.Title>
		</Dialog.Header>

		<div class="space-y-4 py-2">
			<div class="flex flex-col items-center gap-3">
				<div class="w-20 h-20 rounded-full bg-muted overflow-hidden flex items-center justify-center flex-shrink-0">
					{#if editAvatarPreview}
						<img src={editAvatarPreview} alt="" class="w-full h-full object-cover" />
					{:else}
						<User class="w-10 h-10 text-muted-foreground" />
					{/if}
				</div>
				<Label
					class="cursor-pointer text-sm text-muted-foreground hover:text-foreground transition-colors"
					for="avatar-upload"
				>
					Change photo
				</Label>
				<input
					id="avatar-upload"
					type="file"
					accept="image/*"
					class="hidden"
					onchange={handleAvatarFileChange}
				/>
			</div>

			<div class="space-y-1.5">
				<Label for="edit-profile-name">Name</Label>
				<Input
					id="edit-profile-name"
					bind:value={editName}
					placeholder="Enter a name"
					onkeydown={(e) => { if (e.key === "Enter") handleSaveEdit(); }}
				/>
			</div>
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => open = false}>Cancel</Button>
			<Button onclick={handleSaveEdit} disabled={saving || !editName.trim()}>
				{saving ? "Saving…" : "Save"}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>