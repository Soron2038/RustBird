import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Header from './Header.svelte';

const defaultProps = {
  isPaused: false,
  showSettings: false,
  onTogglePause: vi.fn(async () => {}),
  onToggleSettings: vi.fn(),
};

describe('Header', () => {
  it('shows pause icon when not paused', () => {
    render(Header, { props: { ...defaultProps, isPaused: false } });
    expect(screen.getByText('⏸')).toBeTruthy();
  });

  it('shows play icon when paused', () => {
    render(Header, { props: { ...defaultProps, isPaused: true } });
    expect(screen.getByText('▶')).toBeTruthy();
  });

  it('displays RustBird title', () => {
    render(Header, { props: defaultProps });
    expect(screen.getByText('RustBird')).toBeTruthy();
  });
});
