import { convertFileSrc } from "@tauri-apps/api/core";
import type { AlbumGroup } from "../types";

export function AlbumGrid({
  albums,
  onSelect,
}: {
  albums: AlbumGroup[];
  onSelect: (album: AlbumGroup) => void;
}) {
  if (!albums.length) {
    return <p className="text-baasic-muted">No albums yet. Import some music.</p>;
  }

  return (
    <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
      {albums.map((album) => {
        const art = album.art_path ? convertFileSrc(album.art_path) : null;
        return (
          <button
            key={`${album.artist}-${album.name}`}
            onClick={() => onSelect(album)}
            className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-3 text-left transition hover:border-baasic-gold"
          >
            <div className="mb-3 aspect-square overflow-hidden rounded-lg bg-baasic-panel">
              {art ? (
                <img src={art} alt="" className="h-full w-full object-cover" />
              ) : (
                <div className="flex h-full items-center justify-center text-3xl font-black text-baasic-gold/40">B</div>
              )}
            </div>
            <div className="truncate font-medium">{album.name}</div>
            <div className="truncate text-sm text-baasic-muted">{album.artist}</div>
            <div className="mt-1 text-xs text-baasic-muted">
              {album.track_count} tracks{album.year ? ` · ${album.year}` : ""}
            </div>
          </button>
        );
      })}
    </div>
  );
}
