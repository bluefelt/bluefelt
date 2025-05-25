import { applyPatch } from "fast-json-patch";
import { create } from "zustand";

export type LobbyStoreState = {
  bundleMeta?: any;
  state?: any;
  setInitialState: (bundleMeta: any, initialState: any) => void;
  applyDiff: (diff: any[]) => void;
  reset: () => void;
};

export const useLobbyStore = create<LobbyStoreState>((set) => ({
  bundleMeta: undefined,
  state: undefined,
  setInitialState: (bundleMeta, initialState) =>
    set({ bundleMeta, state: initialState }),
  applyDiff: (diff) =>
    set((current) => {
      if (!current.state) return {} as Partial<LobbyStoreState>;
      const nextState = applyPatch(
        { ...current.state },
        diff,
        true,
        false
      ).newDocument;
      return { state: nextState } as Partial<LobbyStoreState>;
    }),
  reset: () => set({ bundleMeta: undefined, state: undefined }),
}));
