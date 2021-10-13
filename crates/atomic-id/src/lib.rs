#[macro_export]
macro_rules! atomic_id {
    ($static_ident:ident, $type_ident:ident) => {
        static $static_ident: AtomicUsize = AtomicUsize::new(0);

        #[derive(
            Debug,
            Default,
            Copy,
            Clone,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $type_ident(usize);

        impl $type_ident {
            pub fn next() -> Self {
                $type_ident($static_ident.fetch_add(1, Ordering::Relaxed))
            }
        }
        
        impl From<usize> for $type_ident {
            fn from(v: usize) -> Self {
                $type_ident(v)
            }
        }
    };
}
