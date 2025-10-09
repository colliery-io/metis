import "./App.css";
import { ProjectProvider } from "./contexts/ProjectContext";
import { ProjectBrowser } from "./components/ProjectBrowser";

function App() {
  return (
    <ProjectProvider>
      <div className="min-h-screen bg-gray-50">
        <ProjectBrowser />
      </div>
    </ProjectProvider>
  );
}

export default App;