import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  invokeCommand,
  getState,
  toggleSound,
  setVolume,
  setMasterVolume,
  pauseAll,
  resumeAll,
  importSound,
  removeSound,
  setCrossfadeDuration,
  setDialogOpen,
  setDialogClosed,
} from './commands';

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
});

describe('invokeCommand', () => {
  it('returns invoke result on success', async () => {
    mockInvoke.mockResolvedValueOnce({ sounds: [] });
    const result = await invokeCommand('get_state');
    expect(result).toEqual({ sounds: [] });
    expect(mockInvoke).toHaveBeenCalledWith('get_state', undefined);
  });

  it('logs and rethrows on error', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    mockInvoke.mockRejectedValueOnce('backend error');

    await expect(invokeCommand('bad_cmd')).rejects.toBe('backend error');
    expect(consoleSpy).toHaveBeenCalledWith('Command bad_cmd failed:', 'backend error');
    consoleSpy.mockRestore();
  });
});

describe('typed helpers', () => {
  it('toggleSound passes correct args', async () => {
    mockInvoke.mockResolvedValueOnce({});
    await toggleSound('rain');
    expect(mockInvoke).toHaveBeenCalledWith('toggle_sound', { id: 'rain' });
  });

  it('setVolume passes correct args', async () => {
    mockInvoke.mockResolvedValueOnce({});
    await setVolume('rain', 0.5);
    expect(mockInvoke).toHaveBeenCalledWith('set_volume', { id: 'rain', volume: 0.5 });
  });

  it('setMasterVolume passes correct args', async () => {
    mockInvoke.mockResolvedValueOnce({});
    await setMasterVolume(0.8);
    expect(mockInvoke).toHaveBeenCalledWith('set_master_volume', { volume: 0.8 });
  });

  it('pauseAll calls correct command', async () => {
    mockInvoke.mockResolvedValueOnce({});
    await pauseAll();
    expect(mockInvoke).toHaveBeenCalledWith('pause_all', undefined);
  });

  it('resumeAll calls correct command', async () => {
    mockInvoke.mockResolvedValueOnce({});
    await resumeAll();
    expect(mockInvoke).toHaveBeenCalledWith('resume_all', undefined);
  });

  it('importSound passes file_path correctly', async () => {
    mockInvoke.mockResolvedValueOnce({});
    await importSound('/path/to/file.mp3');
    expect(mockInvoke).toHaveBeenCalledWith('import_sound', { filePath: '/path/to/file.mp3' });
  });

  it('removeSound passes correct args', async () => {
    mockInvoke.mockResolvedValueOnce({});
    await removeSound('user_rain');
    expect(mockInvoke).toHaveBeenCalledWith('remove_sound', { id: 'user_rain' });
  });

  it('setCrossfadeDuration passes correct args', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setCrossfadeDuration(3.0);
    expect(mockInvoke).toHaveBeenCalledWith('set_crossfade_duration', { duration: 3.0 });
  });

  it('setDialogOpen calls correct command', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setDialogOpen();
    expect(mockInvoke).toHaveBeenCalledWith('set_dialog_open', undefined);
  });

  it('setDialogClosed calls correct command', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setDialogClosed();
    expect(mockInvoke).toHaveBeenCalledWith('set_dialog_closed', undefined);
  });

  it('getState calls correct command', async () => {
    mockInvoke.mockResolvedValueOnce({ sounds: [], master_volume: 0.8 });
    const result = await getState();
    expect(mockInvoke).toHaveBeenCalledWith('get_state', undefined);
    expect(result).toHaveProperty('sounds');
  });
});
