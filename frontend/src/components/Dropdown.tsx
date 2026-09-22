import { Plus } from "lucide-react";
import { Button } from "./ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";

type EnabledDropdownItem = {
  id: string;
  label: string;
  disabled?: false;
  onSelect: () => void;
};

type DisabledDropdownItem = {
  id: string;
  label: string;
  disabled: true;
};

export type DropdownItem = EnabledDropdownItem | DisabledDropdownItem;

export type DropdownProps = {
  items: readonly DropdownItem[];
  triggerLabel: string;
  menuLabel?: string;
};

export function Dropdown({ items, triggerLabel, menuLabel = "Options" }: DropdownProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        aria-label={triggerLabel}
        render={<Button variant="outline" size="icon" />}
      >
        <Plus aria-hidden="true" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuGroup>
          <DropdownMenuLabel>{menuLabel}</DropdownMenuLabel>
          {items.map((item) => (
            <DropdownMenuItem
              key={item.id}
              disabled={item.disabled}
              onClick={item.disabled ? undefined : item.onSelect}
            >
              {item.label}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
