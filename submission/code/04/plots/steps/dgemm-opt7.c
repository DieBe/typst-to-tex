const char *dgemm_desc = "Opt7: Fully optimized (2-level blocking, packing, register accum, pragmas, restrict)";

#include <float.h>
#define min(a, b) ((a) < (b) ? (a) : (b))

#define S3 1152
#define S2 64

/* Micro-kernel with packing and register accumulation */
static inline void do_block_micro(int lda, int M, int N, int K,
                                  double *restrict A, double *restrict B,
                                  double *restrict C) {
  /* Pack A (M×K) contiguously */
  double pA[M * K];
  for (int k = 0; k < K; ++k) {
    #pragma GCC ivdep
    for (int i = 0; i < M; ++i)
      pA[i + k * M] = A[i + k * lda];
  }

  /* Pack B (K×N) contiguously */
  double pB[K * N];
  for (int j = 0; j < N; ++j)
    #pragma GCC ivdep
    for (int k = 0; k < K; ++k)
      pB[k + j * K] = B[k + j * lda];

  /* Load C tile into local accumulator */
  double cij[M * N];
  for (int j = 0; j < N; ++j) {
    #pragma GCC ivdep
    for (int i = 0; i < M; ++i)
      cij[i + j * M] = C[i + j * lda];
  }

  /* Compute on packed data */
  for (int j = 0; j < N; ++j) {
    for (int k = 0; k < K; ++k) {
      double b_kj = pB[k + j * K];
      #pragma GCC ivdep
      for (int i = 0; i < M; ++i) {
        cij[i + j * M] += pA[i + k * M] * b_kj;
      }
    }
  }

  /* Write C tile back */
  for (int j = 0; j < N; ++j) {
    #pragma GCC ivdep
    for (int i = 0; i < M; ++i)
      C[i + j * lda] = cij[i + j * M];
  }
}

void square_dgemm(int n, double *restrict A, double *restrict B,
                  double *restrict C) {
  /* Two-level blocking - L3 and L2 cache */
  for (int j3 = 0; j3 < n; j3 += S3) {
    int N3 = min(S3, n - j3);
    for (int k3 = 0; k3 < n; k3 += S3) {
      int K3 = min(S3, n - k3);
      for (int i3 = 0; i3 < n; i3 += S3) {
        int M3 = min(S3, n - i3);

        /* L2 blocking within L3 block */
        for (int j2 = j3; j2 < j3 + N3; j2 += S2) {
          int N2 = min(S2, j3 + N3 - j2);
          for (int k2 = k3; k2 < k3 + K3; k2 += S2) {
            int K2 = min(S2, k3 + K3 - k2);
            for (int i2 = i3; i2 < i3 + M3; i2 += S2) {
              int M2 = min(S2, i3 + M3 - i2);

              do_block_micro(n, M2, N2, K2, A + i2 + k2 * n, B + k2 + j2 * n,
                             C + i2 + j2 * n);
            }
          }
        }
      }
    }
  }
}
