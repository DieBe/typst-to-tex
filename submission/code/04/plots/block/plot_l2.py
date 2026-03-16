import matplotlib.pyplot as plt
import numpy as np
import re
import sys
import glob

# ---------------------------------------------------------------------------
# Parse helpers
# ---------------------------------------------------------------------------

def parse_l2_file(path):
    """Parse an L2_opt.sh .out file.

    Returns a list of (s2, perf_pct) tuples in the order they appear.
    Also returns the fixed S3 value if found.
    """
    results = []
    s2 = None
    fixed_s3 = None
    with open(path) as f:
        for line in f:
            # Header line: "Fixed S3: 1152"
            m = re.search(r'Fixed S3:\s*(\d+)', line)
            if m:
                fixed_s3 = int(m.group(1))
                continue
            # e.g.  "[1/N] Testing S2=144 (elapsed: ..."
            m = re.search(r'Testing S2=(\d+)', line)
            if m:
                s2 = int(m.group(1))
                continue
            # e.g.  "  Result: 44.9837%"
            m = re.search(r'Result:\s*([0-9.]+)%', line)
            if m and s2 is not None:
                results.append((s2, float(m.group(1))))
                s2 = None
    return results, fixed_s3


def find_file(default_glob):
    if len(sys.argv) > 1:
        return sys.argv[1]
    files = sorted(glob.glob(default_glob))
    if not files:
        raise FileNotFoundError(
            f"No file found matching '{default_glob}'. "
            "Pass the .out path as the first argument."
        )
    return files[-1]


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

path = find_file("matmul_opt_l2-*.out")
print(f"Reading: {path}")
data, fixed_s3 = parse_l2_file(path)

if not data:
    raise ValueError("No benchmark results found in file – is the run still in progress?")

data.sort(key=lambda x: x[0])
s2_vals   = np.array([d[0] for d in data])
perf_vals = np.array([d[1] for d in data])

best_idx  = np.argmax(perf_vals)
best_s2   = s2_vals[best_idx]
best_perf = perf_vals[best_idx]

s3_label = f"S3={fixed_s3}" if fixed_s3 else "S3=fixed"

# ---------------------------------------------------------------------------
# Plot
# ---------------------------------------------------------------------------

fig, ax = plt.subplots(figsize=(12, 6))

ax.scatter(s2_vals, perf_vals,
           s=100, color='steelblue', marker='o',
           edgecolors='darkblue', linewidths=1.5,
           label='Avg % of peak', zorder=3)
ax.plot(s2_vals, perf_vals,
        color='steelblue', linewidth=1.8, alpha=0.5, zorder=2)

# Highlight best
ax.scatter([best_s2], [best_perf],
           s=220, color='gold', marker='*',
           edgecolors='darkgoldenrod', linewidths=1.5,
           label=f'Best: S2={best_s2}, {best_perf:.2f}%', zorder=4)

ax.axvline(best_s2, color='darkgoldenrod', linewidth=1.2,
           linestyle='--', alpha=0.6)

ax.set_xlabel(f'L2 block size S2  ({s3_label} fixed)', fontsize=13, fontweight='bold')
ax.set_ylabel('Average % of Peak Performance', fontsize=13, fontweight='bold')
ax.set_ylim(0, max(perf_vals) * 1.18)
ax.grid(True, axis='y', alpha=0.3, linestyle='--', zorder=0)
ax.grid(True, axis='x', alpha=0.2, linestyle=':', zorder=0)

ax.set_xticks(s2_vals)
ax.set_xticklabels([str(v) for v in s2_vals], rotation=45, ha='right', fontsize=8)

# Value annotations
for i, (x, y) in enumerate(zip(s2_vals, perf_vals)):
    if i % 2 == 0 or x == best_s2:
        ax.text(x, y + 0.8, f'{y:.1f}%',
                ha='center', va='bottom', fontsize=7,
                color='darkblue', fontweight='bold')

plt.title(
    f'L2 Block Size vs. Performance  ({s3_label} fixed)\n'
    '(Compilation: -O3 -march=znver3 -ffast-math -funroll-loops)',
    fontsize=14, fontweight='bold', pad=16)

ax.legend(fontsize=11, framealpha=0.95)
plt.tight_layout()

out_pdf = 'l2_block_size_performance.pdf'
plt.savefig(out_pdf, format='pdf', bbox_inches='tight')
print(f"Plot saved as {out_pdf}")
plt.show()
