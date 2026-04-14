<script lang="ts">
	import { Plus, Trash, Pencil, X, Check } from "lucide-svelte";
	import { onMount } from "svelte";

	import * as Accordion from "$lib/components/ui/accordion/index.js";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { Label } from "$lib/components/ui/label/index.js";

	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";
	import { tagStore, type Tag, type TagGroup, type TagKind } from "$lib/ts/tagManager.svelte";

	// ─── View state ───────────────────────────────────────────────────────────────
	let search = $state("");
	let activeSection = $state<string>("tags");

	// ─── Inline edit ──────────────────────────────────────────────────────────────
	let editingUid = $state<string | null>(null);
	let editingName = $state("");
	let editingColor = $state<string | null>(null);
	let editError = $state("");

	// ─── Add new ──────────────────────────────────────────────────────────────────
	let addingNew = $state(false);
	let newName = $state("");
	let newColor = $state<string | null>(null);
	let addError = $state("");

	// ─── Delete confirmation ──────────────────────────────────────────────────────
	let deleteConfirmUid = $state<string | null>(null);
	let deleteConfirmKind = $state<"tag" | "group" | null>(null);

	// ─── Group modal ──────────────────────────────────────────────────────────────
	let groupModalOpen = $state(false);
	let groupModalMode = $state<"create" | "edit">("create");
	let groupModalUid = $state<string | null>(null);
	let groupModalName = $state("");
	let groupModalKind = $state<TagKind>("tag");
	let groupModalColor = $state<string | null>(null);
	let groupModalMembers = $state<Set<string>>(new Set());
	let groupModalError = $state("");
	let groupModalSearch = $state("");

	// ─── Derived ──────────────────────────────────────────────────────────────────
	const displayTags = $derived(
		(activeSection === "genres" ? tagStore.genres : tagStore.tags)
			.filter((t) => !search || t.name.toLowerCase().includes(search.toLowerCase()))
			.sort((a, b) => a.name.localeCompare(b.name))
	);

	const allGroups = $derived(tagStore.groups);

	const groupModalTagPool = $derived(
		(groupModalKind === "tag" ? tagStore.tags : tagStore.genres)
			.filter(
				(t) =>
					!groupModalSearch ||
					t.name.toLowerCase().includes(groupModalSearch.toLowerCase())
			)
			.sort((a, b) => a.name.localeCompare(b.name))
	);

	onMount(async () => {
		if (!tagStore.loaded) await tagStore.load();
	});

	// ─── Color palette ────────────────────────────────────────────────────────────
	const COLORS = [
		"#8B7CF8", "#3B82F6", "#10B981", "#F59E0B",
		"#EF4444", "#EC4899", "#14B8A6", "#F97316",
		"#6366F1", "#84CC16", "#06B6D4", "#A78BFA",
	];

	// ─── Helpers ──────────────────────────────────────────────────────────────────
	function groupsForTag(uid: string): TagGroup[] {
		return allGroups.filter((g) => g.member_uids.includes(uid));
	}

	function memberNamesFor(group: TagGroup): string[] {
		return group.member_uids
			.map((uid) => tagStore.byUid.get(uid)?.name ?? "?")
			.sort();
	}

	// ─── Inline edit ──────────────────────────────────────────────────────────────
	function startEdit(tag: Tag) {
		editingUid = tag.uid;
		editingName = tag.name;
		editingColor = tag.color;
		editError = "";
	}

	function cancelEdit() {
		editingUid = null;
		editError = "";
	}

	async function commitEdit() {
		if (!editingUid) return;
		const trimmed = editingName.trim();
		if (!trimmed) { editError = "Name cannot be empty"; return; }
		const conflict = tagStore.byName.get(trimmed.toLowerCase());
		if (conflict && conflict.uid !== editingUid) { editError = "A tag with that name already exists"; return; }
		await tagStore.renameTag(editingUid, trimmed);
		if (editingColor !== tagStore.byUid.get(editingUid)?.color) {
			await tagStore.updateTagColor(editingUid, editingColor);
		}
		editingUid = null;
	}

	function handleEditKeydown(e: KeyboardEvent) {
		if (e.key === "Enter") commitEdit();
		if (e.key === "Escape") cancelEdit();
	}

	// ─── Add new ──────────────────────────────────────────────────────────────────
	function startAdd() {
		addingNew = true;
		newName = "";
		newColor = null;
		addError = "";
	}

	function cancelAdd() {
		addingNew = false;
		addError = "";
	}

	async function commitAdd() {
		const trimmed = newName.trim();
		if (!trimmed) { addError = "Name cannot be empty"; return; }
		const isGenre = activeSection === "genres";
		if (isGenre) await tagStore.addGenre(trimmed, newColor ?? undefined);
		else await tagStore.addTag(trimmed, newColor ?? undefined);
		addingNew = false;
		newName = "";
	}

	function handleAddKeydown(e: KeyboardEvent) {
		if (e.key === "Enter") commitAdd();
		if (e.key === "Escape") cancelAdd();
	}

	// ─── Delete ───────────────────────────────────────────────────────────────────
	function requestDelete(uid: string, kind: "tag" | "group") {
		deleteConfirmUid = uid;
		deleteConfirmKind = kind;
	}

	async function confirmDelete() {
		if (!deleteConfirmUid) return;
		if (deleteConfirmKind === "tag") await tagStore.deleteTag(deleteConfirmUid);
		else if (deleteConfirmKind === "group") await tagStore.deleteGroup(deleteConfirmUid);
		deleteConfirmUid = null;
		deleteConfirmKind = null;
	}

	// ─── Group modal ──────────────────────────────────────────────────────────────
	function openCreateGroup() {
		groupModalMode = "create";
		groupModalUid = null;
		groupModalName = "";
		groupModalKind = activeSection === "genres" ? "genre" : "tag";
		groupModalColor = null;
		groupModalMembers = new Set();
		groupModalSearch = "";
		groupModalError = "";
		groupModalOpen = true;
	}

	function openEditGroup(group: TagGroup) {
		groupModalMode = "edit";
		groupModalUid = group.uid;
		groupModalName = group.name;
		groupModalKind = group.kind;
		groupModalColor = group.color;
		groupModalMembers = new Set(group.member_uids);
		groupModalSearch = "";
		groupModalError = "";
		groupModalOpen = true;
	}

	function toggleGroupMember(uid: string) {
		const next = new Set(groupModalMembers);
		if (next.has(uid)) next.delete(uid);
		else next.add(uid);
		groupModalMembers = next;
	}

	async function commitGroupModal() {
		const trimmed = groupModalName.trim();
		if (!trimmed) { groupModalError = "Name cannot be empty"; return; }
		if (groupModalMembers.size === 0) { groupModalError = "Select at least one member"; return; }
		if (groupModalMode === "create") {
			await tagStore.createGroup(
				trimmed, groupModalKind, [...groupModalMembers], groupModalColor ?? undefined
			);
		} else if (groupModalUid) {
			await tagStore.updateGroup(groupModalUid, {
				name: trimmed,
				color: groupModalColor,
				memberUids: [...groupModalMembers],
			});
		}
		groupModalOpen = false;
	}
