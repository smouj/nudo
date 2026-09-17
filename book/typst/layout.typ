#import "tokens.typ": *
#set page(
  paper: "a4",
  margin: (top: 18mm, bottom: 20mm, left: 22mm, right: 18mm),
  fill: paper,
  header: context if counter(page).get().first() > 1 [
    #set text(font: ("IBM Plex Mono", "Cascadia Mono", "DejaVu Sans Mono"), size: 7pt, fill: graphite)
    NUDO // TECHNICAL BOOK
    #h(1fr)
    ENGINEERING PAPER EDITION
  ],
  footer: context if counter(page).get().first() > 1 [
    #set text(font: ("IBM Plex Mono", "Cascadia Mono", "DejaVu Sans Mono"), size: 7pt, fill: graphite)
    NUDO-BOOK-001 // PRE-ALPHA
    #h(1fr)
    #counter(page).display("1")
  ],
)
#set text(font: ("IBM Plex Serif", "Libertinus Serif", "DejaVu Serif"), size: 10.2pt, fill: ink, lang: "en")
#set par(justify: true, leading: .62em)
#set heading(numbering: none)
#show heading.where(level: 1): it => {
  v(8pt)
  text(font: ("IBM Plex Sans", "DejaVu Sans"), size: 25pt, weight: 800, fill: ink)[
    #text(fill: brand-blue, font: ("IBM Plex Mono", "DejaVu Sans Mono"))[// ]#it.body
  ]
  v(5pt)
  line(length: 100%, stroke: .7pt + rule)
  v(8pt)
}
#show heading.where(level: 2): it => {
  v(10pt)
  text(font: ("IBM Plex Sans", "DejaVu Sans"), size: 15pt, weight: 700)[#text(fill: rust)[§ ]#it.body]
  v(4pt)
}
#show heading.where(level: 3): it => {
  v(7pt)
  text(font: ("IBM Plex Sans", "DejaVu Sans"), size: 11pt, weight: 700, it.body)
  v(3pt)
}
#show raw: it => block(
  inset: 9pt,
  fill: rgb("23211e"),
  stroke: .6pt + black,
  width: 100%,
  breakable: false,
  text(font: ("IBM Plex Mono", "Cascadia Mono", "DejaVu Sans Mono"), size: 8pt, fill: rgb("f0eadc"), it)
)
#set table(stroke: .45pt + rule)
