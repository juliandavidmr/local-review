import { TooltipProvider } from "@/components/ui/tooltip";
import { LocalReviewWorkspace } from "@/features/local-review";
import "./App.css";

function App() {
  return (
    <TooltipProvider>
      <LocalReviewWorkspace />
    </TooltipProvider>
  );
}

export default App;
