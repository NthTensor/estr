# Estr
Easy String Interning.

This is a fork of the excellent [`ustr`] library for rust.

Compared to that library, this crate:
+ Supports `no_std`.
+ Uses a hash function that can be called in `const` contexts.
+ Is more minimalist and less fully featured.

[`ustr`]: https://github.com/anderslanglands/ustr/tree/master