</script>

<AlertDialog.Root open={!!deleteConfirmUid} onOpenChange={(v) => { if (!v) { deleteConfirmUid = null; deleteConfirmKind = null; } }}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete {deleteConfirmKind === "group" ? "group" : "tag"}?</AlertDialog.Title>
			<AlertDialog.Description>
				{#if deleteConfirmKind === "tag"}
					This tag will be removed from every track, album, and artist that has it.
				{:else}
					The group will be deleted. The member tags themselves will not be affected.
				{/if}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action
				class={buttonVariants({ variant: "destructive" })}
				onclick={confirmDelete}
			>Delete</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>

<Dialog.Root bind:open={groupModalOpen}>
	<Dialog.Content class="max-w-lg">
		<Dialog.Header>
			<Dialog.Title>{groupModalMode === "create" ? "New group" : "Edit group"}</Dialog.Title>
		</Dialog.Header>

		<div class="flex flex-col gap-3">
			<div class="flex flex-col gap-1.5">
				<Label>Group name</Label>
				<Input placeholder="e.g. Mood" bind:value={groupModalName} />
			</div>

			<div class="flex gap-4 items-center">
				<div class="flex flex-col gap-1.5">
					<Label>Kind</Label>
					<select
						bind:value={groupModalKind}
						class="h-9 rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm"
					>
						<option value="tag">Tags</option>
						<option value="genre">Genres</option>
					</select>
				</div>
				<div class="flex flex-col gap-1.5">
					<Label>Color</Label>
					<div class="flex gap-1.5 flex-wrap">
						{#each COLORS.slice(0, 8) as c}
							<button
								onclick={() => groupModalColor = c}
								class="w-5 h-5 rounded-full cursor-pointer p-0 transition-transform hover:scale-110"
								style="background: {c}; outline: {groupModalColor === c ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
								aria-label="Color {c}"
							></button>
						{/each}
						<button
							onclick={() => groupModalColor = null}
							class="w-5 h-5 rounded-full cursor-pointer flex items-center justify-center text-[10px] transition-transform hover:scale-110"
							style="background: hsl(var(--muted)); outline: {groupModalColor === null ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
							title="No color"
						>✕</button>
					</div>
				</div>
			</div>

			<div class="flex flex-col gap-1.5">
				<Label>Members ({groupModalMembers.size} selected)</Label>
				<Input type="search" placeholder="Filter…" bind:value={groupModalSearch} />
				<ScrollArea class="h-40 rounded-md border border-border p-2">
					{#if groupModalTagPool.length === 0}
						<p class="text-sm text-muted-foreground text-center py-4">
							No {groupModalKind === "genre" ? "genres" : "tags"} found. Add some first.
						</p>
					{:else}
						<div class="flex flex-wrap gap-1.5">
							{#each groupModalTagPool as tag (tag.uid)}
								<button
									onclick={() => toggleGroupMember(tag.uid)}
									class="inline-flex items-center gap-1 text-xs px-2.5 py-1 rounded-full border cursor-pointer transition-colors"
									class:bg-foreground={groupModalMembers.has(tag.uid)}
									class:text-background={groupModalMembers.has(tag.uid)}
									class:border-transparent={groupModalMembers.has(tag.uid)}
									class:bg-transparent={!groupModalMembers.has(tag.uid)}
									class:text-muted-foreground={!groupModalMembers.has(tag.uid)}
									class:border-border={!groupModalMembers.has(tag.uid)}
								>
									{#if tag.color}
										<span class="w-2 h-2 rounded-full flex-shrink-0" style="background: {tag.color}"></span>
									{/if}
									{tag.name}
								</button>
							{/each}
						</div>
					{/if}
				</ScrollArea>
			</div>

			{#if groupModalError}
				<p class="text-xs text-destructive">{groupModalError}</p>
			{/if}
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => groupModalOpen = false}>Cancel</Button>
			<Button onclick={commitGroupModal}>
				{groupModalMode === "create" ? "Create group" : "Save changes"}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Accordion.Root type="single" bind:value={activeSection} onValueChange={(v) => { if (v) { activeSection = v; search = ""; addingNew = false; } }} class="flex flex-col gap-1">

  <!-- ── Tags ── -->
  <Accordion.Item value="tags" class="border rounded-md px-3">
    <Accordion.Trigger class="py-3 text-sm font-medium hover:no-underline">
      <div>
        <span>Tags</span>
        <Badge variant="secondary" class="ml-auto mr-2 text-[11px]">{tagStore.tags.length}</Badge>
      </div>
    </Accordion.Trigger>
    <Accordion.Content>
      <div class="flex flex-col gap-2 pb-3">
        <div class="flex gap-2 justify-between items-center">
          <span class="text-sm text-muted-foreground">
            {displayTags.length} {displayTags.length === 1 ? "tag" : "tags"}
          </span>
          <div class="flex gap-2 items-center">
            <Button variant="outline" size="sm" onclick={startAdd}>
              <Plus class="w-4 h-4 mr-1" /> Add tag
            </Button>
            <SearchBar bind:search searchCount={displayTags.length} />
          </div>
        </div>

        {#if addingNew && activeSection === "tags"}
          <div class="rounded-md border border-border bg-muted/40 p-3 flex flex-col gap-2">
            <div class="flex gap-2 items-center">
              <span
                class="w-5 h-5 rounded-full flex-shrink-0 border border-border"
                style="background: {newColor ?? 'hsl(var(--muted-foreground))'}"
              ></span>
              <Input
                placeholder="Tag name…"
                bind:value={newName}
                onkeydown={handleAddKeydown}
                autofocus
                class="h-8 text-sm"
              />
              <Button size="sm" onclick={commitAdd}><Check class="w-4 h-4" /></Button>
              <Button size="sm" variant="ghost" onclick={cancelAdd}><X class="w-4 h-4" /></Button>
            </div>
            <div class="flex gap-1.5 flex-wrap">
              {#each COLORS as c}
                <button
                  onclick={() => newColor = c}
                  class="w-5 h-5 rounded-full cursor-pointer p-0 transition-transform hover:scale-110"
                  style="background: {c}; outline: {newColor === c ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
                  aria-label="Color {c}"
                ></button>
              {/each}
              <button
                onclick={() => newColor = null}
                class="w-5 h-5 rounded-full cursor-pointer flex items-center justify-center text-[10px] transition-transform hover:scale-110"
                style="background: hsl(var(--muted)); outline: {newColor === null ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
                title="No color"
              >✕</button>
            </div>
            {#if addError}<p class="text-xs text-destructive">{addError}</p>{/if}
          </div>
        {/if}

        {#if displayTags.length === 0 && !addingNew}
          <p class="text-sm text-muted-foreground text-center py-6">No tags yet.</p>
        {/if}

        <div class="flex flex-col gap-1">
          {#each displayTags as tag (tag.uid)}
            {@const groups = groupsForTag(tag.uid)}
            {#if editingUid === tag.uid}
              <div class="rounded-md border border-border bg-muted/40 p-2 flex flex-col gap-2">
                <div class="flex gap-2 items-center">
                  <span
                    class="w-5 h-5 rounded-full flex-shrink-0 border border-border"
                    style="background: {editingColor ?? 'hsl(var(--muted-foreground))'}"
                  ></span>
                  <Input bind:value={editingName} onkeydown={handleEditKeydown} autofocus class="h-8 text-sm" />
                  <Button size="sm" onclick={commitEdit}><Check class="w-4 h-4" /></Button>
                  <Button size="sm" variant="ghost" onclick={cancelEdit}><X class="w-4 h-4" /></Button>
                </div>
                <div class="flex gap-1.5 flex-wrap">
                  {#each COLORS as c}
                    <button
                      onclick={() => editingColor = c}
                      class="w-5 h-5 rounded-full cursor-pointer p-0 transition-transform hover:scale-110"
                      style="background: {c}; outline: {editingColor === c ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
                      aria-label="Color {c}"
                    ></button>
                  {/each}
                  <button
                    onclick={() => editingColor = null}
                    class="w-5 h-5 rounded-full cursor-pointer flex items-center justify-center text-[10px] transition-transform hover:scale-110"
                    style="background: hsl(var(--muted)); outline: {editingColor === null ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
                    title="No color"
                  >✕</button>
                </div>
                {#if editError}<p class="text-xs text-destructive">{editError}</p>{/if}
              </div>
            {:else}
              <div class="tag-row group flex items-center gap-2 px-2 py-1.5 rounded-md border border-transparent hover:border-border hover:bg-muted/40 transition-colors">
                <span
                  class="w-2.5 h-2.5 rounded-full flex-shrink-0 border border-border"
                  style="background: {tag.color ?? 'hsl(var(--muted-foreground))'}"
                ></span>
                <span class="text-sm flex-1 min-w-0 truncate">{tag.name}</span>
                {#if groups.length > 0}
                  <div class="flex gap-1 flex-shrink-0">
                    {#each groups.slice(0, 2) as g}
                      <Badge variant="secondary" class="text-[11px] px-2 py-0">{g.name}</Badge>
                    {/each}
                    {#if groups.length > 2}
                      <span class="text-xs text-muted-foreground">+{groups.length - 2}</span>
                    {/if}
                  </div>
                {/if}
                <div class="flex gap-1 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Tooltip.Root>
                    <Tooltip.Trigger class={buttonVariants({ variant: "ghost", size: "sm" })} onclick={() => startEdit(tag)}>
                      <Pencil class="w-3.5 h-3.5" />
                    </Tooltip.Trigger>
                    <Tooltip.Content><p>Rename</p></Tooltip.Content>
                  </Tooltip.Root>
                  <Tooltip.Root>
                    <Tooltip.Trigger class={buttonVariants({ variant: "ghost", size: "sm" })} onclick={() => requestDelete(tag.uid, "tag")}>
                      <Trash class="w-3.5 h-3.5 text-destructive" />
                    </Tooltip.Trigger>
                    <Tooltip.Content><p>Delete tag</p></Tooltip.Content>
                  </Tooltip.Root>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    </Accordion.Content>
  </Accordion.Item>

  <!-- ── Genres ── -->
  <Accordion.Item value="genres" class="border rounded-md px-3">
    <Accordion.Trigger class="py-3 text-sm font-medium hover:no-underline">
      
      <div>
        <span>Genres</span>
        <Badge variant="secondary" class="ml-auto mr-2 text-[11px]">{tagStore.genres.length}</Badge>
      </div>
    </Accordion.Trigger>
    <Accordion.Content>
      <div class="flex flex-col gap-2 pb-3">
        <div class="flex gap-2 justify-between items-center">
          <span class="text-sm text-muted-foreground">
            {displayTags.length} {displayTags.length === 1 ? "genre" : "genres"}
          </span>
          <div class="flex gap-2 items-center">
            <Button variant="outline" size="sm" onclick={startAdd}>
              <Plus class="w-4 h-4 mr-1" /> Add genre
            </Button>
            <SearchBar bind:search searchCount={displayTags.length} />
          </div>
        </div>

        {#if addingNew && activeSection === "genres"}
          <div class="rounded-md border border-border bg-muted/40 p-3 flex flex-col gap-2">
            <div class="flex gap-2 items-center">
              <span
                class="w-5 h-5 rounded-full flex-shrink-0 border border-border"
                style="background: {newColor ?? 'hsl(var(--muted-foreground))'}"
              ></span>
              <Input
                placeholder="Genre name…"
                bind:value={newName}
                onkeydown={handleAddKeydown}
                autofocus
                class="h-8 text-sm"
              />
              <Button size="sm" onclick={commitAdd}><Check class="w-4 h-4" /></Button>
              <Button size="sm" variant="ghost" onclick={cancelAdd}><X class="w-4 h-4" /></Button>
            </div>
            <div class="flex gap-1.5 flex-wrap">
              {#each COLORS as c}
                <button
                  onclick={() => newColor = c}
                  class="w-5 h-5 rounded-full cursor-pointer p-0 transition-transform hover:scale-110"
                  style="background: {c}; outline: {newColor === c ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
                  aria-label="Color {c}"
                ></button>
              {/each}
              <button
                onclick={() => newColor = null}
                class="w-5 h-5 rounded-full cursor-pointer flex items-center justify-center text-[10px] transition-transform hover:scale-110"
                style="background: hsl(var(--muted)); outline: {newColor === null ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
                title="No color"
              >✕</button>
            </div>
            {#if addError}<p class="text-xs text-destructive">{addError}</p>{/if}
          </div>
        {/if}

        {#if displayTags.length === 0 && !addingNew}
          <p class="text-sm text-muted-foreground text-center py-6">No genres yet.</p>
        {/if}

        <div class="flex flex-col gap-1">
          {#each displayTags as tag (tag.uid)}
            {@const groups = groupsForTag(tag.uid)}
            {#if editingUid === tag.uid}
              <div class="rounded-md border border-border bg-muted/40 p-2 flex flex-col gap-2">
                <div class="flex gap-2 items-center">
                  <span
                    class="w-5 h-5 rounded-full flex-shrink-0 border border-border"
                    style="background: {editingColor ?? 'hsl(var(--muted-foreground))'}"
                  ></span>
                  <Input bind:value={editingName} onkeydown={handleEditKeydown} autofocus class="h-8 text-sm" />
                  <Button size="sm" onclick={commitEdit}><Check class="w-4 h-4" /></Button>
                  <Button size="sm" variant="ghost" onclick={cancelEdit}><X class="w-4 h-4" /></Button>
                </div>
                <div class="flex gap-1.5 flex-wrap">
                  {#each COLORS as c}
                    <button
                      onclick={() => editingColor = c}
                      class="w-5 h-5 rounded-full cursor-pointer p-0 transition-transform hover:scale-110"
                      style="background: {c}; outline: {editingColor === c ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
                      aria-label="Color {c}"
                    ></button>
                  {/each}
                  <button
                    onclick={() => editingColor = null}
                    class="w-5 h-5 rounded-full cursor-pointer flex items-center justify-center text-[10px] transition-transform hover:scale-110"
                    style="background: hsl(var(--muted)); outline: {editingColor === null ? '2px solid hsl(var(--foreground))' : '2px solid transparent'}; outline-offset: 2px;"
                    title="No color"
                  >✕</button>
                </div>
                {#if editError}<p class="text-xs text-destructive">{editError}</p>{/if}
              </div>
            {:else}
              <div class="tag-row group flex items-center gap-2 px-2 py-1.5 rounded-md border border-transparent hover:border-border hover:bg-muted/40 transition-colors">
                <span
                  class="w-2.5 h-2.5 rounded-full flex-shrink-0 border border-border"
                  style="background: {tag.color ?? 'hsl(var(--muted-foreground))'}"
                ></span>
                <span class="text-sm flex-1 min-w-0 truncate">{tag.name}</span>
                {#if groups.length > 0}
                  <div class="flex gap-1 flex-shrink-0">
                    {#each groups.slice(0, 2) as g}
                      <Badge variant="secondary" class="text-[11px] px-2 py-0">{g.name}</Badge>
                    {/each}
                    {#if groups.length > 2}
                      <span class="text-xs text-muted-foreground">+{groups.length - 2}</span>
                    {/if}
                  </div>
                {/if}
                <div class="flex gap-1 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Tooltip.Root>
                    <Tooltip.Trigger class={buttonVariants({ variant: "ghost", size: "sm" })} onclick={() => startEdit(tag)}>
                      <Pencil class="w-3.5 h-3.5" />
                    </Tooltip.Trigger>
                    <Tooltip.Content><p>Rename</p></Tooltip.Content>
                  </Tooltip.Root>
                  <Tooltip.Root>
                    <Tooltip.Trigger class={buttonVariants({ variant: "ghost", size: "sm" })} onclick={() => requestDelete(tag.uid, "tag")}>
                      <Trash class="w-3.5 h-3.5 text-destructive" />
                    </Tooltip.Trigger>
                    <Tooltip.Content><p>Delete genre</p></Tooltip.Content>
                  </Tooltip.Root>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    </Accordion.Content>
  </Accordion.Item>

  <!-- ── Groups ── -->
  <Accordion.Item value="groups" class="border rounded-md px-3">
    <Accordion.Trigger class="py-3 text-sm font-medium hover:no-underline">
      
      <div>
        <span>Groups</span>
        <Badge variant="secondary" class="ml-auto mr-2 text-[11px]">{allGroups.length}</Badge>
      </div>
    </Accordion.Trigger>
    <Accordion.Content>
      <div class="flex flex-col gap-2 pb-3">
        <div class="flex justify-between items-center">
          <span class="text-sm text-muted-foreground">
            {allGroups.length} {allGroups.length === 1 ? "group" : "groups"}
          </span>
          <Button variant="outline" size="sm" onclick={openCreateGroup}>
            <Plus class="w-4 h-4 mr-1" /> New group
          </Button>
        </div>

        {#if allGroups.length === 0}
          <p class="text-sm text-muted-foreground text-center py-6">
            No groups yet. Groups let you cluster tags — e.g. "Mood" containing happy, sad, angry.
          </p>
        {:else}
          <div class="flex flex-col gap-2">
            {#each allGroups as group (group.uid)}
              <div class="rounded-md border border-border p-3 flex flex-col gap-2">
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-2 min-w-0">
                    {#if group.color}
                      <span class="w-2.5 h-2.5 rounded-full flex-shrink-0" style="background: {group.color}"></span>
                    {/if}
                    <span class="text-sm font-medium truncate">{group.name}</span>
                    <Badge variant="secondary" class="text-[11px] px-2 py-0">{group.kind}</Badge>
                  </div>
                  <div class="flex gap-1 flex-shrink-0">
                    <Tooltip.Root>
                      <Tooltip.Trigger class={buttonVariants({ variant: "ghost", size: "sm" })} onclick={() => openEditGroup(group)}>
                        <Pencil class="w-3.5 h-3.5" />
                      </Tooltip.Trigger>
                      <Tooltip.Content><p>Edit group</p></Tooltip.Content>
                    </Tooltip.Root>
                    <Tooltip.Root>
                      <Tooltip.Trigger class={buttonVariants({ variant: "ghost", size: "sm" })} onclick={() => requestDelete(group.uid, "group")}>
                        <Trash class="w-3.5 h-3.5 text-destructive" />
                      </Tooltip.Trigger>
                      <Tooltip.Content><p>Delete group</p></Tooltip.Content>
                    </Tooltip.Root>
                  </div>
                </div>
                <div class="flex flex-wrap gap-1">
                  {#each memberNamesFor(group) as name}
                    <Badge variant="outline" class="text-[11px] px-2 py-0 text-muted-foreground">{name}</Badge>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </Accordion.Content>
  </Accordion.Item>

</Accordion.Root>