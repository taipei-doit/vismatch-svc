pub mod traits;

use std::cmp::Ordering;
use std::error::Error;


use serde;
use image::{self, DynamicImage};
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::image_hash::traits::Hasher;
use crate::metric::*;


/// Enumerates all supported hash algorithm.
#[derive(Debug, Clone, Copy)]
pub enum HashType {
    DHASH,
    PHASH,
    AHASH,
}

fn cache_ext(hash_type: HashType) -> String {
    match hash_type {
        HashType::DHASH => "dhash".to_owned(),
        HashType::PHASH => "phash".to_owned(),
        HashType::AHASH => "ahash".to_owned(),
    }
}

/// Make new hasher with default parameters.
/// 
/// TODO: make parameter adjustable
pub fn mk_hasher(hash_type: HashType) -> Box<dyn Hasher> {
    match hash_type {
        HashType::DHASH => {
            Box::new(imagehash::DifferenceHash::new()
                .with_image_size(32, 32)
                .with_hash_size(32, 32)
                .with_resizer(|img, w, h| {
                    // for resizer function, we choose a more smooth one.
                    img.resize_exact(w as u32, h as u32, image::imageops::FilterType::Lanczos3)
                }))
        },
        HashType::PHASH => {
            Box::new(imagehash::PerceptualHash::new()
                .with_image_size(32, 32)
                .with_hash_size(32, 32)
                .with_resizer(|img, w, h| {
                    // for resizer function, we choose a more smooth one.
                    img.resize_exact(w as u32, h as u32, image::imageops::FilterType::Lanczos3)
                }))
        },
        HashType::AHASH => {
            Box::new(imagehash::AverageHash::new()
                .with_image_size(32, 32)
                .with_hash_size(32, 32)
                .with_resizer(|img, w, h| {
                    // for resizer function, we choose a more smooth one.
                    img.resize_exact(w as u32, h as u32, image::imageops::FilterType::Lanczos3)
                }))
        },
    }
}

/// We make a proxy struct for `imagehash::Hash` because it is 
/// so bad, it cannot serialize, cannot measure distance, and
/// even cannot clone. 
/// 
/// The lack of `clone` ability actually drives me nut.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Hash {
    /// The bit vector representation of the hash.
    pub bits: Vec<bool>,
}

impl From<imagehash::Hash> for Hash {
    fn from(value: imagehash::Hash) -> Self {
        Hash {
            bits: value.bits.clone()
        }
    }
}

impl crate::metric::Metrizable for Hash {
    fn dist(&self, other: &Self) -> f64 {
        // we just borrow the already-implmented measure from Hash
        // first make a cast

        let self_hash: imagehash::Hash = imagehash::Hash {
            bits: self.bits.clone()
        };

        let other_hash: imagehash::Hash = imagehash::Hash {
            bits: other.bits.clone()
        };

        self_hash.dist(&other_hash)
    }
}

fn calc_hash(image: &DynamicImage, hash_type: HashType) -> Hash {
    let hasher = mk_hasher(hash_type);
    hasher.hash(image).into()
}

pub fn calc_image_hash(image_path: &Path, hash_type: HashType) 
        -> Result<ImageHashEntry, Box<dyn Error>> {

    let img = image::open(image_path)?;

    let h = calc_hash(&img, hash_type);

    Ok(ImageHashEntry { 
        image_name: image_path.to_owned(), 
        hash_type, 
        hash: h })
}

/// Write hash value to cache file in the same folder
/// of image file located.
pub fn write_hash_cache(image_path: &Path, image_hash: &Hash, hash_type: HashType) -> Result<usize, Box<dyn Error>> {

    let image_path = image_path.to_owned();

    let hash_file_name = image_path.with_added_extension(cache_ext(hash_type));

    // Serialize: using proxy trick.
    let hash_pxy = 
        Hash { bits: image_hash.bits.clone() }; // clone to a already-derived (de)serialize struct.

    let mut f_handle = File::create(hash_file_name)?;

    bincode::serde::encode_into_std_write(
                            &hash_pxy,
                            &mut f_handle,
                            bincode::config::standard())
                                    .map_err(|e| format!("error while serialize ({})", e).into())
}

