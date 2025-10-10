import "./App.css";
import { ProjectProvider, useProject } from "./contexts/ProjectContext";
import { ProjectBrowser } from "./components/ProjectBrowser";
import { KanbanBoard } from "./components/KanbanBoard";
import { ProjectSidebar } from "./components/ProjectSidebar";
import { ProjectInfo } from "./lib/tauri-api";
import React, { useState } from "react";

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
          <div className="flex-1 flex items-center justify-center bg-gray-50">
            <div className="text-center">
              <div className="text-6xl text-gray-300 mb-4">📋</div>
              <h2 className="text-2xl font-semibold text-gray-700 mb-2">Welcome to Metis</h2>
              <p className="text-gray-500 mb-6">Select a project from the sidebar to get started</p>
              <button
                onClick={handleShowProjectBrowser}
                className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Browse Projects
              </button>
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