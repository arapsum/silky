import { ListIcon, MagnifyingGlassIcon, XIcon } from "@phosphor-icons/react";
import { useEffect, useId, useRef, useState } from "react";
import { ThemeToggle } from "@/components/theme/ThemeToggle";

const links = [
  { id: "collection", href: "/shop", label: "Collection" },
  { id: "shop", href: "/shop", label: "Shop all" },
  { id: "house", href: "/", label: "House" },
  { id: "new-in", href: "/#new-arrivals", label: "New in" },
  { id: "in-stock", href: "/shop?stockStatus=inStock", label: "In stock" },
  { id: "t-shirts", href: "/shop?category=t-shirts", label: "T-shirts" },
  { id: "shoes", href: "/shop?category=shoes", label: "Shoes" },
  { id: "accessories", href: "/shop?category=accessories", label: "Accessories" },
  { id: "account", href: "/account", label: "Your account" },
];

export function MobileNavigation() {
  const [isOpen, setIsOpen] = useState(false);
  const menuId = useId();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;

    const firstFocusable = panelRef.current?.querySelector<HTMLElement>("a, input, button");
    firstFocusable?.focus();

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      setIsOpen(false);
      window.requestAnimationFrame(() => triggerRef.current?.focus());
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen]);

  return (
    <div className="mobile-navigation">
      <button
        aria-controls={menuId}
        aria-expanded={isOpen}
        aria-label={isOpen ? "Close navigation" : "Open navigation"}
        className="mobile-navigation__trigger"
        onClick={() => setIsOpen((open) => !open)}
        ref={triggerRef}
        type="button"
      >
        {isOpen ? <XIcon aria-hidden size={22} weight="regular" /> : <ListIcon aria-hidden size={22} weight="regular" />}
      </button>

      {isOpen && (
        <div className="mobile-navigation__panel" id={menuId} ref={panelRef}>
          <nav aria-label="Mobile navigation">
            {links.map((link) => (
              <a href={link.href} key={link.id} onClick={() => setIsOpen(false)}>
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

          <div className="mobile-navigation__theme">
            <span>Appearance</span>
            <ThemeToggle />
          </div>
        </div>
      )}
    </div>
  );
}
