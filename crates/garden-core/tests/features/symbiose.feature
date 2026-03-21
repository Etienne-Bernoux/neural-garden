# language: fr

Fonctionnalité: Symbiose mycorhizienne

  Scénario: Deux plantes forment un lien quand leurs racines se chevauchent
    Soit une île avec du sol riche
    Et deux plantes voisines dont les racines se chevauchent
    Et les deux plantes ont un signal de connexion fort
    Quand la simulation avance de 5 ticks
    Alors un lien mycorhizien existe entre les deux plantes

  Scénario: Les plantes liées échangent de l'énergie
    Soit une île avec du sol riche
    Et deux plantes liées par un réseau mycorhizien
    Et la première plante a beaucoup d'énergie et la seconde peu
    Quand la simulation avance de 10 ticks
    Alors l'écart d'énergie entre les deux plantes a diminué

  Scénario: Un lien se rompt quand les racines ne se chevauchent plus
    Soit deux plantes liées par un réseau mycorhizien
    Quand une plante perd sa racine partagée
    Alors le lien mycorhizien est rompu
