export interface Pagination {
  page: number;
  limit: number;
  totalItems: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: Pagination;
}

export interface ProductCategorySummary {
  id: number;
  pid: string;
  name: string;
  slug: string;
}

export interface ProductVariantSummary {
  pid: string;
  sku: string;
  price: string;
  stockQuantity: number;
}

export interface ProductListItem {
  pid: string;
  slug: string;
  name: string;
  description: string | null;
  information: Record<string, string>;
  category: ProductCategorySummary;
  primaryImage: string | null;
  defaultVariant: ProductVariantSummary | null;
  variantCount: number;
  optionCount: number;
  totalStock: number;
  createdAt: string;
  updatedAt: string;
}

export interface CategoryListItem {
  id: number;
  pid: string;
  name: string;
  slug: string;
  imageLink: string;
  description: string | null;
  parentId: number | null;
  parentName: string | null;
  productCount: number;
}

export interface ProductTag {
  id: number;
  pid: string;
  name: string;
  createdAt: string;
  updatedAt: string;
}

export interface ProductPicture {
  id: number;
  pid: string;
  imageLink: string;
  displayOrder: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface ProductOption {
  id: number;
  pid: string;
  attributeId: number;
  attributePid: string;
  attributeName: string;
  attributeDescription: string | null;
  displayOrder: number | null;
  createdAt: string;
}

export interface VariantOption {
  id: number;
  pid: string;
  attributeId: number;
  attributePid: string;
  attributeName: string;
  attributeDescription: string | null;
  attributeValueId: number;
  attributeValuePid: string;
  value: string;
  createdAt: string;
}

export interface ProductVariant {
  id: number;
  pid: string;
  sku: string;
  price: string;
  stockQuantity: number;
  isDefault: boolean;
  options: VariantOption[];
  pictures: ProductPicture[];
  createdAt: string;
  updatedAt: string;
}

export interface ProductDetail {
  id: number;
  pid: string;
  slug: string;
  name: string;
  description: string | null;
  information: Record<string, string>;
  tags: ProductTag[];
  category: ProductCategorySummary;
  pictures: ProductPicture[];
  options: ProductOption[];
  variants: ProductVariant[];
  createdAt: string;
  updatedAt: string;
}

export interface ApiErrorPayload {
  error?: string;
  code?: string;
}

export interface UserSession {
  pid: string;
  email: string;
  name: string;
  image: string | null;
  verified: boolean;
  roles: string[];
  permissions: string[];
  createdAt: string;
  updatedAt: string;
}

export interface LoginResponse {
  pid: string;
  email: string;
  name: string;
  verified: boolean;
}

export interface Address {
  pid: string;
  addressType: "billing" | "shipping" | "other";
  label: string | null;
  recipientName: string;
  company: string | null;
  lineOne: string;
  lineTwo: string | null;
  city: string;
  region: string | null;
  postalCode: string | null;
  countryCode: string;
  email: string | null;
  phone: string | null;
  isDefault: boolean;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}

export type AddressInput = Omit<Address, "pid" | "createdAt" | "updatedAt" | "deletedAt">;

export interface CartQuoteLine {
  variantPid: string;
  productPid: string | null;
  productSlug: string | null;
  productName: string | null;
  sku: string | null;
  imageUrl: string | null;
  selectedOptions: Record<string, string>;
  requestedQuantity: number;
  availableQuantity: number;
  unitPrice: string | null;
  lineTotal: string | null;
  status: "available" | "insufficient_stock" | "unavailable";
}

export interface CartQuote {
  currency: string;
  subtotal: string;
  shippingTotal: string;
  taxTotal: string;
  grandTotal: string;
  canCheckout: boolean;
  items: CartQuoteLine[];
}

export interface CheckoutResponse {
  orderPid: string;
  checkoutUrl: string;
  expiresAt: string;
}

export interface CheckoutSession {
  orderPid: string;
  orderStatus: string;
  paymentStatus: string;
  attemptStatus: string;
  checkoutUrl: string | null;
  expiresAt: string | null;
}

export interface OrderSummary {
  id: number;
  pid: string;
  orderNumber: number;
  customerName: string;
  customerEmail: string;
  customerImage: string | null;
  status: string;
  paymentStatus: string;
  fulfillmentStatus: string;
  currency: string;
  subtotal: string;
  shippingTotal: string;
  taxTotal: string;
  grandTotal: string;
  placedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface OrderItem {
  pid: string;
  productPid: string;
  variantPid: string;
  productSlug: string | null;
  imageUrl: string | null;
  productName: string;
  sku: string;
  selectedOptions: Record<string, string>;
  quantity: number;
  unitPrice: string;
  lineTotal: string;
}

export interface OrderDetail {
  order: OrderSummary & {
    billingAddressSnapshot: Record<string, unknown>;
    shippingAddressSnapshot: Record<string, unknown>;
    customerNote: string | null;
  };
  items: OrderItem[];
}
