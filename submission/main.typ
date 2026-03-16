#set heading(numbering: "1.")

= Euler warm-up [10 points]

+ This tool is used to dynamically load packages as required, minimizing version conflicts and ensuring only the necessary components are loaded. It also sets environment variables as needed. Modules can be loaded or unloaded, for example:
  ```
      module load stack/2025-06
      module unload stack/2025-06
  ```
  or any other module available on the system. Additionally, the commands
  ```
      module avail
      module spider <module name>
  ```
  can be used to display a list of available modules or to get more information on a specific module respectively.

+ This tool is employed to manage jobs on Euler or other supercomputers. Slurm is accessed via the login nodes and facilitates access to the compute nodes. Users can request specific hardware resources and submit code for execution. Slurm verifies user permissions for the requested hardware and queues the job. Once the hardware becomes available, the code is executed on the compute node.

+ The C++ code in `01/hello_world.cpp` outputs `Hello World from {$HOSTNAME}`, where `{$HOSTNAME}` represents the system's hostname.

+ The script is located in `01/job_1_4.sh`, with its output and error logs in `01/1_4.out` and `01/1_4.err`.

+ The script is located in `01/job_1_5.sh`, with its output and error logs in `01/1_5.out` and `01/1_5.err`.

= Performance characteristics [50 points]

== Peak performance [16 Points]

The AMD website (@amd_epyc_7763_spec, @amd_epyc_7H12_spec) provides specifications for this model, while the Euler CPU Node Documentation @euler_docs offers details specific to Euler VII. Reference @uops is used for FMA and superscalability factors. The base clock frequency is taken from @amd_epyc_7H12_spec and @amd_epyc_7763_spec.

- Epyc 7H12 / Phase 1:
  $ P_(upright("Core")) = 2 times 4 times 2 times 2.6 upright("GHz") = 41.6 thin "GFlop/s" $
  $ P_(upright("CPU")) = 64 times P_(upright("core")) = 2662.4 thin "GFlop/s" $
  $ P_(upright("node")) = 2 times P_(upright("CPU")) = 5324.8 thin "GFlop/s" $
  $ P_(upright("cluster")) = 292 times P_(upright("node")) = 1554.8 thin "TFlop/s" $

- Epyc 7763 / Phase 2:
  $ P_(upright("Core")) = 2 times 4 times 2 times 2.45 upright("GHz") = 39.2 thin "GFlop/s" $
  $ P_(upright("CPU")) = 64 times P_(upright("core")) = 2508.8 thin "GFlop/s" $
  $ P_(upright("node")) = 2 times P_(upright("CPU")) = 5017.6 thin "GFlop/s" $
  $ P_(upright("cluster")) = 248 times P_(upright("node")) = 1244.3 thin "TFlop/s" $

- Total:
  $ 1554.8 thin "TFlop/s" + 1244.3 thin "TFlop/s" = 2799.1 thin "TFlop/s" $

== Memory Hierarchies [8 Points]

=== Cache and main memory size

Using the commands provided in the task description, we obtain the data in @tab:perf_mem_7763. The output files are available as `02/Memory_7H12.txt` and `02/Memory_7763.txt`, while the corresponding topology plots are included as @fig:epyc_7h12 and @fig:epyc_7763. The job files used to produce this output are `02/job_2_1.sh` and `02/job_2_2.sh`.

The key difference is that the EPYC 7H12 shares L3 cache across 4 cores while the EPYC 7763 shares it across 8 cores. Main memory is partitioned into 4 NUMA nodes per CPU, yielding 8 NUMA nodes per node, for both phase 1 and 2.

#figure(
  table(
    columns: 2,
    stroke: 0.5pt,
    align: (left, center),
    [*Memory level*], [*Size*],
    [Main memory], [256 GB],
    [L3 cache], [512 MiB],
    [L2 cache], [64 MiB],
    [L1i cache], [4 MiB],
    [L1d cache], [4 MiB],
  ),
  caption: [Memory hierarchy of an Euler VII (phase 1/2) node with an AMD EPYC 7H12/7763 CPU. Each core has its own L1 and L2 cache. L3 cache is shared among groups of 4 cores in phase 1 and in groups of 8 cores in phase 2.],
) <tab:perf_mem_7763>

#figure(
  image("images/EPYC_7H12.pdf", width: 100%),
  caption: [Topology of an Euler VII (phase 1) node with an AMD EPYC 7H12 CPU.],
) <fig:epyc_7h12>

#figure(
  image("images/EPYC_7763.pdf", width: 100%),
  caption: [Topology of an Euler VII (phase 2) node with an AMD EPYC 7763 CPU.],
) <fig:epyc_7763>

== Bandwidth: STREAM benchmark [10 Points]

The following resources are available: source code (`02/stream.c`), batch scripts (`02/job_2_3.sh`, `02/job_2_4.sh`), output files (`02/2_3_7H12.out`, `02/2_3_7763.out`).

