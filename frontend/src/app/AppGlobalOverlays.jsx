import CommandPalette from "../components/CommandPalette";
import ToastContainer from "../components/ToastContainer";
import TutorialOverlay from "../components/TutorialOverlay";
import { createTutorialSteps } from "../data/tutorialSteps";
import { useTutorial } from "../hooks/useTutorial";
import { useI18n } from "../i18n";

export default function AppGlobalOverlays({
  commandPaletteOpen,
  onCloseCommandPalette,
}) {
  const { tutorialOpen, closeTutorial } = useTutorial();
  const { t } = useI18n();
  const tutorialSteps = createTutorialSteps(t);

  return (
    <>
      {tutorialOpen && (
        <TutorialOverlay steps={tutorialSteps} onClose={closeTutorial} />
      )}
      <CommandPalette
        open={commandPaletteOpen}
        onClose={onCloseCommandPalette}
      />
      <ToastContainer />
    </>
  );
}
