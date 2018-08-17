use std::collections::HashMap;
use std::hash::Hash;


pub fn vecs2map<K: Eq + Hash, V>(keys: Vec<K>, values: Vec<V>) -> HashMap<K, V>
{
    let mut ret = HashMap::new();
    keys.into_iter().zip(values.into_iter())
        .for_each(|(key, value)| { ret.insert(key, value); });
    ret
}
