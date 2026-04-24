import type { ObservationEvent } from "../types";
import { fmtDuration } from "../lib/format";

interface ObservationPanelProps {
  observations: ObservationEvent[];
  onSeek: (time: number) => void;
  isAnalyzing: boolean;
  onStartAnalysis: () => void;
}

export function ObservationPanel({ observations, onSeek, isAnalyzing, onStartAnalysis }: ObservationPanelProps) {
  return (
    <div className="panel observation-panel">
      <div className="panel-title">Observations</div>
      <div className="toolbar">
        <button type="button" onClick={onStartAnalysis} disabled={isAnalyzing}>
          {isAnalyzing ? "Analyzing..." : "Run Analysis"}
        </button>
      </div>
      <div className="list-wrap">
        {observations.length === 0 ? (
          <div className="empty-state">
            {isAnalyzing ? "Analysis in progress..." : "No observations found. Run analysis to detect vehicles and license plates."}
          </div>
        ) : (
          observations.map((obs) => (
            <button
              key={obs.id}
              type="button"
              className="list-row observation-row"
              onClick={() => onSeek(obs.startTimeSec)}
            >
              <div className="obs-time">[{fmtDuration(obs.startTimeSec)}]</div>
              <div className="obs-content">
                {renderObservationType(obs)}
                <div className="obs-meta">
                  <span className="confidence">{(obs.confidence * 100).toFixed(0)}% confidence</span>
                  {obs.isUserConfirmed && <span className="badge ok">Verified</span>}
                </div>
              </div>
            </button>
          ))
        )}
      </div>
    </div>
  );
}

function renderObservationType(obs: ObservationEvent) {
  const type = obs.observationType;
  if ('vehicle' in type) {
    return (
      <div className="obs-type">
        🚗 {type.vehicle.color || "Unknown"} {type.vehicle.vehicleType}
      </div>
    );
  } else if ('licensePlate' in type) {
    return (
      <div className="obs-type">
        🪪 {type.licensePlate.text}
      </div>
    );
  }
  return <div className="obs-type">Unknown detection</div>;
}
