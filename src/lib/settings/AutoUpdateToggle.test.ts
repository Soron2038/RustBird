import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { invoke } from '@tauri-apps/api/core';
import AutoUpdateToggle from './AutoUpdateToggle.svelte';

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
});

describe('AutoUpdateToggle', () => {
  it('renders the label', () => {
    render(AutoUpdateToggle, { props: { enabled: false } });
    expect(screen.getByText(/Automatisch nach Updates suchen/)).toBeTruthy();
  });

  it('shows hint text', () => {
    render(AutoUpdateToggle, { props: { enabled: false } });
    expect(screen.getByText(/Wird beim nächsten Start geprüft/)).toBeTruthy();
  });

  it('toggle off → on calls set_auto_update_enabled with true', async () => {
    mockInvoke.mockResolvedValueOnce({});
    render(AutoUpdateToggle, { props: { enabled: false } });
    const pill = screen.getByRole('switch');
    fireEvent.click(pill);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith('set_auto_update_enabled', { enabled: true }),
    );
  });

  it('toggle on → off calls set_auto_update_enabled with false', async () => {
    mockInvoke.mockResolvedValueOnce({});
    render(AutoUpdateToggle, { props: { enabled: true } });
    const pill = screen.getByRole('switch');
    fireEvent.click(pill);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith('set_auto_update_enabled', { enabled: false }),
    );
  });

  it('Enter key triggers toggle', async () => {
    mockInvoke.mockResolvedValueOnce({});
    render(AutoUpdateToggle, { props: { enabled: false } });
    const pill = screen.getByRole('switch');
    fireEvent.keyDown(pill, { key: 'Enter' });
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith('set_auto_update_enabled', { enabled: true }),
    );
  });
});
