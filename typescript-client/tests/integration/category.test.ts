import { Code } from '../../src';
import { 
  getTestClient, 
  disconnectTestClient, 
  generateTestId,
  TestClient 
} from '../utils/test-client';

describe('Category Integration Tests', () => {
  let client: TestClient;
  const createdCategoryIds: string[] = [];

  beforeAll(async () => {
    client = await getTestClient();
  });

  afterAll(async () => {
    // Clean up categories in reverse order (children before parents)
    for (const id of createdCategoryIds.reverse()) {
      try {
        await client.catalog.deleteCategory({ id });
      } catch (error) {
        // Ignore errors during cleanup
      }
    }
    
    await disconnectTestClient();
  });

  describe('Category CRUD Operations', () => {
    test('should create a root category', async () => {
      const slug = generateTestId('category');
      
      const response = await client.catalog.createCategory({
        name: 'Test Root Category',
        slug,
        shortDescription: 'A test root category',
        fullDescription: 'This is a test root category for integration testing',
        parentId: undefined,
        displayOrder: 1,
        seo: {
          metaTitle: 'Test Category SEO Title',
          metaDescription: 'Test category for integration testing',
          keywords: ['test', 'category'],
        },
        isActive: true,
        parentSlug: undefined,
      });

      expect(response.status?.code).toBe(Code.OK);
      expect(response.category).toBeValidCategory();
      expect(response.category?.name).toBe('Test Root Category');
      expect(response.category?.slug).toBe(slug);
      expect(response.category?.parentId).toBeUndefined(); // Root categories have no parent
      expect(response.category?.level).toBe(0);
      
      if (response.category?.id) {
        createdCategoryIds.push(response.category.id);
      }
    });

    test('should create a child category', async () => {
      // First create a parent category
      const parentSlug = generateTestId('parent');
      const parentResponse = await client.catalog.createCategory({
        name: 'Parent Category',
        slug: parentSlug,
        shortDescription: 'Parent category',
        fullDescription: undefined,
        parentId: undefined,
        displayOrder: 1,
        seo: undefined,
        isActive: true,
        parentSlug: undefined,
      });

      expect(parentResponse.status?.code).toBe(Code.OK);
      const parentId = parentResponse.category?.id;
      
      if (parentId) {
        createdCategoryIds.push(parentId);
        
        // Create child category
        const childSlug = generateTestId('child');
        const childResponse = await client.catalog.createCategory({
          name: 'Child Category',
          slug: childSlug,
          shortDescription: 'Child category',
          fullDescription: undefined,
          parentId,
          displayOrder: 1,
          seo: undefined,
          isActive: true,
          parentSlug: undefined,
        });

        expect(childResponse.status?.code).toBe(Code.OK);
        expect(childResponse.category).toBeValidCategory();
        expect(childResponse.category?.name).toBe('Child Category');
        expect(childResponse.category?.parentId).toBe(parentId);
        expect(childResponse.category?.level).toBe(1);
        expect(childResponse.category?.ancestors).toContain(parentId);
        
        if (childResponse.category?.id) {
          createdCategoryIds.push(childResponse.category.id);
        }
      }
    });

    test('should get a category by ID', async () => {
      // Create a category
      const createResponse = await client.catalog.createCategory({
        name: 'Category to Get',
        slug: generateTestId('category'),
        shortDescription: 'Test category',
        fullDescription: undefined,
        parentId: undefined,
        displayOrder: 1,
        seo: undefined,
        isActive: true,
        parentSlug: undefined,
      });

      expect(createResponse.status?.code).toBe(Code.OK);
      const categoryId = createResponse.category?.id;
      
      if (categoryId) {
        createdCategoryIds.push(categoryId);
        
        // Get the category
        const getResponse = await client.catalog.getCategory({ id: categoryId });
        
        expect(getResponse.status?.code).toBe(Code.OK);
        expect(getResponse.category).toBeValidCategory();
        expect(getResponse.category?.id).toBe(categoryId);
        expect(getResponse.category?.name).toBe('Category to Get');
      }
    });

    test('should get a category by slug', async () => {
      const slug = generateTestId('category-slug');
      
      // Create a category
      const createResponse = await client.catalog.createCategory({
        name: 'Category with Slug',
        slug,
        shortDescription: 'Test category',
        fullDescription: undefined,
        parentId: undefined,
        displayOrder: 1,
        seo: undefined,
        isActive: true,
        parentSlug: undefined,
      });

      expect(createResponse.status?.code).toBe(Code.OK);
      
      if (createResponse.category?.id) {
        createdCategoryIds.push(createResponse.category.id);
        
        // Get by slug
        const getResponse = await client.catalog.getCategoryBySlug({ slug });
        
        expect(getResponse.status?.code).toBe(Code.OK);
        expect(getResponse.category).toBeValidCategory();
        expect(getResponse.category?.slug).toBe(slug);
        expect(getResponse.category?.name).toBe('Category with Slug');
      }
    });

    test('should update a category', async () => {
      // Create a category
      const createResponse = await client.catalog.createCategory({
        name: 'Category to Update',
        slug: generateTestId('category'),
        shortDescription: 'Original description',
        fullDescription: undefined,
        parentId: undefined,
        displayOrder: 1,
        seo: undefined,
        isActive: true,
        parentSlug: undefined,
      });

      expect(createResponse.status?.code).toBe(Code.OK);
      const categoryId = createResponse.category?.id;
      
      if (categoryId) {
        createdCategoryIds.push(categoryId);
        
        // Update the category
        const updateResponse = await client.catalog.updateCategory({
          id: categoryId,
          name: 'Updated Category Name',
          shortDescription: 'Updated description',
          fullDescription: 'Now with full description',
          displayOrder: 2,
          seo: {
            metaTitle: 'Updated SEO Title',
            metaDescription: 'Updated SEO description',
            keywords: ['updated', 'keywords'],
          },
          isActive: false,
        });
        
        expect(updateResponse.status?.code).toBe(Code.OK);
        expect(updateResponse.category?.name).toBe('Updated Category Name');
        expect(updateResponse.category?.shortDescription).toBe('Updated description');
        expect(updateResponse.category?.fullDescription).toBe('Now with full description');
        expect(updateResponse.category?.displayOrder).toBe(2);
        expect(updateResponse.category?.isActive).toBe(false);
      }
    });

    test('should delete a category', async () => {
      // Create a category
      const createResponse = await client.catalog.createCategory({
        name: 'Category to Delete',
        slug: generateTestId('category'),
        shortDescription: 'Will be deleted',
        fullDescription: undefined,
        parentId: undefined,
        displayOrder: 1,
        seo: undefined,
        isActive: true,
        parentSlug: undefined,
      });

      expect(createResponse.status?.code).toBe(Code.OK);
      const categoryId = createResponse.category?.id;
      
      if (categoryId) {
        // Delete the category
        const deleteResponse = await client.catalog.deleteCategory({ id: categoryId });
        expect(deleteResponse.status?.code).toBe(Code.OK);
        
        // Verify it's deleted
        const getResponse = await client.catalog.getCategory({ id: categoryId });
        expect(getResponse.status?.code).toBe(Code.NOT_FOUND);
        
        // Remove from cleanup list
        const index = createdCategoryIds.indexOf(categoryId);
        if (index > -1) {
          createdCategoryIds.splice(index, 1);
        }
      }
    });
  });

  describe('Category Tree', () => {
    test('should get category tree', async () => {
      // Create a hierarchy
      const rootSlug = generateTestId('root');
      const rootResponse = await client.catalog.createCategory({
        name: 'Tree Root',
        slug: rootSlug,
        shortDescription: 'Root of tree',
        fullDescription: undefined,
        parentId: undefined,
        displayOrder: 1,
        seo: undefined,
        isActive: true,
        parentSlug: undefined,
      });

      expect(rootResponse.status?.code).toBe(Code.OK);
      
      if (rootResponse.category?.id) {
        createdCategoryIds.push(rootResponse.category.id);
        
        // Create children
        for (let i = 1; i <= 3; i++) {
          const childResponse = await client.catalog.createCategory({
            name: `Tree Child ${i}`,
            slug: generateTestId(`child-${i}`),
            shortDescription: `Child ${i}`,
            fullDescription: undefined,
            parentId: rootResponse.category?.id,
            displayOrder: i,
            seo: undefined,
            isActive: true,
            parentSlug: undefined,
          });
          
          if (childResponse.category?.id) {
            createdCategoryIds.push(childResponse.category.id);
          }
        }
        
        // Get the tree
        const treeResponse = await client.catalog.getCategoryTree({
          includeInactive: false,
          maxDepth: undefined,
        });
        
        expect(treeResponse.status?.code).toBe(Code.OK);
        expect(treeResponse.tree).toBeDefined();
        expect(Array.isArray(treeResponse.tree)).toBe(true);
        
        // Find our root in the tree
        const ourRoot = treeResponse.tree?.find(
          node => node.id === rootResponse.category?.id
        );
        expect(ourRoot).toBeDefined();
        expect(ourRoot?.children).toHaveLength(3);
      }
    });

    test('should respect maxDepth in category tree', async () => {
      // Create a deep hierarchy
      let parentId: string | undefined;
      const categoryIds: string[] = [];
      
      for (let level = 0; level < 4; level++) {
        const response = await client.catalog.createCategory({
          name: `Level ${level} Category`,
          slug: generateTestId(`level-${level}`),
          shortDescription: `Level ${level}`,
          fullDescription: undefined,
          parentId,
          displayOrder: 1,
          seo: undefined,
          isActive: true,
          parentSlug: undefined,
        });
        
        if (response.category?.id) {
          categoryIds.push(response.category.id);
          createdCategoryIds.push(response.category.id);
          parentId = response.category.id;
        }
      }
      
      // Get tree with maxDepth = 3 to include levels 0, 1, and 2
      const treeResponse = await client.catalog.getCategoryTree({
        includeInactive: false,
        maxDepth: 3,
      });
      
      expect(treeResponse.status?.code).toBe(Code.OK);
      
      // Find our root category
      const ourRoot = treeResponse.tree?.find(
        node => node.id === categoryIds[0]
      );
      
      if (ourRoot) {
        // With maxDepth = 3, we expect to get depth 0, 1, and 2 (3 levels total)
        // The 4th level (depth 3) should be cut off
        expect(ourRoot.children).toHaveLength(1);
        expect(ourRoot.children[0].children).toHaveLength(1);
        expect(ourRoot.children[0].children[0].children).toHaveLength(0);
      }
    });
  });

  describe('Category Validation', () => {
    test('should reject category with duplicate slug', async () => {
      const slug = generateTestId('duplicate-slug');
      
      // Create first category
      const firstResponse = await client.catalog.createCategory({
        name: 'First Category',
        slug,
        shortDescription: 'First',
        fullDescription: undefined,
        parentId: undefined,
        displayOrder: 1,
        seo: undefined,
        isActive: true,
        parentSlug: undefined,
      });
      
      expect(firstResponse.status?.code).toBe(Code.OK);
      
      if (firstResponse.category?.id) {
        createdCategoryIds.push(firstResponse.category.id);
        
        // Try to create second with same slug
        const secondResponse = await client.catalog.createCategory({
          name: 'Second Category',
          slug, // Same slug
          shortDescription: 'Second',
          fullDescription: undefined,
          parentId: undefined,
          displayOrder: 1,
          seo: undefined,
          isActive: true,
          parentSlug: undefined,
        });
        
        expect(secondResponse.status?.code).toBe(Code.ALREADY_EXISTS);
      }
    });

    test('should reject category with invalid parent', async () => {
      const response = await client.catalog.createCategory({
        name: 'Orphan Category',
        slug: generateTestId('orphan'),
        shortDescription: 'No parent',
        fullDescription: undefined,
        parentId: '507f1f77bcf86cd799439011', // Valid ObjectId but doesn't exist
        displayOrder: 1,
        seo: undefined,
        isActive: true,
        parentSlug: undefined,
      });
      
      expect(response.status?.code).toBe(Code.NOT_FOUND);
    });
  });
});