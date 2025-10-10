import "./App.css";
import { ProjectProvider, useProject } from "./contexts/ProjectContext";
import { ProjectBrowser } from "./components/ProjectBrowser";
import { KanbanBoard } from "./components/KanbanBoard";
import { ProjectSidebar } from "./components/ProjectSidebar";
import { ProjectInfo } from "./lib/tauri-api";
import { useState } from "react";

function AppContent() {
  const { currentProject, setCurrentProject, loadProject } = useProject();
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

  // Show full-screen project browser when explicitly requested
  if (showProjectBrowser) {
    return (
      <div className="h-screen flex flex-col">
        <div className="p-4 bg-white border-b border-gray-200 flex items-center justify-between">
          <h1 className="text-xl font-semibold text-gray-900">Select Project</h1>
          <button
            onClick={handleBackFromBrowser}
            className="px-4 py-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded-lg transition-colors"
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

  // Main app layout with sidebar
  return (
    <div className="h-screen flex">
      <ProjectSidebar
        onProjectSelect={handleProjectSelect}
        onShowProjectBrowser={handleShowProjectBrowser}
      />
      <div className="flex-1 flex flex-col overflow-hidden">
        {currentProject ? (
          <KanbanBoard onBackToProjects={() => setCurrentProject(null)} />
        ) : (
          <div className="flex-1 flex items-center justify-center bg-white">
            <div className="text-center">
              <div className="w-64 h-32 border-2 border-gray-300 rounded-lg flex items-center justify-center bg-gray-50">
                <span className="text-2xl font-medium text-gray-600">Metis</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function App() {
  return (
    <ProjectProvider>
      <div className="min-h-screen bg-gray-50">
        <AppContent />
      </div>
    </ProjectProvider>
  );
}

export default App;