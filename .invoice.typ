#set page(
  paper: "a4",
  margin: 8%,
)
#set text(font: "Roboto", 13pt)

#let table_color() = rgb(110, 140, 180, 205)
#let horiz_line_color() = rgb(53, 80, 220, 100)
#let sep_par() = 28pt

#grid(
  columns: (1fr, auto),
  align(left, text(23pt)[Timothée CERCUEIL]),
  align(right)[LOGO ICI]
)

#align(left, text(14pt)[
  5 rue du Rhin, 44470 Carquefou \
  timothee.cercueil\@pm.me \
  Entrepreneur Individuel \
  SIRET: 02340234023402340
])

#v(sep_par())

#grid(columns: (1fr, 1fr), column-gutter: 10%,
  align(left)[
    #text(17pt)[*Facturé à*] \
    Société CASEDI SARL \
    24 rue de la Fosse, 44470 Carquefou \
  ],
  align(right)[Numéro de facture *050-3192394* \
  Créée le *5 Janvier 2024* \
  Date de la prestation: *3 Janvier 2024*],
)

#v(sep_par())

#table(
  stroke: table_color(),
  columns: (3fr, 1fr, 1fr, 1fr),
  [*Prestation*], [*Nombre d'heures*], [*Prix de l'heure*], [*Total HT*],

  "Conseil en informatique", "5", "25€", "125€",
  "Gestion d'un conflit de version dans la base de donnée ", "3", "30€", "90€"
)

#v(sep_par())

#table(
  stroke: table_color(),
  columns: (auto, auto),
  [*Total HT*], [215€],
   [*TVA 20%*], [43€],
  // [*TVA non applicable* - article 293B du CGI], [],
  [*TOTAL TTC*], [258€]
)

#v(230pt)

#set text(font: "Spectral", 12pt)

#line(length: 100%, stroke: horiz_line_color())
Some terms applicable I think but I'm not sure
