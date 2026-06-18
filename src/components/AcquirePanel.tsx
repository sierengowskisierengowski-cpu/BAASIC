import { useEffect, useState } from "react";
import { Check, Copy, Terminal } from "lucide-react";
import { api } from "../lib/api";
import type { AcquireCommand, AcquireTool } from "../types";

export function AcquirePanel({ onImportFolder }: { onImportFolder: () => Promise<void> }) {
  const [query, setQuery] = useState("");
  const [mode, setMode] = useState<"song" | "album" | "artist">("song");
  const [outputDir, setOutputDir] = useState("");
  const [tools, setTools] = useState<AcquireTool[]>([]);
  const [commands, setCommands] = useState<AcquireCommand[]>([]);
  const [copied, setCopied] = useState<string | null>(null);

  useEffect(() => {
    void api.getDefaultDownloadDir().then(setOutputDir);
    void api.getAcquireTools().then(setTools);
  }, []);

  useEffect(() => {
    if (!query.trim()) {
      setCommands([]);
      return;
    }
    void api.getAcquireCommands(query, mode, outputDir || undefined).then(setCommands);
  }, [query, mode, outputDir]);

  const copy = async (cmd: string, id: string) => {
    await navigator.clipboard.writeText(cmd);
    setCopied(id);
    setTimeout(() => setCopied(null), 2000);
  };

  return (
    <div className="max-w-3xl space-y-6">
      <section className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-6">
        <h2 className="text-lg font-semibold">Get Music — Terminal Command Builder</h2>
        <p className="mt-2 text-sm text-baasic-muted">
          Search for a song, album, or artist. BAASIC builds the exact terminal command for tools you
          already have installed (like <code className="text-baasic-gold">yt-dlp</code>). Copy it, run
          it in your terminal, then import the download folder.
        </p>
        <p className="mt-2 text-xs text-baasic-muted">
          Only download music you have the right to access. BAASIC does not download for you — it
          gives you the command to run yourself.
        </p>

        <div className="mt-4 flex flex-wrap gap-2">
          {(["song", "album", "artist"] as const).map((m) => (
            <button
              key={m}
              onClick={() => setMode(m)}
              className={`rounded-full px-4 py-1.5 text-sm capitalize ${
                mode === m ? "bg-baasic-gold text-black" : "border border-baasic-border text-baasic-muted"
              }`}
            >
              {m}
            </button>
          ))}
        </div>

        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder='e.g. "Radiohead Creep" or paste a YouTube/Bandcamp URL'
          className="mt-4 w-full rounded-xl border border-baasic-border bg-baasic-panel px-4 py-3 text-sm outline-none focus:border-baasic-gold"
        />

        <label className="mt-3 block text-xs text-baasic-muted">Download folder</label>
        <input
          value={outputDir}
          onChange={(e) => setOutputDir(e.target.value)}
          className="mt-1 w-full rounded-lg border border-baasic-border bg-baasic-panel px-3 py-2 text-sm outline-none focus:border-baasic-gold"
        />
      </section>

      <section className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-6">
        <h3 className="text-sm font-semibold">Detected tools</h3>
        <div className="mt-3 flex flex-wrap gap-2">
          {tools.map((t) => (
            <span
              key={t.id}
              className={`rounded-full px-3 py-1 text-xs ${
                t.installed ? "bg-green-500/15 text-green-400" : "bg-red-500/15 text-red-400"
              }`}
            >
              {t.name} {t.installed ? `✓ ${t.version}` : "✗ not installed"}
            </span>
          ))}
        </div>
        {!tools.some((t) => t.installed) && (
          <p className="mt-3 text-sm text-baasic-muted">
            Install yt-dlp: <code className="text-baasic-gold">sudo pacman -S yt-dlp ffmpeg</code>
          </p>
        )}
      </section>

      {commands.map((cmd) => (
        <section key={cmd.label} className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-6">
          <div className="flex items-center justify-between gap-3">
            <h3 className="font-medium">{cmd.label}</h3>
            <button
              onClick={() => void copy(cmd.command, cmd.label)}
              className="flex items-center gap-1 rounded-lg border border-baasic-border px-3 py-1.5 text-xs hover:border-baasic-gold"
            >
              {copied === cmd.label ? <Check size={14} /> : <Copy size={14} />}
              {copied === cmd.label ? "Copied!" : "Copy command"}
            </button>
          </div>

          <div className="mt-3 flex items-start gap-2 rounded-lg bg-black/40 p-4 font-mono text-xs leading-relaxed text-green-300">
            <Terminal size={16} className="mt-0.5 shrink-0 text-baasic-muted" />
            <pre className="overflow-x-auto whitespace-pre-wrap">{cmd.command}</pre>
          </div>

          <p className="mt-3 text-sm text-baasic-muted">{cmd.import_hint}</p>
          <button
            onClick={() => void onImportFolder()}
            className="mt-3 text-sm text-baasic-gold hover:underline"
          >
            → Import folder after download
          </button>
        </section>
      ))}
    </div>
  );
}