@tab:stream_7h12 and @tab:stream_7763 show the output from STREAM. Across all measurements, phase 2 shows better results. For both phases, Add and Triad are roughly consistent, while Scale is slightly lower. Copy throughput is notably higher for both, at approximately $31.0 thin "GB/s"$ for phase 1 and $36.6 thin "GB/s"$ for phase 2.

#figure(
  table(
    columns: 5,
    stroke: 0.5pt,
    align: (left, center, center, center, center),
    [*Function*], [*Best Rate MB/s*], [*Avg time*], [*Min time*], [*Max time*],
    [Copy],  [31020.6], [0.150761], [0.140294], [0.163701],
    [Scale], [18001.0], [0.260390], [0.241764], [0.284396],
    [Add],   [20236.0], [0.352662], [0.322594], [0.428797],
    [Triad], [20139.2], [0.356431], [0.324144], [0.466777],
  ),
  caption: [STREAM benchmark results for AMD EPYC 7H12 (Euler VII Phase 1).],
) <tab:stream_7h12>

#figure(
  table(
    columns: 5,
    stroke: 0.5pt,
    align: (left, center, center, center, center),
    [*Function*], [*Best Rate MB/s*], [*Avg time*], [*Min time*], [*Max time*],
    [Copy],  [36563.5], [0.131608], [0.119026], [0.161190],
    [Scale], [22009.5], [0.218883], [0.197733], [0.243171],
    [Add],   [25126.1], [0.281478], [0.259810], [0.359157],
    [Triad], [25357.8], [0.279042], [0.257436], [0.319567],
  ),
  caption: [STREAM benchmark results for AMD EPYC 7763 (Euler VII Phase 2).],
) <tab:stream_7763>

== Performance model: A simple roofline model [16 Points]

The plot was generated using `02/roofline_plot.py`, refer to @fig:roofline_plot. The script also calculates the minimum Operational Intensity required to achieve compute-bound performance. For Euler VII phase 1, the value is $1.34 thin "Flops"/"Byte"$, and for phase 2, it is $1.07 thin "Flops"/"Byte"$.

#figure(
  image("images/roofline_plot.pdf", width: 80%),
  caption: [Roofline model plot for Euler VII nodes.],
) <fig:roofline_plot>

= Auto-vectorization [10 points]

+ Data alignment refers to storing objects at memory addresses that are multiples of their own size. Misaligned memory access can result in significant performance penalties. SIMD/SSE instructions prefer 16-byte alignment for optimal efficiency. Data alignment can be enforced using `__declspec(align(n))` or `__builtin_assume_aligned`, or it may be handled automatically by the compiler.

+ Possible reasons include:
  - Non-contiguous memory access
  - Data-dependencies between loop iterations
  - Pointer aliasing
  - Non-countable loops
  - Function calls within loops
  - Mixing data types

+ Possible ways include:
  - `#pragma ivdep` (ignore assumed dependencies)
  - `#pragma vector always` (force vectorization)
  - `#pragma vector align` (assert the data is aligned)
  - `#pragma loop count(n)` (provide typical trip count for loop)
  - `#pragma omp simd` (force SIMD)

+ Strip-mining splits a loop into a vectorized part and a scalar cleanup loop. Loop blocking partitions the iteration space into smaller blocks to improve cache reuse. Loop interchange reorders nested loops to obtain stride-1 memory access patterns. Loop peeling performs a few scalar iterations until an aligned boundary is reached.

+ We can use user-mandated vectorization with `#pragma omp simd`, which forces vectorization and issues a warning if it fails. For more control, SIMD-enabled functions like `__attribute__((vector))` or `#pragma omp declare simd` allow writing scalar-style functions that the compiler can transform into short vector variants. Vector intrinsics, e.g. `_mm_add_ps` or inline assembly give full manual control.

#pagebreak()

= Matrix multiplication optimization [30 points]

== dgemm_jki.c <sec:jki>

The loop order was modified to `jki` to enhance cache locality, following standard GEMM optimization principles @goto_gemm @blis_toms. @fig:variants compares this approach to the naive implementation, OpenBLAS, and other optimized versions.

The performance of `dgemm_jki.c` demonstrates significant improvement over the naive implementation, achieving $13.21 thin "GFlop/s"$ compared to $3.41 thin "GFlop/s"$ for the naive version. This 3.9x speedup results from the JKI loop order's stride-one access pattern for matrix `C` in the innermost loop, which dramatically reduces cache misses compared to the naive IJK ordering. Despite non-contiguous access for matrix `A`, the JKI ordering achieves 26.3% of peak performance on average versus only 4.6% for the naive implementation.

#figure(
  image("code/04/plots/variants/optimization_levels_performance.pdf", width: 100%),
  caption: [Performance comparison of the naive, jki, OpenBLAS and 1/2/3-level blocking versions.],
) <fig:variants>

== dgemm_blocked.c

The final optimized file is included as `dgemm_blocked.c`.

=== System <sec:system>

Unless otherwise specified, all tests were conducted on a single core of an AMD EPYC 7763. The GCC compiler (version 13.2.0) was used, with flags `-O3 -march=znver3 -ffast-math -funroll-loops` @gcc_manual. The default optimization strategy employed was 2-level blocking, as described in @sec:overview.

