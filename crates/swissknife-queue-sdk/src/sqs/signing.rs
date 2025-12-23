use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

pub fn sign_request(
    method: &str,
    host: &str,
    uri: &str,
    query_string: &str,
    payload: &str,
    access_key_id: &str,
    secret_access_key: &str,
    region: &str,
    service: &str,
) -> (String, String, String) {
    let now = Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();

    let payload_hash = hex::encode(Sha256::digest(payload.as_bytes()));
    let canonical_headers = format!("host:{}\nx-amz-date:{}\n", host, amz_date);
    let signed_headers = "host;x-amz-date";

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method,
        uri,
        query_string,
        canonical_headers,
        signed_headers,
        payload_hash
    );

    let algorithm = "AWS4-HMAC-SHA256";
    let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, region, service);
    let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm, amz_date, credential_scope, canonical_request_hash
    );

    let signing_key = get_signature_key(secret_access_key, &date_stamp, region, service);
    let signature = hex::encode(hmac_sha256(&signing_key, &string_to_sign));

    let authorization_header = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm, access_key_id, credential_scope, signed_headers, signature
    );

    (authorization_header, amz_date, payload_hash)
}

fn get_signature_key(key: &str, date_stamp: &str, region: &str, service: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{}", key).as_bytes(), date_stamp);
    let k_region = hmac_sha256(&k_date, region);
    let k_service = hmac_sha256(&k_region, service);
    hmac_sha256(&k_service, "aws4_request")
}

fn hmac_sha256(key: &[u8], msg: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(msg.as_bytes());
    mac.finalize().into_bytes().to_vec()
}
