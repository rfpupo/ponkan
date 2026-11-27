import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TrashPage } from "./TrashPage";
import type {
  Profile,
  PknPreview,
  Modal,
  DesktopPrefs,
  MainPage,
  TrashEntry,
  TrashConfirmation,
} from "./types";
import { preferenceKey, readPreference } from "./preferences";
import { HomePage } from "./pages/HomePage";
import { SettingsPage } from "./pages/SettingsPage";
import { ProfileRow } from "./components/ProfileRow";
import { AppContextMenu, readClipboard, writeClipboard } from "./AppContextMenu";
import type { Action } from "./contextPolicy";
import { getVersion } from "@tauri-apps/api/app";
import {
  Save,
  Pencil,
  Trash2,
  Users,
  Cat,
  UserPlus,
  Heart,
  Settings,
  Search,
  Play,
  LoaderCircle,
  Check,
  FileUp,
  FileDown,
  House,
  ArrowLeft,
  Info,
  ArrowDownAZ,
  ArrowDownZA,
  Clock,
  Link2,
  Share2,
  Clipboard,
  Copy,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { readLanguage, translate, notificationError, type Language } from "./i18n";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyContent } from "@/components/ui/empty";
import ponkanEmpty from "./ponkan-empty-hd.webp";
import ponkanEmptyLight from "./ponkan-empty-light.png";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { Item, ItemGroup, ItemMedia, ItemContent, ItemTitle } from "@/components/ui/item";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/sonner";
import ponkanLogo from "./ponkan-logo.png";
import { TitleBar } from "./TitleBar";
import { loadProfileImage, removeProfileImage, saveProfileImage } from "./profileImages";
import "./App.css";
import "./MenuPolish.css";

const trashTrackingKey = "ponkan-trash-tracked";
function trackedTrash(): string[] {
  try {
    const a: unknown = JSON.parse(localStorage.getItem(trashTrackingKey) || "[]");
    return Array.isArray(a) ? a.filter((x): x is string => typeof x === "string") : [];
  } catch {
    return [];
  }
}

