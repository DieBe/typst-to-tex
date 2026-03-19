#import"@local/hpclab:0.1.0": *

#show: hpclab


#serieheader(
  "High-Performance Computing Lab for CSE",
  "2026",
  "Benjamin Diethelm",
  "Discussed with: Lukas Ammann, Raphael Caixeta, Gian Laager",
  "Project 3",
  [Due data: Monday 06 April 2026, 18:00 CEST],
)

#heading(level: 3, numbering: none, [System])
We run all benchmarks on an Euler VII (phase II) node on an AMD EPYC 7763 CPU, unless stated differently. `g++` is used to compile the C++ code, with flags `-O3 -ffast-math -funroll-loops -march=native -fopenmp`.

= Task: Implementing the linear algebra functions and the stencil operators [50 Points]
== HPC functions (`linalg.cpp`)
The sequential implementation of the linear algebra functions is straight-forward. All of them (besides of `hpc_norm2()`) consist of a basic `for`-loop over all elements, where the computation is done according to the comments. Note that we avoid calling `hpc` functions in one-another. In the sequential case, this might be the cleaner, shorter and less redundant approach. But keeping in mind that we will be parallelizing the functions, we avoid function calls to multiple functions that will be multi-threaded, as this would lead to more overhead due to less work being done in a single `#omp parallel` region. For the precise implementation, see `mini_app/linalg.cpp` (ignoring all `#pragma omp`).

== Stencil kernel (`operators.cpp`)
We use the formulas provided in the project instructions. In the `diffusion()` function, we first have to set $f_(i,j)^k$ for all $i$ and $j$ according to:
$ f_(i,j)^k = [-(4+alpha)s_(i,j)^k + s_(i-1,j)^k + s_(i+1, j)^k + s_(i,j-1)^k + s_(i,j+1)^k + beta s_(i,j)^k (1-s_(i,j)^k)] + alpha s_(i,j)^(k-1) $
This can be trivially translated to `C++` using double `for`-loops:
```cpp
for (int j = 1; j < jend; j++) {
  for (int i = 1; i < iend; i++) {
    f(i, j) = (-(4 + alpha) * s_new(i, j)
               + s_new(i - 1, j)
               + s_new(i + 1, j)
               + s_new(i, j - 1)
               + s_new(i, j + 1)
               + beta * s_new(i, j) * (1 - s_new(i, j)))
               + alpha * s_old(i, j);
  }
}
```
Then we have to set our dirichlet boundary conditions, which we can do like this for the east + west boundary:
```cpp
for (int j = 0; j <= jend; j++) {
  f(iend, j) = s_new(iend, j) - 0.1;
  f(0, j) = s_new(0, j) - 0.1;
}
```
Note that the structure of the loops has changed slightly compared to the skeleton code. The reasoning for this is explained in #ref(<sec:diff_par>)
North + south boundaries are implemented in the same fashion. The full code is available in `mini_app/operators.cpp`, again ignoring all `#pragma omp` statements.

== Results
Since runtime is not relevant for this part and we are executing sequentially anyways, this code was executed locally instead of on euler.
we run the script with `./main 128 1 0.00` to get the initial state (0 timesteps are not allowed). This gives:
```
================================================================================
                      Welcome to mini-stencil!
version   :: C++ Serial
mesh      :: 128 * 128 dx = 0.00787402
time      :: 1 time steps from 0 .. 0
iteration :: CG 300, Newton 50, tolerance 1e-06
================================================================================
ERROR: CG failed to converge
step 1 ERROR : nonlinear iterations failed to converge
--------------------------------------------------------------------------------
simulation took 0.0449938 seconds
301 conjugate gradient iterations, at rate of 6689.81 iters/second
1 Newton iterations
--------------------------------------------------------------------------------
### 1, 128, 1, 301, 1, 0.0449938 ###
Goodbye!
```
Note that the convergence error is irelevant since our timestep is 0 and we are anyways only interested in the original state. The plot of this initial state (generated with the provided python plotting script) can be found in #ref(<fig:initial>), it look as we would expect.

