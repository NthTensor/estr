use proc_macro::{TokenStream, TokenTree, Literal};

/// Computes an estr-compatible hash for the given string literal.
/// This is equivalent to calling `digest("something").hash()`, with the added benefit
/// of being valid as a `match` pattern.
///
/// # Examples
///
/// ```
/// # use estr_macros::ehash;
/// # let some_string_hash = ehash!("bar");
/// // assume we got `some_string_hash` through some runtime string,
/// // e.g. via
/// // let some_string_hash = estr(some_string).digest().hash();
///
/// match some_string_hash {
///     ehash!("foo") => {
///         println!("got a foo!");
///     }
///     ehash!("bar") => {
///         println!("got a bar!");
///     }
///     ehash!("baz") => {
///         println!("got a baz!");
///     }
///     _ => {
///         println!("got something else!");
///     }
/// }
/// ```
#[proc_macro]
pub fn ehash(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();
    let lit = match (iter.next(), iter.next()) {
        (Some(TokenTree::Literal(lit)), None) => lit,
        _ => panic!("ehash! expects a single string literal"),
    };

    // strip the surrounding quotes
    let s = lit.to_string();
    let s = s.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
        .expect("ehash! expects a string literal");

    // Keep this in sync with `digest`
    let hash = rapidhash::v3::rapidhash_v3_nano_inline::<true, false>;
    let seed = &rapidhash::v3::DEFAULT_RAPID_SECRETS;
    let value = hash(s.as_bytes(), seed);
    TokenStream::from(TokenTree::Literal(Literal::u64_suffixed(value))).into_iter().collect()
}
