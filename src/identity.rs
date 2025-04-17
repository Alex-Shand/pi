use std::{fs, marker::PhantomData, path::PathBuf};

use anyhow::{Result, anyhow};

use crate::utils;

/*
#[typ::union]
pub(crate) type IdentityState = (Created, Unknown)
*/

#[sealed::sealed]
pub(crate) trait IdentityState {}

pub(crate) struct Created;
pub(crate) struct Unknown;

#[sealed::sealed]
impl IdentityState for Created {}

#[sealed::sealed]
impl IdentityState for Unknown {}

pub(crate) struct Identity<T: IdentityState> {
    _state: PhantomData<T>,
    pub(crate) private: PathBuf,
    pub(crate) public: PathBuf,
}

impl Identity<Unknown> {
    pub(crate) fn new_unknown(name: impl AsRef<str>) -> Result<Self> {
        let name = name.as_ref();
        let ssh = utils::home()?.join(".ssh");
        Ok(Self {
            _state: PhantomData,
            private: ssh.join(name),
            public: ssh.join(format!("{name}.pub")),
        })
    }

    pub(crate) fn exists(self) -> Result<Identity<Created>, Identity<Unknown>> {
        if self.private.exists() && self.public.exists() {
            return Ok(Identity {
                _state: PhantomData,
                private: self.private,
                public: self.public,
            });
        }
        Err(self)
    }
}

impl Identity<Created> {
    pub(crate) fn new(name: impl AsRef<str>) -> Result<Self> {
        let name = name.as_ref();
        Identity::new_unknown(name)?
            .exists()
            .map_err(|_| anyhow!("No identity file for {name}"))
    }

    pub(crate) fn private(name: impl AsRef<str>) -> Result<PathBuf> {
        Ok(Identity::new(name)?.private)
    }

    pub(crate) fn delete(self) -> Result<Identity<Unknown>> {
        let Self {
            _state,
            private,
            public,
        } = self;
        [fs::remove_file(&private), fs::remove_file(&public)]
            .into_iter()
            .collect::<Result<(), _>>()
            .map_err(|e| anyhow!(e))?;
        Ok(Identity {
            _state: PhantomData,
            private,
            public,
        })
    }
}
