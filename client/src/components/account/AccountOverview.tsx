import { zodResolver } from "@hookform/resolvers/zod";
import { ArrowRightIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { FormField } from "@/components/forms/FormField";
import { ApiError, orderApi, sessionApi } from "@/lib/api/browser";
import type { OrderSummary, UserSession } from "@/lib/api/types";
import { formatCurrency } from "@/lib/format";

const schema = z.object({
  name: z.string().min(6, "Enter at least 6 characters.").max(32),
  email: z.email("Enter a valid email address."),
  image: z.union([z.literal(""), z.url("Enter a valid image URL.")]),
});
type Values = z.infer<typeof schema>;

export function AccountOverview() {
  const [user, setUser] = useState<UserSession | null>(null);
  const [orders, setOrders] = useState<OrderSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const { register, handleSubmit, reset, formState: { errors, isSubmitting } } = useForm<Values>({ resolver: zodResolver(schema) });

  useEffect(() => {
    Promise.all([sessionApi.me(), orderApi.list(1)])
      .then(([session, history]) => {
        setUser(session);
        setOrders(history.data.slice(0, 3));
        reset({ name: session.name, email: session.email, image: session.image ?? "" });
      })
      .catch((requestError) => setError(requestError instanceof ApiError ? requestError.message : "Your account could not be loaded."));
  }, [reset]);

  async function submit(values: Values) {
    setError(null);
    setSaved(false);
    try {
      const updated = await sessionApi.update({ name: values.name, email: values.email, image: values.image || null });
      setUser(updated);
      setSaved(true);
    } catch (requestError) {
      setError(requestError instanceof ApiError ? requestError.message : "Your profile could not be saved.");
    }
  }

  if (!user && !error) return <div className="account-state">Loading your account...</div>;
  if (!user) return <div className="account-state"><h1>We could not load your account.</h1><p>{error}</p></div>;

  return (
    <div className="account-overview">
      <header className="account-header"><p className="eyebrow">Account details</p><h1>Hello, {user.name.split(" ")[0]}.</h1><p>Keep your contact details current and revisit recent orders.</p></header>
      <section className="account-panel">
        <div className="account-panel__heading"><div><p className="eyebrow">Profile</p><h2>Your details</h2></div>{user.verified && <span>Verified email</span>}</div>
        <form className="profile-form" noValidate onSubmit={handleSubmit(submit)}>
          <FormField error={errors.name?.message} label="Full name" {...register("name")} />
          <FormField error={errors.email?.message} label="Email address" type="email" {...register("email")} />
          <FormField error={errors.image?.message} hint="Use a public image URL." label="Profile image URL" type="url" {...register("image")} />
          {error && <p className="form-error" role="alert">{error}</p>}
          {saved && <p className="form-success" role="status">Profile saved.</p>}
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
