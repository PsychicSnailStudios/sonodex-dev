<script lang="ts">
	import { ArrowDownToLine, Loader2 } from "lucide-svelte";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import { open } from "@tauri-apps/plugin-dialog";
	import { invoke } from "@tauri-apps/api/core";
	import { toast } from "svelte-sonner";
	import { parseUidType } from "$lib/ts/util/helpers";
	import { downloadTrack, downloadAlbum, downloadPlaylist } from "$lib/ts/app/downloadManager";
	import { library } from "$lib/ts/library.svelte";
	import type { Track } from "$lib/ts/util/types";

	let {
		uid,
		size = "icon",
		variant = "outline",
		class: className = "",
	}: {
		uid: string;
		size?: "icon" | "sm" | "default";
		variant?: "outline" | "ghost" | "default";
		class?: string;
	} = $props();

	let downloading = $state(false);

	function trackIsLocal(t: Track): boolean {
		if (t.path && t.path != "" && t.path != t.uid) return true;
		if (t.remote_path && t.remote_path.length > 0) return false;
		return false;
	}

	const allLocal = $derived.by(() => {
		const type = parseUidType(uid);

		if (type === "track") {
			const track = library.tracks.find(t => t.uid === uid);
			if (!track) return true;
			return trackIsLocal(track);
		}

		if (type === "album") {
			const album = library.albums.find(a => a.uid === uid);
			if (!album) return true;
			let trackUids: string[] = [];
			try { trackUids = JSON.parse(album.tracks as string ?? "[]").map((e: any) => e.uid); } catch {}
			if (trackUids.length === 0) return true;
			return trackUids.every(tuid => {
				const t = library.tracks.find(t => t.uid === tuid);
				return t ? trackIsLocal(t) : true;
			});
		}

		if (type === "playlist") {
			const playlist = library.playlists.find(p => p.uid === uid);
			if (!playlist) return true;
			let trackUids: string[] = [];
			try { trackUids = JSON.parse(playlist.tracks ?? "[]").map((e: any) => e.uid); } catch {}
			if (trackUids.length === 0) return true;
			return trackUids.every(tuid => {
				const t = library.tracks.find(t => t.uid === tuid);
				return t ? trackIsLocal(t) : true;
			});
		}

		return true;
	});

	async function ensureDownloadPath(): Promise<boolean> {
		const settings = await invoke<Array<{ key: string; value: string }>>("get_settings");
		const existing = settings.find(s => s.key === "download_path")?.value ?? "";
		if (existing.trim()) return true;

		toast.warning("No download location set. Please choose a folder.", { duration: 6000 });

		const selected = await open({ directory: true, multiple: false });
		if (!selected) return false;

		await invoke("save_setting", { key: "download_path", value: selected as string });
		return true;
	}

	async function handleDownload(e: MouseEvent) {
		e.stopPropagation();
		if (downloading) return;

		const type = parseUidType(uid);

		if (type === "track") {
			const track = library.tracks.find(t => t.uid === uid);
			if (!track?.remote_path) {
				toast.warning("This track has no remote source to download from.");
				return;
			}
		}

		const pathOk = await ensureDownloadPath();
		if (!pathOk) return;

		downloading = true;
		try {
			if (type === "track") {
				await downloadTrack(uid);
			} else if (type === "album") {
				await downloadAlbum(uid);
			} else if (type === "playlist") {
				await downloadPlaylist(uid);
			}
		} finally {
			downloading = false;
		}
	}
</script>

{#if !allLocal}
	<Tooltip.Root>
		<Tooltip.Trigger
			class="{buttonVariants({ variant, size })} {className}"
			onclick={handleDownload}
			disabled={downloading}
			aria-label={downloading ? "Downloading" : "Download for offline use"}
		>
			{#if downloading}
				<Loader2 class="animate-spin" />
			{:else}
				<ArrowDownToLine />
			{/if}
		</Tooltip.Trigger>
		<Tooltip.Content>
			<p>{downloading ? "Downloading…" : "Download for offline use"}</p>
		</Tooltip.Content>
	</Tooltip.Root>
{/if}