To get the final solution, we run `./main 128 100 0.005` as provided in the project manual. We get the following output:
```
================================================================================
                      Welcome to mini-stencil!
version   :: C++ Serial
mesh      :: 128 * 128 dx = 0.00787402
time      :: 100 time steps from 0 .. 0.005
iteration :: CG 300, Newton 50, tolerance 1e-06
================================================================================
--------------------------------------------------------------------------------
simulation took 0.227544 seconds
1513 conjugate gradient iterations, at rate of 6649.26 iters/second
300 Newton iterations
--------------------------------------------------------------------------------
### 1, 128, 100, 1513, 300, 0.227544 ###
Goodbye!
```
It shows no errors, our simulation did $1513$ conjugate gradient iterations and $300$ Newton iterations. This is the same as in the project manual, suggesting a working implementation. This is further confirmed by the plot in #ref(<fig:final_seq>).

#grid(
  columns: 2,
  gutter: 1em,
  [
    #figure(
      image("plots/initial.png", width: 100%),
      caption: [Initial configuration ($t=0$, serial) for a $128 times 128$ grid.],
    ) <fig:initial>
  ],
  [
    #figure(
      image("plots/final_seq.png", width: 100%),
      caption: [Final solution ($t=0.005$, serial) after $1513$ conjugate gradient iterations and $300$ Newton iterations on a $128 times 128$ grid with $100$ timesteps.],
    ) <fig:final_seq>
  ]
)


= Task: Adding OpenMP to the nonlinear PDE mini-app [50 Points]
== Welcome message in `main.cpp` and serial compilation [5 Points]
We use compiler directives according to the project manual and the OpenMP specification @openmp_spec. If we compile with `-fopenmp`, we enter a `#pragma omp parallel sections` followed by `pragam single`, printing the number of threads. This way we print the thread number just once. It would also be possible to print the version from within the `single` region as well. If  we compile without `-fopenmp`, we default to the serial welcome message.
```cpp
#ifdef _OPENMP
std::cout << "version   :: C++ OpenMP" << std::endl;
#pragma omp parallel sections
{
    #pragma single
    std::cout << "threads   :: " << omp_get_num_threads() << std::endl;
}
#else
std::cout << "version   :: C++ Serial" << std::endl;
#endif
```

== Linear algebra kernel [10 Points]
Most of the `hpc` functions can be trivially parallelized by using a `#pragam omp parralel for` in front of each `for`-loop. For the `hpc_dot()` function, we need an additional `reduction(+:result)` clause since we are reducing both vectors `x` and `y` into a single variable. This has been implemented in `mini_app/linalg.cpp`.

== The diffusion stencil [15 Points] <sec:diff_par>
We parallelize both the interior point computation and the boundary computation. This is implemented in `mini_app/operators.cpp`. We use a single `parallel` region to minimize overhead and `nowait` wherever possible to minimize barriers.

For the interior point computation, we use `#pragma omp for collapse(2) nowait` to effectively parallelize the double for loop ofer all combinations of `i` and `j`.

Setting the boundary conditions can be parallelized in the same manner, we put `#pragma omp for nowait` in front of each for loop. We have additionally changed the structure from 4 loops to two loops, going over east + west and north + south boundarys in a single loop each. This should also reduce overhead as we now have fewer loops that do more work each.

== Strong scaling [10 Points]
We evaluate strong scaling by fixing the problem size per curve and increasing the thread count from 1 to 16. The runtime distributions in #ref(<fig:ss_dist>) show that larger grids benefit substantially from parallelization, while very small grids are dominated by overhead. For $64 times 64$, absolute runtimes are already tiny and thread-management/synchronization costs dominate, so additional threads provide little to no benefit and can even hurt stability. $128 times 128$ improves only modestly for the same reason.

For $256 times 256$ and especially $512 times 512$, the work per thread is large enough to amortize OpenMP overhead. This is reflected by the clear downward trend in runtime in #ref(<fig:ss_dist>) and by the speedup curves in #ref(<fig:ss_speed>), where these cases approach much better scaling than the smaller grids.

