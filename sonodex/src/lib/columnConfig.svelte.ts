export type ColumnKey =
	| "number"
	| "artwork"
	| "title"
	| "album"
	| "year"
	| "rating"
	| "duration"
	| "label"
	| "options"

export type ColumnState = {
	visible: Record<ColumnKey, boolean>
}

export const ALL_COLUMNS: ColumnKey[] = [
	"number",
	"artwork",
	"title",
	"album",
	"year",
	"rating",
	"duration",
	"label",
	"options",
]

export const COLUMN_LABELS: Record<ColumnKey, string> = {
	number: "#",
	artwork: "Artwork",
	title: "Title",
	album: "Album",
	year: "Year",
	rating: "Rating",
	duration: "Duration",
	label: "Label",
	options: "Options",
}

export const ALWAYS_VISIBLE: ColumnKey[] = ["options"]

const PRESETS: Record<string, Partial<Record<ColumnKey, boolean>>> = {
	default: {
		artwork: true,
		title: true,
		duration: true,
	},
	library: {
		artwork: true,
		title: true,
		album: true,
		year: true,
		rating: true,
		duration: true,
	},
	album: {
		number: true,
		title: true,
		rating: true,
		duration: true,
	},
	playlist: {
		number: true,
		title: true,
		album: true,
		duration: true,
	},
	simple: {
		artwork: true,
		title: true,
	},
}

export function createColumnState(preset: keyof typeof PRESETS = "default"): ColumnState {
	const base = PRESETS[preset] ?? PRESETS.default
	const visible = $state(
		Object.fromEntries(
			ALL_COLUMNS.map((col) => [
				col,
				ALWAYS_VISIBLE.includes(col) ? true : (base[col] ?? false),
			])
		) as Record<ColumnKey, boolean>
	)
	return { visible }
}