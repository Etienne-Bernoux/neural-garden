# language: fr

Fonctionnalité: Invasion entre plantes

  Scénario: Une plante envahit une cellule d'un voisin plus faible
    Soit une île avec deux plantes adjacentes
    Et la plante A a beaucoup plus d'énergie que la plante B
    Quand la plante A tente de pousser vers la cellule de B
    Alors la plante A a pris la cellule de B

  Scénario: L'invasion échoue contre un défenseur plus fort
    Soit une île avec deux plantes adjacentes
    Et la plante B a plus d'énergie que la plante A
    Quand la plante A tente de pousser vers la cellule de B
    Alors la cellule appartient toujours à B

  Scénario: L'invasion rompt le lien mycorhizien entre les deux plantes
    Soit deux plantes liées par un réseau mycorhizien
    Et la plante A a beaucoup plus d'énergie que la plante B
    Quand la plante A envahit une cellule de B
    Alors le lien mycorhizien entre A et B est rompu
