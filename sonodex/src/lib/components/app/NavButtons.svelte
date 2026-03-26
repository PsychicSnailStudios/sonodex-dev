<script lang="ts">
   import Button from "$lib/components/ui/button/button.svelte";
   import { setSelection } from "$lib/session.svelte";
	import { CircleArrowLeft, CircleArrowRight, CircleX } from "lucide-svelte";

	let { class: className }: ButtonProps = $props();
</script>

<div class="{className} flex justify-start gap-1">
	<Button variant="outline" size="xs">
		<CircleArrowLeft class="w-4 h-4"/>
	</Button>
	<Button variant="outline" size="xs">
		<CircleArrowRight class="w-4 h-4"/>
	</Button>
	<Button variant="outline" size="xs" onclick={() => setSelection(null, "none")}>
		<CircleX class="w-4 h-4"/>
	</Button>
</div>