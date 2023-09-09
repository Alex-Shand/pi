use crate::{utils, Result};

use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Identity {
    pub(crate) private: PathBuf,
    pub(crate) public: PathBuf,
}

impl Identity {
    pub(crate) fn new(name: &str) -> Result<Self> {
        let ssh = utils::home()?.join(".ssh");
        Ok(Identity {
            private: ssh.join(name),
            public: ssh.join(format!("{name}.pub")),
        })
    }

    pub(crate) fn exists(&self) -> bool {
        self.private.exists() && self.public.exists()
    }

    pub(crate) fn delete(&self) -> Result<()> {
        fs::remove_file(&self.private)?;
        fs::remove_file(&self.public)?;
        Ok(())
    }
}
