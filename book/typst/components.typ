#import "tokens.typ": *

#let document-stamp(body) = block(
  inset: 7pt,
  stroke: .7pt + rust,
  fill: paper-warm.lighten(20%),
  radius: 0pt,
  text(font: ("IBM Plex Mono", "Cascadia Mono", "DejaVu Sans Mono"), size: 8pt, fill: rust, body)
)

#let callout(kind, body) = block(
  width: 100%,
  inset: (left: 10pt, right: 9pt, top: 8pt, bottom: 8pt),
  stroke: (left: 2.4pt + brand-blue),
  fill: white.transparentize(68%),
  [#strong(kind) #h(6pt) #body]
)
