import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { ImportProgress } from "../types";

export function ImportPanel({ onDone }: { onDone: () => Promise<void> }) {
  const [progress, setProgress] = useState<ImportProgress | null>(null);
  const [lastResult, setLastResult] = useState<string | null>(null);

  useEffect(() => {
    const id = setInterval(async () => {
      const p = await api.getImportProgress();
      setProgress(p);
    }, 500);
    return () => clearInterval(id);
  }, []);

  const pickAndImport = async () => {
    const selected = await api.pickFolder("Select music folder to import");
    if (!selected) return;
    setLastResult(null);
    const result = await api.importMusicFolder(selected) as {
      imported: number;
      skipped_duplicates: number;
      tagged: number;
      errors: string[];
    };
    setLastResult(
      `Imported ${result.imported}, skipped ${result.skipped_duplicates} duplicates, auto-tagged ${result.tagged}.` +
        (result.errors.length ? ` ${result.errors.length} errors.` : ""),
    );
    await onDone();
  };

  return (
    <div className="max-w-2xl space-y-6">
      <section className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-6">
        <h2 className="text-lg font-semibold">Import Music Folder</h2>
        <p className="mt-2 text-sm text-baasic-muted">
          BAASIC scans your folder, skips duplicate files (by hash and audio fingerprint),
          fingerprints unknown tracks, fetches metadata from AcoustID/MusicBrainz, and downloads album art.
        </p>
        <button
          onClick={() => void pickAndImport()}
          className="mt-4 rounded-full bg-baasic-gold px-6 py-2.5 text-sm font-bold text-black"
        >
          Choose Folder to Import
        </button>
      </section>

      {progress?.active && (
        <section className="rounded-xl border border-baasic-gold/30 bg-baasic-panel-2 p-6">
          <div className="text-sm font-medium text-baasic-gold">{progress.phase}</div>
          <div className="mt-1 truncate text-xs text-baasic-muted">{progress.current_file}</div>
          <div className="mt-3 h-2 overflow-hidden rounded-full bg-baasic-panel">
            <div
              className="h-full bg-baasic-gold transition-all"
              style={{ width: `${progress.total ? (progress.processed / progress.total) * 100 : 0}%` }}
            />
          </div>
          <div className="mt-2 text-xs text-baasic-muted">
            {progress.processed}/{progress.total} · imported {progress.imported} · skipped {progress.skipped_duplicates} · tagged {progress.tagged}
          </div>
        </section>
      )}

      {lastResult && (
        <section className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-4 text-sm">
          {lastResult}
        </section>
      )}

      <section className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-6 text-sm text-baasic-muted">
        <h3 className="font-medium text-baasic-text">What happens on import</h3>
        <ul className="mt-3 list-disc space-y-1 pl-5">
          <li>Exact duplicate files are skipped (SHA-256 hash)</li>
          <li>Same song with different filename is caught via Chromaprint fingerprint</li>
          <li>Blank tags get identified via AcoustID (set API key in Settings)</li>
          <li>Album art is downloaded from Cover Art Archive when available</li>
        </ul>
      </section>
    </div>
  );
}
