import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Mixer from './Mixer.svelte';

const testSounds = [
  {
    id: 'forest',
    name: 'Forest',
    file_path: '/a.mp3',
    is_bundled: true,
    is_active: true,
    volume: 0.7,
  },
  {
    id: 'rain',
    name: 'Rain',
    file_path: '/b.mp3',
    is_bundled: false,
    is_active: true,
    volume: 0.5,
  },
];

describe('Mixer', () => {
  it('renders fader label for each sound', () => {
    render(Mixer, {
      props: {
        sounds: testSounds,
        masterVolume: 0.8,
        onVolumeChange: vi.fn(async () => {}),
        onMasterVolumeChange: vi.fn(async () => {}),
      },
    });
    expect(screen.getByText('Forest')).toBeTruthy();
    expect(screen.getByText('Rain')).toBeTruthy();
  });

  it('shows Now Playing label', () => {
    render(Mixer, {
      props: {
        sounds: testSounds,
        masterVolume: 0.8,
        onVolumeChange: vi.fn(async () => {}),
        onMasterVolumeChange: vi.fn(async () => {}),
      },
    });
    expect(screen.getByText('Now Playing')).toBeTruthy();
  });
});
