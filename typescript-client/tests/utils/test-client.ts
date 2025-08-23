import { createClient, NatsClient, CatalogClient, ClientConfig } from '../../src';

export interface TestClient {
  natsClient: NatsClient;
  catalog: CatalogClient;
}

let testClient: TestClient | null = null;

/**
 * Get or create a test client singleton
 * Reuses the same connection for all tests in a suite
 */
export async function getTestClient(): Promise<TestClient> {
  if (!testClient) {
    const config: ClientConfig = {
      servers: process.env.NATS_TEST_URL || 'nats://localhost:4223',
      maxReconnectAttempts: 5,
      reconnectTimeWait: 1000,
    };
    
    testClient = await createClient(config);
    
    // Wait a bit for services to be ready
    await new Promise(resolve => setTimeout(resolve, 2000));
  }
  
  return testClient;
}

/**
 * Disconnect the test client
 * Call this in afterAll() hooks
 */
export async function disconnectTestClient(): Promise<void> {
  if (testClient) {
    await testClient.natsClient.disconnect();
    testClient = null;
  }
}

/**
 * Generate unique test data identifiers
 */
export function generateTestId(prefix: string): string {
  const timestamp = Date.now();
  const random = Math.floor(Math.random() * 1000);
  return `${prefix}_test_${timestamp}_${random}`;
}

/**
 * Wait for a condition to be true
 */
export async function waitFor(
  condition: () => Promise<boolean>,
  timeout: number = 5000,
  interval: number = 100
): Promise<void> {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    if (await condition()) {
      return;
    }
    await new Promise(resolve => setTimeout(resolve, interval));
  }
  
  throw new Error(`Timeout waiting for condition after ${timeout}ms`);
}

/**
 * Retry a function multiple times
 */
export async function retry<T>(
  fn: () => Promise<T>,
  retries: number = 3,
  delay: number = 1000
): Promise<T> {
  let lastError: any;
  
  for (let i = 0; i < retries; i++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;
      if (i < retries - 1) {
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }
  
  throw lastError;
}