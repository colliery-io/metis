import { Theme } from '../types/theme';

export const lightTheme: Theme = {
  name: 'light',
  colors: {
    background: {
      primary: '#ffffff',
      secondary: '#f9fafb',
      tertiary: '#f3f4f6',
      elevated: '#ffffff',
      overlay: 'rgba(0, 0, 0, 0.5)',
    },
    text: {
      primary: '#111827',
      secondary: '#6b7280',
      tertiary: '#9ca3af',
      inverse: '#ffffff',
    },
    border: {
      primary: '#e5e7eb',
      secondary: '#d1d5db',
      focus: '#3b82f6',
      error: '#ef4444',
    },
    interactive: {
      primary: '#3b82f6',
      primaryHover: '#2563eb',
      primaryActive: '#1d4ed8',
      secondary: '#f3f4f6',
      secondaryHover: '#e5e7eb',
      danger: '#ef4444',
      dangerHover: '#dc2626',
      success: '#10b981',
      warning: '#f59e0b',
    },
    status: {
      draft: '#6b7280',
      active: '#3b82f6',
      completed: '#10b981',
      archived: '#9ca3af',
    },
    documentType: {
      vision: '#8b5cf6',
      strategy: '#06b6d4',
      initiative: '#3b82f6',
      task: '#10b981',
      adr: '#f59e0b',
      backlog: '#6b7280',
    },
  },
};

export const darkTheme: Theme = {
  name: 'dark',
  colors: {
    background: {
      primary: '#0f172a',
      secondary: '#1e293b',
      tertiary: '#334155',
      elevated: '#1e293b',
      overlay: 'rgba(0, 0, 0, 0.8)',
    },
    text: {
      primary: '#f8fafc',
      secondary: '#cbd5e1',
      tertiary: '#94a3b8',
      inverse: '#0f172a',
    },
    border: {
      primary: '#334155',
      secondary: '#475569',
      focus: '#3b82f6',
      error: '#ef4444',
    },
    interactive: {
      primary: '#3b82f6',
      primaryHover: '#2563eb',
      primaryActive: '#1d4ed8',
      secondary: '#334155',
      secondaryHover: '#475569',
      danger: '#ef4444',
      dangerHover: '#dc2626',
      success: '#10b981',
      warning: '#f59e0b',
    },
    status: {
      draft: '#94a3b8',
      active: '#3b82f6',
      completed: '#10b981',
      archived: '#64748b',
    },
    documentType: {
      vision: '#a855f7',
      strategy: '#06b6d4',
      initiative: '#3b82f6',
      task: '#10b981',
      adr: '#f59e0b',
      backlog: '#94a3b8',
    },
  },
};

export const hyperTheme: Theme = {
  name: 'hyper',
  colors: {
    background: {
      primary: '#0a0a0a',
      secondary: '#1a0a1a',
      tertiary: '#2a1a2a',
      elevated: '#1a0a1a',
      overlay: 'rgba(255, 0, 255, 0.2)',
    },
    text: {
      primary: '#ff00ff',
      secondary: '#00ffff',
      tertiary: '#ff0080',
      inverse: '#0a0a0a',
    },
    border: {
      primary: '#ff00ff',
      secondary: '#00ffff',
      focus: '#ffff00',
      error: '#ff0040',
    },
    interactive: {
      primary: '#ff00ff',
      primaryHover: '#ff40ff',
      primaryActive: '#ff80ff',
      secondary: '#2a1a2a',
      secondaryHover: '#3a2a3a',
      danger: '#ff0040',
      dangerHover: '#ff4080',
      success: '#00ff80',
      warning: '#ffff00',
    },
    status: {
      draft: '#ff0080',
      active: '#00ffff',
      completed: '#00ff80',
      archived: '#8040ff',
    },
    documentType: {
      vision: '#ff00ff',
      strategy: '#00ffff',
      initiative: '#ffff00',
      task: '#00ff80',
      adr: '#ff8000',
      backlog: '#ff0080',
    },
  },
};

export const themes = {
  light: lightTheme,
  dark: darkTheme,
  hyper: hyperTheme,
} as const;