use std::collections::BTreeSet;

use super::*;

/// Simple diff implementation instead of the nesting one found in diff-struct,
/// that would run "diff" recursively down into
/// the value stored in the map

#[cfg_attr(feature = "impl_schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleDiff<K: Ord, V> {
    pub altered: BTreeMap<K, V>,
    pub removed: BTreeSet<K>,
}

impl<K: Ord, V> Default for SimpleDiff<K, V> {
    fn default() -> Self {
        Self {
            altered: BTreeMap::new(),
            removed: BTreeSet::new(),
        }
    }
}

impl<K: Ord, V> SimpleDiff<K, V> {
    pub fn new() -> Self {
        Self {
            altered: Default::default(),
            removed: Default::default(),
        }
    }
}

pub trait SimpleDiffTrait<T> {
    fn generate(a: &T, b: &T) -> Self;
    fn apply_to(&self, apply_to: &mut T);
}

// macro_rules! impl_map {
//     ($ty:ty) => {
//         impl<K: Clone + Ord, V: Clone + PartialEq> SimpleDiffTrait<$ty <K, V>> for SimpleDiff<K, V> {
//             fn generate(a: &$ty<K, V>, b: &$ty<K, V>) -> Self {
//                 let mut diff: SimpleDiff<K, V> = Default::default();

//                 // Check for alterations, dont nest into the value struct for diff
//                 for (key, value) in a.iter() {
//                     if let Some(other_value) = b.get(&key) {
//                         // don't store values that don't change
//                         if value != other_value {
//                             diff.altered.insert(key.clone(), other_value.clone());
//                         }
//                     } else {
//                         diff.removed.insert(key.clone());
//                     }
//                 }
//                 // Check what to remove
//                 for (key, value) in b.iter() {
//                     if let None = a.get(&key) {
//                         diff.altered.insert(key.clone(), value.clone());
//                     }
//                 }

//                 diff
//             }
//             fn apply_to(&self, apply_to: &mut $ty<K, V>) {
//                 self.removed.iter().for_each(|del| {
//                     apply_to.remove(del);
//                 });
//                 for (key, change) in &self.altered {
//                     apply_to.insert(key.clone(), change.clone());
//                 }
//             }
//         }
//     };
// }

// impl_map!(BTreeMap);
// impl_map!(DashMap);

impl<K: Clone + Ord, V: Clone + PartialEq> SimpleDiffTrait<BTreeMap<K, V>> for SimpleDiff<K, V> {
    fn generate(a: &BTreeMap<K, V>, b: &BTreeMap<K, V>) -> Self {
        let mut diff: SimpleDiff<K, V> = Default::default();

        // Check for alterations, dont nest into the value struct for diff
        for (key, value) in a.iter() {
            if let Some(other_value) = b.get(&key) {
                // don't store values that don't change
                if value != other_value {
                    diff.altered.insert(key.clone(), other_value.clone());
                }
            } else {
                diff.removed.insert(key.clone());
            }
        }
        // Check what to remove
        for (key, value) in b {
            if let None = a.get(&key) {
                diff.altered.insert(key.clone(), value.clone());
            }
        }

        diff
    }
    fn apply_to(&self, apply_to: &mut BTreeMap<K, V>) {
        self.removed.iter().for_each(|del| {
            apply_to.remove(del);
        });
        for (key, change) in &self.altered {
            apply_to.insert(key.clone(), change.clone());
        }
    }
}
impl<K: Hash + Clone + Ord, V: Clone + PartialEq> SimpleDiffTrait<ConcMap<K, V>>
    for SimpleDiff<K, V>
{
    fn generate(a: &ConcMap<K, V>, b: &ConcMap<K, V>) -> Self {
        let mut diff: SimpleDiff<K, V> = Default::default();

        // Check for alterations, dont nest into the value struct for diff
        for r in a.0.iter() {
            if let Some(other_value) = b.0.get(&r.key()) {
                // don't store values that don't change
                if r.value() != other_value.value() {
                    diff.altered.insert(r.key().clone(), other_value.clone());
                }
            } else {
                diff.removed.insert(r.key().clone());
            }
        }
        // Check what to remove
        for r in &b.0 {
            if let None = a.0.get(&r.key()) {
                diff.altered.insert(r.key().clone(), r.value().clone());
            }
        }

        diff
    }
    fn apply_to(&self, apply_to: &mut ConcMap<K, V>) {
        self.removed.iter().for_each(|del| {
            apply_to.0.remove(del);
        });
        for (key, change) in &self.altered {
            apply_to.0.insert(key.clone(), change.clone());
        }
    }
}

// impl<K: Clone + Ord, V: Clone + PartialEq> SimpleDiff<K, V> {
//     // Generates a diff
//     pub fn generate(a: &BTreeMap<K, V>, b: &BTreeMap<K, V>) -> Self {
//         let mut diff: SimpleDiff<K, V> = Default::default();

//         // Check for alterations, dont nest into the value struct for diff
//         for (key, value) in a.iter() {
//             if let Some(other_value) = b.get(&key) {
//                 // don't store values that don't change
//                 if value != other_value {
//                     diff.altered.insert(key.clone(), other_value.clone());
//                 }
//             } else {
//                 diff.removed.insert(key.clone());
//             }
//         }
//         // Check what to remove
//         for (key, value) in b {
//             if let None = a.get(&key) {
//                 diff.altered.insert(key.clone(), value.clone());
//             }
//         }

//         diff
//     }

//     pub fn apply_to(&self, apply_to: &mut BTreeMap<K, V>) {
//         self.removed.iter().for_each(|del| {
//             apply_to.remove(del);
//         });
//         for (key, change) in &self.altered {
//             apply_to.insert(key.clone(), change.clone());
//         }
//     }
// }
