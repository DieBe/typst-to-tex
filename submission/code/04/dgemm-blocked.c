const char *dgemm_desc = "Blocked dgemm.";

#include <float.h>
#define min(a, b) ((a) < (b) ? (a) : (b))

/* Define default block sizes if not provided via compiler flags */
#ifndef S3
#define S3 1152
#endif

#ifndef S2
#define S2 64
#endif

#ifndef S1
#define S1 32
#endif

/* Definde default amount of blocks to use */
#ifndef L3_ONLY
#ifndef L2_ONLY
#ifndef L1_ONLY
#define L2_ONLY
#endif
#endif
#endif

/* Single micro-kernel for smallest blocks.
 * Packs A and B into contiguous local buffers to eliminate strided access,
 * and accumulates C in a local tile to reduce writes to main memory. */
static inline void do_block_micro(int lda, int M, int N, int K,
                                  double *restrict A, double *restrict B,
                                  double *restrict C) {
  /* Pack A (M×K) contiguously with leading dim M */
  double pA[M * K];
  for (int k = 0; k < K; ++k) {
    #pragma GCC ivdep
    for (int i = 0; i < M; ++i)
      pA[i + k * M] = A[i + k * lda];
  }

  /* Pack B (K×N) contiguously with leading dim K */
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

#ifdef L3_ONLY
  /* Single level blocking - L3 cache */
  for (int j = 0; j < n; j += S3) {
    int N = min(S3, n - j);
    for (int k = 0; k < n; k += S3) {
      int K = min(S3, n - k);
      for (int i = 0; i < n; i += S3) {
        int M = min(S3, n - i);
        do_block_micro(n, M, N, K, A + i + k * n, B + k + j * n, C + i + j * n);
      }
    }
  }
#endif

#ifdef L2_ONLY
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
#endif

#ifdef L1_ONLY
  /* Three-level blocking - L3, L2, and L1 cache */
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

              /* L1 blocking within L2 block */
              for (int j1 = j2; j1 < j2 + N2; j1 += S1) {
                int N1 = min(S1, j2 + N2 - j1);
                for (int k1 = k2; k1 < k2 + K2; k1 += S1) {
                  int K1 = min(S1, k2 + K2 - k1);
                  for (int i1 = i2; i1 < i2 + M2; i1 += S1) {
                    int M1 = min(S1, i2 + M2 - i1);

                    do_block_micro(n, M1, N1, K1, A + i1 + k1 * n,
                                   B + k1 + j1 * n, C + i1 + j1 * n);
                  }
                }
              }
            }
          }
        }
      }
    }
  }
#endif
}
