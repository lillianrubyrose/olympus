mod models;

use crate::models::{File, GetFileParams};
use olympus_net_common::Variable;
use olympus_server::OlympusServer;

type Context = ();

async fn get_file((): Context, params: GetFileParams) -> File {
	dbg!(params.after_action);

	let content = tokio::fs::read(&params.path).await.unwrap();
	File {
		path: params.path,
		size: Variable(content.len() as u64),
		content,
	}
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let mut server = OlympusServer::new(());
	server.register_procedure("getFile", get_file).await;

	println!("Listening @ tcp://127.0.0.1:9999");
	server.run("127.0.0.1:9999".parse()?).await?;
	Ok(())
}
