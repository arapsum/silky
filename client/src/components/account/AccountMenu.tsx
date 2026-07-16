import { SignOutIcon } from "@phosphor-icons/react";
import { useState } from "react";
import { sessionApi } from "@/lib/api/browser";

export function AccountMenu() {
  const [loading, setLoading] = useState(false);
  return (
    <button
      disabled={loading}
      onClick={async () => {
        setLoading(true);
        await sessionApi.logout().catch(() => undefined);
        window.location.assign("/");
      }}
      type="button"
    >
      <SignOutIcon aria-hidden size={16} /> {loading ? "Signing out" : "Sign out"}
    </button>
  );
}
