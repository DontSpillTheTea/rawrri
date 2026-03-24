import { useEffect, useRef } from "react";
import type { PlaybackSnapshot, RecordingPair, VideoAsset, VideoRect } from "../types";
import { fmtBytes, fmtDate } from "../lib/format";

interface PairDetailsProps {
  pair: RecordingPair | null;
  assetsById: Map<string, VideoAsset>;
  playback: PlaybackSnapshot | null;
  onVideoLayoutChange?: (front: VideoRect, rear: VideoRect) => void;
}

function AssetCard({ label, asset }: { label: string; asset: VideoAsset | null }) {
  return (
    <div className="asset-card">
      <h4>{label}</h4>
      {!asset ? (
        <div className="asset-missing">Missing</div>
      ) : (
        <div className="asset-details">
          <div>{asset.filename}</div>
          <div>{fmtDate(asset.parsedTimestamp)}</div>
          <div>Parse status: {asset.parseStatus}</div>
          <div>Sequence: {asset.parsedSequence ?? "unknown"}</div>
          <div>Raw ts: {asset.rawTimestampString ?? "unknown"}</div>
          <div>{fmtBytes(asset.sizeBytes)}</div>
          <div>{asset.path}</div>
        </div>
      )}
    </div>
  );
}

export function PairDetails({ pair, assetsById, playback, onVideoLayoutChange }: PairDetailsProps) {
  const frontSurfaceRef = useRef<HTMLDivElement | null>(null);
  const rearSurfaceRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!onVideoLayoutChange) return;
    const frontElement = frontSurfaceRef.current;
    const rearElement = rearSurfaceRef.current;
    if (!frontElement || !rearElement) return;

    const emitLayout = () => {
      const frontRect = frontElement.getBoundingClientRect();
      const rearRect = rearElement.getBoundingClientRect();
      onVideoLayoutChange(
        {
          x: Math.round(frontRect.x),
          y: Math.round(frontRect.y),
          width: Math.round(frontRect.width),
          height: Math.round(frontRect.height)
        },
        {
          x: Math.round(rearRect.x),
          y: Math.round(rearRect.y),
          width: Math.round(rearRect.width),
          height: Math.round(rearRect.height)
        }
      );
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
  }, [onVideoLayoutChange, pair?.id]);

  if (!pair) {
    return (
      <div className="panel detail-panel">
        <div className="panel-title">Current Pair</div>
        <div className="empty-state">Select a recording pair from the list.</div>
      </div>
    );
  }

  const front = pair.frontAssetId ? assetsById.get(pair.frontAssetId) ?? null : null;
  const rear = pair.rearAssetId ? assetsById.get(pair.rearAssetId) ?? null : null;

  return (
    <div className="panel detail-panel">
      <div className="panel-title">Current Pair</div>
      <div className="pair-meta">
        <div>ID: {pair.id}</div>
        <div>Time: {fmtDate(pair.canonicalStartTime)}</div>
        <div>Reason: {pair.pairingReason}</div>
        <div>Source: {pair.sourceFolder}</div>
      </div>
      <div className="preview-grid">
        <div ref={frontSurfaceRef} className="video-placeholder">
          {playback?.frontLoaded ? "Front embedded mpv surface" : "Front side unavailable"}
        </div>
        <div ref={rearSurfaceRef} className="video-placeholder">
          {playback?.rearLoaded ? "Rear embedded mpv surface" : "Rear side unavailable"}
        </div>
      </div>
      <div className="asset-grid">
        <AssetCard label="Front" asset={front} />
        <AssetCard label="Rear" asset={rear} />
      </div>
      {pair.warnings.length > 0 ? <div className="pair-warning">{pair.warnings.join(", ")}</div> : null}
    </div>
  );
}
