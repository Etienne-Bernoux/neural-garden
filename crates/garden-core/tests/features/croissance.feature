# language: fr

Fonctionnalité: Croissance des plantes

  Scénario: Une graine germe quand le sol est riche
    Soit une île avec du sol riche en carbone et azote
    Et une graine plantée sur ce sol
    Quand la simulation avance de 10 ticks
    Alors la graine a germé et pousse

  Scénario: Une graine meurt si le sol reste pauvre trop longtemps
    Soit une île avec du sol très pauvre
    Et une graine plantée sur ce sol
    Quand la simulation avance de 250 ticks
    Alors la graine est morte

  Scénario: Une plante grandit en ajoutant des cellules canopée
    Soit une île avec du sol riche
    Et une plante germée avec de l'énergie
    Quand la simulation avance de 50 ticks
    Alors la plante a plus d'une cellule de canopée

  Scénario: Une plante développe ses racines
    Soit une île avec du sol riche
    Et une plante germée avec de l'énergie
    Quand la simulation avance de 50 ticks
    Alors la plante a plus d'une cellule de racines

  Scénario: Une plante atteint la maturité
    Soit une île avec du sol très riche
    Et une plante germée avec beaucoup d'énergie
    Quand la simulation avance de 200 ticks
    Alors la plante est mature
