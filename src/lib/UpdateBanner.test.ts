import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { invoke } from '@tauri-apps/api/core';
import UpdateBanner from './UpdateBanner.svelte';

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
});

describe('UpdateBanner', () => {
  it('shows the version that is ready to install', () => {
    render(UpdateBanner, { props: { version: '1.2.0', onDismiss: () => {} } });
    expect(screen.getByText(/RustBird 1\.2\.0 ist bereit/)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Jetzt neu starten' })).toBeTruthy();
  });

  it('restart button invokes install_update and shows progress', async () => {
    // On success the app restarts, so the promise never resolves.
    mockInvoke.mockReturnValueOnce(new Promise(() => {}));
    render(UpdateBanner, { props: { version: '1.2.0', onDismiss: () => {} } });
    await fireEvent.click(screen.getByRole('button', { name: 'Jetzt neu starten' }));
    expect(mockInvoke).toHaveBeenCalledWith('install_update', undefined);
    await waitFor(() => {
      const busy = screen.getByRole('button', { name: /Wird installiert/ }) as HTMLButtonElement;
      expect(busy.disabled).toBe(true);
    });
    expect((screen.getByRole('button', { name: 'Später' }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it('shows the error and re-enables the buttons when the install fails', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('bundle not found'));
    render(UpdateBanner, { props: { version: '1.2.0', onDismiss: () => {} } });
    await fireEvent.click(screen.getByRole('button', { name: 'Jetzt neu starten' }));
    await waitFor(() => expect(screen.getByRole('alert').textContent).toMatch(/bundle not found/));
    const retry = screen.getByRole('button', { name: 'Jetzt neu starten' }) as HTMLButtonElement;
    expect(retry.disabled).toBe(false);
  });

  it('later button calls onDismiss without installing', async () => {
    const onDismiss = vi.fn();
    render(UpdateBanner, { props: { version: '1.2.0', onDismiss } });
    await fireEvent.click(screen.getByRole('button', { name: 'Später' }));
    expect(onDismiss).toHaveBeenCalledOnce();
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
