# Typst To Tex (ttt)

**Problem**: You want to write your papers in Typst, but conferences and journals want LaTeX sources. `ttt` is an attempt at solving that problem. It compiles a Typst document to LaTeX, does most of the heavy lifting automatically, and lets you finish the last mile with inline LaTeX so you still keep a single source of truth.

This repository is a fork of the original `ttt` at commit `ad35218edb6bca45b4cea02cc4ff57f434e732cc`. Since then, the fork has grown several extensions and behavior changes. The main additions are:

- code block support, including language mapping and `lstlisting` output
- better automatic preamble injection for packages such as `adjustbox`, `caption`, `listings`, `xcolor`, `amsmath`, `amsfonts`, and `biblatex`
- bibliography resource extraction from the Typst source
- optional `ttt-eval` support for resolving stateful Typst-derived values
- preprocessing for local imports and some template-specific helpers
- improved handling of figures, grid layout, and labels
- a new math pipeline using Pandoc, described below

Broadly, a Typst or LaTeX document contains three things:

- lightly styled text content
- fancy content like generated figures, code blocks, inline code, and math
- layout

Typst is still a good authoring language for all three, but for journal and conference submissions the final layout often has to be handled by LaTeX.

Lightly styled text content is translated through AST replacement.

Fancy content is handled in a more mixed way. Figures and other generated assets are often rendered to PDFs and inserted into the LaTeX output with `\includegraphics` or similar. Inline code and listings are rendered directly into LaTeX constructs. Some layout-sensitive elements are emitted as raw LaTeX when that is the most reliable representation.

Layout itself is not delegated to Typst when a journal template needs to control the final document. Instead, you can add `latexraw` blocks to your Typst document so that the Typst source stays the source of truth while still allowing template-specific LaTeX structure.

## Usage

Install `ttt` with `cargo install --git https://codeberg.org/TheZoq2/ttt.git`

Create a `template.tex` file which does the latex preamble stuff your document requires. Generally, it is best to use the journal template here, and just replace the content with `%CONTENT%`.
For including generated PDFs, you need to use the `graphicx` and `adjustbox` packages, and for listings
you need some special `float` code

```latex
% template.tex:
\documentclass{article}

\usepackage{graphicx}
\usepackage{adjustbox}

% For listings to render as listings, this is required
\usepackage{float}

\newfloat{Listing}{htpb}{lop}
\DeclareCaptionSubType{Listing}



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

### Sub Figures With Wild Figures

TTT handles figures by mapping them to latex figures, but figures containing sub-figures
cannot be handled automatically. Instead, `ttt` supports "wild figures" in which the figure
content is generated but not inserted anywhere in the document. Instead, you can use
raw latex to place the content where you want it. For example:

```typst
#let mem_command_def = (
  content: [
   ...
  ],
  caption: []
)
#figure(kind: "wild", supplement: "wild", mem_command_def.content) <mem_command_def>

#if not is_ttt {
  // Your normal typst layout goes here
} else {
  #raw_latex("
    \\begin{figure}[t]
      \\begin{minipage}{0.33\\textwidth}
        \\subcaptionbox{\\label{lst:surfer:mem_command_def}}{\\includegraphics[]{../#wild:mem_command_def#}}
      \\end{minipage}
      ...
    \\end{figure}
  ")
}
```

### TTT Specific Customization

Some small changes are sometimes required between standard typst compilation and `ttt`. You can
detect if the document is being compiled with `ttt` with
```typst
#let is_ttt = sys.inputs.at("is_ttt", default: false) != false;
```

#### TTT specific imports and show rules

You can have typst specific imports and show rules like this

```typ
#let show_rules = if is_ttt {
  it => {
    show: it => highlights(ignore_fill_width: true, use_light_theme: true, it)
    it
  }
} else {
  it => it
}
#show: show_rules

#let imports = if is_ttt {
  import "latex-overrides.typ"
  latex-overrides
} else {
  int
}
#import imports : *
```

imports specifically are very useful for overriding the behaviour of typst functions
that rely on state but which can be implemented in latex.

## Context

While full `context` translation is unlikely to ever work with this, it is still sometimes useful to have references in the document. To support this, `ttt` includes a `typst eval`-based escape hatch.

The idea is:

1. compile the document once in a mode that can evaluate Typst state
2. collect the values you want to materialize into a `ttt-state` metadata block
3. run the normal translation pass and substitute `ttt-eval` raw blocks from that collected state

This lets you keep a Typst-native authoring experience for stateful helpers, while still producing LaTeX from a mostly static translation pass.

For example, line-number references can be implemented like this:

```typst
#let result = state("ttt_eval", (:))

#let add(key, value) = {
  result.update(s => {
    s.insert(key, value)
    s
  })
}

#let emit = context {[#metadata(result.final()) <ttt-state>]}
```

In non-`ttt` mode, call `add` to store values that should appear in the final document. In `ttt` mode, emit a `ttt-state` raw block containing the lookup key, and place `emit` somewhere in the document so the collected state is available.

A representative lookup helper looks like this:

```typst
#let lookup_line(name) = {
  let key = "lookup_line(" + name + ")"
  if is_ttt {
    raw(lang: "ttt-eval", key)
  } else {
    context {
      let number = line_numbers.final().at(name, default: none)

      ttt-eval.add(key, str(number))

      if number == none {
        panic(name + " not found in dict. Current dict: ", line_numbers.get())
      }
      [#number]
    }
  }
}
```

## Limitations

The current fork covers the workflows I needed for my papers, and it has also drifted in a few places from the original version. The most notable remaining quirks are:

- Typst infers supplements for things like figures, while LaTeX does not. In Typst you can write `@fig1`, but in LaTeX you need `Figure~\ref{fig1}`. `ttt` currently relies on prefixes such as `fig:`, `tab:` and `lst:` to guess the right wording. With no prefix, a reference is assumed to be a citation.
- Anything relying on Typst `state` is hard to translate directly, because the compiler uses the Typst frontend but not full layout. The `ttt-eval` workflow is the workaround for that.
- Math is no longer handled by a single export path. The fork now tries to convert equations through Pandoc first, and falls back to rendering the math as a PDF when that fails. This is more robust for real-world papers, especially when Typst math contains constructs that do not map cleanly to LaTeX.
- Some Typst elements are still intentionally ignored or approximated when a faithful LaTeX equivalent is not practical.

# TODOs
- [x] Code blocks
- [x] Fix figure includes
