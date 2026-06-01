import ReactDOM from "react-dom/client";
import AppRoot from "./app/AppRoot";
import { installGlobalErrorHandlers } from "./app/installGlobalErrorHandlers";
import { installTestBridge } from "./test/testBridge";
import "./design-system.css";
import "./styles.css";
import "./styles-responsive-panels.css";
import "./shared.css";
import "@xyflow/react/dist/style.css";

installTestBridge();
installGlobalErrorHandlers();

ReactDOM.createRoot(document.getElementById("root")).render(
  <AppRoot />
);
