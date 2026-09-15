use include_dir::{include_dir, Dir};

static RES: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/res");

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let file = RES
        .get_file(file_name)
        .ok_or_else(|| anyhow::anyhow!("Resource not found: {}", file_name))?;

    let text = file
        .contents_utf8()
        .ok_or_else(|| anyhow::anyhow!("Resource is not valid UTF-8: {}", file_name))?;

    Ok(text.to_owned())
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let file = RES
        .get_file(file_name)
        .ok_or_else(|| anyhow::anyhow!("Resource not found: {}", file_name))?;

    Ok(file.contents().to_vec())
}