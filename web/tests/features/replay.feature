# language: fr

Fonctionnalité: Lecture de replay

  Scénario: Le viewer charge et affiche le terrain
    Soit le viewer est ouvert
    Alors le canvas Three.js est visible
    Et le panneau de stats est visible

  Scénario: Les stats s'affichent correctement
    Soit le viewer est ouvert
    Alors la population affichée est supérieure à 0
    Et la saison est affichée

  Scénario: Le play lance la lecture du replay
    Soit le viewer est ouvert
    Quand je clique sur le bouton play
    Et j'attends 2 secondes
    Alors le tick affiché est supérieur à 0

  Scénario: La navigation entre clips fonctionne
    Soit le viewer est ouvert
    Alors l'info clip affiche "Clip 1/1"

  Scénario: Le panneau plante apparaît au clic sur le canvas
    Soit le viewer est ouvert
    Quand je clique sur le canvas
    Alors le panneau de la plante sélectionnée est visible ou caché
