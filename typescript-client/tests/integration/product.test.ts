import { Code } from '../../src';
import { 
  getTestClient, 
  disconnectTestClient, 
  generateTestId,
  TestClient 
} from '../utils/test-client';

describe('Product Integration Tests', () => {
  let client: TestClient;
  const createdProductIds: string[] = [];

  beforeAll(async () => {
    client = await getTestClient();
  });

  afterAll(async () => {
    // Clean up any products that weren't deleted during tests
    for (const id of createdProductIds) {
      try {
        await client.catalog.deleteProduct({ id });
      } catch (error) {
        // Ignore errors during cleanup
      }
    }
    
    await disconnectTestClient();
  });

  describe('Product CRUD Operations', () => {
    test('should create a new product', async () => {
      const productRef = generateTestId('PROD');
      const slug = generateTestId('product');
      
      const response = await client.catalog.createProduct({
        name: 'Test Product',
        productRef,
        slug,
        brand: 'Test Brand',
        longDescription: 'This is a test product',
        productType: 'physical',
        displayOnSite: true,
        definingAttributes: {
          'color': 'red',
          'size': 'large',
        },
        descriptiveAttributes: {
          'material': 'cotton',
          'weight': '100g',
        },
        seoTitle: 'Test Product SEO',
        seoDescription: 'Test product for integration testing',
        seoKeywords: 'test,product',
        taxCode: 'txcd_99999999',
        relatedProducts: [],
        reviews: undefined,
        hierarchicalCategories: undefined,
        listCategories: ['Test Category'],
        defaultVariant: undefined,
        variants: [],
      });

      expect(response.status?.code).toBe(Code.OK);
      expect(response.product).toBeValidProduct();
      expect(response.product?.name).toBe('Test Product');
      expect(response.product?.productRef).toBe(productRef);
      expect(response.product?.slug).toBe(slug);
      
      if (response.product?.id) {
        createdProductIds.push(response.product.id);
      }
    });

    test('should get a product by ID', async () => {
      // First create a product
      const productRef = generateTestId('PROD');
      const createResponse = await client.catalog.createProduct({
        name: 'Product to Get',
        productRef,
        slug: generateTestId('product'),
        brand: 'Test Brand',
        longDescription: 'Product for get test',
        productType: 'physical',
        displayOnSite: true,
        definingAttributes: {},
        descriptiveAttributes: {},
        seoTitle: undefined,
        seoDescription: undefined,
        seoKeywords: undefined,
        taxCode: undefined,
        relatedProducts: [],
        reviews: undefined,
        hierarchicalCategories: undefined,
        listCategories: [],
        defaultVariant: undefined,
        variants: [],
      });

      expect(createResponse.status?.code).toBe(Code.OK);
      const productId = createResponse.product?.id;
      expect(productId).toBeDefined();
      
      if (productId) {
        createdProductIds.push(productId);
        
        // Now get the product
        const getResponse = await client.catalog.getProduct({ id: productId });
        
        expect(getResponse.status?.code).toBe(Code.OK);
        expect(getResponse.product).toBeValidProduct();
        expect(getResponse.product?.id).toBe(productId);
        expect(getResponse.product?.name).toBe('Product to Get');
        expect(getResponse.product?.productRef).toBe(productRef);
      }
    });

    test('should get a product by slug', async () => {
      const slug = generateTestId('product-slug');
      
      // Create a product with specific slug
      const createResponse = await client.catalog.createProduct({
        name: 'Product with Slug',
        productRef: generateTestId('PROD'),
        slug,
        brand: 'Test Brand',
        longDescription: 'Product for slug test',
        productType: 'physical',
        displayOnSite: true,
        definingAttributes: {},
        descriptiveAttributes: {},
        seoTitle: undefined,
        seoDescription: undefined,
        seoKeywords: undefined,
        taxCode: undefined,
        relatedProducts: [],
        reviews: undefined,
        hierarchicalCategories: undefined,
        listCategories: [],
        defaultVariant: undefined,
        variants: [],
      });

      expect(createResponse.status?.code).toBe(Code.OK);
      
      if (createResponse.product?.id) {
        createdProductIds.push(createResponse.product.id);
        
        // Get by slug
        const getResponse = await client.catalog.getProductBySlug({ slug });
        
        expect(getResponse.status?.code).toBe(Code.OK);
        expect(getResponse.product).toBeValidProduct();
        expect(getResponse.product?.slug).toBe(slug);
        expect(getResponse.product?.name).toBe('Product with Slug');
      }
    });

    test('should update a product', async () => {
      // Create a product
      const createResponse = await client.catalog.createProduct({
        name: 'Product to Update',
        productRef: generateTestId('PROD'),
        slug: generateTestId('product'),
        brand: 'Original Brand',
        longDescription: 'Original description',
        productType: 'physical',
        displayOnSite: true,
        definingAttributes: {},
        descriptiveAttributes: {},
        seoTitle: undefined,
        seoDescription: undefined,
        seoKeywords: undefined,
        taxCode: undefined,
        relatedProducts: [],
        reviews: undefined,
        hierarchicalCategories: undefined,
        listCategories: [],
        defaultVariant: undefined,
        variants: [],
      });

      expect(createResponse.status?.code).toBe(Code.OK);
      const productId = createResponse.product?.id;
      
      if (productId) {
        createdProductIds.push(productId);
        
        // Update the product
        const updateResponse = await client.catalog.updateProduct({
          id: productId,
          product: {
            name: 'Updated Product Name',
            brand: 'Updated Brand',
            longDescription: 'Updated description',
            displayOnSite: false,
            productRef: 'TEST-REF-001', // Required field
            definingAttributes: {},
            descriptiveAttributes: {},
            seoTitle: undefined,
            seoDescription: undefined,
            seoKeywords: undefined,
            taxCode: undefined,
            relatedProducts: [],
            listCategories: [],
            variants: [],
          }
        });
        
        expect(updateResponse.status?.code).toBe(Code.OK);
        expect(updateResponse.product?.name).toBe('Updated Product Name');
        expect(updateResponse.product?.brand).toBe('Updated Brand');
        expect(updateResponse.product?.longDescription).toBe('Updated description');
        expect(updateResponse.product?.displayOnSite).toBe(false);
      }
    });

    test('should delete a product', async () => {
      // Create a product
      const createResponse = await client.catalog.createProduct({
        name: 'Product to Delete',
        productRef: generateTestId('PROD'),
        slug: generateTestId('product'),
        brand: 'Test Brand',
        longDescription: 'Product that will be deleted',
        productType: 'physical',
        displayOnSite: true,
        definingAttributes: {},
        descriptiveAttributes: {},
        seoTitle: undefined,
        seoDescription: undefined,
        seoKeywords: undefined,
        taxCode: undefined,
        relatedProducts: [],
        reviews: undefined,
        hierarchicalCategories: undefined,
        listCategories: [],
        defaultVariant: undefined,
        variants: [],
      });

      expect(createResponse.status?.code).toBe(Code.OK);
      const productId = createResponse.product?.id;
      
      if (productId) {
        // Delete the product
        const deleteResponse = await client.catalog.deleteProduct({ id: productId });
        expect(deleteResponse.status?.code).toBe(Code.OK);
        
        // Verify it's deleted
        const getResponse = await client.catalog.getProduct({ id: productId });
        expect(getResponse.status?.code).toBe(Code.NOT_FOUND);
        
        // Remove from cleanup list since it's already deleted
        const index = createdProductIds.indexOf(productId);
        if (index > -1) {
          createdProductIds.splice(index, 1);
        }
      }
    });

    test('should handle non-existent product gracefully', async () => {
      const response = await client.catalog.getProduct({ 
        id: '507f1f77bcf86cd799439011' // Valid MongoDB ObjectId format but doesn't exist
      });
      
      expect(response.status?.code).toBe(Code.NOT_FOUND);
      expect(response.product).toBeUndefined();
    });
  });

  describe('Product Search', () => {
    test('should search products by query', async () => {
      const uniquePrefix = generateTestId('SEARCH');
      
      // Create multiple products
      const products = [
        { name: `${uniquePrefix} Product One`, productRef: `${uniquePrefix}-1` },
        { name: `${uniquePrefix} Product Two`, productRef: `${uniquePrefix}-2` },
        { name: `Different Product`, productRef: generateTestId('DIFF') },
      ];
      
      for (const product of products) {
        const response = await client.catalog.createProduct({
          name: product.name,
          productRef: product.productRef,
          slug: generateTestId('product'),
          brand: 'Test Brand',
          longDescription: 'Search test product',
          productType: 'physical',
          displayOnSite: true,
          definingAttributes: {},
          descriptiveAttributes: {},
          seoTitle: undefined,
          seoDescription: undefined,
          seoKeywords: undefined,
          taxCode: undefined,
          relatedProducts: [],
          reviews: undefined,
          hierarchicalCategories: undefined,
          listCategories: [],
          defaultVariant: undefined,
          variants: [],
        });
        
        if (response.product?.id) {
          createdProductIds.push(response.product.id);
        }
      }
      
      // Search for products with unique prefix
      const searchResponse = await client.catalog.searchProducts({
        query: uniquePrefix,
        categories: [],
        brand: undefined,
        limit: 10,
        offset: undefined,
      });
      
      expect(searchResponse.status?.code).toBe(Code.OK);
      expect(searchResponse.products).toHaveLength(2);
      expect(searchResponse.products.every(p => p.name?.includes(uniquePrefix))).toBe(true);
    });

    test('should limit search results', async () => {
      const searchResponse = await client.catalog.searchProducts({
        query: undefined,
        categories: [],
        brand: undefined,
        limit: 5,
        offset: undefined,
      });
      
      expect(searchResponse.status?.code).toBe(Code.OK);
      expect(searchResponse.products.length).toBeLessThanOrEqual(5);
    });
  });

  describe('Product Validation', () => {
    test('should reject product with missing required fields', async () => {
      const response = await client.catalog.createProduct({
        name: '', // Empty name should fail
        productRef: generateTestId('PROD'),
        slug: generateTestId('product'),
        brand: 'Test Brand',
        longDescription: 'Test',
        productType: 'physical',
        displayOnSite: true,
        definingAttributes: {},
        descriptiveAttributes: {},
        seoTitle: undefined,
        seoDescription: undefined,
        seoKeywords: undefined,
        taxCode: undefined,
        relatedProducts: [],
        reviews: undefined,
        hierarchicalCategories: undefined,
        listCategories: [],
        defaultVariant: undefined,
        variants: [],
      });
      
      expect(response.status?.code).toBe(Code.INVALID_ARGUMENT);
    });

    test('should reject duplicate product ref', async () => {
      const productRef = generateTestId('DUPLICATE');
      
      // Create first product
      const firstResponse = await client.catalog.createProduct({
        name: 'First Product',
        productRef,
        slug: generateTestId('product1'),
        brand: 'Test Brand',
        longDescription: 'First product',
        productType: 'physical',
        displayOnSite: true,
        definingAttributes: {},
        descriptiveAttributes: {},
        seoTitle: undefined,
        seoDescription: undefined,
        seoKeywords: undefined,
        taxCode: undefined,
        relatedProducts: [],
        reviews: undefined,
        hierarchicalCategories: undefined,
        listCategories: [],
        defaultVariant: undefined,
        variants: [],
      });
      
      expect(firstResponse.status?.code).toBe(Code.OK);
      
      if (firstResponse.product?.id) {
        createdProductIds.push(firstResponse.product.id);
        
        // Try to create second product with same productRef
        const secondResponse = await client.catalog.createProduct({
          name: 'Second Product',
          productRef, // Same productRef
          slug: generateTestId('product2'),
          brand: 'Test Brand',
          longDescription: 'Second product',
          productType: 'physical',
          displayOnSite: true,
          definingAttributes: {},
          descriptiveAttributes: {},
          seoTitle: undefined,
          seoDescription: undefined,
          seoKeywords: undefined,
          taxCode: undefined,
          relatedProducts: [],
          reviews: undefined,
          hierarchicalCategories: undefined,
          listCategories: [],
          defaultVariant: undefined,
          variants: [],
        });
        
        expect(secondResponse.status?.code).toBe(Code.ALREADY_EXISTS);
      }
    });
  });
});