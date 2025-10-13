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
      focus: '#7c3aed',
      error: '#ef4444',
    },
    interactive: {
      primary: '#7c3aed',
      primaryHover: '#6d28d9',
      primaryActive: '#5b21b6',
      secondary: '#f3f4f6',
      secondaryHover: '#e5e7eb',
      danger: '#ef4444',
      dangerHover: '#dc2626',
      success: '#10b981',
      warning: '#f59e0b',
    },
    status: {
      draft: '#6b7280',
      active: '#10b981',
      completed: '#6b7280',
      archived: '#9ca3af',
    },
    documentType: {
      vision: '#9775fa',
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
      focus: '#60a5fa',
      error: '#f87171',
    },
    interactive: {
      primary: '#60a5fa',
      primaryHover: '#3b82f6',
      primaryActive: '#2563eb',
      secondary: '#334155',
      secondaryHover: '#475569',
      danger: '#f87171',
      dangerHover: '#dc2626',
      success: '#34d399',
      warning: '#fbbf24',
    },
    status: {
      draft: '#94a3b8',
      active: '#34d399',
      completed: '#64748b',
      archived: '#64748b',
    },
    documentType: {
      vision: '#a78bfa',
      strategy: '#22d3ee',
      initiative: '#60a5fa',
      task: '#34d399',
      adr: '#fbbf24',
      backlog: '#94a3b8',
    },
  },
};

export const hyperTheme: Theme = {
  name: 'hyper',
  colors: {
    background: {
      primary: '#000000',
      secondary: '#0a0a0f',
      tertiary: '#18181b',
      elevated: '#0a0a0f',
      overlay: 'rgba(196, 38, 211, 0.3)',
    },
    text: {
      primary: '#f4f4f5',
      secondary: '#e4e4e7',
      tertiary: '#d4d4d8',
      inverse: '#000000',
    },
    border: {
      primary: '#27272a',
      secondary: '#3f3f46',
      focus: '#c026d3',
      error: '#f43f5e',
    },
    interactive: {
      primary: '#c026d3',
      primaryHover: '#a21caf',
      primaryActive: '#86198f',
      secondary: '#18181b',
      secondaryHover: '#27272a',
      danger: '#f43f5e',
      dangerHover: '#e11d48',
      success: '#4ade80',
      warning: '#facc15',
    },
    status: {
      draft: '#71717a',
      active: '#4ade80',
      completed: '#52525b',
      archived: '#52525b',
    },
    documentType: {
      vision: '#d946ef',
      strategy: '#06b6d4',
      initiative: '#e879f9',
      task: '#4ade80',
      adr: '#facc15',
      backlog: '#71717a',
    },
  },
};

export const themes = {
  light: lightTheme,
  dark: darkTheme,
  hyper: hyperTheme,
} as const;