import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '../../test/utils';

// Mock the entire Tauri plugin module
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

import { DirectoryPicker } from '../DirectoryPicker';

describe('DirectoryPicker', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders with default text', () => {
    const mockOnDirectorySelected = vi.fn();
    
    render(
      <DirectoryPicker onDirectorySelected={mockOnDirectorySelected} />
    );

    expect(screen.getByText('Browse Directory')).toBeInTheDocument();
  });

  it('renders with custom children', () => {
    const mockOnDirectorySelected = vi.fn();
    
    render(
      <DirectoryPicker onDirectorySelected={mockOnDirectorySelected}>
        Custom Button Text
      </DirectoryPicker>
    );

    expect(screen.getByText('Custom Button Text')).toBeInTheDocument();
  });
});