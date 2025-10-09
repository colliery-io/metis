import { describe, it, expect } from 'vitest';
import { render, screen } from '../../test/utils';
import { EmptyState } from '../EmptyState';

describe('EmptyState', () => {
  it('shows general message for all documents', () => {
    render(<EmptyState selectedType="all" />);
    
    expect(screen.getByText('No documents found')).toBeInTheDocument();
    expect(screen.getByText("This project doesn't have any documents yet.")).toBeInTheDocument();
  });

  it('shows specific message for visions', () => {
    render(<EmptyState selectedType="vision" />);
    
    expect(screen.getByText('No visions found')).toBeInTheDocument();
    expect(screen.getByText('Create a vision to define the strategic direction for your project.')).toBeInTheDocument();
  });

  it('shows specific message for tasks', () => {
    render(<EmptyState selectedType="task" />);
    
    expect(screen.getByText('No tasks found')).toBeInTheDocument();
    expect(screen.getByText('Create tasks to organize the specific work that needs to be done.')).toBeInTheDocument();
  });

  it('shows specific message for ADRs', () => {
    render(<EmptyState selectedType="adr" />);
    
    expect(screen.getByText('No ADRs found')).toBeInTheDocument();
    expect(screen.getByText('Create Architectural Decision Records to document important decisions.')).toBeInTheDocument();
  });
});