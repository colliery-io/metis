import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { Theme, ThemeName } from '../types/theme';
import { themes } from '../themes/definitions';

interface ThemeContextType {
  theme: Theme;
  themeName: ThemeName;
  setTheme: (themeName: ThemeName) => void;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

interface ThemeProviderProps {
  children: ReactNode;
  defaultTheme?: ThemeName;
}

export function ThemeProvider({ children, defaultTheme = 'light' }: ThemeProviderProps) {
  const [themeName, setThemeName] = useState<ThemeName>(defaultTheme);

  // Load theme preference from localStorage on mount
  useEffect(() => {
    const stored = localStorage.getItem('metis-theme') as ThemeName;
    if (stored && themes[stored]) {
      setThemeName(stored);
    }
  }, []);

  // Save theme preference to localStorage when changed
  useEffect(() => {
    localStorage.setItem('metis-theme', themeName);
    
    // Apply theme to document root for CSS custom properties
    const root = document.documentElement;
    const theme = themes[themeName];
    
    // Set CSS custom properties for the current theme
    Object.entries(theme.colors).forEach(([category, colors]) => {
      if (typeof colors === 'object' && colors !== null) {
        Object.entries(colors).forEach(([key, value]) => {
          if (typeof value === 'string') {
            root.style.setProperty(`--color-${category}-${key}`, value);
          }
        });
      }
    });
    
    // Set theme name as data attribute for potential CSS selectors
    root.setAttribute('data-theme', themeName);
  }, [themeName]);

  const setTheme = (newThemeName: ThemeName) => {
    setThemeName(newThemeName);
  };

  const toggleTheme = () => {
    const themeOrder: ThemeName[] = ['light', 'dark', 'hyper'];
    const currentIndex = themeOrder.indexOf(themeName);
    const nextIndex = (currentIndex + 1) % themeOrder.length;
    setThemeName(themeOrder[nextIndex]);
  };

  const value: ThemeContextType = {
    theme: themes[themeName],
    themeName,
    setTheme,
    toggleTheme,
  };

  return (
    <ThemeContext.Provider value={value}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextType {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}

// Utility hook to get theme-aware styles
export function useThemedStyles() {
  const { theme } = useTheme();
  
  return {
    // Helper functions for common style patterns
    bg: (level: keyof typeof theme.colors.background) => ({
      backgroundColor: theme.colors.background[level],
    }),
    text: (level: keyof typeof theme.colors.text) => ({
      color: theme.colors.text[level],
    }),
    border: (level: keyof typeof theme.colors.border) => ({
      borderColor: theme.colors.border[level],
    }),
    // Direct access to theme colors
    colors: theme.colors,
  };
}