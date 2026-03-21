# language: fr

Fonctionnalité: Cycle saisonnier

  Scénario: L'hiver ralentit la croissance
    Soit une île avec du sol riche
    Et deux plantes identiques germées
    Et une plante vit un cycle en été et l'autre en hiver
    Quand chaque plante simule 100 ticks dans sa saison
    Alors la plante d'été a plus de biomasse que celle d'hiver

  Scénario: Le printemps accélère la régénération du sol
    Soit une île avec du sol à moitié riche
    Quand la simulation avance de 100 ticks au printemps
    Et la simulation avance de 100 ticks en hiver
    Alors le sol s'est plus régénéré au printemps qu'en hiver

  Scénario: Les plantes affamées meurent plus vite en hiver
    Soit une île avec du sol pauvre
    Et une plante avec très peu d'énergie
    Quand la simulation avance en hiver pendant 100 ticks
    Alors la plante est morte ou mourante
