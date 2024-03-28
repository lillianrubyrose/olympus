use common::models::{DeleteFileParams, File, GetFileParams};
use common::traits::ServerRpc;
use olympus_net_common::{async_trait, Result, Variable};
use olympus_server::OlympusServer;

type Context = ();

pub struct ServerImpl;

#[allow(non_snake_case)]
#[async_trait]
impl ServerRpc<Context> for ServerImpl {
	async fn get_server_version(_context: Context) -> Result<i8> {
		Ok(69)
	}

	async fn get_file(_context: Context, params: GetFileParams) -> Result<File> {
		dbg!(params.after_action);

		let content = tokio::fs::read(&params.path).await?;
		Ok(File {
			path: params.path,
			size: Variable(content.len() as u64),
			content,
		})
	}

	async fn delete_file(_context: Context, _params: DeleteFileParams) -> Result<()> {
		unimplemented!()
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	let mut server = OlympusServer::new(());
	// TODO: make compiler spit out helper function that does this registration and wrapping of procs without params!
	server
		.register_procedure("getServerVersion", |ctx, (): ()| ServerImpl::get_server_version(ctx))
		.await;
	server.register_procedure("getFile", ServerImpl::get_file).await;
	server.register_procedure("deleteFile", ServerImpl::delete_file).await;

	println!("Listening @ tcp://127.0.0.1:9999");
	server.run("127.0.0.1:9999".parse()?).await?;
	Ok(())
}
