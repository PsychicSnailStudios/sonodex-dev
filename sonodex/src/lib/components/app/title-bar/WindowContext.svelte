<script lang="ts">

	// COMPONENTS
	import { Ellipsis } from "lucide-svelte";

	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  
	// VARIABLES
  
	// APP FUNCTIONS
  
	// FUNCTIONS
  
</script>

<DropdownMenu.Root>
    <DropdownMenu.Trigger>
		<Ellipsis class="w-4 h-4"/>
	</DropdownMenu.Trigger>

	<DropdownMenu.Content>
		<DropdownMenu.Group>
			<DropdownMenu.Item>Settings</DropdownMenu.Item>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>