//! Api for external language.  
//! This file provides a trait to be used as an opaque pointer for C or Julia calls used in file libext.rs

use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};

use crate::hnsw::*;
use crate::hnswio::*;
use anndists::dist::distances::Distance;
use log::info;

pub trait AnnT {
    /// type of data vectors
    type Val;
    //
    fn insert_data(&mut self, data: &[Self::Val], id: usize);
    //
    fn search_neighbours(&self, data: &[Self::Val], knbn: usize, ef_s: usize) -> Vec<Neighbour>;
    //
    fn parallel_insert_data(&mut self, data: &[(&Vec<Self::Val>, usize)]);
    //
    fn parallel_search_neighbours(
        &self,
        data: &[Vec<Self::Val>],
        knbn: usize,
        ef_s: usize,
    ) -> Vec<Vec<Neighbour>>;
    ///
    /// dumps a data and graph in 2 files.
    /// Datas are dumped in file filename.hnsw.data and graph in filename.hnsw.graph
    ///
    /// **We do not overwrite old files if they are currently in use by memory map**
    /// If these files already exist , they are not overwritten and a unique filename is generated by concatenating a random number to filename.  
    /// The function returns the basename used for the dump
    fn file_dump(&self, path: &Path, file_basename: &str) -> anyhow::Result<String>;
}

impl<'b, T, D> AnnT for Hnsw<'b, T, D>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync,
    D: Distance<T> + Send + Sync,
{
    type Val = T;
    //
    fn insert_data(&mut self, data: &[Self::Val], id: usize) {
        self.insert((data, id));
    }
    //
    fn search_neighbours(&self, data: &[T], knbn: usize, ef_s: usize) -> Vec<Neighbour> {
        self.search(data, knbn, ef_s)
    }
    fn parallel_insert_data(&mut self, data: &[(&Vec<Self::Val>, usize)]) {
        self.parallel_insert(data);
    }

    fn parallel_search_neighbours(
        &self,
        data: &[Vec<Self::Val>],
        knbn: usize,
        ef_s: usize,
    ) -> Vec<Vec<Neighbour>> {
        self.parallel_search(data, knbn, ef_s)
    }

    /// The main entry point to do a dump.  
    /// It will generate two files one for the graph part of the data. The other for the real data points of the structure.
    /// The names of file are $filename.hnsw.graph for the graph and $filename.hnsw.data.
    ///

    ///
    fn file_dump(&self, path: &Path, file_basename: &str) -> anyhow::Result<String> {
        info!("In Hnsw::file_dump");
        //
        // do not overwrite if mmap is active
        let overwrite = !self.get_datamap_opt();
        let mut dumpinit = DumpInit::new(path, file_basename, overwrite);
        let dumpname = dumpinit.get_basename().clone();
        //
        let res = self.dump(DumpMode::Full, &mut dumpinit);
        //
        dumpinit.flush()?;
        info!("\n End of dump, file basename : {}\n", &dumpname);
        if res.is_ok() {
            Ok(dumpname)
        } else {
            Err(anyhow::anyhow!("unexpected error"))
        }
    } // end of dump
} // end of impl block AnnT for Hnsw<T,D>
