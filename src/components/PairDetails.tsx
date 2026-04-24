import { useEffect, useRef } from "react";
import type { PlaybackSnapshot, RecordingPair, VideoAsset, VideoRect } from "../types";
import { fmtDate, fmtDuration } from "../lib/format";

interface PairDetailsProps {
  pair: RecordingPair | null;
  assetsById: Map<string, VideoAsset>;
  playback: PlaybackSnapshot | null;
  onVideoLayoutChange?: (front: VideoRect, rear: VideoRect) => void;
  pairStartLabel?: string;
  pairEndLabel?: string;
  prevGapLabel?: string;
  nextGapLabel?: string;
}

function AssetCard({
  label,
  asset,
  runtimeDurationSec
}: {
  label: string;
  asset: VideoAsset | null;
  runtimeDurationSec: number | null;
}) {
  return (
    <div className="asset-card">
      <h4>{label}</h4>
      {!asset ? (
        <div className="asset-missing">Missing</div>
      ) : (
        <div className="asset-details">
          <div className="asset-filename">{asset.filename}</div>
          <div>Start: {fmtDate(asset.parsedTimestamp)}</div>
          <div>Duration: {fmtDuration(runtimeDurationSec ?? asset.durationSec)}</div>
          {asset.metadata && (
            <div className="asset-metadata">
              <span className="badge info">{asset.metadata.width}x{asset.metadata.height}</span>
              <span className="badge info">{asset.metadata.codec}</span>
              {asset.metadata.hasAudio && <span className="badge ok">Audio</span>}
              <span className="badge info">{asset.metadata.streamCount} streams</span>
              {asset.metadata.isCorrupt && <span className="badge warn">Corrupt?</span>}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export function PairDetails({
  pair,
  assetsById,
  playback,
  onVideoLayoutChange,
  pairStartLabel,
  pairEndLabel,
  prevGapLabel,
  nextGapLabel
}: PairDetailsProps) {
  const frontSurfaceRef = useRef<HTMLDivElement | null>(null);
  const rearSurfaceRef = useRef<HTMLDivElement | null>(null);
  const lastSentFrontRef = useRef<VideoRect | null>(null);
  const lastSentRearRef = useRef<VideoRect | null>(null);

  useEffect(() => {
    if (!pair) return;
    if (!onVideoLayoutChange) return;
    const frontElement = frontSurfaceRef.current;
    const rearElement = rearSurfaceRef.current;
    if (!frontElement || !rearElement) return;
    lastSentFrontRef.current = null;
    lastSentRearRef.current = null;

    const emitLayout = () => {
      const frontRect = frontElement.getBoundingClientRect();
      const rearRect = rearElement.getBoundingClientRect();
      const nextFront = {
        x: Math.round(frontRect.x),
        y: Math.round(frontRect.y),
        width: Math.round(frontRect.width),
        height: Math.round(frontRect.height)
      };
      const nextRear = {
        x: Math.round(rearRect.x),
        y: Math.round(rearRect.y),
        width: Math.round(rearRect.width),
        height: Math.round(rearRect.height)
      };

      const unchanged =
        lastSentFrontRef.current &&
        lastSentRearRef.current &&
        lastSentFrontRef.current.x === nextFront.x &&
        lastSentFrontRef.current.y === nextFront.y &&
        lastSentFrontRef.current.width === nextFront.width &&
        lastSentFrontRef.current.height === nextFront.height &&
        lastSentRearRef.current.x === nextRear.x &&
        lastSentRearRef.current.y === nextRear.y &&
        lastSentRearRef.current.width === nextRear.width &&
        lastSentRearRef.current.height === nextRear.height;
      if (unchanged) return;

      lastSentFrontRef.current = nextFront;
      lastSentRearRef.current = nextRear;
      onVideoLayoutChange(nextFront, nextRear);
    };

    emitLayout();
    const observer = new ResizeObserver(() => emitLayout());
    observer.observe(frontElement);
    observer.observe(rearElement);
    window.addEventListener("resize", emitLayout);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", emitLayout);
    };
  }, [onVideoLayoutChange, pair, pair?.id]);

  if (!pair) {
    return (
      <div className="panel details-panel">
        <div className="panel-title">Playback Workspace</div>
        <div className="empty-state">Select a recording pair from the list.</div>
      </div>
    );
  }

  const front = pair.frontAssetId ? assetsById.get(pair.frontAssetId) ?? null : null;
  const rear = pair.rearAssetId ? assetsById.get(pair.rearAssetId) ?? null : null;

  return (
    <div className="detail-stack">
      <div className="panel playback-panel">
        <div className="panel-title">Playback Workspace</div>
        <div className="preview-grid">
          <div ref={frontSurfaceRef} className="video-placeholder">
            {playback?.frontLoaded ? "Front" : "Front unavailable"}
          </div>
          <div ref={rearSurfaceRef} className="video-placeholder">
            {playback?.rearLoaded ? "Rear" : "Rear unavailable"}
          </div>
        </div>
      </div>

      <div className="panel details-panel">
        <div className="panel-title">Current Pair Summary</div>
        <div className="pair-summary-grid">
          <div>Start: {pairStartLabel ?? fmtDate(pair.canonicalStartTime)}</div>
          <div>End: {pairEndLabel ?? "Unknown"}</div>
          <div>Front duration: {fmtDuration(playback?.frontDurationSec ?? null)}</div>
          <div>Rear duration: {fmtDuration(playback?.rearDurationSec ?? null)}</div>
          <div>Sync delta: {playback?.syncDeltaSec !== null ? `${playback?.syncDeltaSec.toFixed(2)}s` : "Unknown"}</div>
          <div>Previous: {prevGapLabel ?? "Unknown"}</div>
          <div>Next: {nextGapLabel ?? "Unknown"}</div>
        </div>
        <div className="pair-details-scroll">
          <div className="asset-grid">
            <AssetCard label="Front" asset={front} runtimeDurationSec={playback?.frontDurationSec ?? null} />
            <AssetCard label="Rear" asset={rear} runtimeDurationSec={playback?.rearDurationSec ?? null} />
          </div>
          <div className="pair-meta">
            <div>Pair reason: {pair.pairingReason}</div>
            <div>Source folder: {pair.sourceFolder}</div>
          </div>
          {pair.warnings.length > 0 ? <div className="pair-warning">{pair.warnings.join(", ")}</div> : null}
        </div>
      </div>
    </div>
  );
}
