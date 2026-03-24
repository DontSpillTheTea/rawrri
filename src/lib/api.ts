import { invoke } from "@tauri-apps/api/core";
import type { PlaybackSnapshot, ScanResult } from "../types";

interface ScanFolderArgs {
  rootPath: string;
  recursive?: boolean;
  pairingThresholdMs?: number;
}

export async function scanFolder(args: ScanFolderArgs): Promise<ScanResult> {
  return invoke<ScanResult>("scan_folder", args);
}

interface PlaybackLoadPairArgs {
  pairId: string;
  frontPath: string | null;
  rearPath: string | null;
}

export async function playbackLoadPair(args: PlaybackLoadPairArgs): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_load_pair", args);
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

export async function playbackStop(): Promise<PlaybackSnapshot> {
  return invoke<PlaybackSnapshot>("playback_stop");
}
