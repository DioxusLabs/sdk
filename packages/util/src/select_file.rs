use dioxus::document::{EvalError, eval};
use serde::{Deserialize, Serialize, de::Error};
use serde_json::Value;

/// Represents a file selection with its metadata and data
#[derive(Serialize, Deserialize, Debug)]
pub struct FileSelection<T> {
    /// The file name including the extension but without the full path
    pub name: String,
    /// MIME type: https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/MIME_types/Common_types
    pub r#type: String,
    /// The size of the file in bytes
    pub size: u64,
    /// The data contained in the file in the corresponding encoding if requested
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilePickerOptions {
    /// https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input/file#accept
    pub accept: Option<String>,
    /// https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input/file#capture
    pub capture: Option<String>,
}

impl Default for FilePickerOptions {
    fn default() -> Self {
        Self {
            accept: None,
            capture: None,
        }
    }
}

/// Encoding options for file data
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
enum DataEncoding {
    /// base64-encoded with MIME type
    DataUrl,
    /// UTF-8 string
    Text,
    // Dev Note: There is no point in supporting this at the moment since `serde_json` (which dioxus uses internally)
    // does not support bytes (js's byte buffer from `readAsArrayBuffer`). So we cant send the data back without conversions.
    // At that point, it is just better use `DataUrl` or not request any data and perform the read on the Rust side.
    // /// Raw bytes
    // Bytes,
}

#[derive(Debug, Serialize)]
struct FilePickerOptionsInternal<'a> {
    pub accept: &'a Option<String>,
    pub multiple: bool,
    pub capture: &'a Option<String>,
    /// The encoding to use for data extraction. If none, no data for the actual file is returned
    pub encoding: Option<DataEncoding>,
}

/// Select a single file, returning the contents the data of the file as base64 encoded
pub async fn select_file_base64(
    options: &FilePickerOptions,
) -> Result<Option<FileSelection<String>>, EvalError> {
    let FilePickerOptions { accept, capture } = options;
    select_file_internal(&FilePickerOptionsInternal {
        accept,
        multiple: false,
        capture,
        encoding: Some(DataEncoding::DataUrl),
    })
    .await
}

/// Select multiple files, returning the contents the data of the files as base64 encoded
pub async fn select_files_base64(
    options: &FilePickerOptions,
) -> Result<Vec<FileSelection<String>>, EvalError> {
    let FilePickerOptions { accept, capture } = options;
    select_files_internal(&FilePickerOptionsInternal {
        accept,
        multiple: true,
        capture,
        encoding: Some(DataEncoding::DataUrl),
    })
    .await
}

/// Select a single file, returning the contents the data of the file as utf-8
pub async fn select_file_text(
    options: &FilePickerOptions,
) -> Result<Option<FileSelection<String>>, EvalError> {
    let FilePickerOptions { accept, capture } = options;
    select_file_internal(&FilePickerOptionsInternal {
        accept,
        multiple: false,
        capture,
        encoding: Some(DataEncoding::Text),
    })
    .await
}

/// Select multiple files, returning the contents the data of the files as utf-8
pub async fn select_files_text(
    options: &FilePickerOptions,
) -> Result<Vec<FileSelection<String>>, EvalError> {
    let FilePickerOptions { accept, capture } = options;
    select_files_internal(&FilePickerOptionsInternal {
        accept,
        multiple: true,
        capture,
        encoding: Some(DataEncoding::Text),
    })
    .await
}

/// Select a single file, returning no contents of the file
pub async fn select_file(
    options: &FilePickerOptions,
) -> Result<Option<FileSelection<()>>, EvalError> {
    let FilePickerOptions { accept, capture } = options;
    let result: Option<FileSelection<Value>> = select_file_internal(&FilePickerOptionsInternal {
        accept,
        multiple: false,
        capture,
        encoding: None,
    })
    .await?;
    result.map(map_to_unit).transpose()
}

/// Select multiple files, returning no contents of the files
pub async fn select_files(
    options: &FilePickerOptions,
) -> Result<Vec<FileSelection<()>>, EvalError> {
    let FilePickerOptions { accept, capture } = options;
    let result: Vec<FileSelection<Value>> = select_files_internal(&FilePickerOptionsInternal {
        accept,
        multiple: true,
        capture,
        encoding: None,
    })
    .await?;
    result
        .into_iter()
        .map(map_to_unit)
        .collect()
}

fn map_to_unit(file: FileSelection<Value>) -> Result<FileSelection<()>, EvalError> {
    let FileSelection {
        name,
        r#type,
        size,
        data,
    } = file;

    if data != Value::Null {
        return Err(EvalError::Serialization(serde_json::Error::custom(
            "Expected no file data but received non-null data. This indicates a mismatch between encoding settings and returned data.",
        )));
    }

    Ok(FileSelection {
        name,
        r#type,
        size,
        data: (),
    })
}

const SELECT_FILE_SCRIPT: &str = r#"
const attrs = await dioxus.recv();

const input = document.createElement("input");
input.type = "file";
if (attrs.accept) input.accept = attrs.accept;
if (attrs.multiple) input.multiple = true;
if (attrs.capture) input.capture = attrs.capture;

input.onchange = async () => {
    const files = input.files;
    input.remove();

    if (!files || files.length === 0) {
        if (attrs.multiple) {
            dioxus.send([]);
        } else {
            dioxus.send(null);
        }
        return;
    }

    const readFile = (file) => new Promise((resolve) => {
        const base = {
            name: file.name,
            type: file.type,
            size: file.size,
        };

        if (attrs.encoding === undefined || attrs.encoding === null) {
            resolve({
                ...base,
                data: null,
            });
            return;
        }

        const reader = new FileReader();
        reader.onload = () => {
            resolve({
                ...base,
                data: reader.result,
            });
        };

        switch (attrs.encoding) {
            case "text":
                reader.readAsText(file);
                break;
            case "data_url":
                reader.readAsDataURL(file);
                break;
            default:
                console.error("Unsupported encoding:", attrs.encoding);
                throw new Error("Unsupported encoding");
        }
    });

    const readFiles = await Promise.all([...files].map(readFile));
    if (attrs.multiple) {
        dioxus.send(readFiles);
    } else {
        dioxus.send(readFiles[0]);
    }
};

input.click();"#;

async fn select_file_internal<'a, T>(
    options: &'a FilePickerOptionsInternal<'a>,
) -> Result<Option<FileSelection<T>>, EvalError>
where
    T: for<'de> Deserialize<'de>,
{
    let mut eval = eval(SELECT_FILE_SCRIPT);
    eval.send(options)?;
    let data = eval.recv().await?;
    Ok(data)
}

async fn select_files_internal<'a, T>(
    options: &'a FilePickerOptionsInternal<'a>,
) -> Result<Vec<FileSelection<T>>, EvalError>
where
    T: for<'de> Deserialize<'de>,
{
    let mut eval = eval(SELECT_FILE_SCRIPT);
    eval.send(options)?;
    let data = eval.recv().await?;
    Ok(data)
}
