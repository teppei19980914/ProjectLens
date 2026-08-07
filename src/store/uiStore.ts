import { create } from "zustand";

export type ViewName = "home" | "analyze" | "result" | "export" | "settings" | "auth" | "history";

interface UiState {
  view: ViewName;
  navigate: (view: ViewName) => void;
}

export const useUiStore = create<UiState>((set) => ({
  view: "home",
  navigate: (view) => set({ view }),
}));
