import {
  ArrowLeftIcon,
  ArrowRightIcon,
  ArrowsOutSimpleIcon,
  XIcon,
} from "@phosphor-icons/react";
import { useEffect, useRef, useState } from "react";
import styles from "./ProductGallery.module.css";

export interface GalleryImage {
  id: string;
  src: string;
  alt: string;
}

interface ProductGalleryProps {
  images: GalleryImage[];
}

const MAX_VISIBLE_THUMBNAILS = 4;

export function ProductGallery({ images }: ProductGalleryProps) {
  const [activeId, setActiveId] = useState(images[0]?.id);
  const [dialogIndex, setDialogIndex] = useState<number | null>(null);
  const dialogRef = useRef<HTMLDialogElement>(null);
  const lastTriggerRef = useRef<HTMLButtonElement | null>(null);
  const activeIndex = Math.max(0, images.findIndex((image) => image.id === activeId));
  const activeImage = images[activeIndex];
  const hasThumbnails = images.length > 1;
  const hasOverflow = images.length > MAX_VISIBLE_THUMBNAILS;
  const visibleImages = hasOverflow ? images.slice(0, MAX_VISIBLE_THUMBNAILS - 1) : images;
  const remainingImageCount = images.length - visibleImages.length;
  const dialogImage = dialogIndex === null ? null : images[dialogIndex];

  useEffect(() => {
    const dialog = dialogRef.current;

    if (dialogIndex === null || !dialog || dialog.open) return;

    if (typeof dialog.showModal === "function") {
      dialog.showModal();
    } else {
      dialog.setAttribute("open", "");
    }
  }, [dialogIndex]);

  if (!activeImage) return null;

  const selectImage = (index: number) => {
    setActiveId(images[index]?.id);
  };

  const openGallery = (index: number, trigger: HTMLButtonElement) => {
    lastTriggerRef.current = trigger;
    selectImage(index);
    setDialogIndex(index);
  };

  const closeGallery = () => {
    const dialog = dialogRef.current;

    if (dialog && typeof dialog.close === "function" && dialog.open) {
      dialog.close();
      return;
    }

    setDialogIndex(null);
    lastTriggerRef.current?.focus();
  };

  const handleDialogClose = () => {
    setDialogIndex(null);
    lastTriggerRef.current?.focus();
  };

  const moveDialog = (direction: -1 | 1) => {
    if (dialogIndex === null) return;

    const nextIndex = (dialogIndex + direction + images.length) % images.length;
    selectImage(nextIndex);
    setDialogIndex(nextIndex);
  };

  return (
    <div className={`${styles.gallery} ${hasThumbnails ? "" : styles.gallerySingle}`}>
      {hasThumbnails && (
        <div aria-label="Product image thumbnails" className={styles.thumbnails} role="group">
          {visibleImages.map((image, index) => {
            const isActive = image.id === activeImage.id;

            return (
              <button
                aria-label={`View product image ${index + 1}`}
                aria-pressed={isActive}
                className={`${styles.thumbnail} ${isActive ? styles.thumbnailActive : ""}`}
                key={image.id}
                onClick={() => selectImage(index)}
                type="button"
              >
                <img alt="" decoding="async" height="220" loading="lazy" src={image.src} width="176" />
              </button>
            );
          })}

          {hasOverflow && (
            <button
              aria-label={`View all ${images.length} product images`}
              aria-pressed={activeIndex >= visibleImages.length}
              className={`${styles.thumbnail} ${styles.thumbnailMore} ${activeIndex >= visibleImages.length ? styles.thumbnailActive : ""}`}
              onClick={(event) => openGallery(visibleImages.length, event.currentTarget)}
              type="button"
            >
              <img
                alt=""
                decoding="async"
                height="220"
                loading="lazy"
                src={images[visibleImages.length].src}
                width="176"
              />
              <span className={styles.thumbnailMoreContent}>
                <strong>+{remainingImageCount}</strong>
                <span>View all</span>
              </span>
            </button>
          )}
        </div>
      )}

      <button
        aria-label={`Open image ${activeIndex + 1} of ${images.length} in gallery`}
        className={styles.main}
        onClick={(event) => openGallery(activeIndex, event.currentTarget)}
        type="button"
      >
        <img
          alt={activeImage.alt}
          className={styles.mainImage}
          decoding="async"
          fetchPriority="high"
          height="1000"
          key={activeImage.id}
          loading="eager"
          src={activeImage.src}
          width="800"
        />
        <span aria-hidden="true" className={styles.expandHint}>
          <ArrowsOutSimpleIcon size={18} />
        </span>
      </button>

      {dialogImage && dialogIndex !== null && (
        <dialog
          aria-label="Product image gallery"
          className={styles.dialog}
          onClose={handleDialogClose}
          onKeyDown={(event) => {
            if (event.key === "ArrowLeft") moveDialog(-1);
            if (event.key === "ArrowRight") moveDialog(1);
          }}
          ref={dialogRef}
        >
          <div className={styles.dialogHeader}>
            <p>
              Image {dialogIndex + 1} of {images.length}
            </p>
            <button aria-label="Close image gallery" onClick={closeGallery} type="button">
              <XIcon size={20} />
            </button>
          </div>

          <div className={styles.dialogStage}>
            <button aria-label="Previous product image" onClick={() => moveDialog(-1)} type="button">
              <ArrowLeftIcon size={22} />
            </button>
            <img
              alt={dialogImage.alt}
              decoding="async"
              height="1000"
              key={dialogImage.id}
              src={dialogImage.src}
              width="800"
            />
            <button aria-label="Next product image" onClick={() => moveDialog(1)} type="button">
              <ArrowRightIcon size={22} />
            </button>
          </div>

          <div aria-label="All product images" className={styles.dialogThumbnails} role="group">
            {images.map((image, index) => (
              <button
                aria-label={`Show product image ${index + 1}`}
                aria-pressed={index === dialogIndex}
                className={`${styles.dialogThumbnail} ${index === dialogIndex ? styles.dialogThumbnailActive : ""}`}
                key={image.id}
                onClick={() => {
                  selectImage(index);
                  setDialogIndex(index);
                }}
                type="button"
              >
                <img alt="" decoding="async" height="160" loading="lazy" src={image.src} width="128" />
              </button>
            ))}
          </div>
        </dialog>
      )}
    </div>
  );
}
