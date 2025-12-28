import { Theme } from '../types/theme';

// Light Theme: "Editorial" - warm, refined, print-inspired
export const lightTheme: Theme = {
  name: 'light',
  colors: {
    background: {
      primary: '#faf9f7',      // Warm off-white
      secondary: '#f5f3f0',    // Soft cream
      tertiary: '#ebe8e4',     // Light taupe
      elevated: '#ffffff',
      overlay: 'rgba(45, 49, 66, 0.6)',
    },
    text: {
      primary: '#2d3142',      // Deep charcoal ink
      secondary: '#5c6070',    // Warm gray
      tertiary: '#8a8d9a',     // Muted gray
      inverse: '#faf9f7',
    },
    border: {
      primary: '#e2ded8',      // Warm border
      secondary: '#d4cfc7',
      focus: '#5b7c65',        // Sage focus
      error: '#c65d5d',        // Muted red
    },
    interactive: {
      primary: '#2d3142',      // Deep charcoal
      primaryHover: '#1a1d2b',
      primaryActive: '#0f1118',
      secondary: '#f0ede9',
      secondaryHover: '#e5e1db',
      danger: '#c65d5d',
      dangerHover: '#a84848',
      success: '#5b7c65',      // Sage green
      warning: '#c9a227',      // Warm gold
    },
    status: {
      draft: '#8a8d9a',
      active: '#5b7c65',       // Sage green
      completed: '#6b7280',
      archived: '#a8a5a0',
    },
    documentType: {
      vision: '#9d6b53',       // Terracotta
      strategy: '#4a7c8f',     // Muted teal
      initiative: '#5b7c65',   // Sage green
      task: '#c9a227',         // Warm gold
      adr: '#7a6b8a',          // Dusty purple
      backlog: '#8a8d9a',
    },
  },
};

// Dark Theme: "Midnight Observatory" - deep, sophisticated, blue-forward
export const darkTheme: Theme = {
  name: 'dark',
  colors: {
    background: {
      primary: '#0a0e14',      // True deep black
      secondary: '#111820',    // Navy undertone
      tertiary: '#1a2332',     // Dark slate
      elevated: '#151c28',
      overlay: 'rgba(0, 0, 0, 0.85)',
    },
    text: {
      primary: '#e8eaed',      // Soft white
      secondary: '#a4aab5',    // Cool gray
      tertiary: '#6b7280',     // Muted gray
      inverse: '#0a0e14',
    },
    border: {
      primary: '#1e2836',      // Subtle border
      secondary: '#2a3646',
      focus: '#60a5fa',        // Sky blue focus
      error: '#e57373',
    },
    interactive: {
      primary: '#60a5fa',      // Sky blue accent
      primaryHover: '#3b82f6',
      primaryActive: '#2563eb',
      secondary: '#1a2332',
      secondaryHover: '#243040',
      danger: '#e57373',
      dangerHover: '#c62828',
      success: '#60a5fa',      // Keep consistent with primary
      warning: '#f0c36d',      // Warm gold
    },
    status: {
      draft: '#6b7280',
      active: '#60a5fa',       // Sky blue
      completed: '#5a6675',
      archived: '#4a5260',
    },
    documentType: {
      vision: '#a78bfa',       // Soft purple
      strategy: '#38bdf8',     // Bright sky blue
      initiative: '#60a5fa',   // Sky blue
      task: '#7dd3fc',         // Light cyan-blue
      adr: '#f0c36d',          // Warm gold
      backlog: '#78909c',      // Blue gray
    },
  },
};

// Hyper Theme: "Neon Cyberpunk" - intense, electric, dramatic
export const hyperTheme: Theme = {
  name: 'hyper',
  colors: {
    background: {
      primary: '#000000',      // Pure black
      secondary: '#05050a',    // Near black with hint of blue
      tertiary: '#0f0f18',     // Dark with purple tint
      elevated: '#08080f',
      overlay: 'rgba(192, 38, 211, 0.4)',
    },
    text: {
      primary: '#f0f0f5',      // Bright white
      secondary: '#c8c8d0',    // Light gray
      tertiary: '#8888a0',     // Muted purple-gray
      inverse: '#000000',
    },
    border: {
      primary: '#1a1a28',      // Dark border with purple
      secondary: '#2a2a40',
      focus: '#e040fb',        // Hot pink
      error: '#ff1744',        // Neon red
    },
    interactive: {
      primary: '#e040fb',      // Hot pink/fuchsia
      primaryHover: '#ea80fc',
      primaryActive: '#d500f9',
      secondary: '#12121c',
      secondaryHover: '#1c1c2c',
      danger: '#ff1744',
      dangerHover: '#ff5252',
      success: '#00e676',      // Neon green
      warning: '#ffea00',      // Electric yellow
    },
    status: {
      draft: '#6a6a80',
      active: '#00e676',       // Neon green
      completed: '#4a4a5a',
      archived: '#3a3a48',
    },
    documentType: {
      vision: '#e040fb',       // Hot pink
      strategy: '#00e5ff',     // Electric cyan
      initiative: '#ea80fc',   // Light fuchsia
      task: '#00e676',         // Neon green
      adr: '#ffea00',          // Electric yellow
      backlog: '#6a6a80',
    },
  },
};

export const themes = {
  light: lightTheme,
  dark: darkTheme,
  hyper: hyperTheme,
} as const;