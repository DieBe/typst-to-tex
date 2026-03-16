#!/bin/bash
#SBATCH --job-name=matmul_opt_l1
#SBATCH --output=%x-%j.out
#SBATCH --error=%x-%j.err
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --constraint=EPYC_7763
#SBATCH --time=24:00:00

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

# Cache parameters
L2_SIZE=$((512 * 1024))  # 512 KiB
L1_SIZE=$((32 * 1024))   # 32 KiB
BLOCKS_IN_CACHE=3
MIN_BLOCK=8
STEP=8

# Calculate max sizes (rounded down to multiple of 8)
calc_max_size() {
    local cache_bytes=$1
    local raw_size=$(awk "BEGIN {print int(sqrt($cache_bytes / ($BLOCKS_IN_CACHE * 8)))}")
    echo $((raw_size / 8 * 8))
}

MAX_S2=$(calc_max_size $L2_SIZE)
MAX_S1=$(calc_max_size $L1_SIZE)

echo ""
echo "###############################################"
echo "# L1 Cache Optimization (Grid Search)"
echo "###############################################"
echo "Fixed S3: $FIXED_S3"
echo "L2 range: $MIN_BLOCK to $MAX_S2 (step $STEP)"
echo "L1 range: $MIN_BLOCK to $MAX_S1 (step $STEP)"
echo "###############################################"
echo ""

# Calculate total combinations
s2_count=$(( (MAX_S2 - MIN_BLOCK) / STEP + 1 ))
s1_count=$(( (MAX_S1 - MIN_BLOCK) / STEP + 1 ))
total_combinations=$((s2_count * s1_count))

echo "Total combinations to test: $total_combinations"
echo ""

best_s2=0
best_s1=0
best_perf=0

current_combo=0
start_time=$(date +%s)

for s2 in $(seq $MAX_S2 -$STEP $MIN_BLOCK); do
    for s1 in $(seq $MAX_S1 -$STEP $MIN_BLOCK); do
        current_combo=$((current_combo + 1))

        # Build complete CFLAGS
        complete_cflags="${BASE_CFLAGS} -DS3=${FIXED_S3} -DS2=${s2} -DS1=${s1}"

        # Calculate ETA
        current_time=$(date +%s)
        elapsed=$((current_time - start_time))
        eta="N/A"
        if [ $current_combo -gt 1 ]; then
            avg_time_per_combo=$((elapsed / (current_combo - 1)))
            remaining_combos=$((total_combinations - current_combo))
            eta_seconds=$((avg_time_per_combo * remaining_combos))
            eta="${eta_seconds}s (~$((eta_seconds/60))m)"
        fi

        echo "[$current_combo/$total_combinations] Testing S2=$s2, S1=$s1 (elapsed: ${elapsed}s, ETA: ${eta})"

        # Clean and compile
        make clean > /dev/null 2>&1
        compile_output=$(make benchmark-blocked-l1 CFLAGS="${complete_cflags}" LDLIBS="-lopenblas -lm" 2>&1)
        compile_status=$?

        if [ $compile_status -ne 0 ] || [ ! -f "./benchmark-blocked-l1" ]; then
            echo "  ERROR: Compilation failed!"
            continue
        fi

        # Run and extract average percentage
        perf=$(stdbuf -oL ./benchmark-blocked-l1 2>&1 | grep -v '^#' | awk '{sum+=$6; count++} END {if(count>0) print sum/count; else print 0}')

        echo "  Result: ${perf}%"

        # Compare performance
        is_better=$(awk "BEGIN {print ($perf > $best_perf) ? 1 : 0}")

        if [ "$is_better" -eq 1 ]; then
            best_perf=$perf
            best_s2=$s2
            best_s1=$s1
            echo "  >>> NEW BEST: S2=${best_s2}, S1=${best_s1} with ${best_perf}% <<<"
        fi
        echo ""
    done
done

echo ""
echo "###############################################"
echo "# L1 OPTIMIZATION COMPLETE"
echo "###############################################"
echo "Best S3: $FIXED_S3"
echo "Best S2: $best_s2"
echo "Best S1: $best_s1"
echo "Best Performance: ${best_perf}%"
echo "###############################################"
echo ""

# Run final benchmark with optimal settings
echo "Running final benchmark with optimal settings..."
make clean
make benchmark-blocked-l1 CFLAGS="${BASE_CFLAGS} -DS3=${FIXED_S3} -DS2=${best_s2} -DS1=${best_s1}" LDLIBS="-lopenblas -lm"
echo ""
echo "==== Final Benchmark ===="
stdbuf -oL ./benchmark-blocked-l1 | tee timing_blocked_l1_optimal.data
