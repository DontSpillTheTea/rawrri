import type { PlaybackSnapshot, RecordingPair, VideoAsset } from "../types";
import { fmtBytes, fmtDate } from "../lib/format";

interface PairDetailsProps {
  pair: RecordingPair | null;
  assetsById: Map<string, VideoAsset>;
  playback: PlaybackSnapshot | null;
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

export function PairDetails({ pair, assetsById, playback }: PairDetailsProps) {
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
        <div className="video-placeholder">
          {playback?.frontLoaded ? "Front playing in external mpv window" : "Front side unavailable"}
        </div>
        <div className="video-placeholder">
          {playback?.rearLoaded ? "Rear playing in external mpv window" : "Rear side unavailable"}
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
