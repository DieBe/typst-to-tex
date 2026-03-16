#!/bin/bash
#SBATCH --job-name=refine_flags
#SBATCH --output=%x-%j.out
#SBATCH --error=%x-%j.err
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --constraint=EPYC_7763
#SBATCH --time=01:30:00

# AI has been used in the writing of this file.

module load stack gcc openblas
module list

export OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1

COMMIT_HASH=$(git rev-parse HEAD)
echo "Current commit: $COMMIT_HASH"
echo "=========================================="
echo "Refining best flag combination"
echo "Base: -O3 -march=znver3 -fno-trapping-math -ffast-math -fopenmp-simd"
echo "=========================================="
echo ""

mkdir -p refined_results
SUMMARY_FILE="refined_results/summary.txt"
> $SUMMARY_FILE

run_test() {
    local test_name="$1"
    local cflags="$2"
    local output_file="refined_results/${test_name}.data"

    echo "=========================================="
    echo "Test: $test_name"
    echo "CFLAGS: $cflags"
    echo "=========================================="

    # Clean previous builds
    make clean > /dev/null 2>&1

    # Compile
    gcc -c -Wall -std=gnu99 $cflags benchmark.c -o benchmark.o
    gcc -c -Wall -std=gnu99 $cflags -DL2_ONLY -o dgemm-blocked-l2.o dgemm-blocked.c
    gcc -o benchmark-blocked-l2 benchmark.o dgemm-blocked-l2.o -lopenblas -lm

    if [ $? -ne 0 ]; then
        echo "COMPILATION FAILED for $test_name"
        echo "$test_name|FAILED|FAILED" >> $SUMMARY_FILE
        echo ""
        return 1
    fi

    # Run benchmark
    echo "Running benchmark..."
    stdbuf -oL ./benchmark-blocked-l2 | tee "$output_file"

    # Extract best Gflop/s
    best_gflops=$(awk '$1 == "Size:" && $3 == "Gflop/s:" {print $4}' "$output_file" | sort -n | tail -1)

    # Extract average percentage
    avg_percentage=$(grep "^# Average percentage" "$output_file" | grep -oE "[0-9]+\.[0-9]+")

    # Save to summary
    echo "$test_name|$best_gflops|$avg_percentage" >> $SUMMARY_FILE

    echo ""
    echo "Result: Best=$best_gflops Gflop/s, Average=$avg_percentage%"
    echo ""
}

# Define base flags
BASE="-O3 -march=znver3 -fno-trapping-math -ffast-math -fopenmp-simd"

# Test 1: Base configuration (for reference)
run_test "00_base" "$BASE"

# Test 2-10: Base + each additional flag individually

run_test "01_base_plus_funroll-loops" "$BASE -funroll-loops"

run_test "02_base_plus_fno-stack-protector" "$BASE -fno-stack-protector"

run_test "03_base_plus_fprefetch-loop-arrays" "$BASE -fprefetch-loop-arrays"

run_test "04_base_plus_fomit-frame-pointer" "$BASE -fomit-frame-pointer"

run_test "05_base_plus_falign-loops-64" "$BASE -falign-loops=64"


# Clean up
make clean > /dev/null 2>&1

echo ""
echo "=========================================="
echo "ALL TESTS COMPLETED"
echo "=========================================="
echo ""
echo "RESULTS SORTED BY AVERAGE PERCENTAGE:"
echo "=========================================="
printf "%-40s | %10s | %10s\n" "Test Name" "Best GF/s" "Avg %"
echo "--------------------------------------------------------------------------------"

sort -t'|' -k3 -rn "$SUMMARY_FILE" | while IFS='|' read -r name gflops percentage; do
    printf "%-40s | %10s | %10s\n" "$name" "$gflops" "$percentage"
done

echo ""
echo "RESULTS SORTED BY BEST GFLOP/S:"
echo "=========================================="
printf "%-40s | %10s | %10s\n" "Test Name" "Best GF/s" "Avg %"
echo "--------------------------------------------------------------------------------"

sort -t'|' -k2 -rn "$SUMMARY_FILE" | while IFS='|' read -r name gflops percentage; do
    printf "%-40s | %10s | %10s\n" "$name" "$gflops" "$percentage"
done

echo "=========================================="
echo ""
echo "Detailed results saved in: refined_results/"
echo "Summary saved in: $SUMMARY_FILE"
