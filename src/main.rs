
use std::error::Error;          // standard error trait
use std::time::Instant;         // calculate time difference
use std::collections::HashMap;  // hashmap support
use image::DynamicImage;        // image IO
use itertools::Itertools;       // functional pattern support to make life easier

// asynchronous execution and management
use tokio::sync::RwLock;    // shared object management
use std::sync::Arc;         // shared object reference

// HTTP related libs
use axum::http::{Response, StatusCode}; // HTTP
use axum::response::IntoResponse;       // convert to response
use axum::routing::{post};              // HTTP method
use axum::body::Body;                   // plain response body
use axum::extract::{Json, State};       // response types
use axum::{Router, http};               // router
use tokio::net::TcpListener;            // listener
use std::net::SocketAddr;               // socker definition

// filesystem and os-related libraries
use std::path::{Path, PathBuf};      // filesystem path operations
use std::fs::{read_dir, create_dir}; // filesystem utils

// internal libraries
use vismatch_svc::{
    HasSingleImage,         // trait for getting image from request object
    base64_to_image, 
    dist_entry_to_api_sim_entry, image_hash::*};     // our packaged hash algorithms

use vismatch_svc::project_mgmt::{
    load_or_calc_project_hashes     
};
use vismatch_svc::api::*;           // API structure


type ProjectHashDict = Arc<RwLock<HashMap<String, Vec<ImageHashEntry>>>>;

#[derive(Clone)]
struct AppState {
    project_root: String,
    project_dict: ProjectHashDict,
}

// common task definition


async fn save_image_to_project(
    project_root: &str,
    project_name: &str, 
    image: &DynamicImage, 
    image_name: &str,
    hash_type: HashType,
    project_hashes: ProjectHashDict) -> Result<(), Box<dyn Error + Send + Sync>> {

    let project_root = Path::new(project_root);
    let project_path = &project_root.join(project_name);

    // check project dir
    match project_path.is_dir() {
        false => {
            // create project folder
            create_dir(project_path)
                .map_err(|e| format!("cannot create project folder: {}", e.to_string()))?;
        }
        true => {} // continue execution
    }

    let _project_hashes = Arc::clone(&project_hashes);

    let mut project_dict_wlock = _project_hashes.write().await;

    (*project_dict_wlock).insert(project_name.to_owned(), Vec::<ImageHashEntry>::new());

    // now add image name
    let image_target_path = project_path.join(image_name);

    // [NOTE] verbose print
    println!("[*] saving image to <{}>", image_target_path.to_string_lossy());

    // save the image
    image.save(&image_target_path)
        .map_err(|e: image::ImageError| 
            Box::<dyn std::error::Error + Send + Sync>::from(   // I know it's tricky, but we need to cast the error
                format!("error while saving image: {}", e.to_string())))?;

    // now we need to calculate, and update the global hash dict.
    // we clone this, since it will be moved to other thread
    let _image_target_path = image_target_path.clone();

    // we spawn a task to calculate hash.
    let hash_calc_task = 
        tokio::task::spawn_blocking(move || {    
            let image_target_path = _image_target_path;

            // we need type annotation, so we created a new varibale here to hold result.
            let res: Result<ImageHashEntry, Box<dyn Error + Send + Sync>> = 
                fetch_cache_or_calc_hash(
                    &image_target_path, 
                    hash_type,
                    true)
                    .map_err(|f|f.to_string().into());  
            res // return the result
        });

    let hash_result: ImageHashEntry = hash_calc_task.await??; // now we have the calculated hash.

    // now we can update the project hash dict.
    let project_name = project_name;

    if let Some(val) = 
        (*project_dict_wlock).get_mut(project_name) { 
            val.push(hash_result); 
    }

    Ok(()) // All good, return
}


