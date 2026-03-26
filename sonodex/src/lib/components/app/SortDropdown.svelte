<script lang="ts">
	import type { SortState, SortField } from "$lib/sortConfig.svelte"
	import { Button } from "$lib/components/ui/button"
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu"
	import { ArrowDownUp, ChevronUp, ChevronDown } from "lucide-svelte"

	let { sort } = $props<{ sort: SortState }>()

	const fields: { value: SortField; label: string }[] = [
		{ value: "title", label: "Title" },
		{ value: "artist", label: "Artist" },
		{ value: "album", label: "Album" },
		{ value: "year", label: "Year" },
		{ value: "rating", label: "Rating" },
		{ value: "duration", label: "Duration" },
		{ value: "label", label: "Label" },
		{ value: "number", label: "Track Number" },
	]

	function toggleDirection() {
		if (!sort.field) return
		sort.set(sort.field, sort.direction === "asc" ? "desc" : "asc")
	}
</script>

<div class="flex items-center gap-1">
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			<Button variant="outline" size="sm" class="gap-2">
				<ArrowDownUp size={14} />
				{sort.field ? fields.find(f => f.value === sort.field)?.label : "Sort"}
			</Button>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content>
			{#each fields as f}
				<DropdownMenu.Item
					class={sort.active(f.value) ? "bg-accent" : ""}
					onclick={() => sort.active(f.value) ? sort.clear() : sort.set(f.value)}
				>
					{f.label}
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	{#if sort.field}
		<Button variant="outline" size="sm" onclick={toggleDirection} class="px-2">
			{#if sort.direction === "asc"}
				<ChevronUp size={14} />
			{:else}
				<ChevronDown size={14} />
			{/if}
		</Button>
	{/if}
</div>