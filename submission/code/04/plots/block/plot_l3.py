import matplotlib.pyplot as plt
import numpy as np
import re
import sys
import glob

# ---------------------------------------------------------------------------
# Parse helpers
# ---------------------------------------------------------------------------

def parse_l3_file(path):
    """Parse an L3_opt.sh .out file.

    Returns a list of (s3, perf_pct) tuples in the order they appear.
    """
    results = []
    s3 = None
    with open(path) as f:
        for line in f:
            # e.g.  "[1/N] Testing S3=1152 (elapsed: ..."
            m = re.search(r'Testing S3=(\d+)', line)
            if m:
                s3 = int(m.group(1))
                continue
            # e.g.  "  Result: 44.9837%"
            m = re.search(r'Result:\s*([0-9.]+)%', line)
            if m and s3 is not None:
                results.append((s3, float(m.group(1))))
                s3 = None
    return results


def find_file(pattern, default_glob):
    """If a explicit path was given on argv use it, otherwise glob."""
    if len(sys.argv) > 1:
        return sys.argv[1]
    files = sorted(glob.glob(default_glob))
    if not files:
        raise FileNotFoundError(
            f"No file found matching '{default_glob}'. "
            "Pass the .out path as the first argument."
        )
    return files[-1]   # newest


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

path = find_file(None, "matmul_opt_l3-*.out")
print(f"Reading: {path}")
data = parse_l3_file(path)

if not data:
    raise ValueError("No benchmark results found in file – is the run still in progress?")

data.sort(key=lambda x: x[0])          # sort by S3
s3_vals  = np.array([d[0] for d in data])
perf_vals = np.array([d[1] for d in data])

best_idx  = np.argmax(perf_vals)
best_s3   = s3_vals[best_idx]
best_perf = perf_vals[best_idx]

# ---------------------------------------------------------------------------
# Plot
# ---------------------------------------------------------------------------

fig, ax = plt.subplots(figsize=(13, 6))

ax.scatter(s3_vals, perf_vals,
           s=100, color='steelblue', marker='o',
           edgecolors='darkblue', linewidths=1.5,
           label='Avg % of peak', zorder=3)
ax.plot(s3_vals, perf_vals,
        color='steelblue', linewidth=1.8, alpha=0.5, zorder=2)

# Highlight best
ax.scatter([best_s3], [best_perf],
           s=220, color='gold', marker='*',
           edgecolors='darkgoldenrod', linewidths=1.5,
           label=f'Best: S3={best_s3}, {best_perf:.2f}%', zorder=4)

ax.axvline(best_s3, color='darkgoldenrod', linewidth=1.2,
           linestyle='--', alpha=0.6)

ax.set_xlabel('L3 block size S3', fontsize=13, fontweight='bold')
ax.set_ylabel('Average % of Peak Performance', fontsize=13, fontweight='bold')
ax.set_ylim(0, max(perf_vals) * 1.18)
ax.grid(True, axis='y', alpha=0.3, linestyle='--', zorder=0)
ax.grid(True, axis='x', alpha=0.2, linestyle=':', zorder=0)

# Use a sparse tick locator — one tick per 128 units avoids label collision
import matplotlib.ticker as ticker
ax.xaxis.set_major_locator(ticker.MultipleLocator(128))
ax.xaxis.set_minor_locator(ticker.MultipleLocator(64))
plt.xticks(rotation=45, ha='right', fontsize=9)

# Annotate only the best point
ax.text(best_s3, best_perf + 1.0, f'{best_perf:.1f}%',
        ha='center', va='bottom', fontsize=10,
        color='darkgoldenrod', fontweight='bold')

plt.title(
    'L3 Block Size vs. Performance\n'
    '(Compilation: -O3 -march=znver3 -ffast-math -funroll-loops)',
    fontsize=14, fontweight='bold', pad=16)

ax.legend(fontsize=11, framealpha=0.95)
plt.tight_layout()

out_pdf = 'l3_block_size_performance.pdf'
plt.savefig(out_pdf, format='pdf', bbox_inches='tight')
print(f"Plot saved as {out_pdf}")
plt.show()
