import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '../../test/utils';
import { DocumentTypeFilter } from '../DocumentTypeFilter';

describe('DocumentTypeFilter', () => {
  const mockDocumentCounts = {
    all: 10,
    vision: 2,
    initiative: 3,
    task: 4,
    adr: 1
  };

  it('renders all filter options with counts', () => {
    const mockOnTypeChange = vi.fn();
    
    render(
      <DocumentTypeFilter
        selectedType="all"
        onTypeChange={mockOnTypeChange}
        documentCounts={mockDocumentCounts}
      />
    );

    expect(screen.getByText('All Documents')).toBeInTheDocument();
    expect(screen.getByText('(10)')).toBeInTheDocument();
    expect(screen.getByText('Visions')).toBeInTheDocument();
    expect(screen.getByText('(2)')).toBeInTheDocument();
    expect(screen.getByText('Tasks')).toBeInTheDocument();
    expect(screen.getByText('(4)')).toBeInTheDocument();
  });

  it('highlights selected filter', () => {
    const mockOnTypeChange = vi.fn();
    
    render(
      <DocumentTypeFilter
        selectedType="task"
        onTypeChange={mockOnTypeChange}
        documentCounts={mockDocumentCounts}
      />
    );

    const taskButton = screen.getByText('Tasks').closest('button');
    expect(taskButton).toHaveClass('bg-blue-100', 'text-blue-800');
  });

  it('calls onTypeChange when filter is clicked', () => {
    const mockOnTypeChange = vi.fn();
    
    render(
      <DocumentTypeFilter
        selectedType="all"
        onTypeChange={mockOnTypeChange}
        documentCounts={mockDocumentCounts}
      />
    );

    const visionButton = screen.getByText('Visions').closest('button');
    fireEvent.click(visionButton!);

    expect(mockOnTypeChange).toHaveBeenCalledWith('vision');
  });
});