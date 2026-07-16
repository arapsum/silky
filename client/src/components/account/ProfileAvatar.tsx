import { CameraIcon, SpinnerGapIcon, UserIcon } from "@phosphor-icons/react";
import { useId, useRef } from "react";

const ACCEPTED_TYPES = new Set(["image/avif", "image/jpeg", "image/png", "image/webp"]);
const MAX_AVATAR_BYTES = 5 * 1024 * 1024;

type ProfileAvatarProps = {
  image: string | null;
  isUploading: boolean;
  name: string;
  onSelect: (file: File) => void;
};

/** A compact, accessible avatar picker for a signed-in customer's profile. */
export function ProfileAvatar({ image, isUploading, name, onSelect }: ProfileAvatarProps) {
  const inputId = useId();
  const input = useRef<HTMLInputElement>(null);
  const initials = name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0])
    .join("")
    .toUpperCase();

  return (
    <div className="profile-avatar-picker">
      <input
        accept="image/avif,image/jpeg,image/png,image/webp"
        aria-describedby={`${inputId}-help`}
        className="profile-avatar-picker__input"
        disabled={isUploading}
        id={inputId}
        onChange={(event) => {
          const [file] = event.target.files ?? [];
          if (file) onSelect(file);
          event.target.value = "";
        }}
        ref={input}
        type="file"
      />
      <button
        aria-label="Choose a new profile picture"
        className="profile-avatar-picker__image"
        disabled={isUploading}
        onClick={() => input.current?.click()}
        type="button"
      >
        {image ? <img alt="" height="176" src={image} width="176" /> : <span aria-hidden>{initials || <UserIcon size={44} weight="light" />}</span>}
        <span className="profile-avatar-picker__action" aria-hidden>
          {isUploading ? <SpinnerGapIcon className="profile-avatar-picker__spinner" size={18} /> : <CameraIcon size={18} />}
        </span>
      </button>
      <div>
        <strong>Profile picture</strong>
        <p id={`${inputId}-help`}>Choose a JPG, PNG, WebP, or AVIF image up to 5 MB.</p>
      </div>
    </div>
  );
}

export function validateAvatarFile(file: File): string | null {
  if (!ACCEPTED_TYPES.has(file.type)) return "Choose a JPG, PNG, WebP, or AVIF image.";
  if (file.size > MAX_AVATAR_BYTES) return "Choose an image smaller than 5 MB.";
  return null;
}
