#!/bin/bash
#SBATCH --job-name=matmul_opt_l2
#SBATCH --output=%x-%j.out
#SBATCH --error=%x-%j.err
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --constraint=EPYC_7763
#SBATCH --time=12:00:00

# AI has been used in the writing of this file.

# Load modules
module load stack gcc openblas
module list

# Print git hash
COMMIT_HASH=$(git rev-parse HEAD)
echo "Current commit: $COMMIT_HASH"

# Set environment variables
export OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1

# Define the optimization flags
OPT_FLAGS="-O3 -march=znver3 -ffast-math -funroll-loops -fno-trapping-math -fno-stack-protector -fopenmp-simd -fprefetch-loop-arrays -fomit-frame-pointer -falign-loops=64"
BASE_CFLAGS="-Wall -std=gnu99 ${OPT_FLAGS}"

# Fixed S3 from L3 optimization
FIXED_S3=1152

# L2 cache parameters
L2_SIZE=$((512 * 1024))  # 512 KiB
BLOCKS_IN_CACHE=3
MIN_BLOCK=16
STEP=8

# Calculate max S2 (rounded down to multiple of 8)
calc_max_s2() {
    local cache_bytes=$L2_SIZE
    local raw_size=$(awk "BEGIN {print int(sqrt($cache_bytes / ($BLOCKS_IN_CACHE * 8)))}")
    echo $((raw_size / 8 * 8))
}

MAX_S2=$(calc_max_s2)

echo ""
echo "###############################################"
echo "# L2 Cache Optimization"
echo "###############################################"
echo "L2 cache size: $((L2_SIZE/1024))KB"
echo "Fixed S3: $FIXED_S3"
echo "Max S2: $MAX_S2 (rounded to multiple of 8)"
echo "Min S2: $MIN_BLOCK"
echo "Step: $STEP"
echo "###############################################"
echo ""

best_s2=0
best_perf=0

total_iters=$(( (MAX_S2 - MIN_BLOCK) / STEP + 1 ))
current_iter=0
start_time=$(date +%s)

for s2 in $(seq $MAX_S2 -$STEP $MIN_BLOCK); do
    current_iter=$((current_iter + 1))

    # Build complete CFLAGS
    complete_cflags="${BASE_CFLAGS} -DS3=${FIXED_S3} -DS2=${s2}"

    # Calculate ETA
    current_time=$(date +%s)
    elapsed=$((current_time - start_time))
    eta="N/A"
    if [ $current_iter -gt 1 ]; then
        avg_time_per_iter=$((elapsed / (current_iter - 1)))
        remaining_iters=$((total_iters - current_iter))
        eta_seconds=$((avg_time_per_iter * remaining_iters))
        eta="${eta_seconds}s (~$((eta_seconds/60))m)"
    fi

    echo "[$current_iter/$total_iters] Testing S2=$s2 (elapsed: ${elapsed}s, ETA: ${eta})"
    echo "  Compiling benchmark-blocked-l2 with S3=$FIXED_S3, S2=$s2"

    # Clean and compile
    make clean > /dev/null 2>&1
    compile_output=$(make benchmark-blocked-l2 CFLAGS="${complete_cflags}" LDLIBS="-lopenblas -lm" 2>&1)
    compile_status=$?

    if [ $compile_status -ne 0 ] || [ ! -f "./benchmark-blocked-l2" ]; then
        echo "  ERROR: Compilation failed!"
        echo "  Compiler output:"
        echo "$compile_output"
        continue
    fi

    echo "  Running benchmark..."

    # Run and extract average percentage
    perf=$(stdbuf -oL ./benchmark-blocked-l2 2>&1 | grep -v '^#' | awk '{sum+=$6; count++} END {if(count>0) print sum/count; else print 0}')

    echo "  Result: ${perf}%"

    # Compare performance
    is_better=$(awk "BEGIN {print ($perf > $best_perf) ? 1 : 0}")

    if [ "$is_better" -eq 1 ]; then
        best_perf=$perf
        best_s2=$s2
        echo "  >>> NEW BEST: S2=${best_s2} with ${best_perf}% <<<"
    else
        echo "  Current best: S2=${best_s2} with ${best_perf}%"
    fi
    echo ""
done

echo ""
echo "###############################################"
echo "# L2 OPTIMIZATION COMPLETE"
echo "###############################################"
echo "Best Performance: ${best_perf}%"
echo "###############################################"

