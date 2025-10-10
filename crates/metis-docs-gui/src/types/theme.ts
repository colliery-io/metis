export interface ThemeColors {
  // Background colors
  background: {
    primary: string;
    secondary: string;
    tertiary: string;
    elevated: string;
    overlay: string;
  };
  
  // Text colors
  text: {
    primary: string;
    secondary: string;
    tertiary: string;
    inverse: string;
  };
  
  // Border colors
  border: {
    primary: string;
    secondary: string;
    focus: string;
    error: string;
  };
  
  // Interactive colors
  interactive: {
    primary: string;
    primaryHover: string;
    primaryActive: string;
    secondary: string;
    secondaryHover: string;
    danger: string;
    dangerHover: string;
    success: string;
    warning: string;
  };
  
  // Status colors
  status: {
    draft: string;
    active: string;
    completed: string;
    archived: string;
  };
  
  // Document type colors
  documentType: {
    vision: string;
    strategy: string;
    initiative: string;
    task: string;
    adr: string;
    backlog: string;
  };
}

export interface Theme {
  name: string;
  colors: ThemeColors;
}

export type ThemeName = 'light' | 'dark' | 'hyper';