//! SHA-512 domain-separated public transcript and salted Merkle oracles.
//! Arithmetic/proof soundness does not rely on an outer signature quorum.
use crate::field::{encode, F};
use ark_ff::PrimeField;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::{BTreeMap, BTreeSet};

pub type Hash = [u8; 64];
pub fn digest(bytes: &[u8]) -> Hash {
    Sha512::digest(bytes).into()
}

#[derive(Clone)]
pub struct Transcript(Sha512);
impl Transcript {
    pub fn new(statement: &[u8]) -> Self {
        let mut result = Self(Sha512::new());
        result.absorb(b"protocol", b"zkfmi-cocode-independent-normalized-v1");
        result.absorb(b"statement", statement);
        result
    }
    pub fn absorb(&mut self, label: &[u8], bytes: &[u8]) {
        self.0.update((label.len() as u64).to_le_bytes());
        self.0.update(label);
        self.0.update((bytes.len() as u64).to_le_bytes());
        self.0.update(bytes);
    }
    pub fn field(&mut self, label: &[u8]) -> F {
        self.absorb(b"challenge", label);
        let bytes: Hash = self.0.clone().finalize().into();
        self.absorb(b"answer", &bytes);
        F::from_le_bytes_mod_order(&bytes)
    }
    pub fn fields(&mut self, label: &[u8], count: usize) -> Vec<F> {
        (0..count)
            .map(|i| {
                self.absorb(b"index", &(i as u64).to_le_bytes());
                self.field(label)
            })
            .collect()
    }
    pub fn absorb_fields(&mut self, label: &[u8], values: &[F]) {
        self.absorb(
            label,
            &values.iter().flat_map(|x| encode(*x)).collect::<Vec<_>>(),
        );
    }
    pub fn view(&self) -> Hash {
        self.0.clone().finalize().into()
    }
    pub fn query(&mut self, upper: usize) -> usize {
        assert!(upper.is_power_of_two());
        self.absorb(b"query", &(upper as u64).to_le_bytes());
        let value = self.view();
        self.absorb(b"query-result", &value);
        (u64::from_le_bytes(value[..8].try_into().unwrap()) as usize) & (upper - 1)
    }
}

