import { createContext, useContext } from "react";

interface VaultContext {
  reload: () => Promise<void>;
  report: (e: unknown) => void;
}

export const VaultCtx = createContext<VaultContext>({
  reload: async () => {},
  report: () => {},
});

export const useVault = () => useContext(VaultCtx);
