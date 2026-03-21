# language: fr

Fonctionnalité: Exploration et Viewer V2

  Scénario: Le viewer affiche des plantes de formes différentes
    Soit le viewer est ouvert
    Quand je clique sur le bouton play
    Et j'attends 3 secondes
    Alors je capture une vue d'ensemble

  Scénario: Le mode exploration s'active avec la touche V
    Soit le viewer est ouvert
    Quand je presse la touche V
    Et j'attends 1 secondes
    Alors je capture la vue exploration

  Scénario: Le joueur avance avec WASD en exploration
    Soit le viewer est ouvert
    Quand je presse la touche V
    Et je clique sur le canvas pour le pointer lock
    Et je maintiens W pendant 2 secondes
    Alors je capture après déplacement

  Scénario: Le brain-viz s'affiche avec la touche B
    Soit le viewer est ouvert
    Quand je presse la touche B
    Alors le panneau cerveau est visible
    Et je capture le brain-viz