fn leaf_hash(domain: &[u8], index: usize, value: F, salt: &Hash) -> Hash {
    let mut h = Sha512::new();
    h.update(b"cocode-leaf-v1");
    h.update((domain.len() as u64).to_le_bytes());
    h.update(domain);
    h.update((index as u64).to_le_bytes());
    h.update(encode(value));
    h.update(salt);
    h.finalize().into()
}
fn parent(left: &Hash, right: &Hash) -> Hash {
    let mut h = Sha512::new();
    h.update(b"cocode-node-v1");
    h.update(left);
    h.update(right);
    h.finalize().into()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Opening {
    pub value: String,
    pub salt: String,
    pub siblings: Vec<String>,
}

/// Canonical multiproof for a verifier-derived set of query positions. Shared
/// branches are emitted once; the underlying salted leaf/node hash is unchanged.
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BatchLeaf {
    pub index: usize,
    pub value: String,
    pub salt: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BatchOpening {
    pub leaves: Vec<BatchLeaf>,
    /// Level order, then increasing queried-node index; only absent siblings.
    pub siblings: Vec<String>,
}

pub struct Merkle {
    domain: Vec<u8>,
    values: Vec<F>,
    salts: Vec<Hash>,
    levels: Vec<Vec<Hash>>,
}
impl Merkle {
    pub fn new(domain: Vec<u8>, values: Vec<F>) -> Self {
        assert!(values.len().is_power_of_two());
        let salts: Vec<Hash> = (0..values.len())
            .map(|_| {
                let mut salt = [0; 64];
                OsRng.fill_bytes(&mut salt);
                salt
            })
            .collect();
        Self::with_salts(domain, values, salts)
    }
    /// Owner-local recovery of exactly the same persisted book commitment.
    pub fn with_salts(domain: Vec<u8>, values: Vec<F>, salts: Vec<Hash>) -> Self {
        assert!(values.len().is_power_of_two());
        assert_eq!(values.len(), salts.len());
        let first = values
            .iter()
            .zip(&salts)
            .enumerate()
            .map(|(i, (v, s))| leaf_hash(&domain, i, *v, s))
            .collect::<Vec<_>>();
        let mut levels = vec![first];
        while levels.last().unwrap().len() > 1 {
            let next = levels
                .last()
                .unwrap()
                .chunks_exact(2)
                .map(|p| parent(&p[0], &p[1]))
                .collect();
            levels.push(next);
        }
        Self {
            domain,
            values,
            salts,
            levels,
        }
    }
    pub fn root(&self) -> String {
        hex::encode(self.levels.last().unwrap()[0])
    }
    pub fn salts(&self) -> &[Hash] {
        &self.salts
    }
    pub fn values(&self) -> &[F] {
        &self.values
    }
    pub fn open(&self, index: usize) -> Opening {
        let mut at = index;
        let siblings = self.levels[..self.levels.len() - 1]
            .iter()
            .map(|level| {
                let item = hex::encode(level[at ^ 1]);
                at /= 2;
                item
            })
            .collect();
        Opening {
            value: hex::encode(encode(self.values[index])),
            salt: hex::encode(self.salts[index]),
            siblings,
        }
    }
    pub fn domain(&self) -> &[u8] {
        &self.domain
    }

    pub fn open_many(&self, indices: &[usize]) -> Result<BatchOpening, String> {
        let mut positions: BTreeSet<usize> = indices.iter().copied().collect();
        if positions.is_empty() || positions.iter().any(|i| *i >= self.values.len()) {
            return Err("Merkle batch query bounds".into());
        }
        let leaves = positions
            .iter()
            .map(|index| BatchLeaf {
                index: *index,
                value: hex::encode(encode(self.values[*index])),
                salt: hex::encode(self.salts[*index]),
            })
            .collect();
        let mut siblings = Vec::new();
        for level in &self.levels[..self.levels.len() - 1] {
            for index in &positions {
                if !positions.contains(&(index ^ 1)) {
                    siblings.push(hex::encode(level[index ^ 1]));
                }
            }
            positions = positions.into_iter().map(|index| index / 2).collect();
        }
        Ok(BatchOpening { leaves, siblings })
    }
}

pub fn verify_many(
    root: &str,
    domain: &[u8],
    size: usize,
    expected_indices: &[usize],
    proof: &BatchOpening,
) -> Result<BTreeMap<usize, F>, String> {
    let expected: BTreeSet<usize> = expected_indices.iter().copied().collect();
    if !size.is_power_of_two()
        || expected.is_empty()
        || expected.iter().any(|i| *i >= size)
        || proof.leaves.len() != expected.len()
        || proof.siblings.len()
            > expected
                .len()
                .saturating_mul(size.trailing_zeros() as usize)
    {
        return Err("Merkle multiproof shape".into());
    }
    let decode_hash = |encoded: &str| -> Result<Hash, String> {
        let bytes: Hash = hex::decode(encoded)
            .map_err(|_| "Merkle hash hex")?
            .try_into()
            .map_err(|_| "Merkle hash width")?;
        if hex::encode(bytes) != encoded {
            return Err("noncanonical Merkle hash".into());
        }
        Ok(bytes)
    };
    let expected_root = decode_hash(root)?;
    let mut values = BTreeMap::new();
    let mut nodes = BTreeMap::new();
    for (index, leaf) in expected.into_iter().zip(&proof.leaves) {
        if index != leaf.index {
            return Err("Merkle multiproof query set".into());
        }
        let value = crate::algebra::unpack(&leaf.value, 1)?[0];
        let salt = decode_hash(&leaf.salt)?;
        nodes.insert(index, leaf_hash(domain, index, value, &salt));
        values.insert(index, value);
    }
    let mut siblings = proof.siblings.iter();
    for _ in 0..size.trailing_zeros() {
        let mut parents = BTreeMap::new();
        for (&index, node) in &nodes {
            let sibling = match nodes.get(&(index ^ 1)) {
                Some(sibling) => *sibling,
                None => decode_hash(siblings.next().ok_or("missing Merkle batch sibling")?)?,
            };
            let hash = if index & 1 == 0 {
                parent(node, &sibling)
            } else {
                parent(&sibling, node)
            };
            if parents
                .insert(index / 2, hash)
                .is_some_and(|previous| previous != hash)
            {
                return Err("inconsistent Merkle shared branch".into());
            }
        }
        nodes = parents;
    }
    if siblings.next().is_some() || nodes.len() != 1 || nodes.get(&0) != Some(&expected_root) {
        return Err("Merkle multiproof root or trailing data".into());
    }
    Ok(values)
}

pub fn verify(
    root: &str,
    domain: &[u8],
    size: usize,
    index: usize,
    opening: &Opening,
) -> Result<F, String> {
    if !size.is_power_of_two()
        || index >= size
        || opening.siblings.len() != size.trailing_zeros() as usize
    {
        return Err("Merkle path shape mismatch".into());
    }
    let value = crate::algebra::unpack(&opening.value, 1)?[0];
    let salt: Hash = hex::decode(&opening.salt)
        .map_err(|_| "salt hex")?
        .try_into()
        .map_err(|_| "salt length")?;
    let mut current = leaf_hash(domain, index, value, &salt);
    let mut at = index;
    for sibling in &opening.siblings {
        let other: Hash = hex::decode(sibling)
            .map_err(|_| "sibling hex")?
            .try_into()
            .map_err(|_| "sibling length")?;
        current = if at & 1 == 0 {
            parent(&current, &other)
        } else {
            parent(&other, &current)
        };
        at /= 2;
    }
    if hex::encode(current) != root {
        return Err("Merkle root mismatch".into());
    }
    Ok(value)
}
