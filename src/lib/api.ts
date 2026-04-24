import { invoke } from "@tauri-apps/api/core";
import type {
  ObservationEvent,
  PlaybackSnapshot,
  ScanResult,
  VideoRect,
  VideoSurfaceSnapshot
} from "../types";

interface ScanFolderArgs {
  rootPath: string;
  recursive?: boolean;
  pairingThresholdMs?: number;
}

export async function scanFolder(args: ScanFolderArgs): Promise<ScanResult> {
  return invoke<ScanResult>("scan_folder", args as unknown as Record<string, unknown>);
}

interface PlaybackLoadPairArgs {
  pairId: string;
  frontPath: string | null;
  rearPath: string | null;
}

export async function playbackLoadPair(args: PlaybackLoadPairArgs): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_load_pair", args as unknown as Record<string, unknown>);
}


export async function playbackTogglePlayPause(): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_toggle_play_pause");
}

export async function playbackSetPlaying(isPlaying: boolean): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_set_playing", { isPlaying });
}

export async function playbackSeek(playheadSec: number): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_seek", { playheadSec });
}

export async function playbackSetMute(side: "front" | "rear", muted: boolean): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_set_mute", { side, muted });
}

export async function playbackStop(): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_stop");
}

export async function playbackGetState(): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_get_state");
}

export async function updateVideoLayout(front: VideoRect, rear: VideoRect): Promise<VideoSurfaceSnapshot> {
  return invoke<VideoSurfaceSnapshot>("update_video_layout", { front, rear });
}

export async function startAnalysis(assetId: string, pairId: string, path: string): Promise<ObservationEvent[]> {
  return invoke<ObservationEvent[]>("start_analysis", { assetId, pairId, path });
}
