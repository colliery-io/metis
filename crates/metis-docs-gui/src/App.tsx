import "./App.css";
import { ProjectProvider, useProject } from "./contexts/ProjectContext";
import { ProjectBrowser } from "./components/ProjectBrowser";
import { KanbanBoard } from "./components/KanbanBoard";

function AppContent() {
  const { currentProject, setCurrentProject } = useProject();

  const handleBackToProjects = () => {
    setCurrentProject(null);
  };

  if (currentProject) {
    return <KanbanBoard onBackToProjects={handleBackToProjects} />;
  }

  return <ProjectBrowser />;
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