=== Overview <sec:overview>

The optimizations implemented are:

- *Multi-level blocking for cache hierarchy*: The code implements three configurable blocking strategies (1-level, 2-level, and 3-level blocking). The 1-level blocking targets the L3 cache with block size `S3`, 2-level blocking adds L2 cache optimization with block size `S2`, and 3-level blocking further incorporates L1 cache optimization with block size `S1`. This multi-level cache blocking approach is a standard technique in high-performance GEMM implementations @goto_gemm @blis_toms.

- *Hierarchical block size optimization*: A systematic tuning process was employed to find optimal block sizes for each cache level. This kind of empirical parameter search is a common strategy in high-performance BLAS implementations @atlas.

- *Data packing for spatial locality*: The micro-kernel packs both matrices A and B into contiguous local buffers with optimal memory layouts, eliminating strided memory access patterns and ensuring sequential access during computation @goto_gemm @blis_toms.

- *Register-level accumulation*: The micro-kernel loads the C tile into a local accumulator array before computation. All updates to C are performed on this local buffer, significantly reducing memory writes @goto_gemm @blis_toms.

- *Loop dependency annotation*: The code uses `#pragma GCC ivdep` in all innermost loops to inform the compiler that loop iterations are independent and can be safely vectorized @gcc_manual.

- *Pointer aliasing control*: All function parameters use the `restrict` keyword to guarantee to the compiler that pointers do not alias, enabling more aggressive optimizations @gcc_manual.

- *Compiler flag optimization*: The script `04/flag_compare.sh` compares different compiler optimization flags to quantify the individual impact of each flag @gcc_manual @atlas.

=== Analysis <sec:analysis>

This section analyzes the impact of the optimizations introduced in `dgemm_blocked.c` and relates them to the overall performance comparison shown in @fig:variants. In all plots, two complementary metrics are reported: the average percentage of peak (a good indicator of sustained performance across problem sizes) and the peak throughput in $"GFlop/s"$ (which highlights best-case behavior under favorable cache and alignment conditions). Overall, the structure of the optimized implementation (cache blocking + packing + micro-kernel accumulation) is aligned with widely used high-performance GEMM designs @goto_gemm @blis_toms.

The incremental results in @fig:steps show that the largest single improvement comes from switching the loop order to `jki`. Subsequent steps such as packing and register accumulation improve the sustained average further, but their effect is smaller and less uniform across sizes.

#figure(
  image("code/04/plots/steps/optimization_steps_performance.pdf", width: 100%),
  caption: [Performance comparison of the incremental optimizations in `dgemm_blocked.c`.],
) <fig:steps>

For 2-level blocking, @fig:l2 exhibits a pronounced optimum at $S_2 = 64$ (with $S_3$ fixed at the best value from @fig:l3). This sharp peak indicates that once the L2 block becomes too large, the packed panels and the active C tile no longer fit comfortably in L2; when it is too small, the overhead becomes dominant. The selected $S_2 = 64$ therefore represents a sweet spot between locality and overhead on the EPYC 7763 core.

For 3-level blocking, @fig:l1l2_slices and @fig:l1l2_heatmap confirm that only a narrow region of $(S_1, S_2)$ combinations performs well, with the best result at $S_1 = 32$ and $S_2 = 64$. As mentioned in @sec:overview, various compiler flags have been tested. @fig:flags suggests that `-O3 -march=znver3 -ffast-math -funroll-loops` yields the best result.

#figure(
  image("code/04/plots/block/l3_block_size_performance.pdf", width: 100%),
  caption: [Average percentage of peak performance as a function of $S_3$ for 1-level blocking. Best result at $S_3 = 1152$ achieving 34.81% of peak.],
) <fig:l3>

#figure(
  image("code/04/plots/block/l2_block_size_performance.pdf", width: 100%),
  caption: [Average percentage of peak performance as a function of $S_2$ for 2-level blocking, with $S_3 = 1152$ fixed. The optimal $S_2 = 64$ achieves 51.4% of peak.],
) <fig:l2>

#figure(
  image("code/04/plots/block/l1l2_block_slices_performance.pdf", width: 100%),
  caption: [Average percentage of peak performance for 3-level blocking as a function of $S_1$ and $S_2$. Best result at $S_1 = 32$, $S_2 = 64$ achieving 25.26% of peak.],
) <fig:l1l2_slices>

#figure(
  image("code/04/plots/block/l1l2_block_heatmap_performance.pdf", width: 100%),
  caption: [Heatmap of average percentage of peak performance for 3-level blocking as a function of $S_1$ and $S_2$.],
) <fig:l1l2_heatmap>

#figure(
  image("code/04/plots/flag/compilation_flags_performance.pdf", width: 100%),
  caption: [Performance comparison of various compilation flags for `dgemm_blocked.c`.],
) <fig:flags>

#bibliography("sources.bib")
 It also sets environment variables as
