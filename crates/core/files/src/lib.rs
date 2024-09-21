use std::io::Write;

use aes_gcm::{
    aead::{AeadCore, AeadMutInPlace, OsRng},
    Aes256Gcm, Key, KeyInit, Nonce,
};
use revolt_config::{config, report_internal_error, FilesS3};
use revolt_result::{create_error, Result};

use aws_sdk_s3::{
    config::{Credentials, Region},
    Client, Config,
};

use base64::prelude::*;

/// Size of the authentication tag in the buffer
pub const AUTHENTICATION_TAG_SIZE_BYTES: usize = 16;

/// Create an S3 client
pub fn create_client(s3_config: FilesS3) -> Client {
    let provider_name = "my-creds";
    let creds = Credentials::new(
        s3_config.access_key_id,
        s3_config.secret_access_key,
        None,
        None,
        provider_name,
    );

    let config = Config::builder()
        .region(Region::new(s3_config.region))
        .endpoint_url(s3_config.endpoint)
        .credentials_provider(creds)
        .build();

    Client::from_conf(config)
}

/// Create an AES-256-GCM cipher
pub fn create_cipher(key: &str) -> Aes256Gcm {
    let key = &BASE64_STANDARD.decode(key).expect("valid base64 string")[..];
    let key: &Key<Aes256Gcm> = key.into();
    Aes256Gcm::new(key)
}

/// Fetch a file from S3 (and decrypt it)
pub async fn fetch_from_s3(bucket_id: &str, path: &str, nonce: &str) -> Result<Vec<u8>> {
    let config = config().await;
    let client = create_client(config.files.s3);

    // Send a request for the file
    let mut obj =
        report_internal_error!(client.get_object().bucket(bucket_id).key(path).send().await)?;

    // Read the file from remote
    let mut buf = vec![];
    while let Some(bytes) = obj.body.next().await {
        let data = report_internal_error!(bytes)?;
        report_internal_error!(buf.write_all(&data))?;
        // is there a more efficient way to do this?
        // we just want the Vec<u8>
    }

    // Recover nonce as bytes
    let nonce = &BASE64_STANDARD.decode(nonce).unwrap()[..];
    let nonce: &Nonce<typenum::consts::U12> = nonce.into();

    // Decrypt the file
    create_cipher(&config.files.encryption_key)
        .decrypt_in_place(nonce, b"", &mut buf)
        .map_err(|_| create_error!(InternalError))?;

    Ok(buf)
}

/// Encrypt and upload a file to S3 (returning its nonce/IV)
pub async fn upload_to_s3(bucket_id: &str, path: &str, buf: &[u8]) -> Result<String> {
    let config = config().await;
    let client = create_client(config.files.s3);

    // Generate a nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Extend the buffer for in-place encryption
    let mut buf = [buf, &[0; AUTHENTICATION_TAG_SIZE_BYTES]].concat();

    // Encrypt the file in place
    create_cipher(&config.files.encryption_key)
        .encrypt_in_place(&nonce, b"", &mut buf)
        .map_err(|_| create_error!(InternalError))?;

    // Upload the file to remote
    report_internal_error!(
        client
            .put_object()
            .bucket(bucket_id)
            .key(path)
            .body(buf.into())
            .send()
            .await
    )?;

    Ok(BASE64_STANDARD.encode(nonce))
}
