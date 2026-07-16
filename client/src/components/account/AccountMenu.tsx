import { SignOutIcon } from "@phosphor-icons/react";
import { useState } from "react";
import { sessionApi } from "@/lib/api/browser";
import { toast } from "@/lib/toast";

export function AccountMenu() {
  const [loading, setLoading] = useState(false);
  return (
    <button
      disabled={loading}
      onClick={async () => {
        setLoading(true);
        try {
          await sessionApi.logout();
          toast.flash.success("Signed out", "See you again soon.");
          window.location.assign("/");
        } catch {
          toast.error("Could not sign out", "Please try again.");
          setLoading(false);
        }
      }}
      type="button"
    >
      <SignOutIcon aria-hidden size={16} /> {loading ? "Signing out" : "Sign out"}
    </button>
  );
}
