export {};

declare global {
  namespace jest {
    interface Matchers<R> {
      toBeValidProduct(): R;
      toBeValidCategory(): R;
    }
  }
}