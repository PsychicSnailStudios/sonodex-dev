<script lang="ts">

	// COMPONENTS
	import { Ellipsis } from "lucide-svelte";

	import * as DropdownMenu from "$shadcn/dropdown-menu/index.js";
	
	// SCRIPTS
   import { setView } from "$ts/store/state_session.svelte";
	
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
			<DropdownMenu.Item onclick={() => {setView("settings");}}>Settings</DropdownMenu.Item>
			<DropdownMenu.Item onclick={() => {setView("manage");}}>Manage Library</DropdownMenu.Item>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>