function App() {
  const [language, setLanguage] = useState<Language>(readLanguage);
  const t = (key: string) => translate(language, key);
  const languageRef = useRef(language);
  languageRef.current = language;
  const errorText = useCallback((err: unknown) => notificationError(languageRef.current, err), []);
  useEffect(() => {
    localStorage.setItem("ponkan-language", language);
    document.documentElement.lang = language;
  }, [language]);
  const [autoBackup, setAutoBackup] = useState(
    () => localStorage.getItem("ponkan-auto-backup") !== "false",
  );
  useEffect(() => {
    localStorage.setItem("ponkan-auto-backup", String(autoBackup));
  }, [autoBackup]);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [busy, setBusy] = useState(false);
  const [initializing, setInitializing] = useState(true);
  const [listReady, setListReady] = useState(false);
  const [modal, setModal] = useState<Modal>(null);
  const [name, setName] = useState("");
  const [target, setTarget] = useState<Profile | null>(null);
  const [applying, setApplying] = useState<Profile | null>(null);
  const [importPreview, setImportPreview] = useState<PknPreview | null>(null);
  const [importName, setImportName] = useState("");
  const [codeImport, setCodeImport] = useState(false);
  const [shareCode, setShareCode] = useState("");
  const [pastingCode, setPastingCode] = useState(false);
  const codeImportEpoch = useRef(0);
  function openCodeImport() {
    codeImportEpoch.current++;
    setShareCode("");
    setCodeImport(true);
  }
  function closeCodeImport() {
    codeImportEpoch.current++;
    setCodeImport(false);
    setShareCode("");
  }
  async function pasteCode() {
    const epoch = codeImportEpoch.current;
    setPastingCode(true);
    try {
      const value = (await readClipboard()).trim();
      if (epoch !== codeImportEpoch.current) return;
      if (value) setShareCode(value.slice(0, 8704));
      else toast.info(t("Área de transferência vazia."));
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setPastingCode(false);
    }
  }
  async function finishCodeImport(event: FormEvent) {
    event.preventDefault();
    if (!shareCode.trim() || busy) return;
    const ok = await task(
      () => invoke("import_share_code", { input: shareCode.trim() }),
      t("Perfil importado"),
    );
    if (ok) {
      closeCodeImport();
      setQuery("");
      setPage("profiles");
    }
  }
  const [shareTarget, setShareTarget] = useState<Profile | null>(null);
  const [sharing, setSharing] = useState(false);
  const [generatedCode, setGeneratedCode] = useState("");
  const [copiedCode, setCopiedCode] = useState(false);
  const [copyingCode, setCopyingCode] = useState(false);
  function openShare(p: Profile) {
    setGeneratedCode("");
    setCopiedCode(false);
    setShareTarget(p);
  }
  function closeShare() {
    setShareTarget(null);
    setGeneratedCode("");
    setCopiedCode(false);
  }
  async function copyGeneratedCode() {
    if (!generatedCode || copyingCode) return;
    setCopyingCode(true);
    try {
      await writeClipboard(generatedCode);
      setCopiedCode(true);
      toast.success(t("Código copiado"));
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setCopyingCode(false);
    }
  }
  const [exportTarget, setExportTarget] = useState<Profile | null>(null);
  const [exportAuthor, setExportAuthor] = useState("");
  const [exportDescription, setExportDescription] = useState("");
  async function chooseImport() {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    try {
      const p = await invoke<PknPreview | null>("select_pkn");
      if (p) {
        if (p.avatar) {
          const img = await createImageBitmap(await (await fetch(p.avatar)).blob());
          img.close();
        }
        setImportName(p.suggested_name);
        setImportPreview(p);
      }
    } catch (err) {
      void invoke("cancel_pkn").catch(() => {});
      toast.error(errorText(err));
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  function closeImport() {
    if (!busy) {
      setImportPreview(null);
      void invoke("cancel_pkn").catch(() => {});
    }
  }
  async function finishImport(event: FormEvent) {
    event.preventDefault();
    const p = importPreview;
    if (!p || !importName.trim()) return;
    const ok = await task(async () => {
      const imported = await invoke<Profile>("import_pkn", { name: importName.trim() });
      if (p.avatar) {
        try {
          const b = await (await fetch(p.avatar)).blob();
          await saveProfileImage(imported.id, new File([b], "avatar", { type: b.type }));
        } catch {
          toast.warning(t("Perfil importado, mas não foi possível salvar a imagem."));
        }
      }
    }, t("Perfil importado"));
    if (ok) setImportPreview(null);
  }
  function showExport(p: Profile) {
    setExportAuthor(p.author ?? "");
    setExportDescription(p.description ?? "");
    setExportTarget(p);
  }
  const shareCopyLock = useRef(false);
  async function generateShareCode(p: Profile) {
    if (shareCopyLock.current) {
      toast.info(t("Aguarde a cópia do código."));
      return;
    }
    shareCopyLock.current = true;
    setSharing(true);
    try {
      const code = await invoke<string>("generate_share_code", { id: p.id });
      setGeneratedCode(code);
      setCopiedCode(false);
    } catch (err) {
      toast.error(errorText(err));
    } finally {
      shareCopyLock.current = false;
      setSharing(false);
    }
  }
  async function finishExport(event: FormEvent) {
    event.preventDefault();
    if (!exportTarget || lock.current) return;
    lock.current = true;
    setBusy(true);
    try {
      const done = await invoke<boolean>("export_pkn", {
        id: exportTarget.id,
        avatar: avatars[exportTarget.id] ?? null,
        author: exportAuthor.trim() || null,
        description: exportDescription.trim() || null,
      });
      if (done) {
        toast.success(t("Perfil exportado"));
        setExportTarget(null);
      }
    } catch (err) {
      toast.error(errorText(err), { duration: 6500 });
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  const [deleting, setDeleting] = useState<Profile | null>(null);

  const [page, setPage] = useState<MainPage>("home");
  const [settingsFrom, setSettingsFrom] = useState<Exclude<MainPage, "settings">>("home");
  const [trashFrom, setTrashFrom] = useState<"home" | "profiles" | "settings">("profiles");
  function openTrash(from: "home" | "profiles" | "settings") {
    if (from !== "settings" || settingsFrom !== "trash") setTrashFrom(from);
    setPage("trash");
  }
  const [desktop, setDesktop] = useState<DesktopPrefs | null>(null);
  const [desktopBusy, setDesktopBusy] = useState(false);
  const desktopLock = useRef(false);
  const pageRef = useRef(page);
  pageRef.current = page;
  const [trashEntries, setTrashEntries] = useState<TrashEntry[]>([]);
  const [trashConfirmation, setTrashConfirmation] = useState<TrashConfirmation | null>(null);
  const [trashLoaded, setTrashLoaded] = useState(false);
  function openSettings() {
    if (page === "settings") return;
    if (page !== "trash" || trashFrom !== "settings") setSettingsFrom(page);
    setPage("settings");
  }
  const updateDesktop = useCallback(
    async (key: string, value: boolean | number | string) => {
      if (desktopLock.current) return;
      desktopLock.current = true;
      setDesktopBusy(true);
      try {
        setDesktop(await invoke<DesktopPrefs>("update_desktop_preference", { key, value }));
      } catch (e) {
        toast.error(errorText(e));
      } finally {
        desktopLock.current = false;
        setDesktopBusy(false);
      }
    },
    [errorText],
  );
  useEffect(() => {
    void invoke<DesktopPrefs>("desktop_preferences")
      .then(setDesktop)
      .catch((e) => toast.error(errorText(e)));
  }, [errorText]);
  const trayLanguageAttempt = useRef("");
  const desktopLanguage = desktop?.language;
  useEffect(() => {
    if (
      desktopLanguage !== undefined &&
      desktopLanguage !== language &&
      !desktopBusy &&
      trayLanguageAttempt.current !== language
    ) {
      trayLanguageAttempt.current = language;
      void updateDesktop("language", language);
    } else if (desktopLanguage === language) {
      trayLanguageAttempt.current = "";
    }
  }, [desktopLanguage, language, desktopBusy, updateDesktop]);
  useEffect(() => {
    let disposed = false;
    const stops: (() => void)[] = [];
    const add = (event: string, handler: () => void) => {
      void listen(event, handler)
        .then((stop) => {
          if (disposed) stop();
          else stops.push(stop);
        })
        .catch(() => {});
    };
    add("ponkan-open-settings", () => {
      const current = pageRef.current;
      if (current !== "settings") {
        if (current !== "trash" || trashFrom !== "settings") setSettingsFrom(current);
        setPage("settings");
      }
    });
    add("ponkan-desktop-busy", () =>
      toast.info(translate(language, "Aguarde a operação atual terminar.")),
    );
    return () => {
      disposed = true;
      stops.forEach((stop) => stop());
    };
  }, [language, trashFrom]);
  const backPage: MainPage =
    page === "settings" ? settingsFrom : page === "trash" ? trashFrom : "home";
  const backLabel =
    backPage === "settings"
      ? "Voltar às configurações"
      : backPage === "trash"
        ? "Voltar à lixeira"
        : backPage === "profiles"
          ? "Voltar aos perfis"
          : "Voltar ao início";
  const [aboutOpen, setAboutOpen] = useState(false);
  const [appVersion, setAppVersion] = useState("");
  function openAbout() {
    setAboutOpen(true);
    void getVersion()
      .then(setAppVersion)
      .catch(() => setAppVersion(""));
  }
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState("az");
  const [theme, setTheme] = useState(() => readPreference(localStorage, "theme") || "dark");
  const [confirmSave, setConfirmSave] = useState(
    () => readPreference(localStorage, "confirmSave") !== "false",
  );
  const [confirmLoad, setConfirmLoad] = useState(
    () => readPreference(localStorage, "confirmLoad") !== "false",
  );
  const [saving, setSaving] = useState<Profile | null>(null);
  const [favorites, setFavorites] = useState<string[]>(() => {
    try {
      const value: unknown = JSON.parse(localStorage.getItem("ponkan-favorites") || "[]");
      return Array.isArray(value) ? value.filter((x): x is string => typeof x === "string") : [];
    } catch {
      return [];
    }
  });
  const [matches, setMatches] = useState<string[]>([]);
  const [avatars, setAvatars] = useState<Record<string, string>>({});
  const avatarInput = useRef<HTMLInputElement>(null);
  const avatarTarget = useRef<string | null>(null);
  function chooseAvatar(id: string) {
    avatarTarget.current = id;
    avatarInput.current?.click();
  }
  function clearAvatar(id: string) {
    try {
      removeProfileImage(id);
      setAvatars((old) => {
        const next = { ...old };
        delete next[id];
        return next;
      });
    } catch (err) {
      toast.error(errorText(err));
    }
  }
  async function changeAvatar(file: File | undefined) {
    const id = avatarTarget.current;
    avatarTarget.current = null;
    if (!file || !id || !previousProfiles.current.some((p) => p.id === id)) return;
    try {
      const value = await saveProfileImage(id, file);
      if (!previousProfiles.current.some((p) => p.id === id)) {
        removeProfileImage(id);
        return;
      }
      setAvatars((old) => ({ ...old, [id]: value }));
    } catch (err) {
      toast.error(errorText(err));
    }
  }
  const [operation, setOperation] = useState<{ id: string; kind: "save" | "load" } | null>(null);
  const [loaded, setLoaded] = useState<string | null>(null);
  const [leaving, setLeaving] = useState<Profile[]>([]);
  const previousProfiles = useRef<Profile[]>([]);
  const removalTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const loadTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const checkSequence = useRef(0);
  const lock = useRef(false);
  useEffect(() => {
    localStorage.setItem("ponkan-favorites", JSON.stringify(favorites));
  }, [favorites]);
  useEffect(
    () => () => {
      if (removalTimer.current) clearTimeout(removalTimer.current);
      if (loadTimer.current) clearTimeout(loadTimer.current);
    },
    [],
  );
  function toggleFavorite(id: string) {
    const wasFavorite = favorites.includes(id);
    setFavorites((old) => (old.includes(id) ? old.filter((x) => x !== id) : [...old, id]));
    const toastId = "favorite-" + id;
    if (wasFavorite) {
      toast.dismiss(toastId);
    } else {
      toast.success(t("Perfil favoritado"), {
        id: toastId,
        duration: 2000,
        icon: <Heart size={16} fill="currentColor" aria-hidden="true" />,
        className: "ponkan-favorite-toast",
      });
    }
  }
  useEffect(() => {
    if (busy || initializing || page !== "profiles") {
      setMatches([]);
      return;
    }
    let disposed = false;
    let checking = false;
    async function check() {
      if (
        checking ||
        disposed ||
        !document.hasFocus() ||
        document.visibilityState !== "visible" ||
        lock.current
      )
        return;
      checking = true;
      const seq = ++checkSequence.current;
      try {
        const result = await invoke<string[]>("matching_profiles");
        if (!disposed && seq === checkSequence.current && document.hasFocus() && !lock.current)
          setMatches(result);
      } catch {
        if (!disposed && seq === checkSequence.current) setMatches([]);
      } finally {
        checking = false;
      }
    }
    function invalidate() {
      ++checkSequence.current;
      setMatches([]);
    }
    function visibility() {
      if (document.visibilityState === "visible") void check();
      else invalidate();
    }
    window.addEventListener("focus", check);
    window.addEventListener("blur", invalidate);
    document.addEventListener("visibilitychange", visibility);
    const timer = setInterval(check, 60000);
    void check();
    return () => {
      disposed = true;
      invalidate();
      clearInterval(timer);
      window.removeEventListener("focus", check);
      window.removeEventListener("blur", invalidate);
      document.removeEventListener("visibilitychange", visibility);
    };
  }, [busy, initializing, page, profiles]);
  useEffect(() => {
    document.documentElement.classList.toggle(
      "dark",
      theme === "dark" ||
        (theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches),
    );
    localStorage.setItem(preferenceKey("theme"), theme);
    localStorage.setItem(preferenceKey("confirmSave"), String(confirmSave));
    localStorage.setItem(preferenceKey("confirmLoad"), String(confirmLoad));
  }, [theme, confirmSave, confirmLoad]);
  const emptyProfiles = listReady && !initializing && profiles.length === 0;
  const favoriteCount = profiles.filter((p) => favorites.includes(p.id)).length;
  const lastSaved = [...profiles].sort((a, b) => b.updated_at.localeCompare(a.updated_at))[0];
  function openProfiles() {
    setQuery("");
    setPage("profiles");
  }
  const visible = profiles
    .filter((p) => p.name.toLocaleLowerCase().includes(query.toLocaleLowerCase()))
    .sort(
      (a, b) =>
        Number(favorites.includes(b.id)) - Number(favorites.includes(a.id)) ||
        (sort === "az"
          ? a.name.localeCompare(b.name)
          : sort === "za"
            ? b.name.localeCompare(a.name)
            : b.updated_at.localeCompare(a.updated_at)),
    );
  const refresh = useCallback(async () => {
    const result = await invoke<Profile[]>("list_profiles");
    const removed = previousProfiles.current.filter((p) => !result.some((x) => x.id === p.id));
    if (removed.length) {
      setLeaving(removed);
      if (removalTimer.current) clearTimeout(removalTimer.current);
      removalTimer.current = setTimeout(() => setLeaving([]), 160);
    }
    setAvatars(
      Object.fromEntries(
        result.flatMap((p) => {
          const img = loadProfileImage(p.id);
          return img ? [[p.id, img]] : [];
        }),
      ),
    );
    previousProfiles.current = result;
    setProfiles(result);
    setListReady(true);
    try {
      const entries = await invoke<TrashEntry[]>("list_trash");
      setTrashEntries(entries);
      setTrashLoaded(true);
      const active = new Set(result.map((x) => x.id)),
        trashed = new Set(entries.map((x) => x.profile.id));
      const tracked = trackedTrash(),
        gone = tracked.filter((id) => !active.has(id) && !trashed.has(id));
      if (gone.length) {
        for (const id of gone) {
          try {
            removeProfileImage(id);
          } catch (error) {
            console.warn("Could not remove cached profile image", error);
          }
        }
        setFavorites((old) => old.filter((id) => !gone.includes(id)));
      }
      localStorage.setItem(
        trashTrackingKey,
        JSON.stringify(tracked.filter((id) => trashed.has(id))),
      );
    } catch (e) {
      setTrashLoaded(false);
      console.warn("Trash refresh", e);
    }
  }, []);
  useEffect(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    void listen("ponkan-trash-changed", () => {
      if (!lock.current) void refresh().catch(() => {});
    })
      .then((f) => {
        if (disposed) f();
        else stop = f;
      })
      .catch(() => {});
    return () => {
      disposed = true;
      stop?.();
    };
  }, [refresh]);
  useEffect(() => {
    if (page === "trash") void refresh().catch((e) => toast.error(errorText(e)));
  }, [page, refresh, errorText]);
  function moveToTrash(p: Profile) {
    return task(async () => {
      const tracked = trackedTrash();
      if (!tracked.includes(p.id))
        localStorage.setItem(trashTrackingKey, JSON.stringify([...tracked, p.id]));
      await invoke("delete_profile", {
        id: p.id,
        avatar: loadProfileImage(p.id) ?? null,
        favorite: favorites.includes(p.id),
      });
    }, t("Perfil movido para a lixeira"));
  }
  async function restoreTrash(ids: string[]) {
    const ok = await task(
      async () => {
        for (const id of ids) {
          const e = await invoke<TrashEntry>("restore_trash", { id });
          if (e.avatar) {
            try {
              localStorage.setItem("ponkan-avatar:" + id, e.avatar);
            } catch {
              toast.warning(t("Perfil restaurado, mas não foi possível salvar a imagem."));
            }
          }
          setFavorites((old) =>
            e.favorite ? [...new Set([...old, id])] : old.filter((x) => x !== id),
          );
        }
      },
      t(ids.length === 1 ? "Perfil restaurado" : "Perfis restaurados"),
    );
    if (!ok) await refresh().catch(() => {});
    return ok;
  }
  async function purgeTrash(ids: string[]) {
    const ok = await task(async () => {
      for (const id of ids) {
        await invoke("purge_trash", { id });
        try {
          removeProfileImage(id);
        } catch (error) {
          console.warn("Could not remove cached profile image", error);
        }
        setFavorites((old) => old.filter((x) => x !== id));
      }
    }, t("Perfis excluídos permanentemente"));
    if (!ok) await refresh().catch(() => {});
    return ok;
  }

  useEffect(() => {
    refresh()
      .catch((err) => toast.error(errorText(err)))
      .finally(() => setInitializing(false));
  }, [refresh, errorText]);

  useEffect(() => {
    if (initializing) return;
    let disposed = false;
    async function receive() {
      if (
        disposed ||
        lock.current ||
        document.visibilityState !== "visible" ||
        !document.hasFocus()
      )
        return;
      let pending: boolean;
      try {
        pending = await invoke<boolean>("has_pending_share");
      } catch {
        return;
      }
      if (!pending || disposed || lock.current) return;
      lock.current = true;
      try {
        const p = await invoke<Profile | null>("import_next_share");
        if (p && !disposed) {
          await refresh();
          setQuery("");
          setPage("profiles");
          toast.success(translate(language, "Perfil importado"));
        }
      } catch (err) {
        if (!disposed) toast.error(notificationError(language, err));
      } finally {
        lock.current = false;
      }
    }
    const timer = setInterval(() => void receive(), 1000);
    const wake = () => {
      void receive();
    };
    document.addEventListener("visibilitychange", wake);
    window.addEventListener("focus", wake);
    void receive();
    return () => {
      disposed = true;
      clearInterval(timer);
      document.removeEventListener("visibilitychange", wake);
      window.removeEventListener("focus", wake);
    };
  }, [initializing, refresh, language]);

  async function task(
    action: () => Promise<unknown>,
    success: string,
    feedback?: { id: string; kind: "save" | "load" },
  ) {
    if (lock.current) return false;
    lock.current = true;
    ++checkSequence.current;
    setMatches([]);
    setLoaded(null);
    if (loadTimer.current) clearTimeout(loadTimer.current);
    setOperation(feedback ?? null);
    setBusy(true);
    try {
      await action();
      await refresh();
      toast.success(success);
      if (feedback?.kind === "load") {
        setLoaded(feedback.id);
        loadTimer.current = setTimeout(() => setLoaded(null), 1000);
      }
      return true;
    } catch (err) {
      toast.error(errorText(err), { duration: 6500 });
      return false;
    } finally {
      lock.current = false;
      setOperation(null);
      setBusy(false);
    }
  }

  async function submitModal(event: FormEvent) {
    event.preventDefault();
    const clean = name.trim();
    if (!clean) return;
    const ok = await task(
      modal === "rename" && target
        ? () => invoke("rename_profile", { id: target.id, name: clean })
        : () => invoke("save_profile", { name: clean, id: null }),
      modal === "rename" ? t("Nome atualizado") : t("Perfil salvo"),
    );
    if (ok) setModal(null);
  }

  function showModal(kind: Modal, profile: Profile | null = null) {
    setTarget(profile);
    setName(profile?.name ?? "");
    setModal(kind);
  }

  function contextAction(action: Action | "import-file" | "import-code", id?: string) {
    const p = profiles.find((x) => x.id === id),
      entry = trashEntries.find((x) => x.profile.id === id);
    if (action === "profiles") {
      openProfiles();
      return;
    }
    if (action === "settings") {
      openSettings();
      return;
    }
    if (action === "trash") {
      openTrash("home");
      return;
    }
    if (action === "about") {
      openAbout();
      return;
    }
    if (busy || lock.current || sharing) return;
    switch (action) {
      case "new":
        showModal("create");
        break;
      case "import-file":
        void chooseImport();
        break;
      case "import-code":
        openCodeImport();
        break;
      case "share":
        if (p) openShare(p);
        break;
      case "rename":
        if (p) showModal("rename", p);
        break;
      case "delete":
        if (p) setDeleting(p);
        break;
      case "restore":
        if (entry) void restoreTrash([entry.profile.id]);
        break;
      case "purge":
        if (entry) setTrashConfirmation({ ids: [entry.profile.id], name: entry.profile.name });
        break;
      case "empty":
        if (trashEntries.length)
          setTrashConfirmation({ ids: trashEntries.map((e) => e.profile.id) });
        break;
    }
  }
  return (
    <AppContextMenu
      page={page}
      busy={busy || sharing}
      hasTrash={trashEntries.length > 0}
      t={t}
      onAction={contextAction}
      onError={(e) => toast.error(errorText(e))}
    >
      <div className="ponkan-window">
        <TitleBar t={t} />
        <input
          ref={avatarInput}
          type="file"
          accept="image/png,image/jpeg,image/webp"
          hidden
          onChange={(event) => {
            const file = event.currentTarget.files?.[0];
            event.currentTarget.value = "";
            void changeAvatar(file);
          }}
        />
        <div
          className={`linear-shell ${theme === "dark" || (theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches) ? "dark" : ""}`}
        >
          <aside className="linear-sidebar ponkan-sidebar-hidden" hidden>
            <div className="linear-brand">
              <img src={ponkanLogo} alt="pon//kan." draggable={false} />
            </div>
            <nav>
              <button className={page === "home" ? "selected" : ""} onClick={() => setPage("home")}>
                <House size={18} />
                {t("Início")}
              </button>
              <button
                className={page === "profiles" ? "selected" : ""}
                onClick={() => setPage("profiles")}
              >
                <Users size={18} />
                {t("Perfis")}
              </button>
              <button className={page === "settings" ? "selected" : ""} onClick={openSettings}>
                <Settings size={18} />
                {t("Configurações")}
              </button>
            </nav>
          </aside>
          <div className="ponkan-page-frame">
            {page !== "home" && (
              <Button
                variant="ghost"
                size="icon"
                className="ponkan-home-back"
                aria-label={t(backLabel)}
                title={t(backLabel)}
                onClick={() => setPage(backPage)}
              >
                <ArrowLeft size={18} />
              </Button>
            )}
            <main
              className={`linear-main ponkan-page${page === "home" ? " ponkan-home-page" : ""}${page === "profiles" && emptyProfiles ? " ponkan-empty-page" : ""}${page === "trash" && trashLoaded && trashEntries.length === 0 ? " ponkan-empty-page ponkan-trash-empty-page" : ""}`}
              key={page}
            >
              {page !== "home" && (
                <header className="linear-header profiles-heading">
                  <h1>
                    {page === "profiles"
                      ? t("PERFIS")
                      : page === "trash"
                        ? t("Lixeira")
                        : t("Configurações")}
                  </h1>
                  {page !== "settings" && (
                    <div className="ponkan-profile-header-actions">
                      {page === "profiles" && profiles.length > 0 && (
                        <Button
                          className="new-profile-action"
                          disabled={busy}
                          onClick={() => showModal("create")}
                        >
                          <UserPlus size={16} /> {t("Novo perfil")}
                        </Button>
                      )}
                      <Button
                        size="icon-lg"
                        variant="ghost"
                        className="ponkan-settings-shortcut"
                        aria-label={t("Configurações")}
                        title={t("Configurações")}
                        onClick={openSettings}
                      >
                        <Settings size={21} />
                      </Button>
                    </div>
                  )}
                </header>
              )}
              {page === "home" ? (
                <HomePage
                  t={t}
                  language={language}
                  setLanguage={setLanguage}
                  openProfiles={openProfiles}
                  openAbout={openAbout}
                  openSettings={openSettings}
                  listReady={listReady}
                  initializing={initializing}
                  profiles={profiles}
                  favoriteCount={favoriteCount}
                  lastSaved={lastSaved}
                />
              ) : page === "profiles" ? (
                emptyProfiles ? (
                  <Empty className="ponkan-empty-state">
                    <EmptyHeader>
                      <EmptyMedia>
                        <img
                          className="ponkan-theme-dark"
                          src={ponkanEmpty}
                          alt=""
                          draggable={false}
                        />
                        <img
                          className="ponkan-theme-light"
                          src={ponkanEmptyLight}
                          alt=""
                          draggable={false}
                        />
                      </EmptyMedia>
                      <EmptyTitle>{t("Nenhum perfil por aqui...")}</EmptyTitle>
                    </EmptyHeader>
                    <EmptyContent className="ponkan-empty-actions">
                      <Button disabled={busy} onClick={() => showModal("create")}>
                        <UserPlus size={16} /> {t("Novo perfil")}
                      </Button>
                      <div className="ponkan-import-actions">
                        <Button
                          size="icon-lg"
                          variant="outline"
                          className="ponkan-trash-shortcut"
                          aria-label={t("Lixeira")}
                          title={t("Lixeira")}
                          onClick={() => openTrash("profiles")}
                        >
                          <Trash2 size={20} />
                        </Button>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="outline" disabled={busy}>
                              <FileUp size={16} /> {t("Importar")}
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onSelect={() => void chooseImport()}>
                              <FileUp size={16} /> {t("Importar arquivo (.pkn)")}
                            </DropdownMenuItem>
                            <DropdownMenuItem onSelect={openCodeImport}>
                              <Link2 size={16} /> {t("Importar código")}
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </div>
                    </EmptyContent>
                  </Empty>
                ) : (
                  <>
                    {profiles.length > 0 && (
                      <div className="linear-toolbar">
                        <label className="linear-search">
                          <Search size={16} />
                          <input
                            aria-label={t("Buscar perfil")}
                            placeholder={t("Buscar perfil...")}
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                          />
                        </label>
                        <Select value={sort} onValueChange={setSort}>
                          <SelectTrigger
                            aria-label={t("Ordenar perfis")}
                            className="ponkan-sort-trigger"
                          >
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="az">
                              <ArrowDownAZ size={16} aria-hidden="true" />
                              {t("Nome (A – Z)")}
                            </SelectItem>
                            <SelectItem value="za">
                              <ArrowDownZA size={16} aria-hidden="true" />
                              {t("Nome (Z – A)")}
                            </SelectItem>
                            <SelectItem value="recent">
                              <Clock size={16} aria-hidden="true" />
                              {t("Mais recentes")}
                            </SelectItem>
                          </SelectContent>
                        </Select>
                        <div className="ponkan-import-actions">
                          <Button
                            size="icon-lg"
                            variant="outline"
                            className="ponkan-trash-shortcut"
                            aria-label={t("Lixeira")}
                            title={t("Lixeira")}
                            onClick={() => openTrash("profiles")}
                          >
                            <Trash2 size={20} />
                          </Button>
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button variant="outline" disabled={busy}>
                                <FileUp size={16} /> {t("Importar")}
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                              <DropdownMenuItem onSelect={() => void chooseImport()}>
                                <FileUp size={16} /> {t("Importar arquivo (.pkn)")}
                              </DropdownMenuItem>
                              <DropdownMenuItem onSelect={openCodeImport}>
                                <Link2 size={16} /> {t("Importar código")}
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </div>
                      </div>
                    )}
                    <ItemGroup className="linear-list" aria-label={t("Perfis salvos")}>
                      {!initializing && profiles.length > 0 && visible.length === 0 && (
                        <div className="linear-empty">{t("Nenhum perfil encontrado.")}</div>
                      )}
                      {visible.map((profile) => (
                        <ProfileRow
                          key={profile.id}
                          profile={profile}
                          t={t}
                          favorite={favorites.includes(profile.id)}
                          matching={matches.includes(profile.id)}
                          avatar={avatars[profile.id]}
                          busy={busy}
                          operationKind={operation?.id === profile.id ? operation.kind : null}
                          loaded={loaded === profile.id}
                          onFavorite={() => toggleFavorite(profile.id)}
                          onApply={() =>
                            confirmLoad || profile.portable
                              ? setApplying(profile)
                              : void task(
                                  () => invoke("apply_profile", { id: profile.id, autoBackup }),
                                  t("Perfil carregado"),
                                  { id: profile.id, kind: "load" },
                                )
                          }
                          onSave={() =>
                            confirmSave
                              ? setSaving(profile)
                              : void task(
                                  () =>
                                    invoke("save_profile", { id: profile.id, name: profile.name }),
                                  t("Perfil atualizado"),
                                  { id: profile.id, kind: "save" },
                                )
                          }
                          onRename={() => showModal("rename", profile)}
                          onAvatarChange={() => chooseAvatar(profile.id)}
                          onAvatarClear={() => clearAvatar(profile.id)}
                          onShare={() => openShare(profile)}
                          onDelete={() => setDeleting(profile)}
                        />
                      ))}
                      {leaving.map((p) => (
                        <Item
                          variant="outline"
                          className="linear-row ponkan-profile-item ponkan-leaving"
                          key={`leaving-${p.id}`}
                          aria-hidden="true"
                        >
                          <span className="profile-status-slot" />
                          <span className="favorite-action-placeholder" />
                          <ItemMedia>
                            <Avatar size="lg">
                              <AvatarFallback>
                                <Cat size={20} />
                              </AvatarFallback>
                            </Avatar>
                          </ItemMedia>
                          <ItemContent>
                            <ItemTitle>{p.name}</ItemTitle>
                          </ItemContent>
                        </Item>
                      ))}
                    </ItemGroup>
                  </>
                )
              ) : page === "trash" ? (
                <TrashPage
                  entries={trashEntries}
                  busy={busy}
                  loaded={trashLoaded}
                  language={language}
                  t={t}
                  restore={restoreTrash}
                  purge={purgeTrash}
                  confirmation={trashConfirmation}
                  setConfirmation={setTrashConfirmation}
                />
              ) : (
                <SettingsPage
                  t={t}
                  theme={theme}
                  setTheme={setTheme}
                  language={language}
                  setLanguage={setLanguage}
                  confirmLoad={confirmLoad}
                  setConfirmLoad={setConfirmLoad}
                  confirmSave={confirmSave}
                  setConfirmSave={setConfirmSave}
                  autoBackup={autoBackup}
                  setAutoBackup={setAutoBackup}
                  desktop={desktop}
                  desktopBusy={desktopBusy}
                  updateDesktop={updateDesktop}
                  openTrash={openTrash}
                />
              )}
            </main>
          </div>
          <AlertDialog
            open={!!saving}
            onOpenChange={(open) => {
              if (!open && !busy) setSaving(null);
            }}
          >
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle className="ponkan-dialog-title">
                  <Save size={19} /> {t("Sobrescrever")} {saving?.name}?
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {t("As configurações atuais substituirão o perfil salvo.")}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel disabled={busy}>{t("Cancelar")}</AlertDialogCancel>
                <AlertDialogAction
                  disabled={busy}
                  onClick={(e) => {
                    e.preventDefault();
                    const p = saving;
                    if (p)
                      void task(
                        () => invoke("save_profile", { id: p.id, name: p.name }),
                        t("Perfil atualizado"),
                        { id: p.id, kind: "save" },
                      ).then((ok) => {
                        if (ok) setSaving(null);
                      });
                  }}
                >
                  {operation?.kind === "save" && <LoaderCircle size={16} className="ponkan-spin" />}
                  {t("Salvar")}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
          <Dialog
            open={modal !== null}
            onOpenChange={(open) => {
              if (!open && !busy) setModal(null);
            }}
          >
            <DialogContent closeLabel={t("Fechar")} className="sm:max-w-[390px]">
              <DialogHeader>
                <DialogTitle className="ponkan-dialog-title">
                  {modal === "rename" ? <Pencil size={19} /> : <UserPlus size={19} />}{" "}
                  {modal === "rename" ? t("Renomear perfil") : t("Salvar perfil")}
                </DialogTitle>
              </DialogHeader>
              <form onSubmit={(event) => void submitModal(event)} className="space-y-5">
                <Input
                  autoFocus
                  value={name}
                  maxLength={48}
                  onChange={(event) => setName(event.target.value)}
                  placeholder={t("Nome do perfil")}
                />
                <DialogFooter>
                  <Button
                    variant="outline"
                    type="button"
                    onClick={() => setModal(null)}
                    disabled={busy}
                  >
                    {t("Cancelar")}
                  </Button>
                  <Button type="submit" disabled={busy || !name.trim()}>
                    {modal === "rename" ? t("Renomear") : t("Salvar")}
                  </Button>
                </DialogFooter>
              </form>
            </DialogContent>
          </Dialog>

          <AlertDialog
            open={!!applying}
            onOpenChange={(open) => {
              if (!open && !busy) setApplying(null);
            }}
          >
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle className="ponkan-dialog-title">
                  <Play size={19} /> {t("Aplicar")} {applying?.name}?
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {applying?.portable
                    ? t(
                        "O perfil importado aplicará as configurações locais e os campos de nuvem compatíveis. Dados específicos da conta e do computador serão protegidos.",
                      )
                    : t(
                        autoBackup
                          ? "As configurações atuais serão substituídas. Um backup será criado automaticamente."
                          : "As configurações atuais serão substituídas sem criar um backup permanente.",
                      )}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel disabled={busy}>{t("Cancelar")}</AlertDialogCancel>
                <AlertDialogAction
                  disabled={busy}
                  onClick={(event) => {
                    event.preventDefault();
                    const p = applying;
                    if (p)
                      void task(
                        () => invoke("apply_profile", { id: p.id, autoBackup }),
                        t("Perfil carregado"),
                        { id: p.id, kind: "load" },
                      ).then((ok) => {
                        if (ok) setApplying(null);
                      });
                  }}
                >
                  {operation?.kind === "load" && <LoaderCircle size={16} className="ponkan-spin" />}
                  {t("Aplicar")}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>

          <AlertDialog
            open={!!deleting}
            onOpenChange={(open) => {
              if (!open && !busy) setDeleting(null);
            }}
          >
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle className="ponkan-dialog-title ponkan-dialog-destructive">
                  <Trash2 size={19} /> {t("Excluir")} {deleting?.name}?
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {t("O perfil será movido para a lixeira.")}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel disabled={busy}>{t("Cancelar")}</AlertDialogCancel>
                <AlertDialogAction
                  disabled={busy}
                  onClick={(event) => {
                    event.preventDefault();
                    const p = deleting;
                    if (p)
                      void moveToTrash(p).then((ok) => {
                        if (ok) setDeleting(null);
                      });
                  }}
                >
                  {t("Excluir")}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
          <Dialog
            open={codeImport}
            onOpenChange={(open) => {
              if (!open && !busy) closeCodeImport();
            }}
          >
            <DialogContent closeLabel={t("Fechar")} className="sm:max-w-[390px]">
              <DialogHeader>
                <DialogTitle className="ponkan-dialog-title">
                  <Link2 size={19} /> {t("Importar código")}
                </DialogTitle>
              </DialogHeader>
              <form onSubmit={finishCodeImport} className="space-y-4">
                <div className="ponkan-code-input">
                  <Input
                    aria-label={t("Código do perfil")}
                    placeholder="pkn.Perfi!a8K2m7#"
                    value={shareCode}
                    onChange={(e) => setShareCode(e.target.value)}
                    autoComplete="off"
                    spellCheck={false}
                    maxLength={8704}
                    disabled={busy}
                  />
                  <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    aria-label={t("Colar")}
                    title={t("Colar")}
                    disabled={busy || pastingCode}
                    onClick={() => void pasteCode()}
                  >
                    <Clipboard size={17} />
                  </Button>
                </div>
                <DialogFooter>
                  <Button variant="outline" type="button" disabled={busy} onClick={closeCodeImport}>
                    {t("Cancelar")}
                  </Button>
                  <Button type="submit" disabled={busy || !shareCode.trim()}>
                    {busy && <LoaderCircle size={16} className="ponkan-spin" />}
                    {t("Importar")}
                  </Button>
                </DialogFooter>
              </form>
            </DialogContent>
          </Dialog>
          <Dialog
            open={!!importPreview}
            onOpenChange={(open) => {
              if (!open) closeImport();
            }}
          >
            <DialogContent closeLabel={t("Fechar")} className="sm:max-w-[420px]">
              <DialogHeader>
                <DialogTitle className="ponkan-dialog-title">
                  <FileUp size={19} /> {t("Importar perfil")}
                </DialogTitle>
              </DialogHeader>
              {importPreview && (
                <form onSubmit={finishImport} className="space-y-4">
                  <div className="flex items-center gap-3">
                    <Avatar size="lg">
                      {importPreview.avatar && <AvatarImage src={importPreview.avatar} alt="" />}
                      <AvatarFallback>
                        <Cat size={20} />
                      </AvatarFallback>
                    </Avatar>
                    <div className="min-w-0">
                      <strong>{importPreview.manifest.name}</strong>
                      {importPreview.manifest.author && (
                        <p className="text-sm text-muted-foreground">
                          {importPreview.manifest.author}
                        </p>
                      )}
                    </div>
                  </div>
                  {importPreview.manifest.description && (
                    <p className="text-sm break-words max-h-20 overflow-auto">
                      {importPreview.manifest.description}
                    </p>
                  )}
                  <label className="grid gap-1.5 text-sm">
                    {t("Nome do perfil")}
                    <Input
                      value={importName}
                      maxLength={48}
                      onChange={(e) => setImportName(e.target.value)}
                    />
                  </label>
                  {importPreview.duplicate && (
                    <p className="text-sm text-muted-foreground">
                      {t("Já existe um perfil semelhante. Será criada uma cópia independente.")}
                    </p>
                  )}
                  {importPreview.edited && (
                    <p className="text-sm text-muted-foreground">
                      {t("Edições manuais detectadas e validadas.")}
                    </p>
                  )}
                  <p className="text-sm text-muted-foreground">
                    {t(
                      "Importar não altera o jogo. O carregamento exige que a conta de destino tenha configurações locais identificáveis.",
                    )}
                  </p>
                  <DialogFooter>
                    <Button variant="outline" type="button" disabled={busy} onClick={closeImport}>
                      {t("Cancelar")}
                    </Button>
                    <Button type="submit" disabled={busy || !importName.trim()}>
                      {t("Importar perfil")}
                    </Button>
                  </DialogFooter>
                </form>
              )}
            </DialogContent>
          </Dialog>
          <Dialog
            open={!!shareTarget}
            onOpenChange={(open) => {
              if (!open && !sharing && !copyingCode) closeShare();
            }}
          >
            <DialogContent
              closeLabel={t("Fechar")}
              showCloseButton={!sharing}
              className="sm:max-w-[390px]"
            >
              <DialogHeader>
                <DialogTitle className="ponkan-dialog-title">
                  {generatedCode ? <Link2 size={19} /> : <Share2 size={19} />}{" "}
                  {t(generatedCode ? "Código de compartilhamento" : "Compartilhar perfil")}
                </DialogTitle>
                {!generatedCode && (
                  <DialogDescription>{t("Como deseja compartilhar o perfil?")}</DialogDescription>
                )}
              </DialogHeader>
              {generatedCode ? (
                <div className="ponkan-code-input">
                  <Input
                    aria-label={t("Código de compartilhamento")}
                    value={generatedCode}
                    readOnly
                    onFocus={(e) => e.currentTarget.select()}
                    spellCheck={false}
                  />
                  <Button
                    type="button"
                    size="icon"
                    variant="outline"
                    aria-label={t("Copiar código")}
                    title={t("Copiar código")}
                    disabled={copyingCode}
                    onClick={() => void copyGeneratedCode()}
                  >
                    {copiedCode ? <Check size={17} /> : <Copy size={17} />}
                  </Button>
                </div>
              ) : (
                <div className="grid gap-2">
                  <Button
                    className="h-11 justify-start gap-3"
                    disabled={sharing || busy || !shareTarget}
                    onClick={() => {
                      if (shareTarget) void generateShareCode(shareTarget);
                    }}
                  >
                    {sharing ? (
                      <LoaderCircle size={17} className="ponkan-spin" />
                    ) : (
                      <Link2 size={17} />
                    )}{" "}
                    {t("Gerar código de compartilhamento")}
                  </Button>
                  <Button
                    variant="outline"
                    className="h-11 justify-start gap-3"
                    disabled={sharing || busy || !shareTarget}
                    onClick={() => {
                      if (shareTarget) {
                        const profile = shareTarget;
                        closeShare();
                        showExport(profile);
                      }
                    }}
                  >
                    <FileDown size={17} /> {t("Exportar arquivo (.pkn)")}
                  </Button>
                </div>
              )}
            </DialogContent>
          </Dialog>
          <Dialog
            open={!!exportTarget}
            onOpenChange={(open) => {
              if (!open && !busy) setExportTarget(null);
            }}
          >
            <DialogContent closeLabel={t("Fechar")} className="sm:max-w-[420px]">
              <DialogHeader>
                <DialogTitle className="ponkan-dialog-title">
                  <FileDown size={19} /> {t("Exportar perfil")}
                </DialogTitle>
              </DialogHeader>
              <form onSubmit={finishExport} className="space-y-4">
                <strong className="block truncate">{exportTarget?.name}</strong>
                <label className="grid gap-1.5 text-sm">
                  {t("Autor (opcional)")}
                  <Input
                    value={exportAuthor}
                    maxLength={80}
                    onChange={(e) => setExportAuthor(e.target.value)}
                  />
                </label>
                <label className="grid gap-1.5 text-sm">
                  {t("Descrição (opcional)")}
                  <textarea
                    className="min-h-20 w-full rounded-lg border border-input bg-transparent px-2.5 py-2 text-sm"
                    maxLength={512}
                    value={exportDescription}
                    onChange={(e) => setExportDescription(e.target.value)}
                  />
                </label>
                {!exportTarget?.shared && (
                  <p className="text-xs text-muted-foreground">
                    {t(
                      "O arquivo inclui dados originais de configuração e pode conter identificadores da conta e do computador. Compartilhe somente com pessoas de confiança.",
                    )}
                  </p>
                )}
                <DialogFooter>
                  <Button
                    variant="outline"
                    type="button"
                    disabled={busy}
                    onClick={() => setExportTarget(null)}
                  >
                    {t("Cancelar")}
                  </Button>
                  <Button type="submit" disabled={busy}>
                    {t("Exportar (.pkn)")}
                  </Button>
                </DialogFooter>
              </form>
            </DialogContent>
          </Dialog>
          <Dialog open={aboutOpen} onOpenChange={setAboutOpen}>
            <DialogContent closeLabel={t("Fechar")} className="sm:max-w-[360px]">
              <DialogHeader>
                <DialogTitle className="ponkan-dialog-title">
                  <Info size={19} /> {t("Sobre")}
                </DialogTitle>
              </DialogHeader>
              <div className="ponkan-about">
                <img src={ponkanLogo} alt="pon//kan." draggable={false} />
                {appVersion && (
                  <span>
                    {t("Versão")} {appVersion}
                  </span>
                )}
              </div>
              <p className="text-center text-sm text-muted-foreground">
                {t("Gerenciador de perfis do VALORANT.")}
              </p>
              <p className="text-xs leading-relaxed text-muted-foreground">
                {t(
                  "Ponkan não é endossado pela Riot Games e não reflete as opiniões da Riot Games ou de qualquer pessoa oficialmente envolvida na produção ou administração das propriedades da Riot Games. Riot Games e todas as propriedades associadas são marcas comerciais ou registradas da Riot Games, Inc.",
                )}
              </p>
            </DialogContent>
          </Dialog>
          <Toaster richColors position="bottom-right" />
        </div>
      </div>
    </AppContextMenu>
  );
}

export default App;
