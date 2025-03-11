#import "sub.typ" : sub

#set heading(numbering: "1.")

= Typst (paritally) rendered by LaTeX <sec:label>

@fig:figure contains the source code for this document!

#lorem(20)

@sec:label has some cool stuff in it!


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
