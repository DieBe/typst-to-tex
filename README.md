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

While full `context` translation is unlikely to ever work with this, it is still sometimes
useful to have references in the document. To support this, `ttt` supports a `typst eval` based
approach.

This system is based on three parts. First
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
which I will put on typst universe eventually, but for now, just put it in your project.

In non-ttt mode (see above), use the `add` function to add any values you want to render in the final document. In `ttt` mode, emit a `ttt-state` raw block, containing the key that you added with `add`. Finally, put the `emit` part somewhere in your document where it will be rendered.

For example, here is how line number references can be implemented

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

Currently, the system supports what I've needed for my papers, most other things are relatively easy to add support for, it just needs someone to do it. There are some exceptions however:

- Typst infers supplements for things like figures, while latex does not. I.e. you can write `@fig1` in typst but need to write `Figure~\ref{fig1}` in latex. I have not found a way to infer the supplement, so for now, you have to prefix your references with `fig:`, `tab:` and `lst:`. With no prefix, it is assumed that the reference is a citation
- Anything using `state` in typst will be very difficult to support. Currently, the compiler uses the typst frontend but does not do layout, state is resolved during layout which means this technique does not work.

# TODOs
- [x] Code blocks
- [x] Fix figure includes