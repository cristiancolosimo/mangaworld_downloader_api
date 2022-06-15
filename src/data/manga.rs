use scraper::Html;
use scraper::Selector;
use serde::{Serialize,Deserialize};
use std::fs::File;
use std::io::prelude::*;

use crate::data::volume::*;
use crate::data::chapter::*;

#[allow(dead_code)]
#[derive(Debug,Clone,Deserialize, Serialize)]
pub struct MangaGenerics{
	pub genres : Vec<String>,
	pub authors : Vec<String>,
	pub artists : Vec<String>,
	pub year_start : String,
}


#[allow(dead_code)]
#[derive(Debug,Clone,Deserialize, Serialize)]
pub struct Manga{
	pub url:String,
	pub title:String,
	pub cover_url:String,
	pub plot:String,
	pub generics : MangaGenerics,
	pub chapters: Vec<ChapterRef>, //chapters
}

impl MangaGenerics{
	fn new_empty()-> MangaGenerics{
		MangaGenerics{
			genres:Vec::new(),
			authors: Vec::new(),
			artists: Vec::new(),
			year_start: String::new()
		}
	}
}

impl Manga{

	pub fn new_from_file_from_title(title:&String)->Result<Manga, ()>{
		let file_path = format!("/manga/state/{}.json",title.to_lowercase().trim());
		
		match std::fs::read_to_string(file_path){
			Ok(string)=>{
				println!("trovato vecchio stato");
				let manga : Manga = serde_json::from_str(&string).unwrap();
				Ok(manga)
			},
			Err(_)=>{
				println!("non trovato vecchio stato");
				Err(())
			} 
		}
		
	}

	pub fn save_state(&self){
		println!("stato salvato");
		let state_string = serde_json::to_string(&self).unwrap();
		let file_path = format!("/manga/state/{}.json",self.title.clone().to_lowercase().trim());
		let mut file = match File::create(&file_path){
			Ok(file)=> file,
			Err(_)=>  File::open(&file_path).unwrap()
		};
		file.write_all(state_string.as_bytes()).unwrap();
		file.sync_all().unwrap();
	}

	fn get_title(dom : &Html)-> String{
		let selector_title = Selector::parse(".info .name.bigger").unwrap();
		let title = dom.select(&selector_title).next().unwrap().text().next().unwrap();
		String::from(title)
	}
	
	fn get_cover_url(dom : &Html)-> String{
		let selector_cover_url = Selector::parse(".single-comic img").unwrap();
		let corver_url_src = dom.select(&selector_cover_url).next().unwrap().value().attr("src").unwrap();
		String::from(corver_url_src)

	}

	fn get_generics(dom : &scraper::ElementRef)->MangaGenerics{
		let selector_sub_frag = Selector::parse(".col-12").unwrap();
		let mut genres: Vec<String> = Vec::new();
		let mut authors: Vec<String> = Vec::new();
		let mut artists: Vec<String> = Vec::new();
		let mut year_start: String = String::new();

		for element in dom.select(&selector_sub_frag){
			let text = element.text().next().unwrap();
			if text.starts_with("Generi") {
				let genres_temp : Vec<&str>= element.text().skip(1).collect::<Vec<_>>(); //.into_iter().map(|el| { return el.toString()});
				genres = genres_temp.into_iter().map(|el| { return String::from(el)}).collect::<Vec<_>>();

			} else if text.starts_with("Autore") || text.starts_with("Autori"){
				let authors_temp = element.text().skip(1)
				.collect::<Vec<_>>().into_iter()
				.map(|el| { return String::from(el)}).collect::<Vec<_>>();
				authors = authors_temp;

			} else if text.starts_with("Artista") ||text.starts_with("Artisti") {
				let artists_temp = element.text().skip(1)
				.collect::<Vec<_>>().into_iter()
				.map(|el| { return String::from(el)}).collect::<Vec<_>>();
				artists = artists_temp;


			} else if text.starts_with("Anno di uscita"){
				year_start = String::from(element.text().skip(1).next().unwrap());

			}

		}
		MangaGenerics{
			genres,
			authors,
			artists,
			year_start
		}

	}

	fn get_plot(dom:&Html)-> String{
		let selector_plot = Selector::parse("#noidungm").unwrap();
		let plot = dom.select(&selector_plot).next().unwrap().text().next().unwrap();
		String::from(plot)
	}

