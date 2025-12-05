
use std::time::Instant;              // calculate time difference
use std::error::Error;               // standard error trait
use image::DynamicImage;             // image IO
use itertools::Itertools;            // functional pattern support for clean code
use std::collections::HashMap;       // hashmap support
use std::sync::Arc;                  // shared object reference
use std::path::{Path, PathBuf};      // filesystem path operations
use std::fs::{read_dir, create_dir}; // filesystem utils
use vismatch_svc::image_hash::*;     // our packaged hash algorithms
use vismatch_svc::{
    is_image_file,
};
type ProjectHashDict = HashMap<String, Vec<ImageHashEntry>>;


/// Calculate project-wide hash from given path.
fn calc_hash_project(project_path: &Path, hash_type: HashType) -> Result<Vec<ImageHashEntry>, Box<dyn Error>> {
    let project_dir_reader = 
        read_dir(project_path)
            .map_err(|e: std::io::Error| format!("error reading project folder: <{}>", e))?;

    let (images_in_project, _): (Vec<_>, Vec<_>) = 
        project_dir_reader.filter_ok(|f| is_image_file(f))
                .map_ok(|f| f.path())
                .partition_result();

    let (h, _): (Vec<_>, Vec<_>) = images_in_project.into_iter()
                                    .map(|f| fetch_cache_or_calc_hash(&f, hash_type))
                                    .partition_result();
    Ok(h)
}

/// For all images in project folder, try to load hash cache,
/// and calculate if not found hash cache.
fn load_or_calc_project_hashes(project_path: &Path, hash_type: HashType) 
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

/// For a given image and specified project name, calculate
/// the difference list across project images for provided image.
fn calc_sim_in_project(image: &DynamicImage, project_name: &str, project_hashes: Arc<ProjectHashDict>) 
    -> Result<Vec<ImageDistEntry>, Box<dyn Error + Send + Sync>>{
    
    // first, we should check if the project exists.
    match project_hashes.get(project_name) {
        Some(hash_list) => {
            let mut diff_result = calc_similarity_list(image, hash_list);
            diff_result.sort();

            Ok(diff_result)
        },
        None => todo!(),
    }
}

#[tokio::main]
async fn main() {

    // Stage 1: check prerequisites

    let standard_hash_type: HashType = HashType::PHASH;

    let load_all = Instant::now(); // Measure load time

    let project_root: &Path = Path::new("./image_root");

    let is_project_root_exists = 
        project_root.try_exists()
                .expect("[x] can't check existence of project root folder, shutting down.");

    match is_project_root_exists {
        false => {
            match create_dir(project_root) {
                Ok(_) => println!("[*] created project root folder."),
                Err(_) => panic!("[x] cannot create project folder, shutting down."),
            }
        },
        true => {
            match project_root.is_dir() {
                false => panic!("[x] project folderis not valid, shutting down."),
                true => {}, // Do nothing, continue the service process
            }
        }
    }

    // Stage 2: load or calculate hash for children projects

    let child_project_reader = 
        read_dir(project_root)
            .map_err(|e: std::io::Error| format!("error reading root project contents: <{}>", e))
            .unwrap(); // [Panics] Terminates process if cannot access project root.

    let (children_projects, _): (Vec<_>, Vec<_>) = 
        child_project_reader.filter_ok(|f| f.path().is_dir())
                .map_ok(|f| f.path())
                .partition_result();


    // Load and create a list of tuple (project name, [hash entries])
    let (children_project_hashes, _): 
        (Vec<(String, Vec<ImageHashEntry>)>, Vec<_>) = 
            children_projects.into_iter()
                .map(|f: PathBuf| {
                    match load_or_calc_project_hashes(&f, standard_hash_type) {
                        Ok(h) => {
                            let project_name = 
                                f.file_name().ok_or("invalid project name")?;
                            Ok((project_name.to_string_lossy().into_owned(), h))
                        },
                        Err(err) => Err(err),
                    }})
                .partition_result();

    // Create a Arc to wrap shared project hashes.
    let project_name_hash_map: Arc<ProjectHashDict>
            = Arc::new(children_project_hashes.into_iter().collect());

    let load_all_done = load_all.elapsed(); // Measure load time

    // [NOTE] any other init stage thingy goes here.

    println!("[*] initialization stage costs: {:.3?}", load_all_done);
    println!("[v] initialization stage done, strating service...");



    let test_img_path = PathBuf::from("./resources/test_images/test4.jpg");
    let test_img = image::open(test_img_path.clone()).unwrap();
    
    let compared_res = calc_sim_in_project(
        &test_img, 
        "val2017", 
        project_name_hash_map).expect("cannot calculate image distance");


    let top3 = &compared_res[0..3];

    println!("===Top 3 results===\n");

    top3.iter().for_each(|elem| {
        println!("image name: {:?} | score: {}", elem.image_name, elem.distance);
    });

}


