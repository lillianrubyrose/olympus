mod callback;
mod callback_ext;

use std::collections::HashMap;

use crate::callback::{Callback, CallbackHolder};

async fn handler(s: String) -> usize {
	s.len()
}

#[tokio::main]
async fn main() {
	let mut handlers = HashMap::<i16, Box<dyn Callback>>::new();
	handlers.insert(1337, Box::new(CallbackHolder::new(handler)));

	let out = handlers.get(&1337).unwrap();
	let out = out.call("hello world".as_bytes()).await;
	println!("should be 11 - {}", usize::from_be_bytes(out[..8].try_into().unwrap()));
}
