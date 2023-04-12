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

impl<K: Clone + Ord, V: Clone + PartialEq> SimpleDiff<K, V> {
    // Generates a diff
    pub fn generate(a: &BTreeMap<K, V>, b: &BTreeMap<K, V>) -> Self {
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

    pub fn apply_to(&self, apply_to: &mut BTreeMap<K, V>) {
        self.removed.iter().for_each(|del| {
            apply_to.remove(del);
        });
        for (key, change) in &self.altered {
            apply_to.insert(key.clone(), change.clone());
        }
    }
}
