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

  it('displays app title with accent styling', () => {
    const { container } = render(Header, { props: defaultProps });
    expect(container.querySelector('.title')).toBeTruthy();
    expect(container.querySelector('.accent-i')).toBeTruthy();
  });
});
