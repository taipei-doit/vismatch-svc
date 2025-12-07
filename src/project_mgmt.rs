//! Project-wide actions.
//! 
//! This module provides "project-wide" image hashing actions, whichshares the same
//! concept: take a project path, and returns a vec of image hashing result.
//! 
//! The ``
use std::time::Instant;                // calculate time difference
use std::error::Error;                 // standard error trait

use crate::utils::is_image_file;

// functional pattern support for clean code
use itertools::Itertools;

use std::path::Path;      // filesystem path operations
use std::fs::read_dir; // filesystem utils

use crate::image_hash::{
    ImageHashEntry,
    //ImageDistEntry,
    HashType,
    fetch_cache_or_calc_hash,
};

/// Calculate project-wide hash from given path.
pub fn calc_hash_project(project_path: &Path, hash_type: HashType) -> Result<Vec<ImageHashEntry>, Box<dyn Error>> {
    let project_dir_reader = 
        read_dir(project_path)
            .map_err(|e: std::io::Error| format!("error reading project folder: <{}>", e))?;

    let (images_in_project, _): (Vec<_>, Vec<_>) = 
        project_dir_reader.filter_ok(|f| is_image_file(f))
                .map_ok(|f| f.path())
                .partition_result();

    let (h, _): (Vec<_>, Vec<_>) = images_in_project.into_iter()
                                    .map(|f| fetch_cache_or_calc_hash(
                                            &f, 
                                            hash_type, 
                                            false))
                                    .partition_result();
    Ok(h)
}

/// For all images in project folder, try to load hash cache file,
/// and calculate if not found hash cache.
pub fn load_or_calc_project_hashes(project_path: &Path, hash_type: HashType) 
    -> Result<Vec<ImageHashEntry>, Box<dyn Error>> {

    let load_now = Instant::now(); // Measure load time
    
    // Initial check
    project_path.is_dir()
        .then(|| ())
        .ok_or_else( || 
            format!("failed to access project path {:?}", project_path))?;

    let project_name = 
        project_path.file_name().ok_or("invalid project name")?;

    // NOTE: Change standard hash type if needed.
    let hash_list: Vec<ImageHashEntry> = 
        calc_hash_project(project_path, hash_type)?;

    let load_done = load_now.elapsed(); // Measure load time

    // Verbose

    println!("[*] loading project <{:?}> costs: {:.3?}", project_name, load_done);
    println!("[v] loaded {} entries from project <{:?}>", hash_list.len(), project_name);
    
    Ok(hash_list)
}