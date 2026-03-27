import { invoke } from '@tauri-apps/api/core';
import type { AppState } from '$lib/types';

export async function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    console.error(`Command ${cmd} failed:`, error);
    throw error;
  }
}

export const getState = () => invokeCommand<AppState>('get_state');
export const toggleSound = (id: string) => invokeCommand<AppState>('toggle_sound', { id });
export const setVolume = (id: string, volume: number) =>
  invokeCommand<AppState>('set_volume', { id, volume });
export const setMasterVolume = (volume: number) =>
  invokeCommand<AppState>('set_master_volume', { volume });
export const pauseAll = () => invokeCommand<AppState>('pause_all');
export const resumeAll = () => invokeCommand<AppState>('resume_all');
export const importSound = (filePath: string) =>
  invokeCommand<AppState>('import_sound', { file_path: filePath });
export const removeSound = (id: string) => invokeCommand<AppState>('remove_sound', { id });
export const setCrossfadeDuration = (duration: number) =>
  invokeCommand<void>('set_crossfade_duration', { duration });
export const setDialogOpen = () => invokeCommand<void>('set_dialog_open');
export const setDialogClosed = () => invokeCommand<void>('set_dialog_closed');
export const setAutopauseOnLock = (enabled: boolean) =>
  invokeCommand<AppState>('set_autopause_on_lock', { enabled });
