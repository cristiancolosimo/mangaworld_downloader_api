use scraper::Html;
use scraper::Selector;
use serde::{Serialize,Deserialize};


#[derive(Debug,Clone,Deserialize, Serialize)]
pub struct ChapterRef{
	pub url:String,
	pub name:String,
    pub downloaded:bool,
    pub pages: Vec<String>,
    pub volume_part: bool,
    pub volume_name:String
	
}

impl ChapterRef{
   pub async fn get_pages(&mut self) -> Result<bool,Box<dyn std::error::Error>>{ //get pages of chapter if there isn't
    if self.pages.len() > 0 {
        return Ok(true);
    }
    println!("non scansionato prima");
    let url : String = if self.url.ends_with("?style=list") {
        self.url.clone()
    }else{
        let url = self.url.clone();
        let mut url = url.replace("?style=page", "");
        url.push_str("?style=list");
        url
    };

    
    let resp = reqwest::get(url)
    .await?.text().await?;
    let document = Html::parse_document(&resp);

    let img_page_selector = Selector::parse(".page-image").unwrap();
    let pages_urls:Vec<String> = document.select(&img_page_selector).map(|el| {
        el.value().attr("src").unwrap().to_string()
    }).collect();
    self.pages = pages_urls;


    Ok(false)
   }
   pub fn download_pages(&mut self,mangatitle: &String)-> Result<(),Box<dyn std::error::Error>>{
        use std::io::prelude::*;
        use zip::write::FileOptions;
        let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

        if self.downloaded {
            return Ok(());
        }
        
        
        let path: String = if self.volume_part {
            format!("/manga/download/{} {} {}.zip",mangatitle.trim(),self.volume_name.clone().trim(),self.name.clone().trim())
        } else{
            format!("/manga/download/{} {}.zip",mangatitle.trim(),self.name.clone().trim())
        };

        match std::fs::remove_file(&path){ _=>{} };//se succede qualsiasi cosa continua lo stesso 

        

        let file = std::fs::File::create(&path).unwrap();
        
        let mut zip = zip::ZipWriter::new(file);

        for (i,page) in self.pages.iter().enumerate(){
            zip.start_file(format!("{}.jpg",i), options).unwrap();
            zip.write_all(&reqwest::blocking::get(page).unwrap().bytes().unwrap()).unwrap();
        }
        zip.finish().unwrap();
  
        self.downloaded = true;
        Ok(())
   }
}

