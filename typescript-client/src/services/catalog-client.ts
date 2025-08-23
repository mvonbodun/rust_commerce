import { NatsClient } from '../nats-client';
import { NatsConfig } from '../../generated/nats-config';
import {
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
} from '../../generated/product';
import {
  CreateCategoryRequest,
  CategoryResponse,
  GetCategoryRequest,
  GetCategoryBySlugRequest,
  UpdateCategoryRequest,
  DeleteCategoryRequest,
  GetCategoryResponse,
  CategoryTreeRequest,
  CategoryTreeResponse,
} from '../../generated/category';

export class CatalogClient {
  constructor(private natsClient: NatsClient) {}

  // Product methods
  async createProduct(request: ProductCreateRequest): Promise<ProductCreateResponse> {
    return this.natsClient.request(
      NatsConfig.Product.methods.CreateProduct.subject,
      request,
      (msg) => ProductCreateRequest.encode(msg).finish(),
      (data) => ProductCreateResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async getProduct(request: ProductGetRequest): Promise<ProductGetResponse> {
    return this.natsClient.request(
      NatsConfig.Product.methods.GetProduct.subject,
      request,
      (msg) => ProductGetRequest.encode(msg).finish(),
      (data) => ProductGetResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async getProductBySlug(request: ProductGetBySlugRequest): Promise<ProductGetBySlugResponse> {
    return this.natsClient.request(
      NatsConfig.Product.methods.GetProductBySlug.subject,
      request,
      (msg) => ProductGetBySlugRequest.encode(msg).finish(),
      (data) => ProductGetBySlugResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async updateProduct(request: ProductUpdateRequest): Promise<ProductUpdateResponse> {
    return this.natsClient.request(
      NatsConfig.Product.methods.UpdateProduct.subject,
      request,
      (msg) => ProductUpdateRequest.encode(msg).finish(),
      (data) => ProductUpdateResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async deleteProduct(request: ProductDeleteRequest): Promise<ProductDeleteResponse> {
    return this.natsClient.request(
      NatsConfig.Product.methods.DeleteProduct.subject,
      request,
      (msg) => ProductDeleteRequest.encode(msg).finish(),
      (data) => ProductDeleteResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async searchProducts(request: ProductSearchRequest): Promise<ProductSearchResponse> {
    return this.natsClient.request(
      NatsConfig.Product.methods.SearchProducts.subject,
      request,
      (msg) => ProductSearchRequest.encode(msg).finish(),
      (data) => ProductSearchResponse.decode(data),
      { timeout: NatsConfig.Product.methods.SearchProducts.timeoutMs || 10000 }
    );
  }

  async exportProducts(request: ProductExportRequest): Promise<ProductExportResponse> {
    return this.natsClient.request(
      NatsConfig.Product.methods.ExportProducts.subject,
      request,
      (msg) => ProductExportRequest.encode(msg).finish(),
      (data) => ProductExportResponse.decode(data),
      { timeout: NatsConfig.Product.methods.ExportProducts.timeoutMs || 30000 }
    );
  }

  async getProductSlugs(request: GetProductSlugsRequest): Promise<GetProductSlugsResponse> {
    return this.natsClient.request(
      NatsConfig.Product.methods.GetProductSlugs.subject,
      request,
      (msg) => GetProductSlugsRequest.encode(msg).finish(),
      (data) => GetProductSlugsResponse.decode(data),
      { timeout: 10000 }
    );
  }

  // Category methods
  async createCategory(request: CreateCategoryRequest): Promise<CategoryResponse> {
    return this.natsClient.request(
      NatsConfig.Category.methods.CreateCategory.subject,
      request,
      (msg) => CreateCategoryRequest.encode(msg).finish(),
      (data) => CategoryResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async getCategory(request: GetCategoryRequest): Promise<CategoryResponse> {
    return this.natsClient.request(
      NatsConfig.Category.methods.GetCategory.subject,
      request,
      (msg) => GetCategoryRequest.encode(msg).finish(),
      (data) => CategoryResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async getCategoryBySlug(request: GetCategoryBySlugRequest): Promise<CategoryResponse> {
    return this.natsClient.request(
      NatsConfig.Category.methods.GetCategoryBySlug.subject,
      request,
      (msg) => GetCategoryBySlugRequest.encode(msg).finish(),
      (data) => CategoryResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async updateCategory(request: UpdateCategoryRequest): Promise<CategoryResponse> {
    return this.natsClient.request(
      NatsConfig.Category.methods.UpdateCategory.subject,
      request,
      (msg) => UpdateCategoryRequest.encode(msg).finish(),
      (data) => CategoryResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async deleteCategory(request: DeleteCategoryRequest): Promise<GetCategoryResponse> {
    return this.natsClient.request(
      NatsConfig.Category.methods.DeleteCategory.subject,
      request,
      (msg) => DeleteCategoryRequest.encode(msg).finish(),
      (data) => GetCategoryResponse.decode(data),
      { timeout: 5000 }
    );
  }

  async getCategoryTree(request: CategoryTreeRequest): Promise<CategoryTreeResponse> {
    return this.natsClient.request(
      NatsConfig.Category.methods.GetCategoryTree.subject,
      request,
      (msg) => CategoryTreeRequest.encode(msg).finish(),
      (data) => CategoryTreeResponse.decode(data),
      { timeout: NatsConfig.Category.methods.GetCategoryTree.timeoutMs || 10000 }
    );
  }
}