<script lang="ts">
   import * as Tabs from "$lib/components/ui/tabs/index.js";
   import ScrollArea from "../ui/scroll-area/scroll-area.svelte";
   import Settings from "$lib/components/views/Settings.svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
</script>

<div class="flex flex-col gap-2 p-2 border-2 rounded-md h-full w-full overflow-hidden">

	<Tabs.Root value="profile">
		<Tabs.List class="w-full">
			<Tabs.Trigger value="statistics" class="flex-1">Stats</Tabs.Trigger>
			<Tabs.Trigger value="profile" class="flex-1">User</Tabs.Trigger>
			<Tabs.Trigger value="settings" class="flex-1">Settings</Tabs.Trigger>
		</Tabs.List>

		<Tabs.Content value="statistics" class="space-y-3 mt-4">
			<ScrollArea class="min-h-0 min-w-0">
				
			</ScrollArea>
		</Tabs.Content>

		<Tabs.Content value="profile" class="space-y-3 mt-4">
			<ScrollArea class="min-h-0 min-w-0">
				
			</ScrollArea>
		</Tabs.Content>

		<Tabs.Content value="settings" class="space-y-3 mt-4">
			<Settings />
		</Tabs.Content>

	</Tabs.Root>
</div>