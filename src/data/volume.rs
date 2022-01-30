
use crate::data::chapter::ChapterRef;
use serde::{Serialize,Deserialize};

#[allow(dead_code)]
#[derive(Debug,Clone,Deserialize, Serialize)]
pub struct Volume{
	pub name:String,
	pub chapters:Vec<ChapterRef>
}


