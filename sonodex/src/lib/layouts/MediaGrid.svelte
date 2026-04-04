<div class="app-music-grid grid gap-2"><slot /></div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>