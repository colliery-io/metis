import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '../../test/utils';
import { ProjectProvider, useProject } from '../ProjectContext';
import { localStorageMock } from '../../test/setup';

// Mock the tauri-api module
vi.mock('../../lib/tauri-api', () => ({
  MetisAPI: {
    loadProject: vi.fn(),
  },
}));

// Wrapper component for testing hooks
const wrapper = ({ children }: { children: React.ReactNode }) => (
  <ProjectProvider>{children}</ProjectProvider>
);

describe('ProjectContext', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorageMock.getItem.mockReturnValue(null);
  });

  it('provides initial state', () => {
    const { result } = renderHook(() => useProject(), { wrapper });

    expect(result.current.state.currentProject).toBeNull();
    expect(result.current.state.recentProjects).toEqual([]);
    expect(result.current.state.isLoading).toBe(false);
    expect(result.current.state.error).toBeNull();
  });

  it('clears current project', () => {
    const { result } = renderHook(() => useProject(), { wrapper });

    // First set a project manually
    act(() => {
      result.current.dispatch({ 
        type: 'LOAD_PROJECT_SUCCESS', 
        payload: { 
          path: '/test/project', 
          is_valid: true, 
          vision_exists: true 
        } 
      });
    });

    expect(result.current.state.currentProject).not.toBeNull();

    // Then clear it
    act(() => {
      result.current.clearProject();
    });

    expect(result.current.state.currentProject).toBeNull();
  });

  it('throws error when useProject is used outside provider', () => {
    expect(() => {
      renderHook(() => useProject());
    }).toThrow('useProject must be used within a ProjectProvider');
  });
});