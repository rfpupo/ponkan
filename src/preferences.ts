const keys = {
  theme: { current: "ponkan-theme", legacy: "cfg-theme" },
  confirmSave: { current: "ponkan-confirm-save", legacy: "cfg-confirm-save" },
  confirmLoad: { current: "ponkan-confirm-load", legacy: "cfg-confirm-load" },
} as const;
type Preference = keyof typeof keys;
type ReadStore = Pick<Storage, "getItem">;
export function preferenceKey(name: Preference) {
  return keys[name].current;
}
export function readPreference(store: ReadStore, name: Preference) {
  return store.getItem(keys[name].current) ?? store.getItem(keys[name].legacy);
}
