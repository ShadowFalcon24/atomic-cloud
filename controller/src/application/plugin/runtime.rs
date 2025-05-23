#[cfg(feature = "wasm-plugins")]
pub(crate) mod wasm;

#[cfg(feature = "wasm-plugins")]
pub(crate) mod source {
    use std::{
        fmt::{self, Display, Formatter},
        fs,
        path::{Path, PathBuf},
    };

    use anyhow::Result;

    pub struct Source {
        path: PathBuf,
        source: Vec<u8>,
    }

    impl Display for Source {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}", self.path.display())
        }
    }

    impl Source {
        pub fn from_file(path: &Path) -> Result<Self> {
            Ok(Source {
                path: path.to_owned(),
                source: fs::read(path)?,
            })
        }

        pub fn get_source(&self) -> &[u8] {
            &self.source
        }
    }
}