The $1024 times 1024$ case shows the strongest speedup in #ref(<fig:ss_speed>) and reaches close to the ideal line at high thread counts, indicating that the implementation can use the available cores effectively when the workload is sufficiently large.

Some points in #ref(<fig:ss_speed>) show superlinear speedup. This can happen in practice due to cache effects and reduced memory pressure per thread, and can also be amplified by run-to-run noise. Overall, the plots consistently indicate that our OpenMP implementation scales well for medium-to-large problem sizes, while small grids are overhead-limited.



#figure(
  image("plots/strong_scaling_distribution.pdf"),
  caption: [
    Strong-scaling runtime distributions across thread counts ($1,2,4,8,16$) for fixed grid sizes ($64 times 64$ to $1024 times 1024$). Measurements were performed on one Euler VII (phase II) AMD EPYC 7763 node (`--cpus-per-task=16`, `OMP_PROC_BIND=close`, `OMP_PLACES=cores`), with 1 warmup run and 20 measured repetitions per configuration. Each run used 1000 timesteps and total simulation time $t=1.0$.
  ],
) <fig:ss_dist>


  #figure(
    image("plots/strong_scaling_speedup.pdf"),
    caption: [
      Strong-scaling speedup relative to 1 thread for each fixed grid size. Same experimental setup as in #ref(<fig:ss_dist>).
    ],
  ) <fig:ss_speed>



== Weak scaling [10 Points]

In the weak-scaling experiment, we increase the problem size proportionally with the thread count, i.e., from $(64 times 64, 1)$ to $(128 times 128, 4)$, $(256 times 256, 16)$, and $(512 times 512, 64)$ for the base-$64 times 64$ series, and analogously for the other base resolutions. Ideally, the runtime would remain constant as the number of threads and problem size grow proportionally.

The measured runtime distributions in #ref(<fig:ws_dist>) show a clear increase in time with scale factor for all three base resolutions. This indicates that the implementation is far from ideal weak scaling on the tested configurations. The spread of the distributions also grows for larger configurations (especially at 64 threads), pointing to increased run-to-run variability under higher parallel load.

This trend is consistent with the efficiency curves in #ref(<fig:ws_eff>): efficiency decreases monotonically with thread count, from 100% at the 1-thread baseline to only a few percent at 64 threads. The time-to-solution view in #ref(<fig:ws_time>) confirms the same behavior directly: instead of remaining flat (ideal weak scaling), all curves increase strongly with scale factor, with the steepest growth for the largest base resolution.

Overall, the weak-scaling results suggest that communication/synchronization overhead, memory-system pressure, and general parallel overhead dominate as the thread count and global problem size increase, preventing constant-time scaling.


#grid(
  columns: 2,
  gutter: 1em,
  [
#figure(
  image("plots/weak_scaling_distribution.pdf"),
  caption: [
    Weak-scaling runtime distributions across configurations with proportional workload increase ($1 arrow 4 arrow 16 arrow 64$ threads and corresponding scaled grids). Experiments were run on Euler VII (phase II) AMD EPYC 7763 nodes with one warmup and 20 measured repetitions per configuration. Each run used total simulation time $t=1.0$, while timesteps and grid resolution were scaled with the weak-scaling factor.
  ]
) <fig:ws_dist>
  ],
  [
#figure(
  image("plots/weak_scaling_efficiency.pdf"),
  caption: [
    Weak-scaling efficiency relative to the 1-thread baseline for each base resolution. Same experimental setup as in #ref(<fig:ws_dist>).
  ]
) <fig:ws_eff>

#figure(
  image("plots/weak_scaling_time.pdf"),
  caption: [
    Weak-scaling time-to-solution as a function of configuration (thread count and proportionally scaled problem size). Same experimental setup as in #ref(<fig:ws_dist>).
  ]
) <fig:ws_time>
  ]
)


#bibliography("sources.bib")
