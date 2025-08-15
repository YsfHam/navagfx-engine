#[macro_export]
macro_rules! impl_default_loader {
    // Version for sources with optional generics
    ($asset:ty, $loader:ty, $(([$($generics:tt),*] => $source:ty)),+) => {
        $(
            impl<$($generics),*> AssetHasDefaultLoader<$source> for $asset {
                type Loader = $loader;
            }
        )+
    };
}