	fn get_volumes(dom: &scraper::ElementRef)-> Vec<Volume>{
		let volume_selector = Selector::parse(".volume-element").unwrap();
		let volume_selector_name = Selector::parse(".volume-name").unwrap();

		let mut volumi:Vec<Volume> = Vec::new();
		for volume in &mut dom.select(&volume_selector){
			let name = String::from(volume.select(&volume_selector_name).next().unwrap().text().next().unwrap());			

			let chapters = Manga::get_chapters(&volume);

			volumi.push(Volume{
				name:name.replace("Volume ", "vol"), //eseguire il replace da Volume a vol per l'indexing di kavita
				chapters: chapters
			});
		} 
		//.volume-element 
		let volumi: Vec<Volume> = volumi.into_iter().rev().collect();
		//fare un reverse del vettore prima di inviarlo
		volumi
	}
	fn get_chapters(dom: &scraper::ElementRef) -> Vec<ChapterRef>{ //per fare in modo che è compatibile sia con la ricerca di volumi che quella di capitoli indipendenti
		let chapters_selector = Selector::parse(".chapter").unwrap();
		let chapter_text = Selector::parse("span.d-inline-block").unwrap();
		let chapter_url = Selector::parse("a.chap").unwrap();
		
		let mut chapters: Vec<ChapterRef> = Vec::new();
		for chapter in dom.select(&chapters_selector){
			let name = String::from(chapter.select(&chapter_text).next().unwrap().text().next().unwrap());
			let url = String::from(chapter.select(&chapter_url).next().unwrap().value().attr("href").unwrap());
			chapters.push(ChapterRef{
				name: name.replace("Capitolo ", "c"),//eseguire il replace da Capotolo a c per l'indexing di kavita
				url,
				pages:Vec::new(),
				downloaded:false,
				volume_name:String::from(""),
				volume_part:false
			});
		}
		let chapters : Vec<ChapterRef> = chapters.into_iter().rev().collect();
		//chapter

		//fare un reverse prima di inviarlo
		chapters
	}

	pub fn new_empty()-> Manga{
		Manga{
			title:String::new(),
			cover_url:String::new(),
			url:String::new(),
			plot:String::new(),
			chapters: Vec::new(), //chapters
			generics:MangaGenerics::new_empty(),

		}
	}

	/*fn get_nfsw(dom: Html)-> bool{

		false
	}*/
	pub async fn new_from_url(url:String)-> Result<Manga, Box<dyn std::error::Error>>{

		let resp = reqwest::get(&url)
		.await?.text().await?;

		let document = Html::parse_document(&resp);

		let selector_frag = Selector::parse(".meta-data").unwrap();
		let document_frag_meta = document.select(&selector_frag).next().unwrap();

		

		let title = Manga::get_title(&document);
		let cover_url_src = Manga::get_cover_url(&document);
		let generics = Manga::get_generics(&document_frag_meta);
		let plot = Manga::get_plot(&document);

		let selector_frag_chapters_wrapper = Selector::parse(".chapters-wrapper").unwrap(); 
		let document_frag_chapters_wrapper = document.select(&selector_frag_chapters_wrapper).next().unwrap();
		let volumes = Manga::get_volumes(&document_frag_chapters_wrapper);
		
		let oldstate_chapters:Vec<ChapterRef> = match Manga::new_from_file_from_title(&title){
			Ok(manga)=> manga.chapters,
			Err(_)=> Vec::new()
		};

		
		let mut chapters: Vec<ChapterRef> = Vec::new();
		for vol in volumes.into_iter(){
			for cap in vol.chapters.into_iter(){
				chapters.push(ChapterRef{
				name:cap.name,
				url:cap.url,
				pages:cap.pages,
				downloaded:cap.downloaded,
				volume_part:true,
				volume_name:vol.name.clone()
				});
			}
			
		}
		if chapters.len() == 0{
			chapters = Manga::get_chapters(&document_frag_chapters_wrapper);
		}

		if oldstate_chapters.len() > 0{
			let mut temp_chapters = oldstate_chapters;
			let mut temp_chapters_new : Vec<ChapterRef>= chapters[temp_chapters.len()..chapters.len()].to_vec();
			temp_chapters.append(&mut temp_chapters_new);
			chapters = temp_chapters;
		}
		//println!("{:?}",volumes);

		

		//https://www.mangaworld.in/manga/2220/owari-no-seraph/

		//let selector_chapters = Selector::parse(".info .name.bigger").unwrap();
		

		Ok(Manga{
			title,
			cover_url:cover_url_src,
			generics,
			url:url,
			plot,
			chapters
		})

	}
	pub fn download(&mut self){
		for i in 0..self.chapters.len(){
			self.chapters[i].download_pages(&self.title).unwrap();
			self.save_state();
			println!("cap scaricato")
		}
	}
	pub async fn update_pages(&mut self){
		for i in 0..self.chapters.len(){
			let already_scanned = self.chapters[i].get_pages().await.unwrap();
			if !already_scanned {
				self.save_state();
			}
			
		}
	}
	
}
	
	

