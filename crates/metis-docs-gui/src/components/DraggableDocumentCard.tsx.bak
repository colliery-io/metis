import React from 'react';
import { useDraggable } from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';
import { DocumentCard } from './DocumentCard';
import type { DocumentInfo } from '../lib/tauri-api';

export interface DraggableDocumentCardProps {
  document: DocumentInfo;
  onClick?: (document: DocumentInfo) => void;
}

export const DraggableDocumentCard: React.FC<DraggableDocumentCardProps> = ({ 
  document, 
  onClick 
}) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    isDragging,
  } = useDraggable({
    id: document.short_code,
  });

  const style = {
    transform: CSS.Translate.toString(transform),
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...listeners}
      {...attributes}
      className={`${isDragging ? 'z-50' : ''}`}
    >
      <DocumentCard
        document={document}
        onClick={onClick}
      />
    </div>
  );
};