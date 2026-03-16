import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
import numpy as np
import re
import sys
import glob

# ---------------------------------------------------------------------------
# Parse helpers
# ---------------------------------------------------------------------------

def parse_l1l2_file(path):
    """Parse an L1L2_opt.sh .out file (2-D grid search over S2 and S1).

    Returns a list of (s2, s1, perf_pct) and the fixed S3 value.
    """
    results = []
    s2 = None
    s1 = None
    fixed_s3 = None
    with open(path) as f:
        for line in f:
            # Header line: "Fixed S3: 1152"
            m = re.search(r'Fixed S3:\s*(\d+)', line)
            if m:
                fixed_s3 = int(m.group(1))
                continue
            # e.g.  "[1/N] Testing S2=144, S1=32 (elapsed: ..."
            m = re.search(r'Testing S2=(\d+),\s*S1=(\d+)', line)
            if m:
                s2 = int(m.group(1))
                s1 = int(m.group(2))
                continue
            # e.g.  "  Result: 44.9837%"
            m = re.search(r'Result:\s*([0-9.]+)%', line)
            if m and s2 is not None and s1 is not None:
                results.append((s2, s1, float(m.group(1))))
                s2 = None
                s1 = None
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

path = find_file("matmul_opt_l1-*.out")
print(f"Reading: {path}")
data, fixed_s3 = parse_l1l2_file(path)

if not data:
    raise ValueError("No benchmark results found in file – is the run still in progress?")

s3_label = f"S3={fixed_s3}" if fixed_s3 else "S3=fixed"

# Build pivot table ---------------------------------------------------------
s2_unique = sorted(set(d[0] for d in data))
s1_unique = sorted(set(d[1] for d in data))

# grid[i, j] = perf for s2_unique[i], s1_unique[j]; NaN if not tested yet
grid = np.full((len(s2_unique), len(s1_unique)), np.nan)
s2_idx = {v: i for i, v in enumerate(s2_unique)}
s1_idx = {v: i for i, v in enumerate(s1_unique)}

best_s2, best_s1, best_perf = None, None, -np.inf
for s2, s1, perf in data:
    grid[s2_idx[s2], s1_idx[s1]] = perf
    if perf > best_perf:
        best_perf = perf
        best_s2, best_s1 = s2, s1

# ---------------------------------------------------------------------------
# Figure 1: Heat-map
# ---------------------------------------------------------------------------

fig, ax = plt.subplots(figsize=(max(10, len(s1_unique) * 0.55),
                                 max(7,  len(s2_unique) * 0.55)))

cmap = plt.cm.RdYlGn
im = ax.imshow(grid, cmap=cmap, aspect='auto', origin='lower',
               vmin=np.nanmin(grid), vmax=np.nanmax(grid))

plt.colorbar(im, ax=ax, label='Average % of Peak Performance', pad=0.02)

# Annotate cells
for i in range(len(s2_unique)):
    for j in range(len(s1_unique)):
        val = grid[i, j]
        if not np.isnan(val):
            text_color = 'white' if val < (np.nanmin(grid) + np.nanmax(grid)) / 2 else 'black'
            ax.text(j, i, f'{val:.1f}', ha='center', va='center',
                    fontsize=6, color=text_color, fontweight='bold')

# Mark best cell
bi = s2_idx[best_s2]
bj = s1_idx[best_s1]
ax.add_patch(plt.Rectangle((bj - 0.5, bi - 0.5), 1, 1,
                             fill=False, edgecolor='gold',
                             linewidth=2.5, zorder=5))
ax.text(bj, bi + 0.42, '★', ha='center', va='bottom',
        fontsize=10, color='gold', zorder=6)

ax.set_xticks(range(len(s1_unique)))
ax.set_xticklabels([str(v) for v in s1_unique], rotation=45, ha='right', fontsize=8)
ax.set_yticks(range(len(s2_unique)))
ax.set_yticklabels([str(v) for v in s2_unique], fontsize=8)

ax.set_xlabel('L1 block size S1', fontsize=13, fontweight='bold')
ax.set_ylabel('L2 block size S2', fontsize=13, fontweight='bold')

plt.title(
    f'L1 & L2 Block Size Grid-Search vs. Performance  ({s3_label} fixed)\n'
    f'Best: S2={best_s2}, S1={best_s1} → {best_perf:.2f}%  '
    '(Compilation: -O3 -march=znver3 -ffast-math -funroll-loops)',
    fontsize=12, fontweight='bold', pad=14)

plt.tight_layout()
out_heat = 'l1l2_block_heatmap_performance.pdf'
plt.savefig(out_heat, format='pdf', bbox_inches='tight')
print(f"Heat-map saved as {out_heat}")

# ---------------------------------------------------------------------------
# Figure 2: 1-D slice plots – best S1 per S2, and best S2 per S1
# ---------------------------------------------------------------------------

