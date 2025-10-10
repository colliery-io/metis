import "./App.css";
import "./styles/theme.css";
import { ProjectProvider, useProject } from "./contexts/ProjectContext";
import { ThemeProvider, useTheme } from "./contexts/ThemeContext";
import { ThemeToggle } from "./components/ThemeToggle";
import { ProjectBrowser } from "./components/ProjectBrowser";
import { KanbanBoard } from "./components/KanbanBoard";
import { ProjectSidebar } from "./components/ProjectSidebar";
import { ProjectInfo } from "./lib/tauri-api";
import { useState } from "react";

function AppContent() {
  const { currentProject, setCurrentProject, loadProject } = useProject();
  const { themeName } = useTheme();
  const [showProjectBrowser, setShowProjectBrowser] = useState(false);

  const handleProjectSelect = async (project: ProjectInfo) => {
    try {
      await loadProject(project.path);
      setShowProjectBrowser(false);
    } catch (error) {
      console.error('Failed to load project:', error);
    }
  };

  const handleShowProjectBrowser = () => {
    setShowProjectBrowser(true);
  };

  const handleBackFromBrowser = () => {
    setShowProjectBrowser(false);
  };

  const getMascotImage = () => {
    switch (themeName) {
      case 'dark':
        return '/assets/metis-dark.png';
      case 'hyper':
        return '/assets/metis-hyper.png';
      case 'light':
      default:
        return '/assets/metis-light.png';
    }
  };

  // Show full-screen project browser when explicitly requested
  if (showProjectBrowser) {
    return (
      <div className="h-screen flex flex-col">
        <div className="p-4 bg-elevated border-b border-primary flex items-center justify-between">
          <h1 className="text-xl font-semibold text-primary">Select Project</h1>
          <button
            onClick={handleBackFromBrowser}
            className="px-4 py-2 text-secondary hover:text-primary hover:bg-secondary rounded-lg transition-colors"
          >
            ← Back
          </button>
        </div>
        <div className="flex-1">
          <ProjectBrowser />
        </div>
      </div>
    );
  }

  // Main app layout with top bar and sidebar
  return (
    <div className="h-screen flex flex-col">
      {/* Top Bar */}
      <div className="bg-secondary flex items-center">
        {/* Left section - matches sidebar width */}
        <div className="w-1/5 flex items-center justify-between px-6 py-4">
          <img
            src={getMascotImage()}
            alt="Home"
            onClick={() => setCurrentProject(null)}
            style={{ width: '64px', height: '64px', cursor: 'pointer' }}
            className="home-icon-glow"
            title="Home"
          />
          <ThemeToggle />
        </div>
        
        {/* Main content area - matches main content width */}
        <div className="flex-1 flex items-center justify-center py-4">
          <h1 className="text-xl font-semibold text-primary">Metis</h1>
        </div>
      </div>

      {/* Main content area with sidebar */}
      <div className="flex-1 flex overflow-hidden">
        <ProjectSidebar
          onProjectSelect={handleProjectSelect}
          onShowProjectBrowser={handleShowProjectBrowser}
        />
        <div className="flex-1 flex flex-col overflow-hidden">
          {currentProject ? (
            <KanbanBoard onBackToProjects={() => setCurrentProject(null)} />
          ) : (
            <div className="flex-1 flex items-center justify-center">
              <div className="text-center">
                {/* Mascot */}
                <div>
                  <img
                    src={getMascotImage()}
                    alt="Metis mascot"
                    className="mx-auto animate-bounce-gentle filter drop-shadow-glow"
                    style={{ width: '512px', height: '512px' }}
                  />
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function App() {
  return (
    <ThemeProvider>
      <ProjectProvider>
        <div className="h-screen bg-secondary overflow-hidden">
          <AppContent />
        </div>
      </ProjectProvider>
    </ThemeProvider>
  );
}

export default App;