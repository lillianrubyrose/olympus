use std::time::Duration;

use common::models::{File, GetFileParams};
use olympus_client::OlympusClient;
use olympus_net_common::Result;

async fn get_file_handler(_: OlympusClient<()>, file: File) -> Result<()> {
	dbg!(file.path);
	dbg!(file.size);

	let content = String::from_utf8(file.content)?;
	dbg!(content);
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	let mut client = OlympusClient::new(());
	client.on_response("getFile", get_file_handler).await;

	client.connect("127.0.0.1:9999".parse()?).await?;
	client.send(
		"getFile",
		&GetFileParams {
			path: "/home/lily/dev/olympus/Cargo.toml".into(),
			after_action: None,
		},
	)?;

	tokio::time::sleep(Duration::from_millis(100)).await;

	Ok(())
}
