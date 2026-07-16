import { PencilSimpleIcon, PlusIcon, TrashIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { AddressForm } from "./AddressForm";
import { AddressBookSkeleton } from "@/components/loading/StorefrontSkeletons";
import { addressApi, ApiError } from "@/lib/api/browser";
import type { Address } from "@/lib/api/types";

export function AddressBook() {
  const [addresses, setAddresses] = useState<Address[]>([]);
  const [editing, setEditing] = useState<Address | "new" | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    addressApi.list().then(setAddresses).catch((requestError) => setError(requestError instanceof ApiError ? requestError.message : "Addresses could not be loaded.")).finally(() => setLoading(false));
  }, []);

  async function remove(address: Address) {
    if (!window.confirm(`Remove ${address.label ?? address.lineOne}?`)) return;
    try {
      await addressApi.remove(address.pid);
      setAddresses((current) => current.filter((entry) => entry.pid !== address.pid));
    } catch (requestError) {
      setError(requestError instanceof ApiError ? requestError.message : "The address could not be removed.");
    }
  }

  return (
    <div>
      <header className="account-header account-header--action"><div><p className="eyebrow">Saved addresses</p><h1>Places you use.</h1><p>Keep delivery and billing details ready for checkout.</p></div><button className="button-primary" onClick={() => setEditing("new")} type="button"><PlusIcon aria-hidden size={17} /> Add address</button></header>
      {error && <p className="form-error" role="alert">{error}</p>}
      {editing && <AddressForm address={editing === "new" ? undefined : editing} onCancel={() => setEditing(null)} onSaved={(saved) => { setAddresses((current) => editing === "new" ? [...current, saved] : current.map((entry) => entry.pid === saved.pid ? saved : entry)); setEditing(null); }} />}
      {loading ? <AddressBookSkeleton /> : addresses.length ? (
        <div className="address-grid">{addresses.map((address) => <article key={address.pid}><div><span className="status-pill">{address.addressType}</span>{address.isDefault && <span className="status-pill">default</span>}</div><h2>{address.label ?? address.recipientName}</h2><p>{address.recipientName}<br />{address.lineOne}{address.lineTwo && <><br />{address.lineTwo}</>}<br />{address.city}{address.region ? `, ${address.region}` : ""}<br />{address.countryCode}</p><div className="address-card__actions"><button onClick={() => setEditing(address)} type="button"><PencilSimpleIcon aria-hidden size={15} /> Edit</button><button onClick={() => remove(address)} type="button"><TrashIcon aria-hidden size={15} /> Remove</button></div></article>)}</div>
      ) : !editing && <p className="empty-copy">No saved addresses yet. Add one to make checkout quicker.</p>}
    </div>
  );
}
