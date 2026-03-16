import matplotlib.pyplot as plt
import numpy as np

script_data = {
    'Naive (i-j-k)': (2.36, 4.62),
    'JKI loop order': (0.51, 1.25),
    '1-level blocking': (16.10, 38.36),
    '2-level blocking': (21.70, 49.87),
    '3-level blocking': (20.54, 41.88),
    'OpenBLAS': (48.32, 114.78),
}

benchmark_data = {
    'Naive (i-j-k)': (3.41, 4.60167),
    'JKI loop order': (13.21, 26.2688),
    '1-level blocking': (16.36, 38.9291),
    '2-level blocking': (21.99, 49.2477),
    '3-level blocking': (20.54, 39.0783),
    'OpenBLAS': (48.07, 113.183),
}

optimizations = []
for name in ['Naive (i-j-k)', 'JKI loop order', '1-level blocking',
             '2-level blocking', '3-level blocking', 'OpenBLAS']:
    script_gflops, script_pct = script_data[name]
    bench_gflops, bench_pct = benchmark_data[name]

    max_gflops = max(script_gflops, bench_gflops)
    max_pct = max(script_pct, bench_pct)

    optimizations.append((name, max_gflops, max_pct))

names = [opt[0] for opt in optimizations]
best_gflops = [opt[1] for opt in optimizations]
avg_pct = [opt[2] for opt in optimizations]

fig, ax1 = plt.subplots(figsize=(14, 8))

x_pos = np.arange(len(names))

ax1.scatter(x_pos, avg_pct,
           s=150, color='steelblue',
           marker='o', linewidths=2.5,
           edgecolors='darkblue',
           label='Avg % of Peak', zorder=3)

ax1.set_xlabel('Optimization Level', fontsize=13, fontweight='bold')
ax1.set_ylabel('Average % of Peak Performance', fontsize=13, fontweight='bold', color='steelblue')
ax1.tick_params(axis='y', labelcolor='steelblue', labelsize=11)
ax1.set_ylim(0, max(avg_pct) * 1.15)
ax1.grid(True, axis='y', alpha=0.3, linestyle='--', zorder=0)

for i, (x, y) in enumerate(zip(x_pos, avg_pct)):
    ax1.text(x, y + 4, f'{y:.1f}%',
             ha='center', va='bottom', fontsize=10,
             color='darkblue', fontweight='bold')

ax2 = ax1.twinx()

ax2.scatter(x_pos, best_gflops,
           s=150, color='coral',
           marker='D', linewidths=2.5,
           edgecolors='darkred',
           label='Best Gflop/s', zorder=3)

ax2.set_ylabel('Best Gflop/s', fontsize=13, fontweight='bold', color='coral')
ax2.tick_params(axis='y', labelcolor='coral', labelsize=11)
ax2.set_ylim(0, max(best_gflops) * 1.15)

for i, (x, y) in enumerate(zip(x_pos, best_gflops)):
    ax2.text(x, y - 2, f'{y:.2f}',
             ha='center', va='top', fontsize=9,
             color='darkred', fontweight='bold')

ax1.set_xticks(x_pos)
ax1.set_xticklabels(names, rotation=15, ha='right', fontsize=11)

plt.title('DGEMM Optimization Progression\n(Compilation: -O3 -march=znver3 -ffast-math -funroll-loops)',
          fontsize=14, fontweight='bold', pad=20)

lines1, labels1 = ax1.get_legend_handles_labels()
lines2, labels2 = ax2.get_legend_handles_labels()
ax1.legend(lines1 + lines2, labels1 + labels2,
          loc='upper left', fontsize=11, framealpha=0.95)

plt.tight_layout()

plt.savefig('optimization_levels_performance.svg', format='svg', bbox_inches='tight')
plt.savefig('optimization_levels_performance.pdf', format='pdf', bbox_inches='tight')

print("Plot saved as optimization_levels_performance.svg and .pdf")
print("\nMaximum values used:")
for name, gflops, pct in optimizations:
    print(f"{name:20s}: {gflops:6.2f} Gflop/s, {pct:6.2f}% avg peak")

plt.show()
