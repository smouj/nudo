#import "tokens.typ": *
#import "cover.typ": cover
#import "layout.typ": *

#cover()

#set page(
  paper: "a4",
  margin: (top: 18mm, bottom: 20mm, left: 22mm, right: 18mm),
  fill: paper,
  header: context [
    #set text(font: ("IBM Plex Mono", "DejaVu Sans Mono"), size: 7pt, fill: graphite)
    NUDO // TECHNICAL BOOK #h(1fr) ENGINEERING PAPER EDITION
  ],
  footer: context [
    #set text(font: ("IBM Plex Mono", "DejaVu Sans Mono"), size: 7pt, fill: graphite)
    NUDO-BOOK-001 // PRE-ALPHA #h(1fr) #counter(page).display("1")
  ]
)

#outline(title: [Contents], depth: 2)
#pagebreak()
#include "generated/book-content.typ"
