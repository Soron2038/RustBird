import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import AppName from './AppName.svelte';

describe('AppName', () => {
  it('renders the app name text content', () => {
    const { container } = render(AppName);
    expect(container.textContent).toContain('RustB');
    expect(container.textContent).toContain('rd');
  });

  it('has an accent-i element for the styled dot', () => {
    const { container } = render(AppName);
    expect(container.querySelector('.accent-i')).toBeTruthy();
  });
});
