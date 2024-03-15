use std::{future::Future, marker::PhantomData};

use async_trait::async_trait;

pub trait CallbackInput {
	fn deserialize(input: &[u8]) -> Self;
}

pub trait CallbackOutput {
	fn serialize(self) -> Vec<u8>;
}

#[async_trait]
pub trait Callback<Ctx>: Send + Sync {
	async fn call<'a>(&'a self, context: Ctx, input: &'a [u8]) -> Vec<u8>;
}

#[derive(Clone)]
pub struct CallbackHolder<F, T>(F, PhantomData<T>);

impl<F, T> CallbackHolder<F, T> {
	pub fn new(callback: F) -> Self {
		Self(callback, PhantomData)
	}
}

#[async_trait]
impl<Ctx, F, Fut, Res, I> Callback<Ctx> for CallbackHolder<F, I>
where
	Ctx: Send + Sync + 'static,
	F: Fn(Ctx, I) -> Fut + Clone + Send + Sync + 'static,
	Fut: Future<Output = Res> + Send,
	Res: CallbackOutput,
	I: CallbackInput + Send + Sync,
{
	async fn call<'a>(&'a self, context: Ctx, input: &'a [u8]) -> Vec<u8> {
		self.0(context, I::deserialize(input)).await.serialize()
	}
}
