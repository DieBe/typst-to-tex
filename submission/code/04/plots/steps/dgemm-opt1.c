const char *dgemm_desc = "Opt1: No blocking, j-k-i loop order only";

void square_dgemm(int n, double *A, double *B, double *C) {
  /* j-k-i loop order without blocking */
  for (int j = 0; j < n; ++j) {
    for (int k = 0; k < n; ++k) {
      double b_kj = B[k + j * n];
      for (int i = 0; i < n; ++i) {
        C[i + j * n] += A[i + k * n] * b_kj;
      }
    }
  }
}
