<script lang="ts">

	// APP
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";
	import { setMode, mode } from "mode-watcher";

	// COMPONENTS
	import * as Select from "$lib/components/ui/select/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Switch } from "$lib/components/ui/switch/index.js";
	import { Label } from "$lib/components/ui/label/index.js";

	import SpotifyImport from "$lib/components/dialogs/playlists/SpotifyImport.svelte";

	// SCRIPTS
	import { loadLibrary, reloadLibrary } from "$lib/ts/library.svelte";
	import { scanState } from "$lib/ts/app-states/state_session.svelte";
	import { eq, EQ_BANDS, EQ_PRESETS, loadEqSettings, setEqEnabled, setEqBandGain, applyEqPreset,	} from "$lib/ts/app/eqStore.svelte";
	import { applyEqToGraph } from "$lib/ts/audio/audioManager.svelte";
	import { connectLastfm, disconnectLastfm, lastfmIsConnected, onLastfmConnected, } from "$lib/ts/connections/lastfm";
	import { connectSpotify, disconnectSpotify, spotifyIsConnected, getSpotifyPlaylists, importSpotifyPlaylist, onSpotifyConnected, type SpotifyPlaylistSummary, } from "$lib/ts/connections/spotify";
	import { toast } from "svelte-sonner";
	
	// VARIABLES
	let settings = $state<Record<string, string>>({});

	let lastfmConnected = $state(false);
	let spotifyConnected = $state(false);
	let spotifyImportOpen = $state(false);

	let themeOptions = [
		{ value: "system", label: "System" },
		{ value: "light", label: "Light" },
		{ value: "dark", label: "Dark" },
	];

	const PRIORITY_OPTIONS = [
		{ value: "tag", label: "File Tag" },
		{ value: "filename", label: "Filename (override)" },
		{ value: "folder", label: "Folder (override)" },
	];

	const SETTING_LABELS: Record<string, string> = {
		filename_priority_title: "Title",
		filename_priority_artist: "Artist",
		filename_priority_album: "Album",
		filename_priority_year: "Year",
		filename_custom_pattern: "Custom Filename Pattern",
	};

	// APP FUNCTIONS
	onMount(() => {
		let cleanupLastfm: (() => void) | undefined;
		let cleanupSpotify: (() => void) | undefined;

		(async () => {
			await loadSettings();
			await loadEqSettings();

			await listen("enrich:progress", (event: any) => {
				scanState.enriching = true;
				scanState.enrichDone = event.payload.done;
				scanState.enrichTotal = event.payload.total;
				scanState.enrichErrors = event.payload.errors;
			});

			await listen("enrich:done", (event: any) => {
				scanState.enriching = false;
				scanState.enrichErrors = event.payload.errors;
				scanState.status = `Enrichment done. ${event.payload.total - event.payload.errors} updated, ${event.payload.errors} not found.`;
			});

			lastfmConnected = await lastfmIsConnected();
			spotifyConnected = await spotifyIsConnected();

			cleanupLastfm = await onLastfmConnected(async () => {
				lastfmConnected = true;
			});

			cleanupSpotify = await onSpotifyConnected(async () => {
				spotifyConnected = true;
				toast.success("Spotify connected");
			});
		})();

		return () => {
			cleanupLastfm?.();
			cleanupSpotify?.();
		};
	});

	// FUNCTIONS
	async function loadSettings() {
		const raw: { key: string; value: string }[] = await invoke("get_settings");
		settings = Object.fromEntries(raw.map((s) => [s.key, s.value]));
	}

	async function saveSetting(key: string, value: string) {
		settings[key] = value;
		await invoke("save_setting", { key, value });
	}

	async function handleEqToggle(checked: boolean) {
		await setEqEnabled(checked);
		applyEqToGraph();
	}

	async function handleBandChange(index: number, value: number) {
		await setEqBandGain(index, value);
		applyEqToGraph();
	}

	async function handlePreset(preset: (typeof EQ_PRESETS)[number]) {
		await applyEqPreset(preset);
		applyEqToGraph();
	}

	async function handleLastfmConnect() {
		await connectLastfm();
	}

	async function handleLastfmDisconnect() {
		await disconnectLastfm();
		lastfmConnected = false;
	}

	async function handleSpotifyConnect() {
		await connectSpotify();
	}

	async function handleSpotifyDisconnect() {
		await disconnectSpotify();
		spotifyConnected = false;
	}

