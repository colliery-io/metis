import React, { useState } from 'react';
import { useTheme } from '../contexts/ThemeContext';

export const ThemeToggle: React.FC = () => {
  const { themeName, setTheme } = useTheme();
  const [isOpen, setIsOpen] = useState(false);

  const themeLabels = {
    light: 'Light',
    dark: 'Dark', 
    hyper: 'Hyper'
  };

  const themes = Object.keys(themeLabels) as Array<keyof typeof themeLabels>;

  const handleThemeSelect = (theme: keyof typeof themeLabels) => {
    setTheme(theme);
    setIsOpen(false);
  };

  return (
    <div className="flex items-center gap-6">
      <span className="text-secondary" style={{ fontSize: '18px' }}>Theme:</span>
      <div className="relative">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="px-6 py-3 text-secondary hover:text-primary hover:bg-elevated rounded-lg transition-colors min-w-32 bg-transparent border-none"
          style={{ fontSize: '18px' }}
        >
          {themeLabels[themeName]}
        </button>
        
        {isOpen && (
          <div className="absolute right-0 top-full mt-2 bg-elevated rounded-lg shadow-lg z-10 min-w-40 overflow-hidden">
            {themes.map((theme) => (
              <button
                key={theme}
                onClick={() => handleThemeSelect(theme)}
                className={`w-full px-6 py-3 text-left transition-colors border-none ${
                  theme === themeName 
                    ? 'text-primary bg-interactive-secondary' 
                    : 'text-secondary hover:bg-secondary hover:text-primary'
                }`}
                style={{ fontSize: '16px', backgroundColor: theme === themeName ? 'var(--color-interactive-secondary)' : 'transparent' }}
              >
                {themeLabels[theme]}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};