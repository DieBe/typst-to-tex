#!/bin/bash
#SBATCH --job-name=matrixmult
#SBATCH --output=%x-%j.out
#SBATCH --error=%x-%j.err
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --constraint=EPYC_7763
#SBATCH --time=04:00:00

#load some modules & list loaded modules
# GCC
module load stack gcc openblas
# INTEL
#module load stack/2024-06 intel-oneapi-compilers intel-oneapi-mkl
module list

# Print git hash for later rollback
COMMIT_HASH=$(git rev-parse HEAD)
echo "Current commit: $COMMIT_HASH"

# set some environment variables
export OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1
# compile
make clean
make

echo "==== benchmark-naive ======================"
stdbuf -oL ./benchmark-naive | tee timing_naive_dgemm.data
echo
echo "==== benchmark-jki ========================"
stdbuf -oL ./benchmark-jki | tee timing_jki_dgemm.data
echo
echo "==== benchmark-blas ======================="
stdbuf -oL ./benchmark-blas | tee timing_blas_dgemm.data
echo
echo "==== benchmark-blocked-L1 ===================="
stdbuf -oL ./benchmark-blocked-l1 | tee timing_blocked_l1_dgemm.data
echo
echo "==== benchmark-blocked-L2 ===================="
stdbuf -oL ./benchmark-blocked-l2 | tee timing_blocked_l2_dgemm.data
echo
echo "==== benchmark-blocked-L3 ===================="
stdbuf -oL ./benchmark-blocked-l3 | tee timing_blocked_l3_dgemm.data

echo
echo "==== plot results ========================="
gnuplot timing.gp
