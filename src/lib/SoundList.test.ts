import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { tick } from 'svelte';
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

  it('shows bottom fade when list can scroll down', async () => {
    const { container } = render(SoundList, { props: defaultProps });
    const list = container.querySelector('.list') as HTMLElement;
    const bottomFade = container.querySelector('.scroll-fade.bottom') as HTMLElement;

    expect(bottomFade).toBeTruthy();

    Object.defineProperty(list, 'scrollHeight', { value: 300, configurable: true });
    Object.defineProperty(list, 'clientHeight', { value: 150, configurable: true });
    list.dispatchEvent(new Event('scroll'));

    await tick();

    expect(bottomFade.classList.contains('visible')).toBe(true);
  });

  it('shows top fade after scrolling down', async () => {
    const { container } = render(SoundList, { props: defaultProps });
    const list = container.querySelector('.list') as HTMLElement;
    const topFade = container.querySelector('.scroll-fade.top') as HTMLElement;

    expect(topFade).toBeTruthy();

    Object.defineProperty(list, 'scrollTop', { get: () => 50, configurable: true });
    Object.defineProperty(list, 'scrollHeight', { value: 300, configurable: true });
    Object.defineProperty(list, 'clientHeight', { value: 150, configurable: true });
    list.dispatchEvent(new Event('scroll'));

    await tick();

    expect(topFade.classList.contains('visible')).toBe(true);
  });

  it('shows bottom caret when list can scroll down', async () => {
    const { container } = render(SoundList, { props: defaultProps });
    const list = container.querySelector('.list') as HTMLElement;
    const bottomCaret = container.querySelector('.scroll-caret.bottom') as HTMLElement;

    expect(bottomCaret).toBeTruthy();

    Object.defineProperty(list, 'scrollHeight', { value: 300, configurable: true });
    Object.defineProperty(list, 'clientHeight', { value: 150, configurable: true });
    list.dispatchEvent(new Event('scroll'));

    await tick();

    expect(bottomCaret.classList.contains('visible')).toBe(true);
  });

  it('shows top caret after scrolling down', async () => {
    const { container } = render(SoundList, { props: defaultProps });
    const list = container.querySelector('.list') as HTMLElement;
    const topCaret = container.querySelector('.scroll-caret.top') as HTMLElement;

    expect(topCaret).toBeTruthy();

    Object.defineProperty(list, 'scrollTop', { get: () => 50, configurable: true });
    Object.defineProperty(list, 'scrollHeight', { value: 300, configurable: true });
    Object.defineProperty(list, 'clientHeight', { value: 150, configurable: true });
    list.dispatchEvent(new Event('scroll'));

    await tick();

    expect(topCaret.classList.contains('visible')).toBe(true);
  });
});
