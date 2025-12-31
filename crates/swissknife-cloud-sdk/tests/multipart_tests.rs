use swissknife_cloud_sdk::{MultipartUpload, UploadPart};

#[test]
fn test_multipart_upload_creation() {
    let upload = MultipartUpload {
        upload_id: "upload123".to_string(),
        key: "large-file.zip".to_string(),
        bucket: "my-bucket".to_string(),
    };

    assert_eq!(upload.upload_id, "upload123");
    assert_eq!(upload.key, "large-file.zip");
    assert_eq!(upload.bucket, "my-bucket");
}

#[test]
fn test_multipart_upload_clone() {
    let upload = MultipartUpload {
        upload_id: "clone-upload".to_string(),
        key: "clone.zip".to_string(),
        bucket: "bucket".to_string(),
    };

    let cloned = upload.clone();
    assert_eq!(upload.upload_id, cloned.upload_id);
    assert_eq!(upload.key, cloned.key);
    assert_eq!(upload.bucket, cloned.bucket);
}

#[test]
fn test_multipart_upload_debug() {
    let upload = MultipartUpload {
        upload_id: "debug-upload".to_string(),
        key: "debug.zip".to_string(),
        bucket: "debug-bucket".to_string(),
    };

    let debug_str = format!("{:?}", upload);
    assert!(debug_str.contains("debug-upload"));
}

#[test]
fn test_upload_part_creation() {
    let part = UploadPart {
        part_number: 1,
        etag: "etag-part-1".to_string(),
    };

    assert_eq!(part.part_number, 1);
    assert_eq!(part.etag, "etag-part-1");
}

#[test]
fn test_upload_part_various_numbers() {
    for num in 1..=10 {
        let part = UploadPart {
            part_number: num,
            etag: format!("etag-{}", num),
        };
        assert_eq!(part.part_number, num);
    }
}

#[test]
fn test_upload_part_clone() {
    let part = UploadPart {
        part_number: 5,
        etag: "clone-etag".to_string(),
    };

    let cloned = part.clone();
    assert_eq!(part.part_number, cloned.part_number);
    assert_eq!(part.etag, cloned.etag);
}

#[test]
fn test_upload_part_debug() {
    let part = UploadPart {
        part_number: 3,
        etag: "debug-etag".to_string(),
    };

    let debug_str = format!("{:?}", part);
    assert!(debug_str.contains("3") || debug_str.contains("debug-etag"));
}

#[test]
fn test_multipart_upload_serialize() {
    let upload = MultipartUpload {
        upload_id: "upload123".to_string(),
        key: "large-file.zip".to_string(),
        bucket: "bucket".to_string(),
    };

    let json = serde_json::to_string(&upload).unwrap();
    assert!(json.contains("upload123"));
    assert!(json.contains("large-file.zip"));
}

#[test]
fn test_upload_part_serialize() {
    let part = UploadPart {
        part_number: 1,
        etag: "part-etag".to_string(),
    };

    let json = serde_json::to_string(&part).unwrap();
    assert!(json.contains("1"));
    assert!(json.contains("part-etag"));
}
