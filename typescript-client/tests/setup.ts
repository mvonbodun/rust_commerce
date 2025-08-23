import { NatsClient } from '../src/nats-client';

// Extend Jest matchers if needed
declare global {
  namespace jest {
    interface Matchers<R> {
      toBeValidProduct(): R;
      toBeValidCategory(): R;
    }
  }
}

// Add custom matchers
expect.extend({
  toBeValidProduct(received: any) {
    const pass = 
      received &&
      typeof received.id === 'string' &&
      typeof received.name === 'string' &&
      typeof received.productRef === 'string';
    
    if (pass) {
      return {
        message: () => `expected ${received} not to be a valid product`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${received} to be a valid product with id, name, and productRef`,
        pass: false,
      };
    }
  },
  
  toBeValidCategory(received: any) {
    const pass = 
      received &&
      typeof received.id === 'string' &&
      typeof received.name === 'string' &&
      typeof received.slug === 'string';
    
    if (pass) {
      return {
        message: () => `expected ${received} not to be a valid category`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${received} to be a valid category with id, name, and slug`,
        pass: false,
      };
    }
  },
});

// Global test timeout
jest.setTimeout(30000);

// Cleanup function for tests
export async function cleanupTestData(_client: NatsClient): Promise<void> {
  // This would ideally call a test cleanup endpoint
  // For now, tests should clean up their own data
  // The _client parameter is prefixed with _ to indicate it's intentionally unused
}