// Adapted from Shadcn Studio @ss-components/switch-14 (radix-nova).
import type { ComponentProps } from "react";
import { Switch } from "@/components/ui/switch";

type Props = ComponentProps<typeof Switch> & { yes: string; no: string };
export function SettingsSwitch({ yes, no, ...props }: Props) {
  return (
    <div className="ponkan-switch14">
      <Switch {...props} />
      <span className="ponkan-switch-indicator ponkan-switch-no" aria-hidden="true">
        {no}
      </span>
      <span className="ponkan-switch-indicator ponkan-switch-yes" aria-hidden="true">
        {yes}
      </span>
    </div>
  );
}
