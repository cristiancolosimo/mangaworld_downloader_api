

    use serde::{Serialize,Deserialize};

    #[derive(Debug,Clone,Deserialize, Serialize)]
    struct SyncItem{
        sync:bool,
        url:String
    }
    #[derive(Debug,Clone,Deserialize, Serialize)]
    pub struct Synclist{
        lastsync:u128,
        synclist:Vec<SyncItem>
    }
    impl Synclist{
        pub fn get()-> Synclist{
            match std::fs::read_to_string("/manga/synclist.json"){
                Ok(str)=> {
                    match serde_json::from_str(&str){
                        Ok(synclist) => synclist,
                        Err(_)=> panic!("Errore nella lettura della synclist, formato json invalido")
                    }
                },
                Err(_)=> {
                    use std::io::prelude::*;
                    let tamplate = serde_json::to_string(&Synclist{
                        synclist:Vec::new(),
                        lastsync:0
                    }).unwrap();

                    let mut file = match std::fs::File::create("/manga/synclist.json"){
                        Ok(file)=> file,
                        Err(_)=> panic!("Errore nella  creazione della /manga/synclist, errore permessi")
                    };
                    file.write_all(tamplate.as_bytes()).unwrap();
                    file.sync_all().unwrap();
            
                    panic!("Errore nella lettura della synclist, synclist.json non trovato o errore permessi");
                }
            }
        }
        
        pub fn set(&self){
            use std::io::prelude::*;

            let state_string = serde_json::to_string(&self).unwrap();
            let file_path = "/manga/synclist.json";
            let mut file = match std::fs::File::create(&file_path){
                Ok(file)=> file,
                Err(_)=>  std::fs::File::open(&file_path).unwrap()
            };
            file.write_all(state_string.as_bytes()).unwrap();
            file.sync_all().unwrap();
            println!("synclist saved");

        }
        pub async fn sync_manga(&mut self) -> Vec<String> {
            pub use crate::data::manga::Manga;
            let mut title_list  : Vec<String>= Vec::new();
            for i in 0..self.synclist.len(){
                if self.synclist[i].sync {
                    let data = Manga::new_from_url(String::from(self.synclist[i].url.clone())).await.unwrap(); 
                    title_list.push(data.title.clone());
                    data.save_state();
                }
            }
            self.lastsync = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis();
            self.set();
        
            return title_list;
        }
        pub async fn sync_update_pages(&mut self, title_list : &Vec<String>) {
            pub use crate::data::manga::Manga;
            
            for i in 0..title_list.len(){
                
                let mut data = Manga::new_from_file_from_title(&title_list[i].clone()).unwrap(); 
                data.update_pages().await;
                data.save_state();
            
            }
            
        }
        pub async fn sync_download(&mut self, title_list : &Vec<String>){ //da eseguire su un thread diverso per non bloccare il resto delle istruzioni
            pub use crate::data::manga::Manga;
            
            let title_list = title_list.clone();
            let handler = std::thread::spawn( move|| {
                for i in 0..title_list.len(){
                    let mut manga = Manga::new_from_file_from_title(&title_list[i].clone()).unwrap(); 
                    manga.download();
                }
            });
            handler.join().unwrap();

        }
    }
    
