import { zodResolver } from "@hookform/resolvers/zod";
import { ArrowRightIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { FormField } from "@/components/forms/FormField";
import { ProfileAvatar, validateAvatarFile } from "@/components/account/ProfileAvatar";
import { AccountOverviewSkeleton } from "@/components/loading/StorefrontSkeletons";
import { ApiError, orderApi, sessionApi } from "@/lib/api/browser";
import type { OrderSummary, UserSession } from "@/lib/api/types";
import { formatCurrency } from "@/lib/format";
import { uploadAvatarImage } from "@/lib/media";
import { toast } from "@/lib/toast";

const schema = z.object({
  name: z.string().min(6, "Enter at least 6 characters.").max(32),
  email: z.email("Enter a valid email address."),
});
type Values = z.infer<typeof schema>;

export function AccountOverview() {
  const [user, setUser] = useState<UserSession | null>(null);
  const [orders, setOrders] = useState<OrderSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isUploadingAvatar, setIsUploadingAvatar] = useState(false);
  const { register, handleSubmit, reset, formState: { errors, isSubmitting } } = useForm<Values>({
    resolver: zodResolver(schema),
    mode: "onChange",
    reValidateMode: "onChange",
  });

  useEffect(() => {
    Promise.all([sessionApi.me(), orderApi.list(1)])
      .then(([session, history]) => {
        setUser(session);
        setOrders(history.data.slice(0, 3));
        reset({ name: session.name, email: session.email });
      })
      .catch((requestError) => setError(requestError instanceof ApiError ? requestError.message : "Your account could not be loaded."));
  }, [reset]);

  async function submit(values: Values) {
    setError(null);
    try {
      const updated = await sessionApi.update({ name: values.name, email: values.email });
      setUser(updated);
      toast.success("Profile saved", "Your Silk account details are up to date.");
    } catch (requestError) {
      toast.error("Profile not saved", requestError instanceof ApiError ? requestError.message : "Your profile could not be saved.");
    }
  }

  async function uploadAvatar(file: File) {
    const validationError = validateAvatarFile(file);
    if (validationError) {
      toast.error("Choose a different image", validationError);
      return;
    }
    setIsUploadingAvatar(true);
    try {
      const uploaded = await uploadAvatarImage(file);
      const updated = await sessionApi.update({
        name: user?.name ?? "",
        email: user?.email ?? "",
        image: uploaded.imageUrl,
        mediaAssetPid: uploaded.assetPid,
      });
      setUser(updated);
      toast.success("Profile picture updated", "Your new profile picture is ready.");
    } catch (uploadError) {
      toast.error("Profile picture not updated", uploadError instanceof Error ? uploadError.message : "Please try again.");
    } finally {
      setIsUploadingAvatar(false);
    }
  }

  if (!user && !error) return <AccountOverviewSkeleton />;
  if (!user) return <div className="account-state"><h1>We could not load your account.</h1><p>{error}</p></div>;

  return (
    <div className="account-overview">
      <header className="account-header"><p className="eyebrow">Account details</p><h1>Hello, {user.name.split(" ")[0]}.</h1><p>Keep your contact details current and revisit recent orders.</p></header>
      <section className="account-panel">
        <div className="account-panel__heading"><div><p className="eyebrow">Profile</p><h2>Your details</h2></div>{user.verified && <span>Verified email</span>}</div>
        <form className="profile-form" noValidate onSubmit={handleSubmit(submit)}>
          <ProfileAvatar image={user.image} isUploading={isUploadingAvatar} name={user.name} onSelect={uploadAvatar} />
          <FormField error={errors.name?.message} label="Full name" {...register("name")} />
          <FormField error={errors.email?.message} label="Email address" type="email" {...register("email")} />
          <button className="button-primary" disabled={isSubmitting} type="submit">{isSubmitting ? "Saving" : "Save profile"}</button>
        </form>
      </section>
      <section className="account-panel">
        <div className="account-panel__heading"><div><p className="eyebrow">Recent orders</p><h2>On your way</h2></div><a href="/account/orders">All orders <ArrowRightIcon aria-hidden size={15} /></a></div>
        {orders.length ? <div className="order-list compact">{orders.map((order) => <a href={`/account/orders/${order.pid}`} key={order.pid}><span><strong>ORD-{String(order.orderNumber).padStart(6, "0")}</strong><small>{new Intl.DateTimeFormat("en", { dateStyle: "medium" }).format(new Date(order.createdAt))}</small></span><span className="status-pill">{order.status}</span><strong>{formatCurrency(order.grandTotal, order.currency)}</strong></a>)}</div> : <p className="empty-copy">Your first Silk order will appear here.</p>}
      </section>
    </div>
  );
}
