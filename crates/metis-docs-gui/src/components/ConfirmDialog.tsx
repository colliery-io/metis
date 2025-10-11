import React from 'react';
import { useTheme } from '../contexts/ThemeContext';

export interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  isOpen,
  title,
  message,
  confirmText = 'Yes',
  cancelText = 'Cancel',
  onConfirm,
  onCancel,
}) => {
  const { theme } = useTheme();

  if (!isOpen) return null;

  return (
    <div 
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0 }}
    >
      {/* Backdrop */}
      <div 
        className="absolute inset-0 transition-opacity"
        style={{ backgroundColor: theme.colors.background.overlay }}
        onClick={onCancel}
      />
      
      {/* Dialog */}
      <div 
        className="relative rounded-lg shadow-lg p-6 w-full max-w-md mx-4 z-10"
        style={{
          backgroundColor: theme.colors.background.elevated,
          border: `1px solid ${theme.colors.border.primary}`
        }}
      >
        {/* Title */}
        <h3 
          className="font-semibold mb-4"
          style={{ 
            fontSize: '18px', 
            color: theme.colors.text.primary 
          }}
        >
          {title}
        </h3>

        {/* Message */}
        <p 
          className="mb-6 leading-relaxed"
          style={{ 
            fontSize: '14px', 
            color: theme.colors.text.secondary 
          }}
        >
          {message}
        </p>

        {/* Buttons */}
        <div className="flex gap-3 justify-end">
          <button
            onClick={onCancel}
            className="px-4 py-2 rounded-lg font-medium transition-colors"
            style={{
              backgroundColor: theme.colors.background.secondary,
              color: theme.colors.text.primary,
              border: `1px solid ${theme.colors.border.primary}`,
              fontSize: '14px'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = theme.colors.interactive.secondary;
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = theme.colors.background.secondary;
            }}
          >
            {cancelText}
          </button>
          
          <button
            onClick={onConfirm}
            className="px-4 py-2 rounded-lg font-medium transition-colors"
            style={{
              backgroundColor: theme.colors.interactive.primary,
              color: theme.colors.text.inverse,
              border: `1px solid ${theme.colors.interactive.primary}`,
              fontSize: '14px'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = theme.colors.interactive.primaryHover;
              e.currentTarget.style.borderColor = theme.colors.interactive.primaryHover;
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = theme.colors.interactive.primary;
              e.currentTarget.style.borderColor = theme.colors.interactive.primary;
            }}
          >
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
};