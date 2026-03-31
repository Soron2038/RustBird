import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Mixer from './Mixer.svelte';

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
});
