#import "sub.typ" : sub

#let emit-latex(content, latex) = {
  raw(lang: "latexraw", latex)
}

#set page(margin: 1.75in)
#set par(leading: 0.55em, spacing: 0.55em, first-line-indent: 1.8em, justify: true)
#set text(font: "New Computer Modern")
#set figure(placement: top)
#show heading: set block(above: 1.4em, below: 1em)

#set heading(numbering: "1")

= Typst (paritally) rendered by LaTeX <sec:label>

@fig:figure contains the source code for this document! $integral f(x) d x$

#lorem(20)

@sec:label has some cool stuff in it!

#emit-latex([*bold*], "\\textbf{bold}");

// I can even customize with show rules!
#show raw.where(lang: "typst"): it => {
  show regex("[Ii] can even customize"): set text(
    fill: gradient.linear(..color.map.rainbow
  ))
  show regex("[Ww]ith show rules!"): set text(
    fill: gradient.linear(..color.map.rainbow), weight: "bold"
  )
  it
}

#figure(
  raw(read("main.typ"), block: true, lang: "typst"),
  caption: [But the caption is!]
) <fig:figure>

#sub()
