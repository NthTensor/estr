#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

mod bumpalloc;
mod cfg;
mod stringcache;

mod platform {
    use crate::cfg;

    crossfig::switch! {
        cfg::std => {
            pub use parking_lot::Mutex;
        }
        cfg::spin => {
            pub use spin::lock_api::Mutex;
        }
        _ => {
            compile_error!("Must select a `Mutex` provider! Enable either the `std` or `spin` feature.");
        }
    }
}

use alloc::{borrow, boxed, fmt, rc, slice, string, sync};
use core::ops::Deref;
use core::str::FromStr;
use core::{cell, cmp, ptr};

use crate::platform::Mutex;
use crate::stringcache::*;

/// A handle representing a string in the global string cache.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Estr {
    char_ptr: ptr::NonNull<u8>,
}

impl Estr {
    /// Create a new `Estr` from the given `str`.
    ///
    /// You can also use the [`estr`] function.
    ///
    /// # Examples
    ///
    /// ```
    /// use estr::{Estr, estr};
    ///
    /// let e1 = Estr::from("the quick brown fox");
    /// let e2 = estr("the quick brown fox");
    /// assert_eq!(e1, e2);
    /// ```
    pub fn from(string: &str) -> Estr {
        let Digest { hash } = digest(string);
        let mut sc = STRING_CACHE[whichbin(hash)].lock();
        let ptr = sc.insert(string, hash);
        Estr {
            // SAFETY: sc.insert does not give back a null pointer
            char_ptr: unsafe { ptr::NonNull::new_unchecked(ptr as *mut _) },
        }
    }

    pub fn from_existing(string: &str) -> Option<Estr> {
        let Digest { hash } = digest(string);
        let sc = STRING_CACHE[whichbin(hash)].lock();
        sc.get_existing(string, hash).map(|ptr| Estr {
            char_ptr: unsafe { ptr::NonNull::new_unchecked(ptr as *mut _) },
        })
    }

    /// Get the cached `Estr` as a `str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use estr::estr;
    ///
    /// let e_fox = estr("the quick brown fox");
    /// let words: Vec<&str> = e_fox.as_str().split_whitespace().collect();
    /// assert_eq!(words, ["the", "quick", "brown", "fox"]);
    /// ```
    pub fn as_str(&self) -> &'static str {
        // This is safe if:
        // 1) self.char_ptr points to a valid address
        // 2) len is a usize stored usize aligned usize bytes before char_ptr
        // 3) char_ptr points to a valid UTF-8 string of len bytes.
        // All these are guaranteed by StringCache::insert() and by the fact
        // we can only construct a Estr from a valid &str.
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(self.char_ptr.as_ptr(), self.len()))
        }
    }

    /// Get a raw pointer to the `StringCacheEntry`.
    #[inline]
    fn as_string_cache_entry(&self) -> &StringCacheEntry {
        // The allocator guarantees that the alignment is correct and that
        // this pointer is non-null
        unsafe { &*(self.char_ptr.as_ptr().cast::<StringCacheEntry>().sub(1)) }
    }

    /// Get the length (in bytes) of this string.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_string_cache_entry().len
    }

    /// Get the precomputed hash for this string.
    #[inline]
    pub fn digest(&self) -> Digest {
        Digest {
            hash: self.as_string_cache_entry().hash,
        }
    }

    /// Get an owned String copy of this string.
    pub fn to_owned(&self) -> string::String {
        string::ToString::to_string(&self.as_str())
    }
}

// We're safe to impl these because the strings they reference are immutable
// and for all intents and purposes 'static since they're never deleted after
// being created
unsafe impl Send for Estr {}
unsafe impl Sync for Estr {}

impl PartialOrd for Estr {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Estr {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.digest().cmp(&other.digest())
    }
}

impl PartialEq<str> for Estr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Estr> for str {
    fn eq(&self, u: &Estr) -> bool {
        self == u.as_str()
    }
}

impl PartialEq<&str> for Estr {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<Estr> for &str {
    fn eq(&self, u: &Estr) -> bool {
        *self == u.as_str()
    }
}

impl PartialEq<&&str> for Estr {
    fn eq(&self, other: &&&str) -> bool {
        self.as_str() == **other
    }
}

impl PartialEq<Estr> for &&str {
    fn eq(&self, u: &Estr) -> bool {
        **self == u.as_str()
    }
}

impl PartialEq<string::String> for Estr {
    fn eq(&self, other: &string::String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Estr> for string::String {
    fn eq(&self, u: &Estr) -> bool {
        self == u.as_str()
    }
}

impl PartialEq<&string::String> for Estr {
    fn eq(&self, other: &&string::String) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<Estr> for &string::String {
    fn eq(&self, u: &Estr) -> bool {
        *self == u.as_str()
    }
}

impl PartialEq<boxed::Box<str>> for Estr {
    fn eq(&self, other: &boxed::Box<str>) -> bool {
        self.as_str() == &**other
    }
}

impl PartialEq<Estr> for boxed::Box<str> {
    fn eq(&self, u: &Estr) -> bool {
        &**self == u.as_str()
    }
}

impl PartialEq<Estr> for &boxed::Box<str> {
    fn eq(&self, u: &Estr) -> bool {
        &***self == u.as_str()
    }
}

impl PartialEq<borrow::Cow<'_, str>> for Estr {
    fn eq(&self, other: &borrow::Cow<'_, str>) -> bool {
        self.as_str() == &*other
    }
}

impl PartialEq<Estr> for borrow::Cow<'_, str> {
    fn eq(&self, u: &Estr) -> bool {
        &*self == u.as_str()
    }
}

impl PartialEq<&borrow::Cow<'_, str>> for Estr {
    fn eq(&self, other: &&borrow::Cow<'_, str>) -> bool {
        self.as_str() == &**other
    }
}

impl PartialEq<Estr> for &borrow::Cow<'_, str> {
    fn eq(&self, u: &Estr) -> bool {
        &**self == u.as_str()
    }
}

