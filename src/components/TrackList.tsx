import { convertFileSrc } from "@tauri-apps/api/core";
import { Heart, MoreHorizontal, Play } from "lucide-react";
import type { Playlist, Track } from "../types";
import { formatDuration } from "../lib/api";
import { useState } from "react";

interface Props {
  tracks: Track[];
  currentId: number | null;
  playlists: Playlist[];
  onPlay: (track: Track, list?: Track[]) => void;
  onToggleFavorite: (track: Track) => void;
  onAddToPlaylist: (playlistId: number, trackId: number) => void;
}

export function TrackList({ tracks, currentId, playlists, onPlay, onToggleFavorite, onAddToPlaylist }: Props) {
  const [menuTrack, setMenuTrack] = useState<number | null>(null);

  if (!tracks.length) {
    return (
      <div className="flex h-64 flex-col items-center justify-center text-baasic-muted">
        <div className="text-4xl font-black text-baasic-gold/30">B</div>
        <p className="mt-3">No tracks here yet.</p>
        <p className="text-sm">Import a folder or use Get Music to add songs.</p>
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-xl border border-baasic-border">
      <div className="grid grid-cols-[40px_1fr_1fr_1fr_80px_80px] gap-2 border-b border-baasic-border bg-baasic-panel-2 px-4 py-2 text-xs font-semibold uppercase tracking-wide text-baasic-muted">
        <div>#</div>
        <div>Title</div>
        <div>Artist</div>
        <div>Album</div>
        <div>Time</div>
        <div></div>
      </div>
      {tracks.map((track, i) => {
        const active = track.id === currentId;
        const art = track.art_path ? convertFileSrc(track.art_path) : null;
        return (
          <div
            key={track.id}
            onDoubleClick={() => onPlay(track, tracks)}
            className={`group grid grid-cols-[40px_1fr_1fr_1fr_80px_80px] items-center gap-2 border-b border-baasic-border/50 px-4 py-2 text-sm last:border-0 ${
              active ? "bg-baasic-gold/10" : "hover:bg-baasic-panel-2"
            }`}
          >
            <div className="text-baasic-muted">
              <span className="group-hover:hidden">{track.track_number ?? i + 1}</span>
              <button className="hidden group-hover:block text-baasic-gold" onClick={() => onPlay(track, tracks)}>
                <Play size={14} fill="currentColor" />
              </button>
            </div>
            <div className="flex min-w-0 items-center gap-2">
              {art ? (
                <img src={art} alt="" className="h-8 w-8 rounded object-cover" />
              ) : (
                <div className="flex h-8 w-8 items-center justify-center rounded bg-baasic-panel text-xs font-bold text-baasic-gold">
                  B
                </div>
              )}
              <span className="truncate font-medium">{track.title}</span>
            </div>
            <div className="truncate text-baasic-muted">{track.artist}</div>
            <div className="truncate text-baasic-muted">{track.album}</div>
            <div className="text-baasic-muted">{formatDuration(track.duration_ms)}</div>
            <div className="flex items-center justify-end gap-1">
              <button
                onClick={() => void onToggleFavorite(track)}
                className={track.favorite ? "text-red-400" : "text-baasic-muted hover:text-red-400"}
              >
                <Heart size={14} fill={track.favorite ? "currentColor" : "none"} />
              </button>
              <div className="relative">
                <button onClick={() => setMenuTrack(menuTrack === track.id ? null : track.id)} className="text-baasic-muted">
                  <MoreHorizontal size={14} />
                </button>
                {menuTrack === track.id && (
                  <div className="absolute right-0 z-10 mt-1 w-44 rounded-lg border border-baasic-border bg-baasic-panel-2 py-1 shadow-xl">
                    {playlists.map((pl) => (
                      <button
                        key={pl.id}
                        className="block w-full px-3 py-2 text-left text-xs hover:bg-baasic-panel"
                        onClick={() => {
                          void onAddToPlaylist(pl.id, track.id);
                          setMenuTrack(null);
                        }}
                      >
                        Add to {pl.name}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
