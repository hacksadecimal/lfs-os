use crate::protocol::{
    InitResponse, Operation, ProgressResponse, ProtocolError, Request, Response, TransferResponse,
};
use futures::stream::StreamExt;
use object_store::{path::Path, ObjectStore};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::debug;

pub async fn serve(object_store: Arc<dyn ObjectStore>) {
    debug!("Serving git-lfs");
    #[allow(unused_assignments)]
    let mut operation = Operation::Upload;

    #[allow(unused_assignments)]
    let mut remote = String::new();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(256);

    // Handle printing in a separate task so messages can't be accidentally interleaved
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(output) = rx.recv().await {
            debug!("sending: {output}");
            stdout.write_all(output.as_bytes()).await.expect("FUBAR");
            stdout.write_u8(b'\n').await.expect("FUBAR");
            stdout.flush().await.expect("FUBAR");
        }
    });

    for line in std::io::stdin().lines() {
        let Ok(line) = line else {
            let err = line.unwrap_err();
            debug!("Unable to read input: {err}");
            std::process::exit(1);
        };
        let result = serde_json::from_str::<Request>(&line);
        let Ok(request) = result else {
            let err = result.unwrap_err();
            debug!("Unable to parse request: {err}");
            std::process::exit(1);
        };
        match request {
            Request::Terminate => std::process::exit(0),
            Request::Init {
                operation: o,
                remote: r,
                ..
            } => {
                operation = o;
                remote = r;
                debug!("init {operation:?} to {remote}");
                tx.send(InitResponse::new(None).json())
                    .await
                    .expect("FUBAR");
            }
            Request::Upload { oid, size, path } => {
                upload_object(object_store.clone(), tx.clone(), oid, size, path).await
            }
            Request::Download { oid, size } => {
                download_object(object_store.clone(), tx.clone(), oid, size).await
            }
        };
    }
}

async fn download_object(
    object_store: Arc<dyn ObjectStore>,
    output_channel: tokio::sync::mpsc::Sender<String>,
    oid: String,
    size: usize,
) {
    debug!("Got a request to download object: {oid} of size: {size}");

    // Retrieve a specific file
    let path: Path = oid.clone().try_into().unwrap();

    // fetch the bytes from object store
    let mut stream = object_store.get(&path).await.unwrap().into_stream();

    let mut bytes_so_far: usize = 0;

    // let local_path := ".git/lfs/objects/" + oid[:2] + "/" + oid[2:4] + "/" + oid;
    let local_path = local_path(&oid);
    tokio::fs::create_dir_all(local_path.parent().unwrap())
        .await
        .expect("TODO");
    debug!("Directory {:?} exists", local_path.parent());
    debug!("Attempting to create file: {:?}", &local_path);
    let mut destination = File::create(&local_path)
        .await
        .expect("Unable to create local LFS object file");
    while let Some(bytes) = stream.next().await {
        let bytes = bytes.unwrap();
        let new_bytes = bytes.len();
        bytes_so_far += new_bytes;
        output_channel
            .send(ProgressResponse::new(oid.clone(), bytes_so_far, new_bytes).json())
            .await
            .expect("FUBAR");
        destination.write_all(&bytes).await.expect("TODO");
    }
    output_channel
        .send(
            TransferResponse::new(
                oid.clone(),
                Ok(Some(String::from(local_path.to_str().unwrap()))),
            )
            .json(),
        )
        .await
        .expect("FUBAR");
}

async fn upload_object(
    object_store: Arc<dyn ObjectStore>,
    output_channel: tokio::sync::mpsc::Sender<String>,
    oid: String,
    size: usize,
    path: String,
) {
    debug!("Got a request to upload object: {oid} of size: {size} from path {path}");

    let object_path: Path = oid.clone().try_into().unwrap();
    let Ok(uploader) = object_store.put_multipart(&object_path).await else {
        output_channel
            .send(
                TransferResponse::new(
                    oid.clone(),
                    Err(ProtocolError::new(
                        1,
                        String::from("Unable to create object in remote store"),
                    )),
                )
                .json(),
            )
            .await
            .expect("FUBAR");
        return;
    };

    let (_upload_id, mut upload_sink) = uploader;

    let mut local_file = File::open(&path)
        .await
        .expect("Unable to create local LFS object file");
    let mut buf = vec![0u8; 4096];
    let mut bytes_so_far = 0;
    while let Ok(chunk_size) = local_file.read(&mut buf).await {
        if chunk_size == 0 {
            break;
        }
        bytes_so_far += chunk_size;
        output_channel
            .send(ProgressResponse::new(oid.clone(), bytes_so_far, chunk_size).json())
            .await
            .expect("FUBAR");
        upload_sink
            .write_all(&buf[..chunk_size])
            .await
            .expect("TODO");
    }
    upload_sink.shutdown().await.expect("TODO");
    output_channel
        .send(TransferResponse::new(oid.clone(), Ok(None)).json())
        .await
        .expect("FUBAR");
}

fn local_path(oid: &str) -> PathBuf {
    PathBuf::from(format!(
        ".git/lfs/objects/{}/{}/{}",
        &oid[..2],
        &oid[2..4],
        oid
    ))
}
