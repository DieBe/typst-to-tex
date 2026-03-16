# AI has been used in the writing of this file.

import numpy as np
import matplotlib.pyplot as plt

# Define constants
P_peak_ph1 = 41.6
BW_ph1 = 31.0206
P_peak_ph2 = 39.2
BW_ph2 = 36.5635

# Roofline function
def roofline(I, P_peak, BW):
    return np.minimum(P_peak, I * BW)

# Calculate ridge points
ridge_ph1 = P_peak_ph1 / BW_ph1
ridge_ph2 = P_peak_ph2 / BW_ph2

# Define Operational Intensity range (x-axis)
I_vals = np.logspace(np.log10(2**-6), np.log10(2**10), 500)

# Calculate Performance values (y-axis)
P_vals_ph1 = roofline(I_vals, P_peak_ph1, BW_ph1)
P_vals_ph2 = roofline(I_vals, P_peak_ph2, BW_ph2)

# Create figure and axis
fig, ax = plt.subplots(figsize=(10, 7))

# Plot Phase 1
ax.plot(I_vals, P_vals_ph1, label=f"Phase 1 (EPYC 7H12)", color='blue', linewidth=2)
ax.scatter([ridge_ph1], [P_peak_ph1], color='blue', edgecolor='darkblue', s=100, label=f"Ridge Ph1 ({ridge_ph1:.2f} F/B)")

# Plot Phase 2
ax.plot(I_vals, P_vals_ph2, label=f"Phase 2 (EPYC 7763)", color='red', linewidth=2, linestyle='--')
ax.scatter([ridge_ph2], [P_peak_ph2], color='red', edgecolor='darkred', s=100, marker='s', label=f"Ridge Ph2 ({ridge_ph2:.2f} F/B)")

# Annotate regions
ax.text(0.1, P_peak_ph1 * 0.5, "Memory Bound", fontsize=10, color='black', ha='left')
ax.text(10.0, P_peak_ph1 * 1.1, "Compute Bound", fontsize=10, color='black', ha='left')

# Configure axes
ax.set_xscale('log')
ax.set_yscale('log')
ax.set_xlabel("Operational Intensity (Flops/Byte)", fontsize=12, fontweight='bold')
ax.set_ylabel("Performance (GFlops/s)", fontsize=12, fontweight='bold')
ax.set_title("Roofline Model: Euler VII Single Core", fontsize=14, fontweight='bold')
ax.grid(True, which='both', linestyle='--', alpha=0.5)
ax.legend(fontsize=10, loc='lower right')

# Save and display the plot
plt.tight_layout()
plt.savefig("../images/roofline_plot.pdf", format='pdf', bbox_inches='tight')
print("Plot saved as ../images/roofline_plot.pdf")

plt.show()
