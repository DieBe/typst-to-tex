# Getting Started With TTT

This guide shows the fastest way to use `ttt` in a new Typst project.

## 1. Install `ttt`

If you are using this repository directly:

```bash
cargo install --path . --force
```

If you install from git:

```bash
cargo install --git https://codeberg.org/TheZoq2/ttt.git
```

## 2. Create a LaTeX template

Create a `template.tex` in your project with `%CONTENT%` where translated body text should go.

Example:

```tex
\documentclass{article}

\usepackage{graphicx}
\usepackage{adjustbox}
\usepackage{float}

\newfloat{Listing}{htpb}{lop}
\DeclareCaptionSubType{Listing}

\begin{document}
%CONTENT%
\end{document}
```

Notes:

- `graphicx` and `adjustbox` are needed for figure/image includes.
- `float` and `Listing` setup are needed if you use listing-style figures.

## 3. Add `ttt.toml`

Create `ttt.toml` in the same directory where you run `ttt`:

```toml
content_main = "main.typ"
template = "template.tex"
```

Optional keys:

- `eval_main` for `ttt-eval` support.
- `figure_placement` to control figure placement (default: `[t]`).
- `inline_wrapper` for inline math/image fallback rendering.
- `pandoc_preamble` if your math conversion needs Typst preamble code.

## 4. Run translation

```bash
ttt
```

or

```bash
ttt --minor
```

for detailed minor diagnostics.

Output:

- `<content_main>.tex` (for example `main.typ.tex`)
- `generated/` folder with figure/math/code assets used by LaTeX

## 5. Compile LaTeX

From the same directory:

```bash
pdflatex main.typ.tex
```

## 6. Recommended workflow

1. Edit Typst source (`main.typ` and included files).
2. Run `ttt`.
3. Compile generated LaTeX (`pdflatex`, `latexmk`, etc.).
4. Iterate.

## Code block support

- Inline raw code is translated to `\texttt{...}`.
- Block raw code (triple backticks in Typst) is translated to `lstlisting`.
- `ttt` auto-injects `\usepackage{listings}` in generated output if needed.

## Figure support

- Figures are exported into `generated/*.pdf`.
- Generated LaTeX includes those assets directly.
- Keep the generated `.tex` and `generated/` directory together.

## Common pitfalls

- Run `ttt` in the directory that contains `ttt.toml`.
- Ensure `content_main` and `template` paths are correct relative to that directory.
- If references look wrong, use label prefixes (`fig:`, `tab:`, `lst:`) as described in `README.md`.
