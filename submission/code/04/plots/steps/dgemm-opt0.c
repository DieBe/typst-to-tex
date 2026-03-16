const char* dgemm_desc = "Opt0: Naive, three-loop dgemm (i-j-k order)";

void square_dgemm(int n, double* A, double* B, double* C) {
  for (int i = 0; i < n; ++i) {
    for (int j = 0; j < n; ++j) {
      for(int k = 0; k < n; ++k) {
        C[i+j*n] += A[i+k*n] * B[k+j*n];
      }
    }
  }
}
