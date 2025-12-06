use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SimilarImageEntry {
	pub image_name: String,	  // the name of image
	pub distance: f32,		  // distance score, lower is closer
	pub data: Option<String>, // image data as base64 string.
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CompareImageReq {
	project_name: String,
	data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CompareImageResp {
	success: bool,
	message: String,
	project_name: String, // the name of project
	compare_result: Vec<SimilarImageEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UploadImageReq {
	project_name: String,
	data: String,
	with_image: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UploadImageResp {
	success: bool,
	message: String,
	token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RemoveImageReq {
	token: String, // image removal token.
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RemoveImageResp {
	success: bool,
	message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

	/// Consistency test
    #[test]
    fn test_serialize() {

        // --- Shared Test Data ---
        let smallest_png_1: String = 
            "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQYGWNgYGD4DwABBAEAqXIB5QAAAABJRU5ErkJggg".to_owned();

        // 1x1 GIF
        let smallest_png_2: String = 
            "R0lGODlhAQABAIAAAAAAAP///yH5BAAAAAAALAAAAAABAAEAAAICRAEAOw".to_owned();

        // ---------------------------------------------------------
        // 1. Test CompareImageResp & CompareImageReq
        // ---------------------------------------------------------
        println!("--- Testing CompareImageResp ---");
        let ent1: SimilarImageEntry = SimilarImageEntry {
            image_name: "img01".to_owned(),
            distance: 3.0,
            data: None,
        };

        let ent2: SimilarImageEntry = SimilarImageEntry {
            image_name: "img02".to_owned(),
            distance: 8.7,
            data: Some(smallest_png_1.clone()),
        };

        let comp_resp: CompareImageResp = CompareImageResp {
            project_name: "some_project".to_owned(),
            success: true,
            message: "success".to_owned(),
            compare_result: vec![ent1, ent2],
        };

        let comp_resp_json: String = serde_json::to_string_pretty(&comp_resp).unwrap();
        let comp_resp_deserialized: CompareImageResp = serde_json::from_str(&comp_resp_json).unwrap();
        println!("{}\n", comp_resp_json);
        assert_eq!(comp_resp, comp_resp_deserialized);

        println!("--- Testing CompareImageReq ---");
        let comp_req: CompareImageReq = CompareImageReq {
            project_name: "some_project".to_owned(),
            data: smallest_png_2.clone(),
        };

        let comp_req_json: String = serde_json::to_string_pretty(&comp_req).unwrap();
        let comp_req_deserialized: CompareImageReq = serde_json::from_str(&comp_req_json).unwrap();
        println!("{}\n", comp_req_json);
        assert_eq!(comp_req, comp_req_deserialized);


        // ---------------------------------------------------------
        // 2. Test UploadImageReq & UploadImageResp
        // ---------------------------------------------------------
        println!("--- Testing UploadImageReq ---");
        let upload_req: UploadImageReq = UploadImageReq {
            project_name: "some_project".to_owned(),
            data: smallest_png_1.clone(),
			with_image: true,
        };

        let upload_req_json: String = serde_json::to_string_pretty(&upload_req).unwrap();
        let upload_req_deserialized: UploadImageReq = serde_json::from_str(&upload_req_json).unwrap();
        println!("{}\n", upload_req_json);
        assert_eq!(upload_req, upload_req_deserialized);


        println!("--- Testing UploadImageResp ---");
        let upload_resp: UploadImageResp = UploadImageResp {
            success: true,
            message: "image uploaded and indexed successfully".to_owned(),
            token: "abc-123-unique-token-xyz".to_owned(),
        };

        let upload_resp_json: String = serde_json::to_string_pretty(&upload_resp).unwrap();
        let upload_resp_deserialized: UploadImageResp = serde_json::from_str(&upload_resp_json).unwrap();
        println!("{}\n", upload_resp_json);
        assert_eq!(upload_resp, upload_resp_deserialized);

		let upload_resp2: UploadImageResp = UploadImageResp {
            success: false,
            message: "duplication".to_owned(),
            token: "".to_owned(),
        };

        let upload_resp2_json: String = serde_json::to_string_pretty(&upload_resp2).unwrap();
        let upload_resp2_deserialized: UploadImageResp = serde_json::from_str(&upload_resp2_json).unwrap();
        println!("{}\n", upload_resp2_json);
        assert_eq!(upload_resp2, upload_resp2_deserialized);

        // ---------------------------------------------------------
        // 3. Test RemoveImageReq & RemoveImageResp
        // ---------------------------------------------------------

        println!("--- Testing RemoveImageReq ---");
        let remove_req: RemoveImageReq = RemoveImageReq {
            token: "abc-123-unique-token-xyz".to_owned(),
        };

        let remove_req_json = serde_json::to_string_pretty(&remove_req).unwrap();
        let remove_req_deserialized: RemoveImageReq = serde_json::from_str(&remove_req_json).unwrap();
        println!("{}\n", remove_req_json);
        assert_eq!(remove_req, remove_req_deserialized);


        println!("--- Testing RemoveImageResp ---");
        let remove_resp: RemoveImageResp = RemoveImageResp {
            success: false,
            message: "token expired or invalid".to_owned(),
        };

        let remove_resp_json: String = serde_json::to_string_pretty(&remove_resp).unwrap();
        let remove_resp_deserialized: RemoveImageResp = serde_json::from_str(&remove_resp_json).unwrap();
        println!("{}\n", remove_resp_json);
        assert_eq!(remove_resp, remove_resp_deserialized);
    }
}