# Getting Started With TTT

This guide reflects the current fork of `ttt`, which has added support for code blocks, improved preamble injection, `ttt-eval`, and a new math translation path that first tries Pandoc and falls back to PDF rendering when needed.

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

This fork supports a few additional configuration options beyond the original project, especially for math conversion and stateful helper workflows.

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
- `pandoc_preamble` is especially useful with the fork's new math approach, because math is first converted through Pandoc and only falls back to a rendered PDF when that conversion fails.

## 4. Run translation

During translation, the fork now also injects LaTeX packages automatically when it detects that they are needed, including `listings`, `xcolor`, `adjustbox`, `caption`, `amsmath`, `amsfonts`, and `biblatex`.

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

Code blocks are one of the fork's major additions compared with the original version. Raw blocks now become `lstlisting`, and common language names are mapped to matching `listings` languages when possible.

- Inline raw code is translated to `\texttt{...}`.
- Block raw code (triple backticks in Typst) is translated to `lstlisting`.
- `ttt` auto-injects `\usepackage{listings}` in generated output if needed.

## Figure support

Figures are still exported into `generated/*.pdf`, but the fork has better handling for figures inside grids, wildcard figures, and automatic label/reference inference.

- Figures are exported into `generated/*.pdf`.
- Generated LaTeX includes those assets directly.
- Keep the generated `.tex` and `generated/` directory together.

## Math support

Math translation changed significantly in this fork.

- The compiler first tries to convert Typst equations to LaTeX through Pandoc.
- If Pandoc conversion fails, `ttt` falls back to rendering the equation to a PDF and includes that PDF inline.
- The default inline wrapper can be customized through `inline_wrapper` if your template needs different spacing or baseline behavior.
- `pandoc_preamble` can be used to prepend Typst imports or helper definitions before math is handed to Pandoc.

This makes math much more robust than the original single-path approach, especially for documents with more complex equations or Typst constructs that Pandoc handles better than direct translation.

## Common pitfalls

- Run `ttt` in the directory that contains `ttt.toml`.
- Ensure `content_main` and `template` paths are correct relative to that directory.
- If references look wrong, use label prefixes (`fig:`, `tab:`, `lst:`) as described in `README.md`.
- If math output looks odd, try supplying a `pandoc_preamble` or adjusting `inline_wrapper` to better match your template.
