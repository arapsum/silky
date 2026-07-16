import { ListIcon, MagnifyingGlassIcon, XIcon } from "@phosphor-icons/react";
import { useId, useState } from "react";

const links = [
  { href: "/shop", label: "Shop all" },
  { href: "/#new-arrivals", label: "New in" },
  { href: "/shop?category=t-shirts", label: "T-shirts" },
  { href: "/shop?category=shoes", label: "Shoes" },
  { href: "/shop?category=accessories", label: "Accessories" },
  { href: "/account", label: "Your account" },
];

export function MobileNavigation() {
  const [isOpen, setIsOpen] = useState(false);
  const menuId = useId();

  return (
    <div className="mobile-navigation">
      <button
        aria-controls={menuId}
        aria-expanded={isOpen}
        aria-label={isOpen ? "Close navigation" : "Open navigation"}
        className="mobile-navigation__trigger"
        onClick={() => setIsOpen((open) => !open)}
        type="button"
      >
        {isOpen ? <XIcon aria-hidden size={22} /> : <ListIcon aria-hidden size={22} />}
      </button>

      {isOpen && (
        <div className="mobile-navigation__panel" id={menuId}>
          <nav aria-label="Mobile navigation">
            {links.map((link) => (
              <a href={link.href} key={link.href} onClick={() => setIsOpen(false)}>
                {link.label}
              </a>
            ))}
          </nav>

          <form action="/shop" className="mobile-navigation__search" method="get" role="search">
            <MagnifyingGlassIcon aria-hidden size={18} />
            <label className="sr-only" htmlFor={`${menuId}-search`}>
              Search the catalogue
            </label>
            <input id={`${menuId}-search`} name="search" placeholder="Search Silk" type="search" />
          </form>
        </div>
      )}
    </div>
  );
}
