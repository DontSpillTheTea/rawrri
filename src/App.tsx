import { useEffect, useMemo, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  playbackGetState,
  playbackLoadPair,
  playbackSeek,
  playbackSetPlaying,
  playbackStop,
  playbackTogglePlayPause,
  scanFolder,
  updateVideoLayout
} from "./lib/api";
import { fmtDuration } from "./lib/format";
import type { PlaybackSnapshot, ScanResult, VideoRect, VideoSurfaceSnapshot } from "./types";
import { RecordingList } from "./components/RecordingList";
import { PairDetails } from "./components/PairDetails";
import { useKeyboardPairNav } from "./hooks/useKeyboardPairNav";

const DEFAULT_THRESHOLD_MS = 3000;
const SMALL_SEEK_SECONDS = 2;
const LARGE_SEEK_SECONDS = 10;
const SLIDER_DEBOUNCE_MS = 50;
const PLAYBACK_POLL_MS = 250;

export default function App() {
  const [activeFolder, setActiveFolder] = useState<string | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [selectedPairId, setSelectedPairId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [playbackError, setPlaybackError] = useState<string | null>(null);
  const [playback, setPlayback] = useState<PlaybackSnapshot | null>(null);
  const [surface, setSurface] = useState<VideoSurfaceSnapshot | null>(null);
  const [sliderPlayheadSec, setSliderPlayheadSec] = useState(0);
  const [isSliderDragging, setIsSliderDragging] = useState(false);
  const seekDebounceRef = useRef<number | null>(null);
  const layoutDebounceRef = useRef<number | null>(null);

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

  useEffect(() => {
    if (!isSliderDragging) {
      setSliderPlayheadSec(playback?.playheadSec ?? 0);
    }
  }, [playback?.playheadSec, isSliderDragging]);

  useEffect(() => {
    if (!selectedPair) {
      void playbackStop()
        .then((snapshot) => {
          setPlayback(snapshot);
        })
        .catch((playErr) => {
          setPlaybackError(playErr instanceof Error ? playErr.message : String(playErr));
        });
      return;
    }

    const frontPath = selectedPair.frontAssetId ? (assetsById.get(selectedPair.frontAssetId)?.path ?? null) : null;
    const rearPath = selectedPair.rearAssetId ? (assetsById.get(selectedPair.rearAssetId)?.path ?? null) : null;

    setPlaybackError(null);
    void playbackLoadPair({
      pairId: selectedPair.id,
      frontPath,
      rearPath
    })
      .then((snapshot) => {
        setPlayback(snapshot);
      })
      .catch((playErr) => {
        setPlaybackError(playErr instanceof Error ? playErr.message : String(playErr));
      });
  }, [selectedPair, assetsById]);

  useEffect(() => {
    if (!selectedPair) return;
    const timer = window.setInterval(() => {
      void playbackGetState()
        .then((snapshot) => {
          setPlayback(snapshot);
          if (snapshot.lastError) {
            setPlaybackError(snapshot.lastError);
          }
        })
        .catch((playErr) => {
          setPlaybackError(playErr instanceof Error ? playErr.message : String(playErr));
        });
    }, PLAYBACK_POLL_MS);
    return () => window.clearInterval(timer);
  }, [selectedPair?.id]);

  async function pickFolderAndScan() {
    setError(null);
    const folderPath = await open({
      directory: true,
      multiple: false,
      title: "Select dashcam footage folder"
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

  async function togglePlayPause() {
    setPlaybackError(null);
    try {
      const snapshot = await playbackTogglePlayPause();
      setPlayback(snapshot);
    } catch (playErr) {
      setPlaybackError(playErr instanceof Error ? playErr.message : String(playErr));
    }
  }

  async function seekTo(playheadSec: number) {
    setPlaybackError(null);
    try {
      const snapshot = await playbackSeek(Math.max(0, playheadSec));
      setPlayback(snapshot);
    } catch (playErr) {
      setPlaybackError(playErr instanceof Error ? playErr.message : String(playErr));
    }
  }

  async function seekRelative(offsetSec: number) {
    const base = playback?.playheadSec ?? 0;
    await seekTo(base + offsetSec);
  }

  useKeyboardPairNav({
    canNavigate: pairs.length > 0,
    onNext: () => selectRelative(1),
    onPrev: () => selectRelative(-1)
  });

  useEffect(() => {
    return () => {
      if (seekDebounceRef.current !== null) {
        window.clearTimeout(seekDebounceRef.current);
      }
      if (layoutDebounceRef.current !== null) {
        window.clearTimeout(layoutDebounceRef.current);
      }
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const isInput = target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA");
      if (isInput) return;

      if (event.code === "Space") {
        event.preventDefault();
        void togglePlayPause();
        return;
      }

      if (event.code === "ArrowLeft") {
        event.preventDefault();
        void seekRelative(event.shiftKey ? -LARGE_SEEK_SECONDS : -SMALL_SEEK_SECONDS);
        return;
      }

      if (event.code === "ArrowRight") {
        event.preventDefault();
        void seekRelative(event.shiftKey ? LARGE_SEEK_SECONDS : SMALL_SEEK_SECONDS);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [playback]);

  function onSliderInput(nextValue: number) {
    setIsSliderDragging(true);
    setSliderPlayheadSec(nextValue);
    if (seekDebounceRef.current !== null) {
      window.clearTimeout(seekDebounceRef.current);
    }
    seekDebounceRef.current = window.setTimeout(() => {
      void seekTo(nextValue);
    }, SLIDER_DEBOUNCE_MS);
  }

  function onSliderCommit() {
    setIsSliderDragging(false);
    if (seekDebounceRef.current !== null) {
      window.clearTimeout(seekDebounceRef.current);
      seekDebounceRef.current = null;
    }
    void seekTo(sliderPlayheadSec);
  }

  async function setPlaying(isPlaying: boolean) {
    setPlaybackError(null);
    try {
      const snapshot = await playbackSetPlaying(isPlaying);
      setPlayback(snapshot);
    } catch (playErr) {
      setPlaybackError(playErr instanceof Error ? playErr.message : String(playErr));
    }
  }

  function handleVideoLayoutChange(front: VideoRect, rear: VideoRect) {
    const dpr = window.devicePixelRatio || 1;
    const nativeFront: VideoRect = {
      x: Math.round(front.x * dpr),
      y: Math.round(front.y * dpr),
      width: Math.round(front.width * dpr),
      height: Math.round(front.height * dpr)
    };
    const nativeRear: VideoRect = {
      x: Math.round(rear.x * dpr),
      y: Math.round(rear.y * dpr),
      width: Math.round(rear.width * dpr),
      height: Math.round(rear.height * dpr)
    };
    if (layoutDebounceRef.current !== null) {
      window.clearTimeout(layoutDebounceRef.current);
    }
    layoutDebounceRef.current = window.setTimeout(() => {
      void updateVideoLayout(nativeFront, nativeRear)
        .then((snapshot) => {
          setSurface(snapshot);
        })
        .catch((layoutErr) => {
          setPlaybackError(layoutErr instanceof Error ? layoutErr.message : String(layoutErr));
        });
    }, 30);
  }

  return (
    <div className="app-shell">
      <header className="topbar">
        <div>
          <h1>rawrii</h1>
          <p>Fast paired browser for front/rear dashcam footage.</p>
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
      {playbackError ? <div className="error-banner">Playback: {playbackError}</div> : null}

      <div className="panel transport-panel">
        <div className="panel-title">Playback Transport</div>
        <div className="transport-row">
          <button type="button" onClick={() => void setPlaying(true)} disabled={!selectedPair}>
            Play
          </button>
          <button type="button" onClick={() => void setPlaying(false)} disabled={!selectedPair}>
            Pause
          </button>
          <button type="button" onClick={() => void seekRelative(-SMALL_SEEK_SECONDS)} disabled={!selectedPair}>
            -2s
          </button>
          <button type="button" onClick={() => void seekRelative(SMALL_SEEK_SECONDS)} disabled={!selectedPair}>
            +2s
          </button>
          <button type="button" onClick={() => void seekRelative(-LARGE_SEEK_SECONDS)} disabled={!selectedPair}>
            -10s
          </button>
          <button type="button" onClick={() => void seekRelative(LARGE_SEEK_SECONDS)} disabled={!selectedPair}>
            +10s
          </button>
          <span>
            Status: {playback?.isPlaying ? "playing" : "paused"} | Playhead:{" "}
            {fmtDuration(playback?.playheadSec ?? 0)} / {fmtDuration(playback?.pairDurationSec ?? 0)}
          </span>
        </div>
        <div className="status-row diagnostics-row">
          <span>
            Front: {fmtDuration(playback?.frontTimeSec ?? 0)} / {fmtDuration(playback?.frontDurationSec ?? 0)}
          </span>
          <span>
            Rear: {fmtDuration(playback?.rearTimeSec ?? 0)} / {fmtDuration(playback?.rearDurationSec ?? 0)}
          </span>
          <span>Sync delta: {(playback?.syncDeltaSec ?? 0).toFixed(2)}s</span>
        </div>
        <input
          className="scrub-slider"
          type="range"
          min={0}
          max={Math.max(playback?.pairDurationSec ?? 0, 1)}
          step={0.1}
          value={sliderPlayheadSec}
          onChange={(event) => onSliderInput(Number(event.target.value))}
          onMouseUp={onSliderCommit}
          onTouchEnd={onSliderCommit}
          disabled={!selectedPair}
        />
        <div className="status-row">
          <span>Space: play/pause</span>
          <span>Left/Right: +/-2s</span>
          <span>Shift+Left/Right: +/-10s</span>
        </div>
      </div>

      <div className="panel transport-panel">
        <div className="panel-title">Embedded Surface Debug</div>
        <div className="status-row diagnostics-row">
          <span>Embedded: {surface?.frontWid || surface?.rearWid ? "yes" : "no"}</span>
          <span>Front wid: {surface?.frontWid ?? "n/a"}</span>
          <span>Rear wid: {surface?.rearWid ?? "n/a"}</span>
          <span>Front visible: {surface?.frontVisible ? "yes" : "no"}</span>
          <span>Rear visible: {surface?.rearVisible ? "yes" : "no"}</span>
          <span>Debug host visuals: {surface?.debugVisualHosts ? "on" : "off"}</span>
        </div>
      </div>

      <main className="main-layout">
        <RecordingList pairs={pairs} selectedPairId={selectedPairId} onSelectPair={setSelectedPairId} />
        <PairDetails
          pair={selectedPair}
          assetsById={assetsById}
          playback={playback}
          onVideoLayoutChange={handleVideoLayoutChange}
        />
      </main>
    </div>
  );
}
