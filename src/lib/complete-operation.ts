// Async settlement belongs outside React's compiler lowering. Owners supply all
// lifecycle/identity guards; this helper makes no retry or presentation decisions.
export async function completeOperation<T>(
  operation: () => Promise<T>,
  failed: ((error: unknown) => T | Promise<T>) | undefined,
  settled: () => void,
): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    if (failed) return await failed(error);
    throw error;
  } finally {
    settled();
  }
}
