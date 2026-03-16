#!/bin/bash
#SBATCH --job-name=matmul_opt_study
#SBATCH --output=%x-%j.out
#SBATCH --error=%x-%j.err
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --constraint=EPYC_7763
#SBATCH --time=08:00:00

# Load modules
module load stack gcc openblas
module list

# Print git hash for reproducibility
COMMIT_HASH=$(git rev-parse HEAD 2>/dev/null || echo "no git repo")
echo "Current commit: $COMMIT_HASH"

# Set environment variables
export OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1

# Compile all versions
make clean
make

echo "========================================"
echo "Optimization Steps"
echo "========================================"
echo ""

# Run each optimization level
echo "==== Opt0: Naive (i-j-k) ======================"
stdbuf -oL ./benchmark-opt0 | tee timing_opt0_dgemm.data
echo ""

echo "==== Opt1: j-k-i loop order ==================="
stdbuf -oL ./benchmark-opt1 | tee timing_opt1_dgemm.data
echo ""

echo "==== Opt2: Single-level blocking =============="
stdbuf -oL ./benchmark-opt2 | tee timing_opt2_dgemm.data
echo ""

echo "==== Opt3: + restrict ========================="
stdbuf -oL ./benchmark-opt3 | tee timing_opt3_dgemm.data
echo ""

echo "==== Opt4: + ivdep pragmas ===================="
stdbuf -oL ./benchmark-opt4 | tee timing_opt4_dgemm.data
echo ""

echo "==== Opt5: + register accumulation ============"
stdbuf -oL ./benchmark-opt5 | tee timing_opt5_dgemm.data
echo ""

echo "==== Opt6: + data packing ====================="
stdbuf -oL ./benchmark-opt6 | tee timing_opt6_dgemm.data
echo ""

echo "==== Opt7: + multi-level blocking ============="
stdbuf -oL ./benchmark-opt7 | tee timing_opt7_dgemm.data
echo ""

echo "==== BLAS reference ==========================="
stdbuf -oL ./benchmark-blas | tee timing_blas_dgemm.data
echo ""

echo "========================================"
echo "All benchmarks completed!"
echo "========================================"
