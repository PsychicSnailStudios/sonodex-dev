<script lang="ts">
	import type { ColumnState, ColumnKey } from "$lib/columnConfig.svelte"
	import { ALL_COLUMNS, COLUMN_LABELS, ALWAYS_VISIBLE } from "$lib/columnConfig.svelte"
	import { Toggle } from "$lib/components/ui/toggle/index.js"
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

	let { columns } = $props<{ columns: ColumnState }>()

	const toggleable = ALL_COLUMNS.filter((col) => !ALWAYS_VISIBLE.includes(col))
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>...</DropdownMenu.Trigger>
  <DropdownMenu.Content>
    <DropdownMenu.Group>
		<div class="flex flex-col flex-wrap gap-1">
			<p class="text-xs text-muted-foreground">Display Columns</p>
			{#each toggleable as col (col)}
				<Toggle
					pressed={columns.visible[col as ColumnKey]}
					onPressedChange={(v) => (columns.visible[col as ColumnKey] = v)}
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
</DropdownMenu.Root>
