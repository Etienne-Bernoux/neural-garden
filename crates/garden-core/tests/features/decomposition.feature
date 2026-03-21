# language: fr

Fonctionnalité: Décomposition des plantes mortes

  Scénario: Une plante morte enrichit progressivement le sol
    Soit une île avec du sol pauvre
    Et une plante morte en décomposition sur ce sol
    Quand la simulation avance de 30 ticks
    Alors le sol sous la plante est plus riche qu'avant

  Scénario: La décomposition s'étale sur plusieurs ticks
    Soit une plante morte en décomposition
    Quand la simulation avance de 10 ticks
    Alors la plante est toujours en décomposition
    Quand la simulation avance de 50 ticks supplémentaires
    Alors la plante est complètement décomposée