</script>

<SpotifyImport bind:open={spotifyImportOpen} />

<div class="flex flex-col gap-2 p-2 border-2 rounded-md h-full w-full overflow-hidden">
	<h2 class="h2">Settings</h2>

	<ScrollArea class="h-full w-full min-h-0 min-w-0">
		<div class="flex flex-col gap-4 p-2 pr-4">
			<h3 class="font-semibold">General</h3>
			<div class="flex flex-col gap-2 p-2 bg-muted rounded-md">
				<label class="text-sm">Theme</label>
				<Select.Root
					type="single"
					value={mode.current}
					onValueChange={(value) => setMode(value as "system" | "light" | "dark")}
					>
					<Select.Trigger class="w-[180px]">{mode.current}</Select.Trigger>
					<Select.Content>
						{#each themeOptions as opt}
						<Select.Item value={opt.value}>{opt.label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
				
			</div>
	
			<h3 class="font-semibold">EQ</h3>
			<div class="flex flex-col gap-4 p-2 bg-muted rounded-md">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-semibold">Equalizer</p>
						<p class="text-xs text-muted-foreground">10-band graphic EQ applied to audio output.</p>
					</div>
					<Switch checked={eq.enabled} onCheckedChange={handleEqToggle} />
				</div>
			
				<div class="flex flex-wrap gap-2">
					{#each EQ_PRESETS as preset}
						<Button
							variant="outline"
							class="text-xs h-7 px-2"
							onclick={() => handlePreset(preset)}
						>{preset.name}</Button>
					{/each}
				</div>
			
				<div class="flex items-end justify-between gap-1 pt-2" class:opacity-50={!eq.enabled} class:pointer-events-none={!eq.enabled}>
					{#each EQ_BANDS as freq, i}
						<div class="flex flex-col items-center gap-1 flex-1">
							<span class="text-xs text-muted-foreground tabular-nums">
								{eq.gains[i] > 0 ? "+" : ""}{eq.gains[i]}
							</span>
							<div class="relative flex justify-center" style="height: 120px;">
								<input
									type="range"
									min="-12"
									max="12"
									step="0.5"
									value={eq.gains[i]}
									oninput={(e) => handleBandChange(i, parseFloat((e.target as HTMLInputElement).value))}
									style="writing-mode: vertical-lr; direction: rtl; width: 28px; height: 120px; cursor: pointer; accent-color: hsl(var(--primary));"
								/>
							</div>
							<span class="text-xs text-muted-foreground">
								{freq >= 1000 ? `${freq / 1000}k` : freq}
							</span>
						</div>
					{/each}
				</div>
			
				<div class="flex justify-between text-xs text-muted-foreground px-1 mt-1">
					<span>-12 dB</span>
					<span>0 dB</span>
					<span>+12 dB</span>
				</div>
			</div>
	
			<h3 class="font-semibold">Local Metadata</h3>
			<div class="flex flex-col gap-2 p-2 bg-muted rounded-md">
	
				<div class="space-y-2">
				<h2 class="text-sm font-semibold">Metadata Priority</h2>
				<p class="text-xs text-muted-foreground">File Tag uses embedded tags with filename as fallback. Filename override prefers the filename. Folder uses the folder structure as the override source.</p>
				{#each ["filename_priority_title", "filename_priority_artist", "filename_priority_album", "filename_priority_year"] as key}
					<div class="flex items-center justify-between gap-4">
					<label class="text-sm w-24">{SETTING_LABELS[key]}</label>
					<select
						class="flex-1 border rounded px-3 py-2 text-sm bg-background"
						value={settings[key] ?? "tag"}
						onchange={(e) => saveSetting(key, (e.target as HTMLSelectElement).value)}
					>
						{#each PRIORITY_OPTIONS as opt}
						<option value={opt.value}>{opt.label}</option>
						{/each}
					</select>
					</div>
				{/each}
				</div>
	
				<div class="space-y-1">
				<label class="text-sm font-medium">Custom Filename Pattern</label>
				<p class="text-xs text-muted-foreground">Use tokens: {"{title}"} {"{artist}"} {"{album}"} {"{year}"}. Example: <code>{"{artist}"} - {"{year}"} - {"{title}"}</code></p>
				<input
					class="w-full border rounded px-3 py-2 text-sm bg-background"
					placeholder="{'{artist}'} - {'{album}'} - {'{year}'} - {'{title}'}"
					value={settings["filename_custom_pattern"] ?? ""}
					onchange={(e) => saveSetting("filename_custom_pattern", (e.target as HTMLInputElement).value)}
				/>
				</div>
	
				<div class="space-y-2">
				<h2 class="text-sm font-semibold">Artist Parsing</h2>
				<p class="text-xs text-muted-foreground">Attempt to split artist names containing "&" into separate artists when the result matches already-known artists.</p>
				<div class="flex items-center justify-between gap-4">
					<label class="text-sm">Try parse &amp;</label>
					<Switch checked={settings["scan_try_parse_ampersand"] === "true"} onCheckedChange={(checked) => saveSetting("scan_try_parse_ampersand", checked ? "true" : "false")} />
				</div>
				</div>
			</div>

			<h3 class="font-semibold">Scan Behaviour</h3>
			<div class="flex flex-col gap-2 p-2 bg-muted rounded-md">
				<p class="text-xs text-muted-foreground">These run automatically when a path is scanned. Enrichment and lyrics fetch require an internet connection and will slow down scanning.</p>
				<div class="flex items-center justify-between gap-4">
					<div>
						<label class="text-sm font-medium">Auto-enrich tracks</label>
						<p class="text-xs text-muted-foreground">Fetch missing metadata from APIs during scan.</p>
					</div>
					<Switch checked={settings["auto_enrich_tracks"] === "true"} onCheckedChange={(checked) => saveSetting("auto_enrich_tracks", checked ? "true" : "false")} />
				</div>
				<div class="flex items-center justify-between gap-4">
					<div>
						<label class="text-sm font-medium">Auto-enrich albums</label>
						<p class="text-xs text-muted-foreground">Fetch album metadata after tracks are scanned.</p>
					</div>
					<Switch checked={settings["auto_enrich_albums"] === "true"} onCheckedChange={(checked) => saveSetting("auto_enrich_albums", checked ? "true" : "false")} />
				</div>
				<div class="flex items-center justify-between gap-4">
					<div>
						<label class="text-sm font-medium">Auto-fetch lyrics</label>
						<p class="text-xs text-muted-foreground">Fetch lyrics from lrclib for each track during scan.</p>
					</div>
					<Switch checked={settings["auto_fetch_lyrics"] === "true"} onCheckedChange={(checked) => saveSetting("auto_fetch_lyrics", checked ? "true" : "false")} />
				</div>
			</div>
	
			<h3 class="font-semibold">Connected Accounts</h3>
			<div class="flex flex-col gap-4 p-2 bg-muted rounded-md">
	
				<!-- Last.fm -->
				<div class="flex items-center justify-between gap-4">
					<div class="flex flex-col gap-0.5">
							<p class="text-sm font-semibold">Last.fm</p>
							<p class="text-xs text-muted-foreground">
								{#if lastfmConnected}
									Connected — scrobbles will be sent automatically.
								{:else}
									Connect to enable scrobbling. Requires an API key in Metadata APIs below.
								{/if}
							</p>
					</div>
					{#if lastfmConnected}
							<Button variant="outline" onclick={handleLastfmDisconnect}>Disconnect</Button>
					{:else}
							<Button onclick={handleLastfmConnect}>Connect</Button>
					{/if}
				</div>
	
				{#if !lastfmConnected}
				<div class="space-y-1">
					<label class="text-sm font-medium">Last.fm Shared Secret</label>
					<p class="text-xs text-muted-foreground">Required alongside your API key to complete authentication.</p>
					<input
							class="w-full border rounded px-3 py-2 text-sm bg-background"
							placeholder="Your Last.fm shared secret"
							type="password"
							value={settings["api_lastfm_secret"] ?? ""}
							onchange={(e) => saveSetting("api_lastfm_secret", (e.target as HTMLInputElement).value)}
					/>
				</div>
				{/if}
	
				<div class="border-t border-border" />
	
				<!-- Spotify -->
				<div class="flex items-center justify-between gap-4">
					<div class="flex flex-col gap-0.5">
							<p class="text-sm font-semibold">Spotify</p>
							<p class="text-xs text-muted-foreground">
								{#if spotifyConnected}
									Connected — playlists can be imported and metadata enriched.
								{:else}
									Connect to import playlists and use Spotify for metadata enrichment.
								{/if}
							</p>
					</div>
					{#if spotifyConnected}
						<Button variant="outline" onclick={handleSpotifyDisconnect}>Disconnect</Button>
						<Button variant="outline" onclick={() => spotifyImportOpen = true}>Import Playlists</Button>
					{:else}
							<Button onclick={handleSpotifyConnect}>Connect</Button>
					{/if}
				</div>
	
				{#if !spotifyConnected}
				<div class="space-y-1">
					<label class="text-sm font-medium">Spotify Client ID</label>
					<p class="text-xs text-muted-foreground">
							Get this from your app at
							<a href="https://developer.spotify.com/dashboard" target="_blank" class="underline underline-offset-2">developer.spotify.com/dashboard</a>.
							Set <code>sonodex://spotify-callback</code> as a Redirect URI in your app settings.
					</p>
					<input
							class="w-full border rounded px-3 py-2 text-sm bg-background"
							placeholder="Your Spotify client ID"
							value={settings["spotify_client_id"] ?? ""}
							onchange={(e) => saveSetting("spotify_client_id", (e.target as HTMLInputElement).value)}
					/>
				</div>
				{/if}	
			</div>
	
			<h3 class="font-semibold">Metadata API's</h3>
			<div class="flex flex-col gap-2 p-2 bg-muted rounded-md">
				<div class="space-y-2">
				<h2 class="text-sm font-semibold">Online Metadata Enrichment</h2>
				<p class="text-xs text-muted-foreground">Fetch missing metadata from MusicBrainz and TheAudioDB. Primary API is tried first; falls back to the other if not found.</p>
	
				<div class="flex items-center justify-between gap-4">
					<label class="text-sm">Primary API</label>
					<select
					class="flex-1 border rounded px-3 py-2 text-sm bg-background"
					value={settings["enrich_primary_api"] ?? "musicbrainz"}
					onchange={(e) => saveSetting("enrich_primary_api", (e.target as HTMLSelectElement).value)}
					>
					<option value="musicbrainz">MusicBrainz</option>
					<option value="audiodb">TheAudioDB</option>
					</select>
				</div>
	
				<div class="space-y-1">
					<label class="text-sm font-medium">TheAudioDB API Key</label>
					<p class="text-xs text-muted-foreground">Leave blank to use the free tier.</p>
					<input
					class="w-full border rounded px-3 py-2 text-sm bg-background"
					placeholder="Pro API key (optional)"
					value={settings["api_audiodb_key"] ?? ""}
					onchange={(e) => saveSetting("api_audiodb_key", (e.target as HTMLInputElement).value)}
					/>
				</div>
				</div>
	
				<div class="space-y-2">
				<h2 class="text-sm font-semibold">Enrichment Field Priority</h2>
				<p class="text-xs text-muted-foreground">"Local" keeps your existing value and only fills blanks. "API" overwrites with API data if available.</p>
				{#each [
					{ key: "enrich_priority_title", label: "Title" },
					{ key: "enrich_priority_artists", label: "Artists" },
					{ key: "enrich_priority_album_artist", label: "Album Artist" },
					{ key: "enrich_priority_album", label: "Album" },
					{ key: "enrich_priority_year", label: "Year" },
					{ key: "enrich_priority_genres", label: "Genres" },
					{ key: "enrich_priority_bpm", label: "BPM" },
					{ key: "enrich_priority_key", label: "Key" },
					{ key: "enrich_priority_artwork", label: "Artwork" },
				] as { key, label }}
					<div class="flex items-center justify-between gap-4">
					<label class="text-sm w-28">{label}</label>
					<select
						class="flex-1 border rounded px-3 py-2 text-sm bg-background"
						value={settings[key] ?? "local"}
						onchange={(e) => saveSetting(key, (e.target as HTMLSelectElement).value)}
					>
						<option value="local">Local (fill blanks only)</option>
						<option value="api">API (overwrite)</option>
					</select>
					</div>
				{/each}
				</div>
			</div>
		</div>
	</ScrollArea>
</div>