export const artworkCache = new Map<string, string | null>();
export const artworkInflight = new Map<string, Promise<string | null>>();