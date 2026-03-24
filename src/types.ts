export type VideoSide = "front" | "rear";
export type HealthStatus = "ok" | "warning" | "error";
export type ParseStatus = "parsed";

export interface VideoAsset {
  id: string;
  path: string;
  filename: string;
  side: VideoSide;
  parseStatus: ParseStatus;
  parsedSequence: number | null;
  rawTimestampString: string | null;
  parsedTimestamp: string | null;
  durationSec: number | null;
  resolution: { width: number; height: number } | null;
  codec: string | null;
  sizeBytes: number;
  modifiedAt: string;
  health: HealthStatus;
  warnings: string[];
}

export interface RecordingPair {
  id: string;
  frontAssetId: string | null;
  rearAssetId: string | null;
  canonicalStartTime: string | null;
  estimatedDurationSec: number | null;
  pairingConfidence: number;
  pairingReason: string;
  sourceFolder: string;
  warnings: string[];
}

export interface ScanDiagnostics {
  totalFilesDiscovered: number;
  parserMatchedFiles: number;
  parserSkippedFiles: number;
  parserFailedFiles: number;
  validPairs: number;
  partialPairs: number;
  pairingThresholdMs: number;
}

export interface ScanResult {
  rootPath: string;
  scannedAt: string;
  assets: VideoAsset[];
  pairs: RecordingPair[];
  diagnostics: ScanDiagnostics;
  errors: string[];
}

export interface PlaybackSnapshot {
  activePairId: string | null;
  isPlaying: boolean;
  playheadSec: number;
  frontLoaded: boolean;
  rearLoaded: boolean;
}
