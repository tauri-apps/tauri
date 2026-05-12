export type TestStatus = 'pass' | 'fail' | 'skip';
export type TestCategory = 'auto' | 'side-effect' | 'manual';

export interface TestCase {
  name: string;
  category: TestCategory;
  fn: () => Promise<void>;
}

export interface TestResult {
  name: string;
  category: TestCategory;
  status: TestStatus;
  duration: number;
  error?: string;
}

export interface TestReport {
  timestamp: string;
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  results: TestResult[];
}

const TEST_TIMEOUT_MS = 5000;

function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`Timeout after ${timeoutMs}ms`));
    }, timeoutMs);
    promise
      .then((v) => {
        clearTimeout(timer);
        resolve(v);
      })
      .catch((e) => {
        clearTimeout(timer);
        reject(e);
      });
  });
}

export async function runTests(
  tests: TestCase[],
  onProgress?: (result: TestResult, index: number, total: number) => void
): Promise<TestReport> {
  const results: TestResult[] = [];

  for (let i = 0; i < tests.length; i++) {
    const test = tests[i];

    if (test.category === 'manual') {
      const result: TestResult = {
        name: test.name,
        category: test.category,
        status: 'skip',
        duration: 0,
      };
      results.push(result);
      onProgress?.(result, i, tests.length);
      continue;
    }

    const start = performance.now();
    let result: TestResult;

    try {
      await withTimeout(test.fn(), TEST_TIMEOUT_MS);
      result = {
        name: test.name,
        category: test.category,
        status: 'pass',
        duration: Math.round(performance.now() - start),
      };
    } catch (e: any) {
      result = {
        name: test.name,
        category: test.category,
        status: 'fail',
        duration: Math.round(performance.now() - start),
        error: e?.message || String(e),
      };
    }

    results.push(result);
    onProgress?.(result, i, tests.length);
  }

  return {
    timestamp: new Date().toISOString(),
    total: results.length,
    passed: results.filter((r) => r.status === 'pass').length,
    failed: results.filter((r) => r.status === 'fail').length,
    skipped: results.filter((r) => r.status === 'skip').length,
    results,
  };
}
