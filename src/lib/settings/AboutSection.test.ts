import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { openUrl } from '@tauri-apps/plugin-opener';
import AboutSection from './AboutSection.svelte';

describe('AboutSection', () => {
  it('displays author credit', () => {
    render(AboutSection);
    expect(screen.getByText(/soron2038/)).toBeTruthy();
  });

  it('shows the xeno-canto link', () => {
    render(AboutSection);
    expect(screen.getByText(/xeno-canto\.org/)).toBeTruthy();
  });

  it('shows version after mount', async () => {
    render(AboutSection);
    await waitFor(() => {
      expect(screen.getByText(/1\.0\.0/)).toBeTruthy();
    });
  });

  it('opens xeno-canto.org in browser when link is clicked', async () => {
    render(AboutSection);
    fireEvent.click(screen.getByText(/xeno-canto\.org/));
    await waitFor(() => expect(vi.mocked(openUrl)).toHaveBeenCalledWith('https://xeno-canto.org'));
  });
});
