import { apiRequest } from "@/lib/api/browser";

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

type MediaAsset = { pid: string; secureUrl: string };

export type UploadedAvatar = {
  assetPid: string;
  imageUrl: string;
};

/** Uploads a customer-selected avatar through the authenticated profile-media flow. */
export async function uploadAvatarImage(file: File): Promise<UploadedAvatar> {
  const signature = await apiRequest<UploadSignature>("/media/profile/sign", {
    method: "POST",
    body: JSON.stringify({ kind: "user" }),
  });
  const body = new FormData();
  body.set("file", file);
  body.set("api_key", signature.apiKey);
  body.set("timestamp", String(signature.timestamp));
  body.set("folder", signature.folder);
  body.set("public_id", signature.publicId);
  body.set("signature", signature.signature);

  const response = await fetch(`https://api.cloudinary.com/v1_1/${signature.cloudName}/image/upload`, {
    method: "POST",
    body,
  });
  if (!response.ok) throw new Error("Your image could not be uploaded.");

  const uploaded = await response.json() as CloudinaryUploadResponse;
  const expectedPublicIds = new Set([signature.publicId, `${signature.folder}/${signature.publicId}`]);
  if (!uploaded.secure_url || !uploaded.public_id || !expectedPublicIds.has(uploaded.public_id)) {
    throw new Error("Cloudinary returned an invalid profile image.");
  }

  const asset = await apiRequest<MediaAsset>(`/media/profile/${signature.assetPid}/finalize`, {
    method: "PUT",
    body: JSON.stringify({
      publicId: uploaded.public_id,
      assetId: uploaded.asset_id,
      secureUrl: uploaded.secure_url,
      format: uploaded.format,
      bytes: uploaded.bytes,
      width: uploaded.width,
      height: uploaded.height,
    }),
  });

  return { assetPid: asset.pid, imageUrl: asset.secureUrl };
}
