const char *dgemm_desc = "Opt2: Single-level blocking (j-k-i order, block size 64)";

#define min(a, b) ((a) < (b) ? (a) : (b))

#define BLOCK_SIZE 64

void square_dgemm(int n, double *A, double *B, double *C) {
  /* Single-level blocking with j-k-i order */
  for (int j = 0; j < n; j += BLOCK_SIZE) {
    int N = min(BLOCK_SIZE, n - j);
    for (int k = 0; k < n; k += BLOCK_SIZE) {
      int K = min(BLOCK_SIZE, n - k);
      for (int i = 0; i < n; i += BLOCK_SIZE) {
        int M = min(BLOCK_SIZE, n - i);
        
        /* Micro-kernel */
        for (int jj = j; jj < j + N; ++jj) {
          for (int kk = k; kk < k + K; ++kk) {
            double b_kj = B[kk + jj * n];
            for (int ii = i; ii < i + M; ++ii) {
              C[ii + jj * n] += A[ii + kk * n] * b_kj;
            }
          }
        }
      }
    }
  }
}
