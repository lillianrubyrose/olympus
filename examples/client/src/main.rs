use std::time::Duration;

use common::models::{File, GetFileParams};
use olympus_client::OlympusClient;

async fn get_file_handler(_: OlympusClient<()>, file: File) {
	dbg!(file.path);
	dbg!(file.size);

	let content = String::from_utf8_lossy(&file.content);
	dbg!(content);
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let mut client = OlympusClient::new(());
	client.on_response("getFile", get_file_handler).await;

	client.connect("127.0.0.1:9999".parse()?).await?;
	client
		.send(
			"getFile",
			&GetFileParams {
				path: "/home/lily/dev/olympus/Cargo.toml".into(),
				after_action: None,
			},
		)
		.unwrap();

	tokio::time::sleep(Duration::from_millis(100)).await;

	Ok(())
}
