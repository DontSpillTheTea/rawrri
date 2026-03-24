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
  pairDurationSec: number | null;
  frontTimeSec: number | null;
  rearTimeSec: number | null;
  frontDurationSec: number | null;
  rearDurationSec: number | null;
  syncDeltaSec: number | null;
  frontLoaded: boolean;
  rearLoaded: boolean;
  frontMuted: boolean;
  rearMuted: boolean;
  lastError: string | null;
}

export interface VideoRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface NativeRect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export interface VideoSurfaceSnapshot {
  frontWid: number | null;
  rearWid: number | null;
  parentHwndRaw: number | null;
  frontHwndRaw: number | null;
  rearHwndRaw: number | null;
  frontVisible: boolean;
  rearVisible: boolean;
  frontWindowRect: NativeRect | null;
  rearWindowRect: NativeRect | null;
  frontClientRect: NativeRect | null;
  rearClientRect: NativeRect | null;
  lastFrontLayout: VideoRect | null;
  lastRearLayout: VideoRect | null;
  debugVisualHosts: boolean;
}
