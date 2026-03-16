// assignment.typ — Typst equivalent of assignment.sty

// ─── state variables ───────────────────────────────────────────────
#let _is-assignment = state("is-assignment", false)
#let _due-date = state("due-date", none)

#let set-assignment() = _is-assignment.update(true)
#let set-duedate(d) = _due-date.update(d)

#let if-assignment(body) = {
  context {
    if _is-assignment.get() == true { body }
  }
}
#let if-not-assignment(body) = {
  context {
    if _is-assignment.get() != true { body }
  }
}

// ─── page & text setup ─────────────────────────────────────────────
// LaTeX geometry derived from assignment.sty:
//   top    = 1in + voffset(-5mm) + topmargin(0) + headheight(0) + headsep(0) = 20.4mm
//   left   = 1in + oddsidemargin(0mm) = 25.4mm
//   textwidth  = 165mm  →  right  = 210 - 25.4 - 165 = 19.6mm
//   textheight = 255mm  →  bottom = 297 - 20.4 - 255 = 21.6mm
#let apply-layout(body) = {
  set page(
    paper: "a4",
    margin: (
      top: 20.4mm,
      bottom: 21.6mm,
      left: 25.4mm,
      right: 19.6mm,
    ),
    numbering: "1",           // plain page style
    number-align: center,
  )
  set text(
    font: "New Computer Modern",
    size: 10pt,
    lang: "en",
  )
  // KOMA-Script scrartcl-like headings
  set heading(numbering: "1.")
  show heading.where(level: 1): it => {
    v(1.2em)
    block(text(size: 14pt, weight: "bold", it))
    v(0.6em)
  }
  show heading.where(level: 2): it => {
    v(1em)
    block(text(size: 12pt, weight: "bold", it))
    v(0.5em)
  }
  show heading.where(level: 3): it => {
    v(0.8em)
    block(text(size: 10.5pt, weight: "bold", it))
    v(0.4em)
  }
  set par(justify: true)
  set enum(indent: 0.5em, body-indent: 0.5em, spacing: 0.65em)
  set list(indent: 0.5em, body-indent: 0.5em, spacing: 0.65em)
  body
}

#set list(marker: ([•], [--], [\*]))
// ─── points annotation ─────────────────────────────────────────────
#let punkte(n) = {
  h(1fr)
  emph[(#n Points)]
}

// ─── series header ─────────────────────────────────────────────────
// Reproduces \serieheader{#1}{#2}{#3}{#4}{#5}{#6}
#let serieheader(
  course,        // #1  e.g. "High-Performance Computing Lab for CSE"
  year,          // #2  e.g. "2026"
  student,       // #3
  discussed,     // #4
  title,         // #5
  extra,         // #6  shown when not in assignment mode
) = {
  // suppress page number on first page
  set page(header: none, footer: none) if false // handled via counter below
  context {
    // --- ETH logo (40 % of text width) ---
    align(left, image("ETHlogo_13.pdf", width: 40%))
    v(0.3em)

    // --- first line: course + year, both bold ---
    block(width: 100%)[
      #set text(size: 12pt)
      #strong(course) #h(1fr) #strong(year)
    ]
    v(0.15em)

    // --- second line: student / discussed ---
    block(width: 100%)[
      #set text(size: 12pt)
      #student #h(1fr) #discussed
    ]

    v(0.8em)
    line(length: 100%, stroke: 0.4pt)
    v(0.2em)

    // --- title + due-date / extra ---
    block(width: 100%)[
      #set text(size: 14pt)
      #strong(title)
      #h(1fr)
      #set text(size: 12pt)
      #{
        if _is-assignment.get() == true {
          let d = _due-date.get()
          if d != none [Due date: ~ #d]
        } else {
          extra
        }
      }
    ]

    v(0.4em)
    line(length: 100%, stroke: 0.4pt)
    v(2.2em)
  }
}

// ─── LaTeX logo helper ─────────────────────────────────────────────
#let LaTeX = {
  set text(font: "New Computer Modern")
  [L]
  h(-0.36em)
  text(size: 0.7em, baseline: -0.2em)[A]
  h(-0.15em)
  [T]
  h(-0.1667em)
  text(baseline: 0.2em)[E]
  h(-0.125em)
  [X]
}

// ─── assignment policy box ─────────────────────────────────────────
#let assignmentpolicy() = {
  block(
    width: 100%,
    stroke: 0.8pt + black,
    inset: 0pt,
    radius: 0pt,
  )[
    // title bar
    #block(
      width: 100%,
      fill: black,
      inset: (x: 8pt, y: 5pt),
    )[
      #text(fill: white, weight: "bold", size: 10pt)[HPC Lab for CSE 2025 --- Submission Instructions]
    ]
    // body
    #v(-1.2em)
    #block(
      width: 100%,
      fill: luma(230),
      inset: (x: 8pt, y: 5pt),
      stroke: (rest: black + 0.8pt, top: none)
    )[
      #set text(size: 10pt)
      #set par(justify: true)
      #set list(indent: 0.5em, body-indent: 0.5em, spacing: 0.65em)
      #set enum(indent: 0.5em, body-indent: 0.5em, spacing: 0.65em)
      #set list(marker: ([•], [--], [\*]))

      *Following these instructions is mandatory*: Submissions that do not
      comply *will not be considered*.

      - *Submission Platform*:
        Assignments must be submitted via
        #link("https://sam-up.math.ethz.ch/?lecture=401-3670-00")[SAM-UP].

      - *Required Files*: Your submission must include:
        - *Report*: A PDF summarizing your results and observations,
          based on this #LaTeX template.
          - File name:
            `project_number_lastname_firstname.pdf`
          - You are allowed to discuss all questions with anyone you like;
            however: (i) your submission must list anyone you discussed
            problems with and (ii) you must write up your submission
            independently.
        - *Source Code Archive*: A *compressed tar archive*,
          named `project_number_lastname_firstname.tar.gz`,
          containing:
          - All source code files.
          - Build files and scripts (e.g., `Makefile`, batch job
            scripts, plot scripts, etc).
            If you modified any provided files, ensure they still work as
            expected.
          - The full #LaTeX source of your report including all needed
            files to build the report (figures, listings, etc).
        - *Size Limit*: The total submission (PDF + archive) must not
          exceed *25 MB*.
        - *File Naming Rules*: Use only `ASCII` characters
          (avoid spaces, special symbols, and non-English letters).

      - *Grading Process*: The TAs will evaluate your project based on:
        - Your report write-up.
        - Your code implementation.
        - Benchmarking your code's performance.
    ]
  ]
}
