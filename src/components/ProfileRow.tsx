import {
  Cat,
  Check,
  Ellipsis,
  Heart,
  ImageOff,
  ImagePlus,
  LoaderCircle,
  Pencil,
  Play,
  Save,
  SaveCheck,
  Share2,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Item, ItemActions, ItemContent, ItemMedia, ItemTitle } from "@/components/ui/item";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Profile } from "../types";

type Props = {
  profile: Profile;
  t: (key: string) => string;
  favorite: boolean;
  matching: boolean;
  avatar?: string;
  busy: boolean;
  operationKind: "load" | "save" | null;
  loaded: boolean;
  onFavorite: () => void;
  onApply: () => void;
  onSave: () => void;
  onRename: () => void;
  onAvatarChange: () => void;
  onAvatarClear: () => void;
  onShare: () => void;
  onDelete: () => void;
};

export function ProfileRow({
  profile,
  t,
  favorite,
  matching,
  avatar,
  busy,
  operationKind,
  loaded,
  onFavorite,
  onApply,
  onSave,
  onRename,
  onAvatarChange,
  onAvatarClear,
  onShare,
  onDelete,
}: Props) {
  return (
    <Item
      variant="outline"
      role="listitem"
      className="linear-row ponkan-profile-item"
      data-context-profile={profile.id}
      data-favorite={favorite}
    >
      <span className="profile-status-slot">
        {matching && (
          <span
            className="profile-status-dot"
            role="img"
            aria-label={t("Perfil atual confirmado")}
            title={t("Configurações atuais correspondem ao perfil")}
          />
        )}
      </span>
      <Button
        className="favorite-action"
        variant="ghost"
        size="icon"
        aria-label={`${favorite ? t("Desfavoritar") : t("Favoritar")} ${profile.name}`}
        aria-pressed={favorite}
        title={favorite ? t("Remover dos favoritos") : t("Favoritar")}
        disabled={busy}
        onClick={() => onFavorite()}
      >
        <Heart size={20} className={favorite ? "favorite-filled" : ""} />
      </Button>
      <ItemMedia>
        <Avatar size="lg" className="ponkan-profile-avatar">
          {avatar && <AvatarImage src={avatar} alt="" draggable={false} />}
          <AvatarFallback>
            <Cat size={23} className="profile-avatar" aria-hidden="true" />
          </AvatarFallback>
        </Avatar>
      </ItemMedia>
      <ItemContent className="min-w-0">
        <ItemTitle className="ponkan-profile-name" title={profile.name}>
          <span>{profile.name}</span>
        </ItemTitle>
      </ItemContent>
      <ItemActions className="ponkan-profile-actions">
        <Button className="load-action" disabled={busy} onClick={onApply}>
          {operationKind === "load" ? (
            <LoaderCircle size={16} className="ponkan-spin" />
          ) : loaded ? (
            <Check size={16} className="ponkan-feedback" />
          ) : (
            <Play size={16} />
          )}{" "}
          {t("Aplicar")}
        </Button>
        <Button
          variant="outline"
          size="icon"
          className="profile-save-action"
          title={t("Salvar alterações")}
          disabled={busy}
          aria-label={`${t("Salvar alterações em")} ${profile.name}`}
          onClick={onSave}
        >
          {operationKind === "save" ? (
            <LoaderCircle size={16} className="ponkan-spin" />
          ) : matching ? (
            <SaveCheck
              size={16}
              className="ponkan-feedback"
              aria-label={t("Configurações correspondentes")}
            />
          ) : (
            <Save size={16} />
          )}
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              aria-label={`${t("Opções de")} ${profile.name}`}
              disabled={busy}
            >
              <Ellipsis size={22} />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onSelect={onRename}>
              <Pencil size={15} /> {t("Renomear")}
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={onAvatarChange}>
              <ImagePlus size={16} /> {t("Alterar imagem")}
            </DropdownMenuItem>
            {avatar && (
              <DropdownMenuItem onSelect={onAvatarClear}>
                <ImageOff size={16} /> {t("Remover imagem")}
              </DropdownMenuItem>
            )}
            <DropdownMenuItem onSelect={onShare}>
              <Share2 size={16} /> {t("Compartilhar")}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem variant="destructive" onSelect={onDelete}>
              <Trash2 size={15} /> {t("Excluir")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </ItemActions>
    </Item>
  );
}
