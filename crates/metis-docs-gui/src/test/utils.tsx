import { render, RenderOptions } from '@testing-library/react';
import { ReactElement } from 'react';
import { ProjectProvider } from '../contexts/ProjectContext';

// Custom render function that includes providers
const AllTheProviders = ({ children }: { children: React.ReactNode }) => {
  return (
    <ProjectProvider>
      {children}
    </ProjectProvider>
  );
};

const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>,
) => render(ui, { wrapper: AllTheProviders, ...options });

export * from '@testing-library/react';
export { customRender as render };