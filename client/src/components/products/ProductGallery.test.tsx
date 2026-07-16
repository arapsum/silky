import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { ProductGallery, type GalleryImage } from "./ProductGallery";

const images: GalleryImage[] = Array.from({ length: 7 }, (_, index) => ({
  id: `image-${index + 1}`,
  src: `/products/image-${index + 1}.jpg`,
  alt: `Product view ${index + 1}`,
}));

afterEach(cleanup);

describe("ProductGallery", () => {
  it("keeps long galleries to four thumbnail slots", () => {
    render(<ProductGallery images={images} />);

    expect(screen.getAllByRole("button", { name: /View product image/i })).toHaveLength(3);
    expect(screen.getByRole("button", { name: "View all 7 product images" })).toBeTruthy();
    expect(screen.getByText("+4")).toBeTruthy();
  });

  it("opens the overflow gallery at the fourth image", async () => {
    const user = userEvent.setup();
    render(<ProductGallery images={images} />);

    await user.click(screen.getByRole("button", { name: "View all 7 product images" }));

    const dialog = await waitFor(() => screen.getByRole("dialog", { name: "Product image gallery" }));
    expect(screen.getByText("Image 4 of 7")).toBeTruthy();
    expect(within(dialog).getByRole("img", { name: "Product view 4" })).toBeTruthy();
  });

  it("supports arrow-key navigation inside the gallery", async () => {
    const user = userEvent.setup();
    render(<ProductGallery images={images} />);

    await user.click(screen.getByRole("button", { name: /Open image 1 of 7/i }));
    const dialog = await screen.findByRole("dialog", { name: "Product image gallery" });
    fireEvent.keyDown(dialog, { key: "ArrowRight" });

    expect(screen.getByText("Image 2 of 7")).toBeTruthy();
    expect(within(dialog).getByRole("img", { name: "Product view 2" })).toBeTruthy();
  });

  it("shows every thumbnail when the gallery has four images", () => {
    render(<ProductGallery images={images.slice(0, 4)} />);

    expect(screen.getAllByRole("button", { name: /View product image/i })).toHaveLength(4);
    expect(screen.queryByRole("button", { name: /View all/i })).toBeNull();
  });
});
