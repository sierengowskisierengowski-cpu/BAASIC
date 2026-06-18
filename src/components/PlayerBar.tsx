import {
  Pause,
  Play,
  SkipBack,
  SkipForward,
  Shuffle,
  Repeat,
  Repeat1,
  Volume2,
  VolumeX,
} from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { usePlayer } from "../hooks/usePlayer";
import { formatDuration } from "../lib/api";

export function PlayerBar({ player }: { player: ReturnType<typeof usePlayer> }) {
  const {
    currentTrack,
    isPlaying,
    shuffle,
    repeat,
    volume,
    muted,
    position,
    duration,
    togglePlay,
    next,
    prev,
    seek,
    setVolume,
    setMuted,
    cycleRepeat,
    toggleShuffle,
  } = player;

  const art = currentTrack?.art_path ? convertFileSrc(currentTrack.art_path) : null;
  const progress = duration > 0 ? (position / duration) * 100 : 0;

  return (
    <footer className="border-t border-baasic-border bg-baasic-panel px-4 py-3">
      <div className="mb-2 flex items-center gap-3">
        <span className="w-10 text-right text-xs text-baasic-muted">{formatDuration(position * 1000)}</span>
        <input
          type="range"
          min={0}
          max={duration || 1}
          step={0.1}
          value={position}
          onChange={(e) => seek(Number(e.target.value))}
          className="h-1 flex-1 cursor-pointer accent-baasic-gold"
          style={{
            background: `linear-gradient(to right, #e8a838 ${progress}%, #2a2a3a ${progress}%)`,
          }}
        />
        <span className="w-10 text-xs text-baasic-muted">
          {formatDuration((duration || 0) * 1000)}
        </span>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex min-w-0 flex-1 items-center gap-3">
          <div className="h-12 w-12 shrink-0 overflow-hidden rounded-lg bg-baasic-panel-2">
            {art ? (
              <img src={art} alt="" className="h-full w-full object-cover" />
            ) : (
              <div className="flex h-full items-center justify-center font-bold text-baasic-gold">B</div>
            )}
          </div>
          <div className="min-w-0">
            <div className="truncate font-medium">{currentTrack?.title ?? "Nothing playing"}</div>
            <div className="truncate text-sm text-baasic-muted">
              {currentTrack ? `${currentTrack.artist} · ${currentTrack.album}` : "Select a track"}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={toggleShuffle}
            className={`rounded-full p-2 ${shuffle ? "text-baasic-gold" : "text-baasic-muted hover:text-baasic-text"}`}
            title="Shuffle"
          >
            <Shuffle size={18} />
          </button>
          <button onClick={prev} className="rounded-full p-2 text-baasic-muted hover:text-baasic-text" title="Previous">
            <SkipBack size={20} fill="currentColor" />
          </button>
          <button
            onClick={togglePlay}
            disabled={!currentTrack}
            className="flex h-10 w-10 items-center justify-center rounded-full bg-baasic-gold text-black disabled:opacity-40"
            title={isPlaying ? "Pause" : "Play"}
          >
            {isPlaying ? <Pause size={20} fill="currentColor" /> : <Play size={20} fill="currentColor" className="ml-0.5" />}
          </button>
          <button onClick={next} className="rounded-full p-2 text-baasic-muted hover:text-baasic-text" title="Next">
            <SkipForward size={20} fill="currentColor" />
          </button>
          <button
            onClick={cycleRepeat}
            className={`rounded-full p-2 ${repeat !== "off" ? "text-baasic-gold" : "text-baasic-muted hover:text-baasic-text"}`}
            title="Repeat"
          >
            {repeat === "one" ? <Repeat1 size={18} /> : <Repeat size={18} />}
          </button>
        </div>

        <div className="flex flex-1 items-center justify-end gap-2">
          <button onClick={() => setMuted(!muted)} className="text-baasic-muted hover:text-baasic-text">
            {muted || volume === 0 ? <VolumeX size={18} /> : <Volume2 size={18} />}
          </button>
          <input
            type="range"
            min={0}
            max={1}
            step={0.01}
            value={muted ? 0 : volume}
            onChange={(e) => {
              setMuted(false);
              setVolume(Number(e.target.value));
            }}
            className="w-24 accent-baasic-gold"
          />
        </div>
      </div>
    </footer>
  );
}
