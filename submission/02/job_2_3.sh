#!/bin/bash
#SBATCH --job-name=2_3_7763      # Job name    (default: sbatch)
#SBATCH --output=%x-%j.out       # Output file (default: slurm-%j.out)
#SBATCH --error=%x-%j.err        # Error file  (default: slurm-%j.out)
#SBATCH --ntasks=1               # Number of tasks
#SBATCH --cpus-per-task=1        # Number of CPUs per task
#SBATCH --mem-per-cpu=8G         # Memory per CPU
#SBATCH --time=00:01:00          # Wall clock time limit (H:M:S)
#SBATCH --constraint="EPYC_7763"
#SBATCH --nodes=1

# exit immediately if any command exits with a non-zero status
set -e



# print CPU model
srun lscpu

module load stack gcc
srun gcc -O3 -march=native -mcmodel=large -DSTREAM_TYPE=double -DSTREAM_ARRAY_SIZE=272000000 -DNTIMES=20 stream.c.c -o stream_7763
srun stream_7763
