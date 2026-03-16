# Some possible configs ########################################################

# Euler GCC config
# (module load stack gcc openblas)
CC = gcc                         # compiler (& linker)
OPT = -O3 -march=znver3 -ffast-math -funroll-loops #-O3 -march=znver3 -ffast-math -funroll-loops -fno-trapping-math -fno-stack-protector -fopenmp-simd -fprefetch-loop-arrays -fomit-frame-pointer -falign-loops=64                    # optimization flags
CFLAGS = -Wall -std=gnu99 $(OPT) # standard compiler flags
LDLIBS = -lopenblas -lm         # linker options

# Euler INTEL config
# (module load stack/2024-06 intel-oneapi-compilers intel-oneapi-mkl)
# module load stack intel-oneapi-compilers intel-mkl
#CC = icx                          # Intel oneAPI C compiler
#OPT = -O3 -march=native -ffast-math -funroll-loops -fno-trapping-math \
      -fomit-frame-pointer -qopenmp-simd -qopt-prefetch=4 \
      -falign-loops=64 -fp-model fast=2
#CFLAGS = -Wall -std=gnu99 $(OPT)
#LDLIBS = -qmkl -lm               # Use Intel MKL instead of OpenBLAS

################################################################################

targets = benchmark-naive benchmark-jki benchmark-blocked benchmark-blocked-l1 benchmark-blocked-l2 benchmark-blocked-l3 benchmark-blas
objects = benchmark.o dgemm-naive.o dgemm-jki.o dgemm-blocked.o dgemm-blocked-l1.o dgemm-blocked-l2.o dgemm-blocked-l3.o dgemm-blas.o
results = timing_naive_dgemm.data   \
          timing_jki_dgemm.data     \
          timing_blas_dgemm.data    \
          timing_blocked_dgemm.data \
          timing_blocked_l1_dgemm.data \
          timing_blocked_l2_dgemm.data \
          timing_blocked_l3_dgemm.data \
          timing.pdf

.PHONY: default
default: all

.PHONY: all
all: $(targets)

benchmark-naive: benchmark.o dgemm-naive.o
	$(CC) -o $@ $^ $(LDLIBS)
benchmark-jki: benchmark.o dgemm-jki.o
	$(CC) -o $@ $^ $(LDLIBS)
benchmark-blocked: benchmark.o dgemm-blocked-l1.o
	$(CC) -o $@ $^ $(LDLIBS)

benchmark-blocked-l1: benchmark.o dgemm-blocked-l1.o
	$(CC) -o $@ $^ $(LDLIBS)
benchmark-blocked-l2: benchmark.o dgemm-blocked-l2.o
	$(CC) -o $@ $^ $(LDLIBS)
benchmark-blocked-l3: benchmark.o dgemm-blocked-l3.o
	$(CC) -o $@ $^ $(LDLIBS)

dgemm-blocked-l1.o: dgemm-blocked.c
	$(CC) -c $(CFLAGS) -DL1_ONLY -o $@ $<
dgemm-blocked-l2.o: dgemm-blocked.c
	$(CC) -c $(CFLAGS) -DL2_ONLY -o $@ $<
dgemm-blocked-l3.o: dgemm-blocked.c
	$(CC) -c $(CFLAGS) -DL3_ONLY -o $@ $<
benchmark-blas: benchmark.o dgemm-blas.o
	$(CC) -o $@ $^ $(LDLIBS)


%.o: %.c
	$(CC) -c $(CFLAGS) $<

.PHONY: clean
clean:
	$(RM) $(targets) $(objects)

.PHONY: cleanall
cleanall: clean
	$(RM) $(results)
