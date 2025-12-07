use core::hash::{BuildHasherDefault, Hasher};

use byteorder::{ByteOrder, NativeEndian};
use hashbrown::{HashMap, HashSet};

use super::Estr;

/// A standard `HashMap` using `Estr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type EstrMap<V> = HashMap<Estr, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `HashSet` using `Estr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type EstrSet = HashSet<Estr, BuildHasherDefault<IdentityHasher>>;

/// The worst hasher in the world -- the identity hasher.
#[doc(hidden)]
#[derive(Default)]
pub struct IdentityHasher {
    hash: u64,
}

impl Hasher for IdentityHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() == 8 {
            self.hash = NativeEndian::read_u64(bytes);
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}
