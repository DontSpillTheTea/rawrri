import type { RecordingPair } from "../types";
import { fmtDate, fmtDuration } from "../lib/format";

interface RecordingListProps {
  pairs: RecordingPair[];
  selectedPairId: string | null;
  onSelectPair: (pairId: string) => void;
}

export function RecordingList({ pairs, selectedPairId, onSelectPair }: RecordingListProps) {
  return (
    <div className="panel list-panel">
      <div className="panel-title">Recordings ({pairs.length})</div>
      {pairs.length === 0 ? (
        <div className="empty-state">No valid K6 MP4 recordings found in this folder yet.</div>
      ) : null}
      <div className="list-wrap">
        {pairs.map((pair) => {
          const isSelected = pair.id === selectedPairId;
          const hasFront = Boolean(pair.frontAssetId);
          const hasRear = Boolean(pair.rearAssetId);
          return (
            <button
              key={pair.id}
              type="button"
              className={`list-row ${isSelected ? "selected" : ""}`}
              onClick={() => onSelectPair(pair.id)}
            >
              <div className="list-row-main">
                <span className="pair-time">{fmtDate(pair.canonicalStartTime)}</span>
                <span className="pair-duration">{fmtDuration(pair.estimatedDurationSec)}</span>
              </div>
              <div className="list-row-sub">
                <span className={`badge ${hasFront ? "ok" : "warn"}`}>F {hasFront ? "yes" : "missing"}</span>
                <span className={`badge ${hasRear ? "ok" : "warn"}`}>R {hasRear ? "yes" : "missing"}</span>
                <span>{pair.warnings.length > 0 ? `${pair.warnings.length} warning(s)` : "Healthy"}</span>
              </div>
              {pair.warnings.length > 0 ? <div className="pair-warning">{pair.warnings.join(", ")}</div> : null}
            </button>
          );
        })}
      </div>
    </div>
  );
}
