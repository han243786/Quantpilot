export function installGlobalErrorHandlers() {
  window.addEventListener("unhandledrejection", (event) => {
    console.error("[UnhandledRejection]", event.reason);
  });
}
