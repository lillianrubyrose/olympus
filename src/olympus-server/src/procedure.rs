use std::{future::Future, marker::PhantomData};

use async_trait::async_trait;
pub use olympus_net_common::{ProcedureInput, ProcedureOutput, bytes::BytesMut};

#[async_trait]
pub trait Procedure<Ctx>: Send + Sync {
	async fn call(&self, context: Ctx, input: BytesMut) -> BytesMut;
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
	Ctx: Send + Sync + 'static,
	F: Fn(Ctx, I) -> Fut + Clone + Send + Sync + 'static,
	Fut: Future<Output = Res> + Send,
	Res: ProcedureOutput,
	I: ProcedureInput + Send + Sync,
{
	async fn call(&self, context: Ctx, mut input: BytesMut) -> BytesMut {
		self.0(context, I::deserialize(&mut input)).await.serialize()
	}
}
