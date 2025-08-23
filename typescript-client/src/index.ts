// Core client
import { NatsClient as NatsClientClass, ClientConfig } from './nats-client';
export { NatsClient, ClientConfig } from './nats-client';

// Service clients  
import { CatalogClient as CatalogClientClass } from './services/catalog-client';
export { CatalogClient } from './services/catalog-client';

// Generated types - Product (selective export to avoid conflicts)
export {
  Product,
  ProductCreateRequest,
  ProductCreateResponse,
  ProductGetRequest,
  ProductGetResponse,
  ProductGetBySlugRequest,
  ProductGetBySlugResponse,
  ProductUpdateRequest,
  ProductUpdateResponse,
  ProductDeleteRequest,
  ProductDeleteResponse,
  ProductSearchRequest,
  ProductSearchResponse,
  ProductExportRequest,
  ProductExportResponse,
  GetProductSlugsRequest,
  GetProductSlugsResponse,
  ProductVariant,
  Reviews,
} from '../generated/product';

// Generated types - Category (selective export)
export {
  CategoryResponse,
  CreateCategoryRequest,
  GetCategoryRequest,
  GetCategoryBySlugRequest,
  UpdateCategoryRequest,
  DeleteCategoryRequest,
  GetCategoryResponse,
  CategoryTreeRequest,
  CategoryTreeResponse,
  CategoryTreeNode,
} from '../generated/category';

// Generated types - Common
export { Code } from '../generated/common/code';
export { Status } from '../generated/common/status';

// Generated types - Events (selective export)
export {
  ProductCreatedEvent,
  ProductUpdatedEvent,
  ProductDeletedEvent,
  CategoryCreatedEvent,
  CategoryUpdatedEvent,
  CategoryDeletedEvent,
  CategoryTreeRebuiltEvent,
  OrderPlacedEvent,
  InventoryUpdatedEvent,
} from '../generated/events';

// NATS configuration
export { NatsConfig, NatsConfigType } from '../generated/nats-config';

// Helper function to create a connected client
export async function createClient(config: ClientConfig): Promise<{
  natsClient: NatsClientClass;
  catalog: CatalogClientClass;
}> {
  const natsClient = new NatsClientClass(config);
  await natsClient.connect();
  
  return {
    natsClient,
    catalog: new CatalogClientClass(natsClient),
  };
}