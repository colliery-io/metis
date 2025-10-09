import '@testing-library/jest-dom';

// Mock Tauri API for testing
const mockInvoke = vi.fn();
const mockOpen = vi.fn();

// @ts-ignore
global.__TAURI__ = {
  invoke: mockInvoke,
};

// @ts-ignore
global.__TAURI_PLUGIN_DIALOG__ = {
  open: mockOpen,
};

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

// Reset mocks before each test
beforeEach(() => {
  vi.clearAllMocks();
  localStorageMock.getItem.mockClear();
  localStorageMock.setItem.mockClear();
  localStorageMock.removeItem.mockClear();
  localStorageMock.clear.mockClear();
});

// Export mocks for use in tests
export { mockInvoke, mockOpen, localStorageMock };