/// For a given image and specified project name, calculate
/// the difference list across project images for provided image.
async fn calc_sim_in_project(image: DynamicImage, project_name: &str, project_hashes: ProjectHashDict) 
    -> Result<Vec<ImageDistEntry>, Box<dyn Error + Send + Sync>>{
    // println!("[*] enter calculation blk");

    let calc_start = Instant::now(); // Measure calc time

    let image = image.clone();
    let project_dict_rlock = project_hashes.read().await;

    // first, we should check if the project exists.
    match (*project_dict_rlock).get(project_name) {

        // If exists, then calculate the distance.
        Some(hash_list) => {
            let hash_list = hash_list.clone();

            // This involves image resizing, which is a cpu task.
            // So we put it in seprated thread. 
            let diff_calc_task = 
                tokio::task::spawn_blocking(move || {            
                    let res = calc_similarity_list(&image, &hash_list);  
                    res
                });

            let mut diff_result = diff_calc_task.await?;
            diff_result.sort();

            let calc_done = calc_start.elapsed(); // Measure load time

            println!("[*] calculation task done: {:.3?}", calc_done);
            // println!("[*] leave calculation blk");
            
            Ok(diff_result)

        },
        None => Err(format!("project <{}> not found in current database", project_name).into()),
    }
}

// here's are the service handlers

async fn compare_handler(
    State(state): State<AppState>, 
    Json(payload): Json<CompareImageReq>)
    -> Result<Json<CompareImageResp>, AppError> {
    
    // 1. we first get the image from data b64 string
    let image_target 
        = payload.get_image()
            .map_err(|e| AppError::InternalError(e.to_string()))?;

    // 2. 
    let result = calc_sim_in_project(
        image_target, 
        &payload.project_name, 
        state.project_dict
    ).await.map_err(|e| AppError::BadRequest(e.to_string()));

    match result {
        Ok(dist_vec) => {

            // [NOTE] we pick the top-3 entries from closest images, change if needed.
            let sim_vec: Vec<SimilarImageEntry> = (&dist_vec[0..3])
                .iter().map(
                    |x| dist_entry_to_api_sim_entry(
                        x, 
                        payload.with_image))
                .collect();
            
            Ok(Json(CompareImageResp {
            success: true,
            message: "success".to_owned(),
            project_name: payload.project_name,
            compare_result: sim_vec,
        }))},
        Err(e) => Err(e),
    }
}

async fn upload_handler(
    State(state): State<AppState>, 
    Json(payload): Json<UploadImageReq>)
    -> Result<Json<UploadImageResp>, AppError> {
    
    // 1. we first collect parameters we need

    let project_root = state.project_root;
    let project_name = payload.project_name;
    let image_name = payload.image_name;

    // [NOTE] conside resize to save spaces.
    let image = base64_to_image(&payload.data)
                .map_err(|e| format!("cannot create image from b64: {}", e.to_string()))
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let project_dict = Arc::clone(&state.project_dict);
    

    println!("[*] received upload request on <{}>", project_name); // [NOTE] verbose

    // do saving image, return 500 if failed
    save_image_to_project(
        &project_root,
        &project_name,
        &image,
        &image_name,
        HashType::PHASH, // [NOTE] [WARN] change here later
        project_dict
    ).await.map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(UploadImageResp {
        success: true,
        message: "image uploaded and indexed successfully".to_owned(),
        token: "dummy-deletion-token".to_string(), // [WARN] [NOTE] change later to proper uuid
    }))

}


/// Handler for "404 not found" error, returning plain text body.
async fn not_found_handler() -> Response<Body> { 
    (
        StatusCode::NOT_FOUND,
        [(http::header::CONTENT_TYPE, "application/json")],
        "Knock, knock. Anyone here?\n\nSorry, this door seems to be missing! Maybe try another link?".to_owned()
    ).into_response()
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
                false => panic!("[x] project folder is not valid, shutting down."),
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
    let project_name_hash_map: ProjectHashDict
            = Arc::new(RwLock::new(children_project_hashes.into_iter().collect()));

    let load_all_done = load_all.elapsed(); // Measure load time

    // [NOTE] any other init stage thingy goes here.

    println!("[*] initialization stage costs: {:.3?}", load_all_done);
    println!("[v] initialization stage done, strating service...");

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener: TcpListener = 
        TcpListener::bind(addr).await.unwrap();

    println!("[*] image comparison service listening on {}", addr);


    // Stage 3: starting service
    let axum_state: AppState = AppState { 
        project_root: project_root.to_string_lossy().to_string(),
        project_dict: project_name_hash_map };

    let axum_app: Router = Router::new()
                    .route("/diff", post(compare_handler))
                    .route("/upload", post(upload_handler))
                    .with_state(axum_state)
                    .fallback(not_found_handler);

    axum::serve(listener, axum_app).await.unwrap();
}


