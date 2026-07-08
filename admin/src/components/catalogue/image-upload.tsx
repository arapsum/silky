import { ImageSquareIcon, UploadSimpleIcon, XIcon } from "@phosphor-icons/react";
import { useRef, useState, type DragEvent, type ReactNode } from "react";
import { toast } from "sonner";

import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";

const IMAGE_MAX_BYTES = 1024 * 1024 * 5;
const IMAGE_TYPES = new Set(["image/jpeg", "image/png", "image/webp"]);

export type ImageDraft = {
  id: string;
  file: File;
  preview: string;
};

type ImageDropzoneProps = {
  label: string;
  multiple?: boolean;
  onFiles: (files: File[]) => void;
  className?: string;
};

type ImagePreviewSlotProps = {
  preview?: string;
  icon?: ReactNode;
  className?: string;
};

type ImageGridProps = {
  images: ImageDraft[];
  onRemove: (image: ImageDraft) => void;
};

export function createImageDraft(file: File): ImageDraft {
  return {
    id: crypto.randomUUID(),
    file,
    preview: URL.createObjectURL(file),
  };
}

export function imageFileError(file: File, label = "Image") {
  if (!IMAGE_TYPES.has(file.type)) {
    return "Upload a PNG, JPG or WebP image";
  }

  if (file.size > IMAGE_MAX_BYTES) {
    return `${label} must be 5MB or smaller`;
  }

  return null;
}

export function validImageFiles(files: File[], label?: string) {
  return files.filter((file) => {
    const error = imageFileError(file, label);

    if (error) {
      toast.error(error);
      return false;
    }

    return true;
  });
}

export function ImageDropzone({ label, multiple, onFiles, className }: ImageDropzoneProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [isDragging, setIsDragging] = useState(false);

  function handleFiles(files: FileList | File[]) {
    onFiles(validImageFiles(Array.from(files)));
  }

  function onDrop(event: DragEvent<HTMLButtonElement>) {
    event.preventDefault();
    setIsDragging(false);
    handleFiles(event.dataTransfer.files);
  }

  return (
    <>
      <input
        ref={inputRef}
        type="file"
        accept="image/png,image/jpeg,image/webp"
        multiple={multiple}
        className="sr-only"
        onChange={(event) => {
          if (event.target.files) handleFiles(event.target.files);
          event.target.value = "";
        }}
      />
      <button
        type="button"
        onClick={() => inputRef.current?.click()}
        onDragOver={(event) => {
          event.preventDefault();
          setIsDragging(true);
        }}
        onDragLeave={() => setIsDragging(false)}
        onDrop={onDrop}
        className={cn(
          "flex min-h-28 items-center justify-center gap-3 rounded-xl border border-dashed bg-background px-4 text-sm font-medium outline-none transition-colors hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/30",
          isDragging && "border-primary bg-primary/5",
          className,
        )}
      >
        <span className="flex size-10 items-center justify-center rounded-full bg-muted">
          <UploadSimpleIcon className="size-5 text-muted-foreground" aria-hidden />
        </span>
        <span className="truncate">{label}</span>
      </button>
    </>
  );
}

export function ImagePreviewSlot({ preview, icon, className }: ImagePreviewSlotProps) {
  return (
    <div
      className={cn(
        "flex min-h-28 items-center justify-center overflow-hidden rounded-xl border bg-muted/40",
        className,
      )}
    >
      {preview ? (
        <img src={preview} alt="" className="h-full w-full object-cover" />
      ) : (
        (icon ?? <ImageSquareIcon className="size-9 text-muted-foreground" aria-hidden />)
      )}
    </div>
  );
}

export function ImageGrid({ images, onRemove }: ImageGridProps) {
  if (images.length === 0) return null;

  return (
    <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
      {images.map((image) => (
        <div key={image.id} className="overflow-hidden border bg-background">
          <div className="aspect-square bg-muted">
            <img src={image.preview} alt="" className="h-full w-full object-cover" />
          </div>
          <div className="flex items-center justify-between gap-2 p-2">
            <span className="truncate text-xs text-muted-foreground">{image.file.name}</span>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Remove image"
              onClick={() => onRemove(image)}
            >
              <XIcon className="size-4" />
            </Button>
          </div>
        </div>
      ))}
    </div>
  );
}