fig2, (ax_left, ax_right) = plt.subplots(1, 2, figsize=(14, 6))

# Left: for each S2 row, take the max over S1
best_by_s2 = [(s2, np.nanmax(grid[s2_idx[s2], :])) for s2 in s2_unique]
s2_arr  = np.array([b[0] for b in best_by_s2])
perf_s2 = np.array([b[1] for b in best_by_s2])

ax_left.scatter(s2_arr, perf_s2, s=100, color='steelblue', marker='o',
                edgecolors='darkblue', linewidths=1.5, zorder=3,
                label='Best % over all S1')
ax_left.plot(s2_arr, perf_s2, color='steelblue', linewidth=1.8, alpha=0.5, zorder=2)

best_s2_row = s2_arr[np.argmax(perf_s2)]
best_s2_perf = np.nanmax(perf_s2)
ax_left.scatter([best_s2_row], [best_s2_perf], s=220, color='gold', marker='*',
                edgecolors='darkgoldenrod', linewidths=1.5, zorder=4,
                label=f'Best: S2={best_s2_row}, {best_s2_perf:.2f}%')
ax_left.axvline(best_s2_row, color='darkgoldenrod', linewidth=1.2,
                linestyle='--', alpha=0.6)

for x, y in zip(s2_arr, perf_s2):
    ax_left.text(x, y + 0.5, f'{y:.1f}%', ha='center', va='bottom',
                 fontsize=7, color='darkblue', fontweight='bold')

ax_left.set_xlabel('L2 block size S2', fontsize=12, fontweight='bold')
ax_left.set_ylabel('Best Avg % of Peak Performance', fontsize=12, fontweight='bold')
ax_left.set_xticks(s2_arr)
ax_left.set_xticklabels([str(v) for v in s2_arr], rotation=45, ha='right', fontsize=8)
ax_left.set_ylim(0, best_s2_perf * 1.2)
ax_left.grid(True, axis='y', alpha=0.3, linestyle='--')
ax_left.grid(True, axis='x', alpha=0.2, linestyle=':')
ax_left.legend(fontsize=10, framealpha=0.95)
ax_left.set_title('Best performance per S2\n(optimised over all S1)',
                   fontsize=11, fontweight='bold')

# Right: for each S1 col, take the max over S2
best_by_s1 = [(s1, np.nanmax(grid[:, s1_idx[s1]])) for s1 in s1_unique]
s1_arr  = np.array([b[0] for b in best_by_s1])
perf_s1 = np.array([b[1] for b in best_by_s1])

ax_right.scatter(s1_arr, perf_s1, s=100, color='coral', marker='D',
                 edgecolors='darkred', linewidths=1.5, zorder=3,
                 label='Best % over all S2')
ax_right.plot(s1_arr, perf_s1, color='coral', linewidth=1.8, alpha=0.5, zorder=2)

best_s1_col = s1_arr[np.argmax(perf_s1)]
best_s1_perf = np.nanmax(perf_s1)
ax_right.scatter([best_s1_col], [best_s1_perf], s=220, color='gold', marker='*',
                 edgecolors='darkgoldenrod', linewidths=1.5, zorder=4,
                 label=f'Best: S1={best_s1_col}, {best_s1_perf:.2f}%')
ax_right.axvline(best_s1_col, color='darkgoldenrod', linewidth=1.2,
                 linestyle='--', alpha=0.6)

for x, y in zip(s1_arr, perf_s1):
    ax_right.text(x, y + 0.5, f'{y:.1f}%', ha='center', va='bottom',
                  fontsize=7, color='darkred', fontweight='bold')

ax_right.set_xlabel('L1 block size S1', fontsize=12, fontweight='bold')
ax_right.set_ylabel('Best Avg % of Peak Performance', fontsize=12, fontweight='bold')
ax_right.set_xticks(s1_arr)
ax_right.set_xticklabels([str(v) for v in s1_arr], rotation=45, ha='right', fontsize=8)
ax_right.set_ylim(0, best_s1_perf * 1.2)
ax_right.grid(True, axis='y', alpha=0.3, linestyle='--')
ax_right.grid(True, axis='x', alpha=0.2, linestyle=':')
ax_right.legend(fontsize=10, framealpha=0.95)
ax_right.set_title('Best performance per S1\n(optimised over all S2)',
                    fontsize=11, fontweight='bold')

fig2.suptitle(
    f'L1 & L2 Block Size Optimisation  ({s3_label} fixed)\n'
    '(Compilation: -O3 -march=znver3 -ffast-math -funroll-loops)',
    fontsize=13, fontweight='bold', y=1.02)

plt.tight_layout()
out_slice = 'l1l2_block_slices_performance.pdf'
plt.savefig(out_slice, format='pdf', bbox_inches='tight')
print(f"Slice plot saved as {out_slice}")

plt.show()
