import { DotsThreeIcon, EyeIcon, PencilSimpleIcon, TrashIcon } from "@phosphor-icons/react";
import type { ReactElement } from "react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "#/components/ui/dropdown-menu";

type CatalogueRowActionsProps = {
  itemName: string;
  itemType: string;
  view: ReactElement;
  edit: ReactElement;
  isDeleting: boolean;
  deleteDisabled?: boolean;
  deleteDisabledReason?: string;
  onDelete: () => void;
};

export function CatalogueRowActions({
  itemName,
  itemType,
  view,
  edit,
  isDeleting,
  deleteDisabled = false,
  deleteDisabledReason,
  onDelete,
}: CatalogueRowActionsProps) {
  const itemTypeLabel = itemType.toLowerCase();
  const deleteItem = (
    <DropdownMenuItem
      variant="destructive"
      className="rounded-md"
      disabled={deleteDisabled}
      title={deleteDisabledReason}
    >
      <TrashIcon className="size-4" aria-hidden />
      Delete
    </DropdownMenuItem>
  );

  return (
    <AlertDialog>
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              className="rounded-lg"
              aria-label={`Actions for ${itemName}`}
            />
          }
        >
          <DotsThreeIcon className="size-4" aria-hidden />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-40 rounded-lg">
          <DropdownMenuItem className="rounded-md" render={view}>
            <EyeIcon className="size-4" aria-hidden />
            View
          </DropdownMenuItem>
          <DropdownMenuItem className="rounded-md" render={edit}>
            <PencilSimpleIcon className="size-4" aria-hidden />
            Edit
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          {deleteDisabled ? (
            deleteItem
          ) : (
            <AlertDialogTrigger nativeButton={false} render={deleteItem} />
          )}
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialogContent className="rounded-lg">
        <AlertDialogHeader>
          <AlertDialogTitle>Delete {itemTypeLabel}?</AlertDialogTitle>
          <AlertDialogDescription>
            {itemName} will be removed from the active catalogue.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
          <AlertDialogAction variant="destructive" disabled={isDeleting} onClick={onDelete}>
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
