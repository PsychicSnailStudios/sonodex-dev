<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Input } from "$lib/components/ui/input";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

	let { uid, rating }: { uid: string; rating: number | null } = $props();

	let value = $state(rating != null ? String(Math.round(rating * 10) / 10) : "—");
	let saving = $state(false);

	async function save() {
		const parsed = parseFloat(value);
		const clamped = isNaN(parsed) ? null : Math.min(10, Math.max(0, Math.round(parsed * 10) / 10));

		if (clamped !== null) {
			value = String(clamped);
		}

		saving = true;
		try {
			await invoke("update_track_metadata", {
				uid,
				update: { rating: clamped },
			});
		} finally {
			saving = false;
		}
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === "Enter") {
			(e.currentTarget as HTMLInputElement).blur();
		}
	}
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
	<span class="text-sm font-mono">{value}</span>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content>
    <DropdownMenu.Group>
		<Input
			type="number"
			min="0"
			max="10"
			step="0.1"
			bind:value
			onblur={save}
			{onkeydown}
			disabled={saving}
			class="h-7 text-sm text-center"
			placeholder="1-10"
		/>
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>

