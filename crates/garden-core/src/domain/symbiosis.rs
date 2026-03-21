/// Lien mycorhizien entre deux plantes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MycorrhizalLink {
    plant_a: u64,
    plant_b: u64,
}

impl MycorrhizalLink {
    /// Cree un nouveau lien, normalisant l'ordre pour que le plus petit id vienne en premier.
    pub fn new(plant_a: u64, plant_b: u64) -> Self {
        if plant_a <= plant_b {
            Self { plant_a, plant_b }
        } else {
            Self {
                plant_a: plant_b,
                plant_b: plant_a,
            }
        }
    }

    pub fn plant_a(&self) -> u64 {
        self.plant_a
    }

    pub fn plant_b(&self) -> u64 {
        self.plant_b
    }

    pub fn contains(&self, plant_id: u64) -> bool {
        self.plant_a == plant_id || self.plant_b == plant_id
    }

    pub fn other(&self, plant_id: u64) -> Option<u64> {
        if self.plant_a == plant_id {
            Some(self.plant_b)
        } else if self.plant_b == plant_id {
            Some(self.plant_a)
        } else {
            None
        }
    }
}

/// Reseau de liens mycorhiziens entre plantes.
pub struct SymbiosisNetwork {
    links: Vec<MycorrhizalLink>,
}

impl Default for SymbiosisNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbiosisNetwork {
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }

    /// Reconstruit un reseau a partir d'une liste de liens.
    /// Utilise pour la deserialisation.
    pub(crate) fn from_links(links: Vec<MycorrhizalLink>) -> Self {
        Self { links }
    }

    /// Retourne une reference sur tous les liens.
    pub fn links(&self) -> &[MycorrhizalLink] {
        &self.links
    }

    /// Cree un lien s'il n'existe pas deja. Retourne true si cree.
    pub fn create_link(&mut self, plant_a: u64, plant_b: u64) -> bool {
        if plant_a == plant_b {
            return false;
        }
        let link = MycorrhizalLink::new(plant_a, plant_b);
        if self.links.contains(&link) {
            return false;
        }
        self.links.push(link);
        true
    }

    /// Supprime un lien s'il existe. Retourne true si supprime.
    pub fn remove_link(&mut self, plant_a: u64, plant_b: u64) -> bool {
        let link = MycorrhizalLink::new(plant_a, plant_b);
        if let Some(pos) = self.links.iter().position(|l| *l == link) {
            self.links.swap_remove(pos);
            true
        } else {
            false
        }
    }

    /// Supprime tous les liens impliquant une plante. Retourne les liens supprimes.
    pub fn remove_plant(&mut self, plant_id: u64) -> Vec<MycorrhizalLink> {
        let mut removed = Vec::new();
        let mut i = 0;
        while i < self.links.len() {
            if self.links[i].contains(plant_id) {
                removed.push(self.links.swap_remove(i));
            } else {
                i += 1;
            }
        }
        removed
    }

    /// Tous les liens impliquant une plante.
    pub fn links_of(&self, plant_id: u64) -> Vec<&MycorrhizalLink> {
        self.links.iter().filter(|l| l.contains(plant_id)).collect()
    }

    /// Verifie si deux plantes sont liees.
    pub fn are_linked(&self, plant_a: u64, plant_b: u64) -> bool {
        let link = MycorrhizalLink::new(plant_a, plant_b);
        self.links.contains(&link)
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }
}

/// Calcule le montant a transferer entre deux plantes.
/// Retourne (montant_a_vers_b, montant_b_vers_a) base sur la difference d'energie.
pub fn calculate_exchange(energy_a: f32, energy_b: f32, transfer_rate: f32) -> (f32, f32) {
    let diff = energy_a - energy_b;
    if diff > 0.0 {
        (diff * transfer_rate, 0.0)
    } else {
        (0.0, (-diff) * transfer_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_lien_normalise_lordre() {
        let link = MycorrhizalLink::new(5, 3);
        assert_eq!(link.plant_a(), 3);
        assert_eq!(link.plant_b(), 5);
    }

    #[test]
    fn le_lien_contient_les_plantes() {
        let link = MycorrhizalLink::new(5, 3);
        assert!(link.contains(3));
        assert!(link.contains(5));
        assert!(!link.contains(7));
    }

    #[test]
    fn le_lien_retourne_lautre_plante() {
        let link = MycorrhizalLink::new(5, 3);
        assert_eq!(link.other(3), Some(5));
        assert_eq!(link.other(5), Some(3));
        assert_eq!(link.other(7), None);
    }

    #[test]
    fn le_reseau_cree_un_lien() {
        let mut net = SymbiosisNetwork::new();
        assert!(net.create_link(1, 2));
        assert_eq!(net.link_count(), 1);
    }

    #[test]
    fn le_reseau_refuse_les_liens_dupliques() {
        let mut net = SymbiosisNetwork::new();
        assert!(net.create_link(1, 2));
        assert!(!net.create_link(1, 2));
        assert!(!net.create_link(2, 1));
        assert_eq!(net.link_count(), 1);
    }

    #[test]
    fn le_reseau_refuse_les_auto_liens() {
        let mut net = SymbiosisNetwork::new();
        assert!(!net.create_link(3, 3));
        assert_eq!(net.link_count(), 0);
    }

    #[test]
    fn le_reseau_supprime_un_lien() {
        let mut net = SymbiosisNetwork::new();
        net.create_link(1, 2);
        assert!(net.remove_link(1, 2));
        assert_eq!(net.link_count(), 0);
    }

    #[test]
    fn le_reseau_supprime_une_plante() {
        let mut net = SymbiosisNetwork::new();
        net.create_link(1, 2);
        net.create_link(1, 3);
        net.create_link(1, 4);
        net.create_link(2, 3);
        let removed = net.remove_plant(1);
        assert_eq!(removed.len(), 3);
        assert_eq!(net.link_count(), 1);
    }

    #[test]
    fn le_reseau_detecte_les_liens() {
        let mut net = SymbiosisNetwork::new();
        net.create_link(1, 2);
        assert!(net.are_linked(1, 2));
        assert!(net.are_linked(2, 1));
        net.remove_link(1, 2);
        assert!(!net.are_linked(1, 2));
    }

    #[test]
    fn echange_energie_coule_du_haut_vers_le_bas() {
        let (a_to_b, b_to_a) = calculate_exchange(80.0, 20.0, 0.1);
        assert!((a_to_b - 6.0).abs() < f32::EPSILON);
        assert!((b_to_a - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn echange_equilibre_pas_de_transfert() {
        let (a_to_b, b_to_a) = calculate_exchange(50.0, 50.0, 0.1);
        assert!((a_to_b - 0.0).abs() < f32::EPSILON);
        assert!((b_to_a - 0.0).abs() < f32::EPSILON);
    }
}
