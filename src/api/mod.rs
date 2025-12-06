use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimilarImageEntry {
	pub image_name: String,	  // the name of image
	pub distance: f32,		  // distance score, lower is closer
	pub data: Option<String>, // image data as base64 string.
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompareImageReq {
	pub project_name: String,
	pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompareImageResp {
	pub project_name: String, // the name of project
	pub compare_result: Vec<SimilarImageEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadImageReq {
	project_name: String,
	data: String,
}

pub struct RemoveImageReq {

}