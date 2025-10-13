import React, { useState, useEffect } from 'react';
import { useTheme } from '../contexts/ThemeContext';

export interface InitProjectDialogProps {
  isOpen: boolean;
  directoryName: string;
  onConfirm: (prefix: string) => void;
  onCancel: () => void;
}

export const InitProjectDialog: React.FC<InitProjectDialogProps> = ({
  isOpen,
  directoryName,
  onConfirm,
  onCancel,
}) => {
  const { theme } = useTheme();
  const [prefix, setPrefix] = useState('');
  const [error, setError] = useState('');

  // Generate default prefix when dialog opens
  useEffect(() => {
    if (isOpen && directoryName) {
      const defaultPrefix = directoryName.toUpperCase().replace(/[^A-Z]/g, '').slice(0, 8);
      setPrefix(defaultPrefix.length >= 2 ? defaultPrefix : 'PROJ');
      setError('');
    }
  }, [isOpen, directoryName]);

  const validatePrefix = (value: string): string => {
    if (value.length < 2) return 'Prefix must be at least 2 characters';
    if (value.length > 8) return 'Prefix must be no more than 8 characters';
    if (!/^[A-Z]+$/.test(value)) return 'Prefix must contain only uppercase letters';
    return '';
  };

  const handlePrefixChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value.toUpperCase().replace(/[^A-Z]/g, '');
    setPrefix(value);
    setError(validatePrefix(value));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const validationError = validatePrefix(prefix);
    if (validationError) {
      setError(validationError);
      return;
    }
    onConfirm(prefix);
  };

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
        className="relative rounded-lg shadow-lg p-6 z-10"
        style={{
          backgroundColor: theme.colors.background.elevated,
          border: `1px solid ${theme.colors.border.primary}`,
          width: '400px',
          maxWidth: '90vw'
        }}
      >
        <form onSubmit={handleSubmit}>
          {/* Title */}
          <h3 
            className="font-semibold mb-4"
            style={{ 
              fontSize: '18px', 
              color: theme.colors.text.primary 
            }}
          >
            Initialize Metis Project
          </h3>

          {/* Message */}
          <p 
            className="mb-4 leading-relaxed"
            style={{ 
              fontSize: '14px', 
              color: theme.colors.text.secondary 
            }}
          >
            The directory "{directoryName}" will be initialized as a new Metis project.
          </p>

          {/* Project Prefix Input */}
          <div className="mb-4">
            <label 
              className="block mb-2 font-medium"
              style={{ 
                fontSize: '14px', 
                color: theme.colors.text.primary 
              }}
            >
              Project Prefix
            </label>
            <input
              type="text"
              value={prefix}
              onChange={handlePrefixChange}
              className="w-full px-3 py-2 rounded-lg transition-colors"
              style={{
                backgroundColor: theme.colors.background.primary,
                border: `1px solid ${error ? theme.colors.border.error : theme.colors.border.primary}`,
                color: theme.colors.text.primary,
                fontSize: '14px'
              }}
              placeholder="PROJ"
              maxLength={8}
              autoFocus
            />
            <p 
              className="mt-1 text-xs"
              style={{ 
                color: theme.colors.text.tertiary 
              }}
            >
              2-8 uppercase letters (used for document IDs like {prefix || 'PROJ'}-V-0001)
            </p>
            {error && (
              <p 
                className="mt-1 text-xs"
                style={{ 
                  color: theme.colors.border.error 
                }}
              >
                {error}
              </p>
            )}
          </div>

          {/* Buttons */}
          <div className="flex gap-3 justify-end">
            <button
              type="button"
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
              Cancel
            </button>
            
            <button
              type="submit"
              disabled={!!error || !prefix}
              className="px-4 py-2 rounded-lg font-medium transition-colors"
              style={{
                backgroundColor: error || !prefix ? theme.colors.background.tertiary : theme.colors.interactive.primary,
                color: error || !prefix ? theme.colors.text.tertiary : theme.colors.text.inverse,
                border: `1px solid ${error || !prefix ? theme.colors.border.secondary : theme.colors.interactive.primary}`,
                fontSize: '14px',
                cursor: error || !prefix ? 'not-allowed' : 'pointer'
              }}
              onMouseEnter={(e) => {
                if (!error && prefix) {
                  e.currentTarget.style.backgroundColor = theme.colors.interactive.primaryHover;
                  e.currentTarget.style.borderColor = theme.colors.interactive.primaryHover;
                }
              }}
              onMouseLeave={(e) => {
                if (!error && prefix) {
                  e.currentTarget.style.backgroundColor = theme.colors.interactive.primary;
                  e.currentTarget.style.borderColor = theme.colors.interactive.primary;
                }
              }}
            >
              Initialize Project
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};