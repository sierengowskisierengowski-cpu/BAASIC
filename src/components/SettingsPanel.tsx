import { useEffect, useState } from "react";
import type { AppConfig } from "../types";
import { api } from "../lib/api";

export function SettingsPanel({
  config,
  onSave,
}: {
  config: AppConfig;
  onSave: (c: AppConfig) => Promise<void>;
}) {
  const [key, setKey] = useState(config.acoustid_api_key ?? "");
  const [downloadFolder, setDownloadFolder] = useState(config.download_folder ?? "");
  const [autoImport, setAutoImport] = useState(config.auto_import_downloads);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (!downloadFolder) {
      void api.getDefaultDownloadDir().then(setDownloadFolder);
    }
  }, [downloadFolder]);

  const save = async () => {
    await onSave({
      ...config,
      acoustid_api_key: key || null,
      download_folder: downloadFolder || null,
      auto_import_downloads: autoImport,
    });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="max-w-xl space-y-6">
      <section className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-6">
        <h2 className="text-lg font-semibold">AcoustID API Key</h2>
        <p className="mt-2 text-sm text-baasic-muted">
          Auto-identifies songs with blank tags via audio fingerprint. Free at{" "}
          <a href="https://acoustid.org/new-application" className="text-baasic-gold hover:underline">
            acoustid.org
          </a>
        </p>
        <input
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder="Your AcoustID API key"
          className="mt-4 w-full rounded-lg border border-baasic-border bg-baasic-panel px-3 py-2 text-sm outline-none focus:border-baasic-gold"
        />
      </section>

      <section className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-6">
        <h2 className="text-lg font-semibold">Download folder</h2>
        <p className="mt-2 text-sm text-baasic-muted">
          Where Get Music commands save files. BAASIC can auto-import new downloads on startup.
        </p>
        <input
          value={downloadFolder}
          onChange={(e) => setDownloadFolder(e.target.value)}
          className="mt-4 w-full rounded-lg border border-baasic-border bg-baasic-panel px-3 py-2 text-sm outline-none focus:border-baasic-gold"
        />
        <label className="mt-4 flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={autoImport}
            onChange={(e) => setAutoImport(e.target.checked)}
            className="accent-baasic-gold"
          />
          Auto-import download folder on startup (skips duplicates, fingerprints unknown tracks)
        </label>
        <button
          onClick={() => void save()}
          className="mt-4 rounded-full bg-baasic-gold px-5 py-2 text-sm font-bold text-black"
        >
          {saved ? "Saved!" : "Save Settings"}
        </button>
      </section>

      <section className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-6 text-sm text-baasic-muted">
        <h3 className="font-medium text-baasic-text">Keyboard shortcuts</h3>
        <ul className="mt-3 space-y-1">
          <li><kbd className="rounded bg-baasic-panel px-1.5">Space</kbd> Play / Pause</li>
          <li><kbd className="rounded bg-baasic-panel px-1.5">→</kbd> Next track</li>
          <li><kbd className="rounded bg-baasic-panel px-1.5">←</kbd> Previous track</li>
        </ul>
      </section>
    </div>
  );
}
