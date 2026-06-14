import ReactDOM from "react-dom/client";
import AppRoot from "./app/AppRoot";
import { installGlobalErrorHandlers } from "./app/installGlobalErrorHandlers";
import { installTestBridge } from "./test/testBridge";
import "./styleEntrypoint";

installTestBridge();
installGlobalErrorHandlers();

ReactDOM.createRoot(document.getElementById("root")).render(
  <AppRoot />
);
