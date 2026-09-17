#import "tokens.typ": *
#import "brand.typ": brand-mark
#let cover() = {
  set page(margin: 0mm, fill: paper-warm)
  block(width: 100%, height: 100%, inset: 20mm, stroke: 10mm + ink)[
    #block(width: 100%, height: 100%, inset: 7mm, stroke: .7pt + rust)[
      #set text(font: ("IBM Plex Mono", "Cascadia Mono", "DejaVu Sans Mono"), fill: brand-blue, size: 10pt, weight: 700)
      #brand-mark() #h(6pt) // TECHNICAL PUBLICATION

      #v(62mm)
      #text(font: ("IBM Plex Sans", "DejaVu Sans"), size: 46pt, weight: 900, fill: ink)[NUDO]
      #linebreak()
      #text(font: ("IBM Plex Sans", "DejaVu Sans"), size: 18pt, fill: graphite)[The Technical Book]
      #v(9mm)
      #text(font: ("IBM Plex Mono", "DejaVu Sans Mono"), size: 8.5pt, weight: 700, fill: rust)[ENGINEERING PAPER EDITION]
      #linebreak()
      #text(font: ("IBM Plex Mono", "DejaVu Sans Mono"), size: 8.5pt, fill: graphite)[A programming language for humans and agents]

      #v(1fr)
      #table(columns: (1.2fr, 1fr, 1fr), stroke: .6pt + graphite,
        inset: 7pt,
        [DOCUMENT\\#strong[NUDO-BOOK-001]], [REVISION\\#strong[R0]], [STATUS\\#strong[LIVING / PRE-ALPHA]])
    ]
  ]
  pagebreak()
}
