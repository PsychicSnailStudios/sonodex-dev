<script lang="ts">
	import { EllipsisIcon, List, TextAlignJustify } from "lucide-svelte";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import Button from "../../ui/button/button.svelte";
	import ScrollArea from "../../ui/scroll-area/scroll-area.svelte";
	import { Toggle } from "$lib/components/ui/toggle/index.js";

	import type { ColumnState, ColumnKey } from "$lib/ts/app/columnConfig.svelte"
	import { ALL_COLUMNS, COLUMN_LABELS } from "$lib/ts/app/columnConfig.svelte"
	import SortDropdown from "$lib/components/app-ui/track-table/SortDropdown.svelte";

	import { SortState } from "$lib/ts/app/sortConfig.svelte";

	let { sort, cols, compact, onCompactChange } = $props<{ sort: SortState, cols: ColumnState, compact: boolean, onCompactChange: (v: boolean) => void }>();
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon"><EllipsisIcon/></Button>
		{/snippet}
	</DropdownMenu.Trigger>

	<DropdownMenu.Content>
		<DropdownMenu.Group>
			<DropdownMenu.Label>View As</DropdownMenu.Label>
			<Toggle onPressedChange={() => onCompactChange(!compact)} >
				{#if compact}
					<List />
					List
				{:else}
					<TextAlignJustify />
					Compact
				{/if}
			</Toggle>
		</DropdownMenu.Group>

		<DropdownMenu.Separator />

		<DropdownMenu.Group>
			<DropdownMenu.Label>Sort</DropdownMenu.Label>
			<SortDropdown sort={sort} />
		</DropdownMenu.Group>
		
		<DropdownMenu.Separator />

		<DropdownMenu.Group>
			<DropdownMenu.Label>View Columns</DropdownMenu.Label>
			<div class="flex flex-col flex-wrap gap-1">
				<p class="text-xs text-muted-foreground">Display Columns</p>
				{#each ALL_COLUMNS as col (col)}
					<Toggle
						pressed={cols.visible[col as ColumnKey]}
						onPressedChange={(v) => (cols.visible[col as ColumnKey] = v)}
						size="sm"
						variant="outline"
						class="text-sm justify-start"
					>
						{COLUMN_LABELS[col as ColumnKey]}
					</Toggle>
				{/each}
			</div>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
	<!-- <div>
		<ScrollArea class="h-[250px]">
			<div>
				
			</div>
		</ScrollArea>
	</div> -->
</DropdownMenu.Root>