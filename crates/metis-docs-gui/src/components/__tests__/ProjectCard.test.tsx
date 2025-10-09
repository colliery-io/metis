import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '../../test/utils';
import { ProjectCard } from '../ProjectCard';
import { ProjectInfo } from '../../lib/tauri-api';

const mockValidProject: ProjectInfo = {
  path: '/path/to/valid/project',
  is_valid: true,
  vision_exists: true,
};

describe('ProjectCard', () => {
  it('renders valid project correctly', () => {
    const mockOnSelect = vi.fn();
    
    render(
      <ProjectCard
        project={mockValidProject}
        onSelect={mockOnSelect}
      />
    );

    expect(screen.getByText('project')).toBeInTheDocument();
    expect(screen.getByText('/path/to/valid/project')).toBeInTheDocument();
    expect(screen.getByText('Valid Metis project')).toBeInTheDocument();
    expect(screen.getByText('✅')).toBeInTheDocument();
  });

  it('calls onSelect when clicked', () => {
    const mockOnSelect = vi.fn();
    
    render(
      <ProjectCard
        project={mockValidProject}
        onSelect={mockOnSelect}
      />
    );

    // Click on the main container
    const container = screen.getByText('project').closest('[class*="cursor-pointer"]');
    fireEvent.click(container!);
    expect(mockOnSelect).toHaveBeenCalledWith('/path/to/valid/project');
  });
});