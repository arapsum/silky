import { env } from "#/env.ts";

type CloudinaryUploadResponse = {
  secure_url?: string;
  url?: string;
};

async function uploadImage(file: File, folder: string, label: string) {
  const cloudName = env.VITE_CLOUDINARY_CLOUD_NAME;
  const uploadPreset = env.VITE_CLOUDINARY_UPLOAD_PRESET;

  if (!cloudName || !uploadPreset) {
    throw new Error(`${label} upload is not configured`);
  }

  const body = new FormData();
  body.set("file", file);
  body.set("upload_preset", uploadPreset);
  body.set("folder", folder);

  const response = await fetch(`https://api.cloudinary.com/v1_1/${cloudName}/image/upload`, {
    method: "POST",
    body,
  });

  if (!response.ok) {
    throw new Error(`Unable to upload ${label.toLowerCase()}`);
  }

  const result = (await response.json()) as CloudinaryUploadResponse;
  const imageUrl = result.secure_url ?? result.url;

  if (!imageUrl) {
    throw new Error(`${label} upload did not return an image URL`);
  }

  return imageUrl;
}

export function uploadAvatarImage(file: File) {
  return uploadImage(file, "silk/users", "Avatar");
}

export function uploadCategoryImage(file: File) {
  return uploadImage(file, "silk/categories", "Category image");
}