cfg::std! {
    use std::path::Path;
    use std::ffi::OsStr;

    impl PartialEq<Estr> for Path {
        fn eq(&self, u: &Estr) -> bool {
            self == Path::new(u)
        }
    }

    impl PartialEq<Estr> for &Path {
        fn eq(&self, u: &Estr) -> bool {
            *self == Path::new(u)
        }
    }

    impl PartialEq<Estr> for OsStr {
        fn eq(&self, u: &Estr) -> bool {
            self == OsStr::new(u)
        }
    }

    impl PartialEq<Estr> for &OsStr {
        fn eq(&self, u: &Estr) -> bool {
            *self == OsStr::new(u)
        }
    }
}

impl<T: ?Sized> AsRef<T> for Estr
where
    str: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.as_str().as_ref()
    }
}

impl FromStr for Estr {
    type Err = ();

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Estr::from(s))
    }
}

impl From<&str> for Estr {
    fn from(s: &str) -> Estr {
        Estr::from(s)
    }
}

impl From<Estr> for &'static str {
    fn from(s: Estr) -> &'static str {
        s.as_str()
    }
}

impl From<Estr> for string::String {
    fn from(u: Estr) -> Self {
        string::String::from(u.as_str())
    }
}

impl From<Estr> for boxed::Box<str> {
    fn from(u: Estr) -> Self {
        boxed::Box::from(u.as_str())
    }
}

impl From<Estr> for rc::Rc<str> {
    fn from(u: Estr) -> Self {
        rc::Rc::from(u.as_str())
    }
}

impl From<Estr> for sync::Arc<str> {
    fn from(u: Estr) -> Self {
        sync::Arc::from(u.as_str())
    }
}

impl From<Estr> for borrow::Cow<'static, str> {
    fn from(u: Estr) -> Self {
        borrow::Cow::Borrowed(u.as_str())
    }
}

impl From<string::String> for Estr {
    fn from(s: string::String) -> Estr {
        Estr::from(&s)
    }
}

impl From<&string::String> for Estr {
    fn from(s: &string::String) -> Estr {
        Estr::from(&**s)
    }
}

impl From<boxed::Box<str>> for Estr {
    fn from(s: boxed::Box<str>) -> Estr {
        Estr::from(&*s)
    }
}

impl From<rc::Rc<str>> for Estr {
    fn from(s: rc::Rc<str>) -> Estr {
        Estr::from(&*s)
    }
}

impl From<sync::Arc<str>> for Estr {
    fn from(s: sync::Arc<str>) -> Estr {
        Estr::from(&*s)
    }
}

impl From<borrow::Cow<'_, str>> for Estr {
    fn from(s: borrow::Cow<'_, str>) -> Estr {
        Estr::from(&*s)
    }
}

impl Default for Estr {
    fn default() -> Self {
        Estr::from("")
    }
}

impl Deref for Estr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for Estr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Estr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "u!({:?})", self.as_str())
    }
}

#[inline(always)]
pub const fn digest(string: &str) -> Digest {
    let hash = rapidhash::v3::rapidhash_v3_nano_inline::<true, false>;
    let seed = &rapidhash::v3::DEFAULT_RAPID_SECRETS;
    Digest {
        hash: hash(string.as_bytes(), seed),
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Digest {
    hash: u64,
}

impl Digest {
    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl PartialEq<Digest> for Estr {
    fn eq(&self, other: &Digest) -> bool {
        self.digest() == *other
    }
}

impl PartialOrd<Digest> for Estr {
    fn partial_cmp(&self, other: &Digest) -> Option<cmp::Ordering> {
        Some(self.digest().cmp(other))
    }
}

impl PartialEq<Estr> for Digest {
    fn eq(&self, other: &Estr) -> bool {
        *self == other.digest()
    }
}

impl PartialOrd<Estr> for Digest {
    fn partial_cmp(&self, other: &Estr) -> Option<cmp::Ordering> {
        Some(self.cmp(&other.digest()))
    }
}

/// Create a new `Estr` from the given `str`.
///
/// # Examples
///
/// ```
/// use estr::estr;
///
/// let e1 = estr("the quick brown fox");
/// let e2 = estr("the quick brown fox");
/// assert_eq!(e1, e2);
/// ```
#[inline]
pub fn estr(s: &str) -> Estr {
    Estr::from(s)
}

/// Create a new `Estr` from the given `str` but only if it already exists in
/// the string cache.
///
/// # Examples
///
/// ```
/// use estr::{estr, existing_estr};
///
/// let e1 = existing_estr("the quick brown fox");
/// let e2 = estr("the quick brown fox");
/// let e3 = existing_estr("the quick brown fox");
/// assert_eq!(e1, None);
/// assert_eq!(e3, Some(e2));
/// ```
#[inline]
pub fn existing_estr(s: &str) -> Option<Estr> {
    Estr::from_existing(s)
}

static STRING_CACHE: [Mutex<cell::LazyCell<StringCache>>; NUM_BINS] =
    [const { Mutex::new(cell::LazyCell::new(StringCache::new)) }; NUM_BINS];

// Use the top bits of the hash to choose a bin
#[inline]
fn whichbin(hash: u64) -> usize {
    ((hash >> TOP_SHIFT as u64) % NUM_BINS as u64) as usize
}
