import { PlusIcon, TrashIcon } from "@phosphor-icons/react";

import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";

export type ProductInformationEntry = {
  id: string;
  key: string;
  value: string;
};

function entryId() {
  return crypto.randomUUID();
}

export function informationEntries(information: Record<string, string>): ProductInformationEntry[] {
  return Object.entries(information).map(([key, value]) => ({
    id: entryId(),
    key,
    value,
  }));
}

export function productInformation(entries: ProductInformationEntry[]) {
  const information: Record<string, string> = {};

  for (const entry of entries) {
    const key = entry.key.trim();
    const value = entry.value.trim();

    if (!key && !value) continue;
    if (!key || !value) throw new Error("Product information needs both a key and a value");
    if (information[key]) throw new Error(`Product information key “${key}” is duplicated`);

    information[key] = value;
  }

  return information;
}

export function ProductInformationFields({
  entries,
  onChange,
}: {
  entries: ProductInformationEntry[];
  onChange: (entries: ProductInformationEntry[]) => void;
}) {
  function addEntry() {
    onChange([...entries, { id: entryId(), key: "", value: "" }]);
  }

  function updateEntry(id: string, patch: Partial<ProductInformationEntry>) {
    onChange(entries.map((entry) => (entry.id === id ? { ...entry, ...patch } : entry)));
  }

  function removeEntry(id: string) {
    onChange(entries.filter((entry) => entry.id !== id));
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <p className="text-sm text-muted-foreground">
          Add flexible product details such as material, fit, care instructions, or origin.
        </p>
        <Button type="button" variant="outline" size="sm" onClick={addEntry}>
          <PlusIcon className="size-4" />
          Add information
        </Button>
      </div>

      {entries.length ? (
        <div className="space-y-2">
          {entries.map((entry) => (
            <div
              key={entry.id}
              className="grid gap-2 sm:grid-cols-[minmax(0,.8fr)_minmax(0,1.2fr)_2.5rem]"
            >
              <Input
                aria-label="Information key"
                value={entry.key}
                placeholder="Key, e.g. Material"
                onChange={(event) => updateEntry(entry.id, { key: event.target.value })}
              />
              <Input
                aria-label="Information value"
                value={entry.value}
                placeholder="Value, e.g. 100% Cotton"
                onChange={(event) => updateEntry(entry.id, { value: event.target.value })}
              />
              <Button
                type="button"
                variant="outline"
                size="icon"
                aria-label={`Remove ${entry.key || "information"}`}
                onClick={() => removeEntry(entry.id)}
              >
                <TrashIcon className="size-4" />
              </Button>
            </div>
          ))}
        </div>
      ) : (
        <div className="border bg-muted/20 px-4 py-5 text-sm text-muted-foreground">
          No additional product information.
        </div>
      )}
    </div>
  );
}
