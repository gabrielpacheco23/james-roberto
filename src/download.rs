use crate::my_instants::MyInstant;
use downloader::{Download, Downloader};
//use std::fs::File;
//use std::fs::OpenOptions;
//use std::io::Write;
use std::path::Path;

pub fn download_instant(instant: &MyInstant) -> bool {
    // let filename = &format!("{}.mp3", &instant.name);
    let dir = Path::new("./audios");
    let mut downloader = Downloader::builder()
        .download_folder(dir)
        .parallel_requests(1)
        .build()
        .unwrap();

    let audio_to_download = Download::new(&instant.url);
    // .file_name(Path::new(filename));
    let res = downloader.download(&[audio_to_download]).unwrap();

    for r in res {
        match r {
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
            Ok(s) => println!("Success: {}", &s),
        }
    }

    let old_name: Vec<&str> = instant.url.split('/').collect();
    let old_name = *old_name.last().unwrap();
    let old_path = &format!("{}/{}", dir.display(), old_name);
    let filename = &format!("{}/{}.mp3", dir.display(), &instant.name);

    //    let commands_file_path = Path::new("./audios/__commands.txt");
    //    if !commands_file_path.exists() {
    //        let _ = File::create(commands_file_path);
    //    }
    //
    //    let mut commands_file = OpenOptions::new()
    //        .append(true)
    //        .open(&commands_file_path)
    //        .unwrap();
    //
    //    if let Err(e) = writeln!(&mut commands_file, "{}", &instant.name) {
    //        eprintln!("Couldn't write to file: {}", e);
    //    }

    std::fs::rename(old_path, filename).unwrap();
    true
}
