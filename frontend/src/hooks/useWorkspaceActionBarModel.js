import { useEffect, useState } from "react";
import { useWorkspaceActionBarActions } from "./useWorkspaceActionBarActions";
import { useWorkspaceActionSelectors } from "./workspaceActionSelectors";

export function useWorkspaceActionBarModel() {
  const [notice, setNotice] = useState(null);
  const selectors = useWorkspaceActionSelectors();
  const actions = useWorkspaceActionBarActions({
    onNotice(nextNotice) {
      setNotice(nextNotice);
    }
  });

  useEffect(() => {
    if (!notice || notice.type === "error") return undefined;
    const timer = window.setTimeout(() => {
      setNotice((current) => (current?.id === notice.id ? null : current));
    }, 2600);
    return () => window.clearTimeout(timer);
  }, [notice]);

  return {
    ...selectors,
    ...actions,
    notice,
    setNotice
  };
}
