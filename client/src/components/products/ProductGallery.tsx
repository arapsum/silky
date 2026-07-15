import { useState } from "react";
import styles from "./ProductGallery.module.css";

export interface GalleryImage {
  id: string;
  src: string;
  alt: string;
}

interface ProductGalleryProps {
  images: GalleryImage[];
}

export function ProductGallery({ images }: ProductGalleryProps) {
  const [activeId, setActiveId] = useState(images[0]?.id);
  const activeIndex = Math.max(0, images.findIndex((image) => image.id === activeId));
  const activeImage = images[activeIndex];
  const hasThumbnails = images.length > 1;

  if (!activeImage) return null;

  return (
    <div className={`${styles.gallery} ${hasThumbnails ? "" : styles.gallerySingle}`}>
      {hasThumbnails && (
        <div aria-label="Product image thumbnails" className={styles.thumbnails} role="group">
          {images.map((image, index) => {
            const isActive = image.id === activeImage.id;

            return (
              <button
                aria-label={`View product image ${index + 1}`}
                aria-pressed={isActive}
                className={`${styles.thumbnail} ${isActive ? styles.thumbnailActive : ""}`}
                key={image.id}
                onClick={() => setActiveId(image.id)}
                type="button"
              >
                <img alt="" decoding="async" height="220" loading="lazy" src={image.src} width="176" />
              </button>
            );
          })}
        </div>
      )}

      <div aria-live="polite" className={styles.main}>
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
      </div>
    </div>
  );
}
