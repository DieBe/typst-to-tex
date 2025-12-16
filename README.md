# Typst To Tex (ttt)

**Problem**: You want to write your papers in typst, but conferences and journals want latex sources. This is an attempt at a solution to that problem. It compiles a typst document to a latex document doing most of the heavy lifting, and allows you to do the final touchups with inline latex, maintaining a source of truth.

Broadly, a Typst or LaTeX document contains three things

- Lightly styled text content
- Fancy content like generated figures, code blocks, inline code, and math
- Layout

Typst is nice for all three but for submissions, we need layout to be done by latex.

 Translating lightly styled text content is easy to do as an AST replacement.

 Translating fancy content to equivalent LaTeX is bordering on impossible, so here typstex generates a pdf with the content and uses `\includepdf` to put it at the right place and shape in the document.

Layout cannot be done by typst if we want to use journal templates, here, you
can add `latexraw` blocks to your typst document to have one source of truth.

## Usage

Install `ttt` with `cargo install --git https://gitlab.com/TheZoq2/ttt.git`

Create a `template.tex` file which does the latex preamble stuff your document requires. Generally, it is best to use the journal template here, and just replace the content with `%CONTENT%`.
For including generated PDFs, you need to use the `graphicx` and `adjustbox` packages

```latex
% template.tex:
\documentclass{article}

\usepackage{graphicx}
\usepackage{adjustbox}


\begin{document}
%CONTENT%
\end{document}
```

Then create a `ttt.toml` config file. By default, you only need to specify the template,
but there are a few more options that can be set there (see the Config struct for details)

Then run
```
ttt your_file.typ
```
which will create `generated` containing the PDFs that will be included, and `your_file.typ.tex` which is the output latex file.

## Limitations

Currently, the system supports what I've needed for my papers, most other things are relatively easy to add support for, it just needs someone to do it. There are some exceptions however:

- Typst infers supplements for things like figures, while latex does not. I.e. you can write `@fig1` in typst but need to write `Figure~\ref{fig1}` in latex. I have not found a way to infer the supplement, so for now, you have to prefix your references with `fig:`, `tab:` and `lst:`. With no prefix, it is assumed that the reference is a citation
- Anything using `state` in typst will be very difficult to support.
Currently, the compiler uses the typst frontend but does not do layout, state
is resolved during layout which means this technique does not work.
