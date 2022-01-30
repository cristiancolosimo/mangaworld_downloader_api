use std::thread;

use crate::data::manga::{Manga};

pub fn file_download_sync(url:String,path:String){
    let mut dest = std::fs::File::create(&path).unwrap();
    let mut response = reqwest::blocking::get(url).unwrap();
    std::io::copy(&mut response, &mut dest).unwrap();

}

