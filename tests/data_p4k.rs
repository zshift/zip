use std::{
    io::{BufReader, BufWriter},
    path::Path,
};

const KEY: [u8; 16] = [
    0x5E, 0x7A, 0x20, 0x02, 0x30, 0x2E, 0xEB, 0x1A, 0x3B, 0xB6, 0x17, 0xC3, 0x0F, 0xDE, 0x1E, 0x47,
];

#[test]
pub fn unpack_data_p4k_test() {
    let archive =
        std::fs::File::open("D:/RobertSpaceIndustries/StarCitizen/LIVE/Data.p4k").unwrap();
    let out_path = Path::new("D:/tmp");

    let mut zip = zip::ZipArchive::new(archive).unwrap();
    for i in 0..10 {
        let zipped_file = zip.by_index_decrypt(i, &KEY).unwrap().unwrap();
        let path = out_path.join(Path::new(zipped_file.name()));
        if !path.parent().unwrap().exists() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        }
        let out_file = std::fs::File::create(path).unwrap();
        let mut reader = BufReader::new(zipped_file);
        let mut writer = BufWriter::new(out_file);

        std::io::copy(&mut reader, &mut writer).unwrap();
    }
}
