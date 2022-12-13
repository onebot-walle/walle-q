use walle_core::resp::RespError;

use crate::error;

#[cfg(feature = "silk")]
pub async fn encode_to_silk(audio_file: &str) -> Result<Vec<u8>, RespError> {
    use silk_rs::encode_silk;
    use tokio::{io::AsyncReadExt, process::Command};
    let temp_pcm = format!("{audio_file}_pcm");
    Command::new("ffmpeg")
        .arg("-i")
        .arg(audio_file)
        .arg("-f")
        .arg("s16le")
        .arg("-ar")
        .arg("24000")
        .arg("-ac")
        .arg("1")
        .arg(&temp_pcm)
        .output()
        .await
        .map_err(|e| error::audio_encode_failed(e))?;
    let mut pcm = Vec::new();
    tokio::fs::File::open(&temp_pcm)
        .await
        .map_err(|e| error::audio_encode_failed(e))?
        .read_to_end(&mut pcm)
        .await
        .map_err(|e| error::file_read_error(e))?;
    encode_silk(pcm, 24000, 24000, true).map_err(|e| error::silk_encode_failed(e))
}

#[cfg(not(feature = "silk"))]
pub async fn encode_to_silk(_: &str) -> Result<Vec<u8>, RespError> {
    Err(error::silk_encode_failed(
        "silk is not supported in this target platform",
    ))
}
