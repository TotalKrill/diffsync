use super::*;
use crate::structs::SimpleDiffTrait;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConcMap<K: Eq + Ord + Hash, V: PartialEq>(pub DashMap<K, V>);

impl<K: Eq + Ord + Hash, V: PartialEq> Default for ConcMap<K, V> {
    fn default() -> Self {
        Self(DashMap::default())
    }
}

impl<K: Clone + Ord + Hash, V: PartialEq + Clone> ConcMap<K, V> {
    fn as_btree(&self) -> BTreeMap<K, V> {
        let mut bmap = BTreeMap::new();
        self.0.iter().for_each({
            |r| {
                // map into btree, heavy operation unfortunately
                let key = r.key().clone();
                let value = r.value().clone();
                bmap.insert(key, value);
            }
        });

        bmap
    }
}

impl<K: Ord + Hash + Clone, V: PartialEq + Hash + Clone> Hash for ConcMap<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bmap = self.as_btree();
        bmap.hash(state)
    }
}

impl<K: Ord + Hash + Clone, V: PartialEq + Hash + Clone> Diff for ConcMap<K, V> {
    type Repr = SimpleDiff<K, V>;

    fn diff(&self, other: &Self) -> Self::Repr {
        let s = self.as_btree();
        let o = other.as_btree();
        SimpleDiff::generate(&s, &o)
    }

    fn apply(&mut self, diff: &Self::Repr) {
        diff.removed.iter().for_each(|del| {
            self.0.remove(del);
        });
        for (key, change) in &diff.altered {
            self.0.insert(key.clone(), change.clone());
        }
    }

    fn identity() -> Self {
        Self(DashMap::new())
    }
}
