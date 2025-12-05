crossfig::alias! {
    pub std: { #[cfg(feature = "std")] },
    pub spin: { #[cfg(feature = "spin")] }
}
