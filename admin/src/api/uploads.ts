import { env } from "#/env.ts";

type CloudinaryUploadResponse = {
  secure_url?: string;
  url?: string;
};

export async function uploadAvatarImage(file: File) {
  const cloudName = env.VITE_CLOUDINARY_CLOUD_NAME;
  const uploadPreset = env.VITE_CLOUDINARY_UPLOAD_PRESET;

  if (!cloudName || !uploadPreset) {
    throw new Error("Avatar upload is not configured");
  }

  const body = new FormData();
  body.set("file", file);
  body.set("upload_preset", uploadPreset);
  body.set("folder", "silk/users");

  const response = await fetch(`https://api.cloudinary.com/v1_1/${cloudName}/image/upload`, {
    method: "POST",
    body,
  });

  if (!response.ok) {
    throw new Error("Unable to upload avatar");
  }

  const result = (await response.json()) as CloudinaryUploadResponse;
  const imageUrl = result.secure_url ?? result.url;

  if (!imageUrl) {
    throw new Error("Avatar upload did not return an image URL");
  }

  return imageUrl;
}
