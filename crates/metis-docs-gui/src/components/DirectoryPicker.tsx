import React from 'react';
import { open } from '@tauri-apps/plugin-dialog';

interface DirectoryPickerProps {
  onDirectorySelected: (path: string) => void;
  disabled?: boolean;
  children?: React.ReactNode;
  className?: string;
}

export function DirectoryPicker({ 
  onDirectorySelected, 
  disabled = false, 
  children,
  className = ""
}: DirectoryPickerProps) {
  const handleClick = async () => {
    if (disabled) return;

    try {
      const result = await open({
        directory: true,
        multiple: false,
        title: 'Select Metis Project Directory',
      });

      if (result && typeof result === 'string') {
        onDirectorySelected(result);
      }
    } catch (error) {
      console.error('Failed to open directory picker:', error);
    }
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      disabled={disabled}
      className={`
        px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 
        disabled:bg-gray-300 disabled:cursor-not-allowed
        ${className}
      `}
    >
      {children || 'Browse Directory'}
    </button>
  );
}