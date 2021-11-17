
use std::{io::{BufRead, BufReader, BufWriter, Write}, path::Path};

const KEY: [u8; 16] = [
    0x5E, 0x7A, 0x20, 0x02, 0x30, 0x2E, 0xEB, 0x1A, 0x3B, 0xB6, 0x17, 0xC3, 0x0F, 0xDE, 0x1E, 0x47,
];

fn main() {
    let archive = std::fs::File::open(r#"D:\RobertSpaceIndustries\StarCitizen\LIVE\Data.p4k"#).unwrap();
    let out_path = Path::new(r#"D:\tmp"#);

    let mut zip = zip::ZipArchive::new(archive).unwrap();
    let len = zip.len();
    for i in 0..len {
        let zipped_file = zip.by_index_decrypt(i, &KEY).unwrap().unwrap();
        let path = out_path.join(Path::new(zipped_file.name()));
        if !path.parent().unwrap().exists() {
            std::fs::create_dir(path.parent().unwrap()).unwrap();
        }
        let out_file = std::fs::File::create(path).unwrap();
        let mut reader = BufReader::new(zipped_file);
        let mut writer = BufWriter::new(out_file);
        let mut length = 1;
        while length > 0 {
            let buffer = reader.fill_buf().unwrap();
            writer.write(buffer).unwrap();
            length = buffer.len();
            reader.consume(length);
        }
    }
}