import matplotlib.pyplot as plt
import numpy as np

# Data from your results
def get_data():
    configs = [
        ("baseline (no flags)", 0.56, 1.37),
        ("-O3", 10.98, 26.75),
        ("-march=znver3", 0.58, 1.44),
        ("-O3 -march=znver3", 21.37, 50.63),
        ("-O3 -march -ffast-math", 21.55, 50.88),
        ("-O3 -march -funroll-loops", 21.26, 50.33),
        ("-O3 -march -fno-trapping-math", 21.36, 50.82),
        ("-O3 -march -fno-stack-protector", 21.57, 50.92),
        ("-O3 -march -fopenmp-simd", 21.83, 50.77),
        ("-O3 -march -fprefetch-loop-arrays", 21.12, 50.27),
        ("-O3 -march -fomit-frame-pointer", 21.61, 51.16),
        ("-O3 -march -falign-loops=64", 21.55, 51.19),
        ("-O3 -march -ffast-math -funroll-loops", 21.68, 51.24),
        ("+ -fno-trapping-math", 20.74, 49.34),
        ("+ -fno-stack-protector", 21.58, 50.54),
        ("+ -fopenmp-simd", 21.45, 50.33),
        ("+ -fprefetch-loop-arrays", 21.22, 50.13),
        ("+ -fomit-frame-pointer", 21.21, 49.82),
        ("all flags", 21.20, 50.29),
    ]

    names = [c[0] for c in configs]
    gflops = [c[1] for c in configs]
    avg_pct = [c[2] for c in configs]

    return names, gflops, avg_pct

# Get data
names, gflops, avg_pct = get_data()

# Sort by average percentage
sorted_indices = np.argsort(avg_pct)[::-1]
names_sorted = [names[i] for i in sorted_indices]
gflops_sorted = [gflops[i] for i in sorted_indices]
avg_pct_sorted = [avg_pct[i] for i in sorted_indices]

# Create figure and primary axis
fig, ax1 = plt.subplots(figsize=(18, 10))

# Position for x-axis
x_pos = np.arange(len(names_sorted))

# Plot average percentage as scatter points
scatter1 = ax1.scatter(x_pos, avg_pct_sorted,
                       s=120, color='steelblue',
                       marker='o', linewidths=2,
                       edgecolors='darkblue',
                       label='Avg % of Peak', zorder=3)

# Configure primary y-axis (Average %)
ax1.set_xlabel('Compilation Flag Configuration', fontsize=13, fontweight='bold')
ax1.set_ylabel('Average % of Peak Performance', fontsize=13, fontweight='bold', color='steelblue')
ax1.tick_params(axis='y', labelcolor='steelblue', labelsize=11)
ax1.set_ylim(0, max(avg_pct_sorted) * 1.12)
ax1.grid(True, axis='y', alpha=0.3, linestyle='--', zorder=0)

# Add value labels for average percentage
for i, (x, y) in enumerate(zip(x_pos, avg_pct_sorted)):
    ax1.text(x, y + 1.5, f'{y:.1f}%',
             ha='center', va='bottom', fontsize=9,
             color='darkblue', fontweight='bold')

# Create secondary y-axis for Gflop/s
ax2 = ax1.twinx()

# Plot Gflop/s as scatter points with different marker
scatter2 = ax2.scatter(x_pos, gflops_sorted,
                       s=120, color='coral',
                       marker='D', linewidths=2,
                       edgecolors='darkred',
                       label='Best Gflop/s', zorder=3)

# Configure secondary y-axis (Gflop/s)
ax2.set_ylabel('Best Gflop/s', fontsize=13, fontweight='bold', color='coral')
ax2.tick_params(axis='y', labelcolor='coral', labelsize=11)
ax2.set_ylim(0, max(gflops_sorted) * 1.12)

# Add value labels for Gflop/s
for i, (x, y) in enumerate(zip(x_pos, gflops_sorted)):
    ax2.text(x, y - 1.2, f'{y:.2f}',
             ha='center', va='top', fontsize=9,
             color='darkred', fontweight='bold')

# Set x-axis labels
ax1.set_xticks(x_pos)
ax1.set_xticklabels(names_sorted, rotation=45, ha='right', fontsize=10)

# Title
plt.title('Impact of Compilation Flags on benchmark-blocked-l2 Performance',
          fontsize=15, fontweight='bold', pad=20)

# Combine legends from both axes
lines1, labels1 = ax1.get_legend_handles_labels()
lines2, labels2 = ax2.get_legend_handles_labels()
ax1.legend(lines1 + lines2, labels1 + labels2,
          loc='upper right', fontsize=12, framealpha=0.95)

# Tight layout
plt.tight_layout()

# Save as SVG (vector format per benchmarking rules)
plt.savefig('compilation_flags_performance.pdf', format='pdf', bbox_inches='tight')

print("Plot saved as compilation_flags_performance.pdf")

plt.show()
