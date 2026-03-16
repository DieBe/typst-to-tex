import matplotlib.pyplot as plt
import numpy as np
import re

# Parse the data from your output
def parse_optimization_data(text):
    """Parse optimization benchmark results from text output"""

    # Extract each optimization section
    sections = re.split(r'====\s+Opt\d+:', text)

    results = []

    for section in sections[1:]:  # Skip first empty split
        lines = section.strip().split('\n')

        # Extract optimization name and description
        name_line = lines[0].strip()
        desc_match = re.search(r'# Description:\s*(.+)', section)
        description = desc_match.group(1) if desc_match else name_line

        # Extract average percentage
        avg_match = re.search(r'# Average percentage of peak performance = ([0-9.]+)', section)
        if avg_match:
            avg_percentage = float(avg_match.group(1))

            # Extract all Gflop/s values to find best
            gflops_values = []
            for line in lines:
                if 'Gflop/s:' in line and 'Size:' in line:
                    gf_match = re.search(r'Gflop/s:\s*([0-9.]+)', line)
                    if gf_match:
                        gflops_values.append(float(gf_match.group(1)))

            best_gflops = max(gflops_values) if gflops_values else 0.0

            # Extract optimization number
            opt_num = re.search(r'^(.+?)\s*=', name_line)
            opt_name = opt_num.group(1).strip() if opt_num else name_line.split('=')[0].strip()

            results.append({
                'name': opt_name,
                'description': description,
                'avg_percentage': avg_percentage,
                'best_gflops': best_gflops
            })

    return results

# Your data (paste the relevant section from your output)
data_text = """
==== Opt0: Naive (i-j-k) ======================
# Description:	Opt0: Naive, three-loop dgemm (i-j-k order)
# Average percentage of peak performance = 4.00416

==== Opt1: j-k-i loop order ===================
# Description:	Opt1: No blocking, j-k-i loop order only
# Average percentage of peak performance = 21.7314

==== Opt2: Single-level blocking ==============
# Description:	Opt2: Single-level blocking (j-k-i order, block size 64)
# Average percentage of peak performance = 22.3825

==== Opt3: + restrict =========================
# Description:	Opt3: No restrict (2-level blocking only)
# Average percentage of peak performance = 24.5696

==== Opt4: + ivdep pragmas ====================
# Description:	Opt4: No ivdep pragmas (2-level blocking, restrict)
# Average percentage of peak performance = 31.1116

==== Opt5: + register accumulation ============
# Description:	Opt5: No register accum (2-level blocking, pragmas, restrict)
# Average percentage of peak performance = 31.5666

==== Opt6: + data packing =====================
# Description:	Opt6: No packing (2-level blocking, register accum, pragmas, restrict)
# Average percentage of peak performance = 32.1219

==== Opt7: + multi-level blocking =============
# Description:	Opt7: Fully optimized (2-level blocking, packing, register accum, pragmas, restrict)
# Average percentage of peak performance = 41.7645
"""

# Manually input the data with best Gflop/s values
optimizations = [
    ('Naive (i-j-k)', 4.00, 3.25),
    ('+ j-k-i loop order', 21.73, 12.73),
    ('+ Single-level blocking', 22.38, 10.51),
    ('+ restrict', 24.57, 12.30),
    ('+ ivdep pragmas', 31.11, 15.38),
    ('+ register accum', 31.57, 14.91),
    ('+ data packing', 32.12, 16.06),
    ('+ multi-level blocking', 41.76, 20.57),
]

names = [opt[0] for opt in optimizations]
avg_pct = [opt[1] for opt in optimizations]
best_gflops = [opt[2] for opt in optimizations]

# Create figure with dual y-axes
fig, ax1 = plt.subplots(figsize=(14, 8))

x_pos = np.arange(len(names))

# Plot average percentage as scatter with connecting line
ax1.scatter(x_pos, avg_pct,
           s=150, color='steelblue',
           marker='o', linewidths=2.5,
           edgecolors='darkblue',
           label='Avg % of Peak', zorder=3)
ax1.plot(x_pos, avg_pct,
         color='steelblue', linewidth=2, alpha=0.5, zorder=2)

# Configure primary y-axis (Average %)
ax1.set_xlabel('Optimization Step', fontsize=13, fontweight='bold')
ax1.set_ylabel('Average % of Peak Performance', fontsize=13, fontweight='bold', color='steelblue')
ax1.tick_params(axis='y', labelcolor='steelblue', labelsize=11)
ax1.set_ylim(0, max(avg_pct) * 1.15)
ax1.grid(True, axis='y', alpha=0.3, linestyle='--', zorder=0)

# Add value labels for average percentage
for i, (x, y) in enumerate(zip(x_pos, avg_pct)):
    ax1.text(x, y + 1.5, f'{y:.1f}%',
             ha='center', va='bottom', fontsize=10,
             color='darkblue', fontweight='bold')

# Create secondary y-axis for Gflop/s
ax2 = ax1.twinx()

# Plot Gflop/s as scatter with connecting line
ax2.scatter(x_pos, best_gflops,
           s=150, color='coral',
           marker='D', linewidths=2.5,
           edgecolors='darkred',
           label='Best Gflop/s', zorder=3)
ax2.plot(x_pos, best_gflops,
         color='coral', linewidth=2, alpha=0.5, zorder=2)

# Configure secondary y-axis (Gflop/s)
ax2.set_ylabel('Best Gflop/s', fontsize=13, fontweight='bold', color='coral')
ax2.tick_params(axis='y', labelcolor='coral', labelsize=11)
ax2.set_ylim(0, max(best_gflops) * 1.15)

# Add value labels for Gflop/s
for i, (x, y) in enumerate(zip(x_pos, best_gflops)):
    ax2.text(x, y - 0.8, f'{y:.2f}',
             ha='center', va='top', fontsize=9,
             color='darkred', fontweight='bold')

# Set x-axis labels
ax1.set_xticks(x_pos)
ax1.set_xticklabels(names, rotation=25, ha='right', fontsize=10)

# Title
plt.title('Optimization Steps Performance - benchmark-blocked-l2\n(Compilation: -O3 -march=znver3 -ffast-math -funroll-loops)',
          fontsize=14, fontweight='bold', pad=20)

# Combine legends
lines1, labels1 = ax1.get_legend_handles_labels()
lines2, labels2 = ax2.get_legend_handles_labels()
ax1.legend(lines1 + lines2, labels1 + labels2,
          loc='upper left', fontsize=11, framealpha=0.95)

# Tight layout
plt.tight_layout()

plt.savefig('optimization_steps_performance.pdf', format='pdf', bbox_inches='tight')

print("Plot saved as optimization_steps_performance.pdf")

plt.show()
