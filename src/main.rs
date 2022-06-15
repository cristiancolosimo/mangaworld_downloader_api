
mod lib;
mod data;
mod synclist;
pub use crate::data::manga::Manga;
pub use crate::synclist::synclist::Synclist;
use std::time::Duration;

fn setup(){
    //mkdir state
    std::fs::create_dir_all("/manga/state").expect("error creation folder in /manga");
    //mkdir download
    std::fs::create_dir_all("/manga/download").expect("error creation folder in /manga");

}
use warp::Filter;
async fn sync(){
    let mut synclist = Synclist::get();
    let manga_list_title = synclist.sync_manga().await;
    synclist.sync_update_pages(&manga_list_title).await;
    synclist.sync_download(&manga_list_title).await;

}
use std::sync::atomic::AtomicBool;
use std::sync::Arc;



async fn web_server(){

    let hello = warp::path!("sync")
    .map(move || {
        println!("reset intervallo");
        //interval.reset();
        "Start sync"
    });

warp::serve(hello)
    .run(([0, 0, 0, 0], 3030))
    .await;


}


#[tokio::main]
async fn main() {
    setup();
    let mut interval = tokio::time::interval(Duration::from_secs(6 * 60*60));

    //interval.clear();
    let  working  = Arc::new(AtomicBool::new(false));
    let  work = working.clone();
    //let web_server = tokio::task::spawn( ||   {
//
    //});
    //web_server.await;
    
    let download_loop  = tokio::task::spawn( async move{
        loop{
            working.store(true,std::sync::atomic::Ordering::Relaxed);
            sync().await;
            working.store(false,std::sync::atomic::Ordering::Relaxed);
            interval.tick().await;
        }
    });
    
    let test_loop = tokio::task::spawn(async move  {
        let mut test_int = tokio::time::interval(Duration::from_secs(10));

        loop {
            println!("valore : {}", work.load(std::sync::atomic::Ordering::Relaxed));
            test_int.tick().await;
        }
    });
        test_loop.await.unwrap();
        download_loop.await.unwrap();
    
    //
    //

}



