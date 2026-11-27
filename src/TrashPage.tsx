import { Trash2, RotateCcw, Cat } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import {
  Item,
  ItemGroup,
  ItemMedia,
  ItemContent,
  ItemTitle,
  ItemActions,
} from "@/components/ui/item";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogCancel,
  AlertDialogAction,
} from "@/components/ui/alert-dialog";
import type { Language, Translate } from "./i18n";
import type { TrashEntry, TrashConfirmation } from "./types";
type Props = {
  entries: TrashEntry[];
  busy: boolean;
  loaded: boolean;
  language: Language;
  t: Translate;
  restore: (ids: string[]) => Promise<boolean>;
  purge: (ids: string[]) => Promise<boolean>;
  confirmation: TrashConfirmation | null;
  setConfirmation: (value: TrashConfirmation | null) => void;
};
export function TrashPage({
  entries,
  busy,
  loaded,
  language,
  t,
  restore,
  purge,
  confirmation,
  setConfirmation,
}: Props) {
  const date = (s: string) =>
    new Intl.DateTimeFormat(language, { dateStyle: "short", timeStyle: "short" }).format(
      new Date(s),
    );
  return (
    <>
      {entries.length > 0 ? (
        <>
          <div className="linear-toolbar ponkan-trash-toolbar">
            <Button
              variant="outline"
              disabled={busy}
              onClick={() => void restore(entries.map((e) => e.profile.id))}
            >
              <RotateCcw size={16} />
              {t("Restaurar tudo")}
            </Button>
            <Button
              variant="outline"
              className="ponkan-trash-danger"
              disabled={busy}
              onClick={() => setConfirmation({ ids: entries.map((e) => e.profile.id) })}
            >
              <Trash2 size={16} />
              {t("Esvaziar lixeira")}
            </Button>
          </div>
          <ItemGroup className="linear-list" aria-label={t("Perfis excluídos")}>
            {entries.map((e) => (
              <Item
                key={e.profile.id}
                data-context-trash={e.profile.id}
                variant="outline"
                role="listitem"
                className="linear-row ponkan-profile-item ponkan-trash-item"
              >
                <ItemMedia>
                  <Avatar size="lg" className="ponkan-profile-avatar">
                    {e.avatar && <AvatarImage src={e.avatar} alt="" draggable={false} />}
                    <AvatarFallback>
                      <Cat size={23} />
                    </AvatarFallback>
                  </Avatar>
                </ItemMedia>
                <ItemContent className="min-w-0">
                  <ItemTitle className="ponkan-profile-name" title={e.profile.name}>
                    <span>{e.profile.name}</span>
                  </ItemTitle>
                  <span className="ponkan-trash-date">
                    {t("Excluído em")} {date(e.deleted_at)}
                  </span>
                </ItemContent>
                <ItemActions className="ponkan-profile-actions">
                  <Button
                    variant="outline"
                    disabled={busy}
                    onClick={() => void restore([e.profile.id])}
                  >
                    <RotateCcw size={16} />
                    {t("Restaurar")}
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="ponkan-trash-danger"
                    disabled={busy}
                    aria-label={t("Excluir permanentemente") + " " + e.profile.name}
                    title={t("Excluir permanentemente")}
                    onClick={() => setConfirmation({ ids: [e.profile.id], name: e.profile.name })}
                  >
                    <Trash2 size={18} />
                  </Button>
                </ItemActions>
              </Item>
            ))}
          </ItemGroup>
        </>
      ) : loaded ? (
        <Empty className="ponkan-empty-state ponkan-trash-empty">
          <EmptyHeader>
            <EmptyMedia>
              <Trash2 size={112} strokeWidth={1.25} />
            </EmptyMedia>
            <EmptyTitle>{t("A lixeira está vazia")}</EmptyTitle>
          </EmptyHeader>
        </Empty>
      ) : null}
      <AlertDialog
        open={!!confirmation}
        onOpenChange={(open) => {
          if (!open && !busy) setConfirmation(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="ponkan-dialog-title ponkan-dialog-destructive">
              <Trash2 size={19} />
              {confirmation?.name
                ? t("Excluir permanentemente") + " " + confirmation.name + "?"
                : t("Esvaziar lixeira") + "?"}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t("Esta ação é permanente e não pode ser desfeita.")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={busy}>{t("Cancelar")}</AlertDialogCancel>
            <AlertDialogAction
              disabled={busy}
              onClick={(event) => {
                event.preventDefault();
                if (confirmation)
                  void purge(confirmation.ids).then((ok) => {
                    if (ok) setConfirmation(null);
                  });
              }}
            >
              {t("Excluir permanentemente")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
