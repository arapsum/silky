import { CheckCircleIcon, SpinnerGapIcon, WarningCircleIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { ApiError, sessionApi } from "@/lib/api/browser";
import { toast } from "@/lib/toast";

export function VerifyEmail({ token }: { token: string }) {
  const [state, setState] = useState<"loading" | "success" | "error">("loading");
  const [message, setMessage] = useState("Confirming your email address...");
  useEffect(() => {
    if (!token) {
      setState("error");
      setMessage("This verification link is incomplete.");
      toast.error("Email not verified", "This verification link is incomplete.");
      return;
    }
    sessionApi.verify(token)
      .then(() => {
        setState("success");
        setMessage("Your email is verified. You can now sign in.");
        toast.success("Email verified", "You can now sign in to Silk.");
      })
      .catch((error) => {
        const description = error instanceof ApiError ? error.message : "This verification link could not be confirmed.";
        setState("error");
        setMessage(description);
        toast.error("Email not verified", description);
      });
  }, [token]);
  return (
    <section className="auth-card verify-card">
      {state === "loading" ? <SpinnerGapIcon aria-hidden className="is-spinning" size={44} /> : state === "success" ? <CheckCircleIcon aria-hidden size={44} weight="thin" /> : <WarningCircleIcon aria-hidden size={44} weight="thin" />}
      <p className="eyebrow">Email verification</p>
      <h1>{state === "success" ? "You are all set." : state === "error" ? "That link did not work." : "One last check."}</h1>
      <p>{message}</p>
      {state !== "loading" && <a className="button-primary" href="/auth/login">Continue to sign in</a>}
    </section>
  );
}
