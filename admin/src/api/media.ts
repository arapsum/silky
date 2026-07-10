import { apiRequest } from "#/api/client.ts";

export type MediaKind = "user" | "category" | "product";

export type MediaAsset = {
  pid: string;
  publicId: string;
  secureUrl: string;
  folder: string;
  status: string;
  format: string | null;
  bytes: number | null;
  width: number | null;
  height: number | null;
};

type UploadSignature = {
  assetPid: string;
  cloudName: string;
  apiKey: string;
  timestamp: number;
  folder: string;
  publicId: string;
  signature: string;
};

type CloudinaryUploadResponse = {
  asset_id?: string;
  bytes?: number;
  format?: string;
  height?: number;
  public_id?: string;
  secure_url?: string;
  width?: number;
};

export type UploadedMedia = {
  assetPid: string;
  imageLink: string;
};

export function listMediaAssets() {
  return apiRequest<MediaAsset[]>("/media", {
    fallback: "Unable to load media assets",
  });
}

export function signMediaUpload(kind: MediaKind, checksum?: string) {
  return apiRequest<UploadSignature>("/media/sign", {
    method: "POST",
    body: JSON.stringify({ kind, checksum }),
    fallback: "Unable to prepare media upload",
  });
}

export async function uploadMedia(file: File, kind: MediaKind): Promise<UploadedMedia> {
  const signature = await signMediaUpload(kind);
  const body = new FormData();
  body.set("file", file);
  body.set("api_key", signature.apiKey);
  body.set("timestamp", String(signature.timestamp));
  body.set("folder", signature.folder);
  body.set("public_id", signature.publicId);
  body.set("signature", signature.signature);

  const response = await fetch(
    `https://api.cloudinary.com/v1_1/${signature.cloudName}/image/upload`,
    { method: "POST", body },
  );

  if (!response.ok) {
    throw new Error(`Unable to upload ${kind} image`);
  }

  const result = (await response.json()) as CloudinaryUploadResponse;
  const expectedPublicIds = new Set([
    signature.publicId,
    `${signature.folder}/${signature.publicId}`,
  ]);
  if (!result.secure_url || !result.public_id || !expectedPublicIds.has(result.public_id)) {
    throw new Error("Cloudinary returned an invalid media asset");
  }

  const asset = await apiRequest<MediaAsset>(`/media/${signature.assetPid}/finalize`, {
    method: "PUT",
    body: JSON.stringify({
      publicId: result.public_id,
      assetId: result.asset_id,
      secureUrl: result.secure_url,
      format: result.format,
      bytes: result.bytes,
      width: result.width,
      height: result.height,
    }),
    fallback: "Unable to register uploaded media",
  });

  return { assetPid: asset.pid, imageLink: asset.secureUrl };
}

export function uploadAvatarImage(file: File) {
  return uploadMedia(file, "user");
}

export function uploadCategoryImage(file: File) {
  return uploadMedia(file, "category");
}

export function uploadProductImage(file: File) {
  return uploadMedia(file, "product");
}
