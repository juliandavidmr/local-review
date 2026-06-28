import { TooltipProvider } from "@/components/ui/tooltip";
import { LocalReviewWorkspace } from "@/features/local-review";
import { useSystemColorScheme } from "@/hooks/useSystemColorScheme";
import "./App.css";

function App() {
  useSystemColorScheme();

  return (
    <TooltipProvider>
      <LocalReviewWorkspace />
    </TooltipProvider>
  );
}

export default App;
