import { useState, useRef, type ReactNode, type MouseEvent } from "react";
import { ContextMenu as CM } from "radix-ui";
import {
  Users,
  Settings,
  Trash2,
  Info,
  UserPlus,
  FileUp,
  Link2,
  Share2,
  Pencil,
  RotateCcw,
  Copy,
  Clipboard,
  Scissors,
  TextSelect,
  ChevronRight,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import {
  contextActions,
  textActions,
  replaceSelection,
  type Page,
  type Kind,
  type Action,
} from "./contextPolicy";
import type { Translate } from "./i18n";
type Field = HTMLInputElement | HTMLTextAreaElement;
type Snapshot = {
  kind: Kind;
  id?: string;
  actions: Action[];
  field?: Field;
  start?: number;
  end?: number;
  text?: string;
  container?: HTMLElement;
  focus?: HTMLElement;
};
type Props = {
  children: ReactNode;
  page: Page;
  hasTrash: boolean;
  busy: boolean;
  t: Translate;
  onAction: (action: Action | "import-file" | "import-code", id?: string) => void;
  onError: (e: unknown) => void;
};
const icons = {
  profiles: Users,
  settings: Settings,
  trash: Trash2,
  about: Info,
  new: UserPlus,
  import: FileUp,
  share: Share2,
  rename: Pencil,
  delete: Trash2,
  restore: RotateCcw,
  purge: Trash2,
  empty: Trash2,
  copy: Copy,
  paste: Clipboard,
  cut: Scissors,
  "select-all": TextSelect,
};
const labels: Record<Action, string> = {
  profiles: "Perfis",
  settings: "Configurações",
  trash: "Lixeira",
  about: "Sobre",
  new: "Novo perfil",
  import: "Importar",
  share: "Compartilhar",
  rename: "Renomear",
  delete: "Excluir",
  restore: "Restaurar",
  purge: "Excluir permanentemente",
  empty: "Esvaziar lixeira",
  copy: "Copiar",
  paste: "Colar",
  cut: "Recortar",
  "select-all": "Selecionar tudo",
};
export const readClipboard = () => invoke<string>("clipboard_read_text");
export const writeClipboard = (text: string) => invoke<void>("clipboard_write_text", { text });
function editableField(target: Element): Field | undefined {
  const el = target.closest("input,textarea");
  if (el instanceof HTMLTextAreaElement && !el.disabled) return el;
  if (
    el instanceof HTMLInputElement &&
    !el.disabled &&
    ["text", "search", "url", "tel"].includes(el.type)
  )
    return el;
  return undefined;
}
function selectedText(target: Element): string {
  const sel = window.getSelection();
  if (!sel || sel.isCollapsed || !sel.rangeCount) return "";
  const range = sel.getRangeAt(0);
  try {
    return range.intersectsNode(target) ? sel.toString() : "";
  } catch {
    return "";
  }
}
function setText(field: Field, start: number, end: number, text: string) {
  if (!field.isConnected || field.disabled || field.readOnly) return;
  if (field instanceof HTMLInputElement) text = text.replace(/[\r\n]/g, "");
  const next = replaceSelection(field.value, start, end, text, field.maxLength);
  field.focus();
  field.setSelectionRange(start, end);
  // Use native edit commands when available so browser undo still works.
  if (document.execCommand("insertText", false, next.value.slice(start, next.caret))) return;
  const proto =
    field instanceof HTMLTextAreaElement
      ? HTMLTextAreaElement.prototype
      : HTMLInputElement.prototype;
  Object.getOwnPropertyDescriptor(proto, "value")?.set?.call(field, next.value);
  field.setSelectionRange(next.caret, next.caret);
  field.dispatchEvent(
    new InputEvent("input", { bubbles: true, inputType: "insertText", data: text }),
  );
}
export function AppContextMenu({ children, page, hasTrash, busy, t, onAction, onError }: Props) {
  const [snapshot, setSnapshot] = useState<Snapshot>({ kind: "page", actions: [] });
  const snapshotRef = useRef(snapshot);
  const picked = useRef(false);
  function inspect(e: MouseEvent) {
    const target = e.target instanceof Element ? e.target : undefined;
    if (!target || target.closest("[data-ponkan-context-menu]")) {
      e.preventDefault();
      return;
    }
    const container =
      target.closest<HTMLElement>(
        '[data-slot="dialog-content"],[data-slot="alert-dialog-content"]',
      ) ?? undefined;
    const field = editableField(target);
    const focus =
      document.activeElement instanceof HTMLElement ? document.activeElement : undefined;
    let next: Snapshot;
    if (field) {
      const start = field.selectionStart ?? 0,
        end = field.selectionEnd ?? 0;
      next = {
        kind: "field",
        field,
        start,
        end,
        text: field.value.slice(start, end),
        actions: textActions({ length: field.value.length, start, end, editable: !field.readOnly }),
        container,
        focus: field,
      };
    } else {
      const selection = selectedText(target);
      const profile = target.closest<HTMLElement>("[data-context-profile]");
      const trash = target.closest<HTMLElement>("[data-context-trash]");
      const kind: Kind = selection
        ? "selection"
        : container || target.closest('[role="menu"],[role="listbox"]')
          ? "dialog"
          : profile
            ? "profile"
            : trash
              ? "trash-item"
              : "page";
      next = {
        kind,
        id: profile?.dataset.contextProfile ?? trash?.dataset.contextTrash,
        text: selection,
        actions: contextActions(page, kind, hasTrash),
        container,
        focus,
      };
    }
    if (!next.actions.length) {
      e.preventDefault();
      return;
    }
    picked.current = false;
    snapshotRef.current = next;
    setSnapshot(next);
  }
  function select(a: Action | "import-file" | "import-code") {
    const s = snapshotRef.current;
    picked.current = true;
    // Let Radix dismiss before triggering a dialog or restoring input focus.
    window.setTimeout(() => {
      if (a === "copy" || a === "cut" || a === "paste" || a === "select-all") {
        void (async () => {
          const field = s.field;
          if (a === "copy") {
            if (s.text) await writeClipboard(s.text);
            if (field?.isConnected) {
              field.focus();
              field.setSelectionRange(s.start ?? 0, s.end ?? 0);
            }
            return;
          }
          if (!field?.isConnected || field.disabled) return;
          if (a === "select-all") {
            field.focus();
            field.select();
            return;
          }
          if (field.readOnly) return;
          if (a === "cut") {
            if (!s.text) return;
            await writeClipboard(s.text);
            setText(field, s.start ?? 0, s.end ?? 0, "");
          }
          if (a === "paste") {
            const value = await readClipboard();
            if (value) setText(field, s.start ?? 0, s.end ?? 0, value);
            else field.focus();
          }
        })().catch(onError);
      } else onAction(a, s.id);
    }, 0);
  }
  function item(a: Action) {
    const Icon = icons[a];
    const destructive = ["delete", "purge", "empty"].includes(a);
    const disabled =
      busy &&
      ["new", "import", "share", "rename", "delete", "restore", "purge", "empty"].includes(a);
    return (
      <CM.Item
        key={a}
        className="ponkan-context-item"
        data-variant={destructive ? "destructive" : "default"}
        disabled={disabled}
        onSelect={() => select(a)}
      >
        <Icon size={14} />
        {t(labels[a])}
      </CM.Item>
    );
  }
  const isHome = page === "home" && snapshot.kind === "page";
  return (
    <CM.Root modal={false}>
      <CM.Trigger asChild onContextMenu={inspect}>
        {children}
      </CM.Trigger>
      <CM.Portal container={snapshot.container}>
        <CM.Content
          data-ponkan-context-menu=""
          className="ponkan-context-menu"
          collisionPadding={8}
          loop
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            if (!picked.current && snapshotRef.current.focus?.isConnected)
              snapshotRef.current.focus.focus();
          }}
        >
          {isHome && <CM.Label className="ponkan-context-label">{t("Ir para")}</CM.Label>}
          {snapshot.actions.map((a, i) =>
            a === "import" ? (
              <CM.Sub key={a}>
                <CM.SubTrigger className="ponkan-context-item" disabled={busy}>
                  <FileUp size={14} />
                  {t("Importar")}
                  <ChevronRight size={14} className="ponkan-context-chevron" />
                </CM.SubTrigger>
                <CM.Portal container={snapshot.container}>
                  <CM.SubContent
                    data-ponkan-context-menu=""
                    className="ponkan-context-menu"
                    sideOffset={5}
                    collisionPadding={8}
                  >
                    <CM.Item className="ponkan-context-item" onSelect={() => select("import-file")}>
                      <FileUp size={14} />
                      {t("Importar arquivo (.pkn)")}
                    </CM.Item>
                    <CM.Item className="ponkan-context-item" onSelect={() => select("import-code")}>
                      <Link2 size={14} />
                      {t("Importar código")}
                    </CM.Item>
                  </CM.SubContent>
                </CM.Portal>
              </CM.Sub>
            ) : (
              <FragmentItem key={a} separator={a === "about" && i > 0} item={item(a)} />
            ),
          )}
        </CM.Content>
      </CM.Portal>
    </CM.Root>
  );
}
function FragmentItem({ separator, item }: { separator: boolean; item: ReactNode }) {
  return (
    <>
      {separator && <CM.Separator className="ponkan-context-separator" />}
      {item}
    </>
  );
}
