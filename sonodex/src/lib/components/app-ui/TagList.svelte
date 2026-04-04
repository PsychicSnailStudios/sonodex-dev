<script lang="ts">

	import X from "@lucide/svelte/icons/x";
	import { Badge } from "$lib/components/ui/badge/index.js";

	let { tags, canEdit = false } = $props<{ tags: string[], canEdit?: boolean }>();

	function removeTag(tag: string) {
		tags = tags.filter(t => t !== tag);
	}

</script>

<div class="flex flex-wrap gap-1">
	{#each tags as tag}
		<Badge variant="outline">
			<span class="text-xs">{tag}</span>
			{#if canEdit}
				<button onclick={() => removeTag(tag)}><X class="cursor-pointer w-4 h-4"/></button>
			{/if}
		</Badge>
	{/each}
</div>