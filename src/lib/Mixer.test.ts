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

const activeSound = {
  id: '1',
  name: 'Rain',
  volume: 0.7,
  is_active: true,
  is_bundled: true,
};

const noop = async () => {};

describe('Mixer', () => {
  it('shows silence indicator when no sounds are active', () => {
    render(Mixer, {
      props: {
        sounds: [],
        masterVolume: 0.7,
        onVolumeChange: noop,
        onMasterVolumeChange: noop,
      },
    });
    expect(screen.getByText('· · ·')).toBeTruthy();
  });

  it('does not show faders when no sounds are active', () => {
    const { queryByText } = render(Mixer, {
      props: {
        sounds: [],
        masterVolume: 0.7,
        onVolumeChange: noop,
        onMasterVolumeChange: noop,
      },
    });
    expect(queryByText('Rain')).toBeNull();
  });

  it('shows fader labels when sounds are provided', () => {
    render(Mixer, {
      props: {
        sounds: [activeSound],
        masterVolume: 0.7,
        onVolumeChange: noop,
        onMasterVolumeChange: noop,
      },
    });
    expect(screen.getByText('Rain')).toBeTruthy();
  });

  it('does not show silence indicator when sounds are provided', () => {
    const { queryByText } = render(Mixer, {
      props: {
        sounds: [activeSound],
        masterVolume: 0.7,
        onVolumeChange: noop,
        onMasterVolumeChange: noop,
      },
    });
    expect(queryByText('· · ·')).toBeNull();
  });

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
