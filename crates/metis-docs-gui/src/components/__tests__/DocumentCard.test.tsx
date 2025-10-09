import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../test/utils';
import { DocumentCard } from '../DocumentCard';
import type { DocumentInfo } from '../../lib/tauri-api';

describe('DocumentCard', () => {
  const mockDocument: DocumentInfo = {
    id: 'test-doc',
    title: 'Test Document',
    short_code: 'TEST-T-0001',
    document_type: 'task',
    phase: 'todo',
    created_at: 1633024800, // Oct 1, 2021
    updated_at: 1633024800,
    filepath: '/path/to/doc',
    archived: false
  };

  it('renders document information correctly', () => {
    render(<DocumentCard document={mockDocument} />);

    expect(screen.getByText('Test Document')).toBeInTheDocument();
    expect(screen.getByText('TEST-T-0001')).toBeInTheDocument();
    expect(screen.getByText('TASK')).toBeInTheDocument();
    expect(screen.getByText('todo')).toBeInTheDocument();
  });

  it('calls onClick handler when clicked', () => {
    const mockOnClick = vi.fn();
    render(<DocumentCard document={mockDocument} onClick={mockOnClick} />);

    const card = screen.getByText('Test Document').closest('div');
    card?.click();

    expect(mockOnClick).toHaveBeenCalledWith(mockDocument);
  });

  it('applies correct type colors', () => {
    const visionDoc = { ...mockDocument, document_type: 'vision' };
    const { rerender } = render(<DocumentCard document={visionDoc} />);

    expect(screen.getByText('VISION')).toHaveClass('bg-purple-100', 'text-purple-800');

    const initiativeDoc = { ...mockDocument, document_type: 'initiative' };
    rerender(<DocumentCard document={initiativeDoc} />);

    expect(screen.getByText('INITIATIVE')).toHaveClass('bg-blue-100', 'text-blue-800');
  });
});