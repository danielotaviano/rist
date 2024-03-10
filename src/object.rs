use std::ffi::CStr;
use std::{fmt, fs, io::Error};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
enum ObjectType {
    Tree,
    Blob,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Tree => write!(f, "tree"),
        }
    }
}

#[derive(Debug)]
pub struct Object {
    hash: String,
    content: Vec<u8>,
    size: i32,
    file_type: ObjectType,
}

impl Object {
    pub fn from_hash(hash: String) -> Self {
        let f = fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .expect("Can't read the file");

        let mut z = ZlibDecoder::new(f);
        let mut s = Vec::new();
        z.read_to_end(&mut s).expect("Invalid zlib content");

        let headers = CStr::from_bytes_until_nul(s.as_slice())
            .expect("Error Trying to read the heards")
            .to_str()
            .expect("Error Trying to parse headers to string");

        let type_size: Vec<&str> = headers.split(' ').collect();

        if type_size.len() != 2 {
            panic!("Try to create a not valid git object")
        };

        let raw_file_type = type_size[0];
        let size: i32 = type_size[1]
            .parse()
            .expect("Error when try to parse size in i32");

        let file_type = match raw_file_type {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            _ => panic!("Unexpected file type"),
        };

        let content = &s[headers.len() + 1..];

        Self {
            hash,
            content: content.to_vec(),
            size,
            file_type,
        }
    }

    pub fn from_path(path: String) -> Self {
        let file_content =
            String::from_utf8(fs::read(path).expect("Unable to read the file").to_vec())
                .expect("Expect only bytes");

        let file_size: i32 = file_content
            .len()
            .try_into()
            .expect("Error when try to convert file size");
        let file_content_with_header = format!("blob {}\0{}", file_size, file_content);

        let mut hasher = Sha1::new();
        hasher.update(file_content_with_header.as_bytes());

        let hash = format!("{:X}", hasher.finalize()).to_lowercase();

        Self {
            content: Vec::from(file_content),
            file_type: ObjectType::Blob,
            hash,
            size: file_size,
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        let folder_name = &self.hash[..2];
        let file_name = &self.hash[2..];

        let file_path = format!(".git/objects/{folder_name}/{file_name}");
        let folder_path = format!(".git/objects/{folder_name}");

        if fs::metadata(&file_path).is_ok() {
            return Ok(());
        };

        let content_string =
            String::from_utf8(self.content.clone()).expect("Error trying to recover content");

        let headers_content = format!("{} {}\0{}", self.file_type, self.size, content_string);

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(headers_content.as_bytes())
            .expect("Unable to write in compress buffer");
        let compressed = e.finish().expect("Unable to compress the content");

        fs::create_dir(folder_path).expect("Erro trying to create directory");
        fs::write(file_path, compressed).expect("Error trying to write in the file");

        Ok(())
    }

    pub fn content(&self, tree_name_only: Option<bool>) -> String {
        match self.file_type {
            ObjectType::Blob => String::from_utf8(self.content.clone())
                .expect("Error Trying to recover the object content"),
            ObjectType::Tree => {
                let name_only = match tree_name_only {
                    Some(name_only) => name_only,
                    _ => panic!("Need to pass tree_name_only param"),
                };

                self.format_tree_content_to_print(&self.content, name_only)
            }
        }
    }

    pub fn hash(&self) -> String {
        self.hash.clone()
    }

    fn format_tree_content_to_print(&self, content: &Vec<u8>, name_only: bool) -> String {
        let mut final_content = Vec::new();
        let mut content_copy = content.clone();
        while content_copy.len() != 0 {
            let header = CStr::from_bytes_until_nul(&content_copy.as_slice())
                .expect("Error when try to extract the bytes until null")
                .to_str()
                .expect("Error when try to convert CStr to a string");

            let byte_sha = &content_copy[header.len() + 1..20 + header.len() + 1];
            let mut sha_1 = Vec::new();
            for b in byte_sha {
                write!(&mut sha_1, "{:02X?}", b).expect("Error when try to write sha_1");
            }

            let sha_1_string = String::from_utf8(sha_1)
                .expect("Unable to transform sha_1 as string")
                .to_lowercase();

            let mut parts = header.split_whitespace();

            let mode = format!(
                "{:06}",
                parts
                    .next()
                    .expect("Error when try to get the mode")
                    .parse::<i32>()
                    .expect("Error when try to parse mode")
            );
            let name = parts.next().expect("Error when try to get the name");

            let object_type = match mode.as_str() {
                "100644" => "blob",
                "100755" => "blob",
                "120000" => "blob",
                "040000" => "tree",
                _ => panic!("The tree has a unsupported object type"),
            };

            let content_to_push = match name_only {
                false => format!("{mode} {object_type} {sha_1_string}    {name}\n"),
                _ => format!("{name}\n"),
            };

            final_content.append(&mut Vec::from(content_to_push));
            content_copy = content_copy[header.len() + 1 + 20..].to_vec();
        }

        String::from_utf8(final_content).expect("Erro Trying to convert final content to string")
    }
}
