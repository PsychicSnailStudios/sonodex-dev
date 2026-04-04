<script lang="ts">

	// COMPONENTS
	import { ArrowDownUp, ChevronUp, ChevronDown } from "lucide-svelte"

	import * as DropdownMenu from "$lib/components/ui/dropdown-menu"
	import { Button } from "$lib/components/ui/button"
	
	// SCRIPTS
	import type { SortState, SortField } from "$lib/ts/app/sortConfig.svelte"

	// PROPS
	let { sort } = $props<{ sort: SortState }>()

	// VARIABLES
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
	
	// FUNCTIONS
	function toggleDirection() {
		if (!sort.field) return
		sort.set(sort.field, sort.direction === "asc" ? "desc" : "asc")
	}
</script>

<div class="flex items-center">
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			<Button variant="ghost" size="sm" class="gap-2">
				<!-- <ArrowDownUp size={14} /> -->
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
		<Button variant="ghost" size="sm" onclick={toggleDirection} class="px-2">
			{#if sort.direction === "asc"}
				<ChevronUp size={14} />
			{:else}
				<ChevronDown size={14} />
			{/if}
		</Button>
	{/if}
</div>