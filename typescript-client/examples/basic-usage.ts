#!/usr/bin/env tsx

import { createClient, Code } from '../src';

async function main() {
  // Create and connect the client
  const client = await createClient({
    servers: 'nats://localhost:4222',
  });

  console.log('✅ Connected to NATS');

  try {
    // Create a product
    console.log('\n📦 Creating a product...');
    const createResponse = await client.catalog.createProduct({
      name: 'Example Product',
      productRef: 'PROD-001',
      slug: 'example-product',
      brand: 'Example Brand',
      longDescription: 'This is an example product created from TypeScript',
      productType: 'physical',
      displayOnSite: true,
      definingAttributes: {},
      descriptiveAttributes: {},
      seoTitle: 'Example Product SEO Title',
      seoDescription: 'Example product for demonstrating TypeScript client',
      seoKeywords: 'example,product,typescript',
      taxCode: 'txcd_99999999',
      relatedProducts: [],
      reviews: undefined,
      hierarchicalCategories: undefined,
      listCategories: ['Electronics'],
      defaultVariant: undefined,
      variants: [],
    });

    if (createResponse.status?.code === Code.OK) {
      console.log('✅ Product created:', createResponse.product?.id);
      
      // Get the product by ID
      if (createResponse.product?.id) {
        console.log('\n🔍 Fetching product by ID...');
        const getResponse = await client.catalog.getProduct({
          id: createResponse.product.id,
        });
        
        if (getResponse.status?.code === Code.OK) {
          console.log('✅ Product found:', getResponse.product?.name);
        }
      }
      
      // Search for products
      console.log('\n🔎 Searching for products...');
      const searchResponse = await client.catalog.searchProducts({
        query: 'Example',
        categories: [],
        brand: undefined,
        limit: 10,
        offset: undefined,
      });
      
      if (searchResponse.status?.code === Code.OK) {
        console.log(`✅ Found ${searchResponse.products.length} products`);
        searchResponse.products.forEach(p => {
          console.log(`  - ${p.name} (${p.productRef})`);
        });
      }
      
      // Delete the product
      if (createResponse.product?.id) {
        console.log('\n🗑️  Deleting product...');
        const deleteResponse = await client.catalog.deleteProduct({
          id: createResponse.product.id,
        });
        
        if (deleteResponse.status?.code === Code.OK) {
          console.log('✅ Product deleted');
        }
      }
    } else {
      console.error('❌ Failed to create product:', createResponse.status?.message);
    }

    // Category operations
    console.log('\n📁 Creating a category...');
    const categoryResponse = await client.catalog.createCategory({
      name: 'Test Category',
      slug: 'test-category',
      shortDescription: 'A test category',
      fullDescription: undefined,
      parentId: undefined,
      displayOrder: 1,
      seo: undefined,
      isActive: true,
      parentSlug: undefined,
    });

    if (categoryResponse.code === Code.OK) {
      console.log('✅ Category created:', categoryResponse.id);
      
      // Get category tree
      console.log('\n🌲 Fetching category tree...');
      const treeResponse = await client.catalog.getCategoryTree({
        includeInactive: false,
        maxDepth: undefined,
      });
      
      if (treeResponse.status?.code === Code.OK) {
        console.log('✅ Category tree fetched');
        console.log(`  Root categories: ${treeResponse.tree?.roots.length || 0}`);
      }
      
      // Delete the category
      console.log('\n🗑️  Deleting category...');
      const deleteCategoryResponse = await client.catalog.deleteCategory({
        id: categoryResponse.id,
      });
      
      if (deleteCategoryResponse.code === Code.OK) {
        console.log('✅ Category deleted');
      }
    }

  } catch (error) {
    console.error('❌ Error:', error);
  } finally {
    // Disconnect from NATS
    await client.natsClient.disconnect();
    console.log('\n👋 Disconnected from NATS');
  }
}

main().catch(console.error);