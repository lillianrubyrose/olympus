use std::{future::Future, marker::PhantomData};

use async_trait::async_trait;
pub use olympus_net_common::{bytes::BytesMut, ProcedureInput, ProcedureOutput, Result};

#[async_trait]
pub trait Procedure<Ctx>: Send {
	async fn call(&self, context: Ctx, input: BytesMut) -> Result<BytesMut>;
}

#[derive(Clone)]
pub struct ProcedureHolder<F, T>(F, PhantomData<T>);

impl<F, T> ProcedureHolder<F, T> {
	pub fn new(procedure: F) -> Self {
		Self(procedure, PhantomData)
	}
}

#[async_trait]
impl<Ctx, F, Fut, Res, I> Procedure<Ctx> for ProcedureHolder<F, I>
where
	Ctx: Send + 'static,
	F: Fn(Ctx, I) -> Fut + Send + Sync,
	Fut: Future<Output = Result<Res>> + Send,
	Res: ProcedureOutput,
	I: ProcedureInput + Send + Sync,
{
	async fn call(&self, context: Ctx, mut input: BytesMut) -> Result<BytesMut> {
		Ok(self.0(context, I::deserialize(&mut input)).await?.serialize())
	}
}
