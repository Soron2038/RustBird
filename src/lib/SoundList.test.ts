import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SoundList from './SoundList.svelte';

const testSounds = [
  {
    id: 'forest',
    name: 'Forest',
    file_path: '/a.mp3',
    is_bundled: true,
    is_active: false,
    volume: 0.7,
  },
  {
    id: 'user_rain',
    name: 'Rain',
    file_path: '/b.mp3',
    is_bundled: false,
    is_active: true,
    volume: 0.5,
  },
];

const defaultProps = {
  sounds: testSounds,
  hasActiveSounds: true,
  onToggle: vi.fn(async () => {}),
  onImport: vi.fn(async () => {}),
  onRemove: vi.fn(async () => {}),
};

describe('SoundList', () => {
  it('renders sound names', () => {
    render(SoundList, { props: defaultProps });
    expect(screen.getByText('Forest')).toBeTruthy();
    expect(screen.getByText('Rain')).toBeTruthy();
  });

  it('shows remove button only for user sounds', () => {
    render(SoundList, { props: defaultProps });
    const removeButtons = screen.getAllByTitle('Remove');
    // Only the non-bundled sound (Rain) should have a remove button
    expect(removeButtons).toHaveLength(1);
  });

  it('shows Sounds section label', () => {
    render(SoundList, { props: defaultProps });
    expect(screen.getByText('Sounds')).toBeTruthy();
  });

  it('shows Add Sound button', () => {
    render(SoundList, { props: defaultProps });
    expect(screen.getByText('＋ Add Sound')).toBeTruthy();
  });
});
