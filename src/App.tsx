import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { scanFolder } from "./lib/api";
import type { ScanResult } from "./types";
import { RecordingList } from "./components/RecordingList";
import { PairDetails } from "./components/PairDetails";
import { useKeyboardPairNav } from "./hooks/useKeyboardPairNav";

const DEFAULT_THRESHOLD_MS = 3000;

export default function App() {
  const [activeFolder, setActiveFolder] = useState<string | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [selectedPairId, setSelectedPairId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const assetsById = useMemo(() => {
    const map = new Map<string, ScanResult["assets"][number]>();
    for (const asset of scanResult?.assets ?? []) {
      map.set(asset.id, asset);
    }
    return map;
  }, [scanResult]);

  const pairs = scanResult?.pairs ?? [];
  const selectedPair = pairs.find((pair) => pair.id === selectedPairId) ?? null;
  const scanState: "idle" | "scanning" | "loaded" | "error" = isScanning
    ? "scanning"
    : error
      ? "error"
      : scanResult
        ? "loaded"
        : "idle";

  async function pickFolderAndScan() {
    setError(null);
    const folderPath = await open({
      directory: true,
      multiple: false,
      title: "Select INNOVV K6 footage folder"
    });

    if (!folderPath || Array.isArray(folderPath)) return;
    setActiveFolder(folderPath);
    setIsScanning(true);

    try {
      const previousSelectedPairId = selectedPairId;
      const result = await scanFolder({
        rootPath: folderPath,
        recursive: true,
        pairingThresholdMs: DEFAULT_THRESHOLD_MS
      });
      setScanResult(result);
      const keepSelection = previousSelectedPairId && result.pairs.some((pair) => pair.id === previousSelectedPairId);
      setSelectedPairId(keepSelection ? previousSelectedPairId : (result.pairs[0]?.id ?? null));
    } catch (scanError) {
      setError(scanError instanceof Error ? scanError.message : String(scanError));
    } finally {
      setIsScanning(false);
    }
  }

  function selectRelative(offset: number) {
    if (pairs.length === 0) return;
    const currentIndex = selectedPairId ? pairs.findIndex((pair) => pair.id === selectedPairId) : -1;
    const nextIndex = Math.min(Math.max(currentIndex + offset, 0), pairs.length - 1);
    setSelectedPairId(pairs[nextIndex].id);
  }

  useKeyboardPairNav({
    canNavigate: pairs.length > 0,
    onNext: () => selectRelative(1),
    onPrev: () => selectRelative(-1)
  });

  return (
    <div className="app-shell">
      <header className="topbar">
        <div>
          <h1>k6player</h1>
          <p>Fast paired browser for INNOVV K6 front/rear footage.</p>
        </div>
        <div className="toolbar">
          <button type="button" onClick={pickFolderAndScan} disabled={isScanning}>
            {isScanning ? "Scanning..." : "Open Folder"}
          </button>
          <button type="button" onClick={() => selectRelative(-1)} disabled={pairs.length === 0}>
            Prev
          </button>
          <button type="button" onClick={() => selectRelative(1)} disabled={pairs.length === 0}>
            Next
          </button>
        </div>
      </header>

      <div className="status-row">
        <span>Folder: {activeFolder ?? "No folder selected"}</span>
        <span>Scan state: {scanState}</span>
        <span>Pairs: {pairs.length}</span>
        <span>Shortcuts: J/K or Up/Down for previous/next pair</span>
      </div>

      {scanResult ? (
        <div className="status-row diagnostics-row">
          <span>Files: {scanResult.diagnostics.totalFilesDiscovered}</span>
          <span>Parser matches: {scanResult.diagnostics.parserMatchedFiles}</span>
          <span>Parser skipped: {scanResult.diagnostics.parserSkippedFiles}</span>
          <span>Parser failed: {scanResult.diagnostics.parserFailedFiles}</span>
          <span>Full pairs: {scanResult.diagnostics.validPairs}</span>
          <span>Partial pairs: {scanResult.diagnostics.partialPairs}</span>
        </div>
      ) : null}

      {error ? <div className="error-banner">{error}</div> : null}
      {!error && (scanResult?.errors.length ?? 0) > 0 ? (
        <div className="error-banner">Scan diagnostics: {scanResult?.errors.join(" | ")}</div>
      ) : null}

      <main className="main-layout">
        <RecordingList pairs={pairs} selectedPairId={selectedPairId} onSelectPair={setSelectedPairId} />
        <PairDetails pair={selectedPair} assetsById={assetsById} />
      </main>
    </div>
  );
}