/// Attempt to load hash value from cache in the same folder of 
/// given image.
/// 
/// It also implemented the `Ord` trait so it's possible to sort a list
/// of measured, images and fetch the most similar images.
pub fn fetch_hash_cache(image_path: &Path, hash_type: HashType) -> Result<ImageHashEntry, Box<dyn Error>> {
    
    let hash_file_name = image_path.with_added_extension(cache_ext(hash_type));

    // try to open the cache corresponding to the given hash type
    let mut f_handle = match File::open(&hash_file_name) {
        Ok(f) => f,
        Err(e) => {
            // Provide a more descriptive error if the file doesn't exist
            return Err(format!("cannot open cache file '{}' with type {:?}: {}",
                                hash_file_name.display(), hash_type, e).into());
        }
    };

    // try to decode
    let hash_pxy: Hash = 
        bincode::serde::decode_from_std_read(
        &mut f_handle,
        bincode::config::standard(),
        ).map_err(|e: bincode::error::DecodeError| format!("cannot deserialize cache file '{}' with type {:?}: {}",
                            hash_file_name.display(), hash_type, e))?;

    let img_hash = Hash {
        bits: hash_pxy.bits.clone(),
    };

    Ok(ImageHashEntry { 
        image_name: image_path.to_owned(), 
        hash_type, 
        hash: img_hash.into() 
    })
}

pub fn fetch_cache_or_calc_hash(image_path: &Path, hash_type: HashType, force_rewrite_cache: bool) -> Result<ImageHashEntry, Box<dyn Error>> {
    
    match fetch_hash_cache(image_path, hash_type) {
        Ok(h) => { // we found exist hash cache
            let h = match force_rewrite_cache {
                true => { // force recalculate
                    match calc_image_hash(image_path, hash_type) {
                        Ok(h_new) => {
                        // now try to write cache, and IGNORE the error.
                        // [NOTE] shoule we catch the error of cache writing?
                        // Hey, cache really looks like catch!
                        write_hash_cache(image_path, &h.hash, hash_type).ok();
                        h_new
                    },
                Err(_err) => h, // calculation error, just return cache
            }
                },
                false => h,
            };
            Ok(h)
        },
        Err(_) => {
            match calc_image_hash(image_path, hash_type) {
                Ok(h) => {

                    // now try to write cache, and IGNORE the error.
                    // [NOTE] shoule we catch the error of cache writing?
                    // Hey, cache really looks like catch!
                    write_hash_cache(image_path, &h.hash, hash_type).ok();
                    Ok(h)
                },
                Err(err) => Err(err),
            }
        },
    }
}

/// The definition of (image name, hash value) pair format.
#[derive(Debug, Clone)]
pub struct ImageHashEntry {
    pub image_name: PathBuf,
    pub hash_type: HashType,
    pub hash: Hash,
}

/// The definition of an entry of image, pair with the distance 
/// of another given image.
#[derive(Debug, Clone)]
pub struct ImageDistEntry {
    pub image_name: PathBuf,
    pub distance: f64,
}

impl PartialEq for ImageDistEntry {
    fn eq(&self, other: &Self) -> bool {
        // Equality is defined by the total comparison being equal.
        self.distance.total_cmp(&other.distance) == Ordering::Equal
    }
}

impl PartialOrd for ImageDistEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // You can use standard partial_cmp here, though total_cmp is also fine.
        self.distance.partial_cmp(&other.distance)
    }
}

impl Eq for ImageDistEntry {}

impl Ord for ImageDistEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Use total_cmp to get a stable, panic-free total ordering.
        self.distance.total_cmp(&other.distance)
    }
}

pub fn calc_distance(image: &DynamicImage, h_entry: &ImageHashEntry) -> ImageDistEntry {
    let hasher = mk_hasher(h_entry.hash_type);
    let h: Hash = hasher.hash(&image).into();
    let h_dist = h.dist(&h_entry.hash);

    ImageDistEntry {
        image_name: h_entry.image_name.clone(),
        distance: h_dist,    
    }
}

fn calc_distance_from_hash(hash: &Hash, h_entry: &ImageHashEntry) -> ImageDistEntry {
    ImageDistEntry {
        image_name: h_entry.image_name.clone(),
        distance: hash.dist(&h_entry.hash),    
    }
}

pub fn calc_similarity_list(image: &image::DynamicImage, hash_list: &Vec<ImageHashEntry>) -> Vec<ImageDistEntry> {
    
    if hash_list.len() == 0 {
        return vec![];
    }
    
    // Important NOTE: we choose the first element from `hash_list`,
    // and use it as the hasher for all element.
    //
    // It speeds up by ignore redundant hash calculation, but less
    // generality, change if needed.
    let hasher = mk_hasher(hash_list[0].hash_type);
    let h: Hash = hasher.hash(&image).into();
    


    hash_list.iter().map(|h_ent: &ImageHashEntry| {
        calc_distance_from_hash(&h, &h_ent)
    }).collect()
}