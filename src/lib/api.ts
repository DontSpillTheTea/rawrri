import { invoke } from "@tauri-apps/api/core";
import type { ScanResult } from "../types";

interface ScanFolderArgs {
  rootPath: string;
  recursive?: boolean;
  pairingThresholdMs?: number;
}

export async function scanFolder(args: ScanFolderArgs): Promise<ScanResult> {
  return invoke<ScanResult>("scan_folder", args);
}
