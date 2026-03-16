#!/bin/bash
#SBATCH --job-name=slurm_job_one # Job name    (default: sbatch)
#SBATCH --output=%x-%j.out       # Output file (default: slurm-%j.out)
#SBATCH --error=%x-%j.err        # Error file  (default: slurm-%j.out)
#SBATCH --ntasks=1               # Number of tasks
#SBATCH --cpus-per-task=1        # Number of CPUs per task
#SBATCH --mem-per-cpu=1G         # Memory per CPU
#SBATCH --time=00:01:00          # Wall clock time limit (H:M:S)
#SBATCH --constraint="EPYC_7H12"
#SBATCH --nodes=1

# exit immediately if any command exits with a non-zero status
set -e



# print CPU model
srun lscpu
srun free -h --si
srun hwloc-ls --whole-system --no-io --of console
srun hwloc-ls --whole-system --no-io --of pdf EPYC_7H12.pdf

