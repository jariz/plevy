use crate::Ino;
use anyhow::{Context, Result};
use sled::{Db as SledDb, IVec, Iter, Serialize, Tree};
use std::convert::TryInto;

#[derive(Clone)]
pub struct Db {
    db: SledDb,
    entries: Tree,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    torrent_hash: String,
    file_index: i64,
}

impl Db {
    pub fn new(db_path: &str) -> Result<Db> {
        let db = sled::open(db_path)
            .with_context(|| format!("Failed to open database path `{}`", db_path))?;

        let entries = db.open_tree("entries")?;
        Ok(Db { db, entries })
    }

    // todo: would rather return a iterator
    pub fn list(&self) -> Result<Vec<Result<(Ino, Entry)>>> {
        debug!("tree len {}", self.entries.len());

        let iter = self
            .entries
            .iter()
            .map(|result| -> Result<(Ino, Entry)> {
                let (key, val) = result?;
                let entry: Entry = bincode::deserialize(&val.to_vec())?;
                debug!("list -> key {:?} entry {:?}", key.serialize(), entry);
                let ino = bincode::deserialize(&key.to_vec())?; // TODO: no unwrap please?
                Ok((ino, entry))
            })
            .collect();
        Ok(iter)
    }

    pub fn add(&self, entry: Entry) -> Result<Ino> {
        let ino = self.db.generate_id()?;

        let serialized = bincode::serialize(&entry)?;
        self.entries.insert(bincode::serialize(&ino)?, serialized)?;

        Ok(ino)
    }

    pub fn get(&self, ino: Ino) -> Result<Option<Entry>> {
        let val = match self.entries.get(bincode::serialize(&ino)?)? {
            Some(val) => val,
            None => {
                return Ok(None);
            }
        };
        let entry: Entry = bincode::deserialize(&val)?;

        Ok(Some(entry))
    }

    pub fn has(&self, ino: Ino) -> Result<bool> {
        Ok(self.entries.contains_key(bincode::serialize(&ino)?)?)
    }
}
