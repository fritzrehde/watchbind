mod operation;

pub use operation::Operation;

use anyhow::Result;
use derive_more::Deref;

#[derive(Clone, Deref)]
pub struct Operations(Vec<Operation>);

impl TryFrom<Vec<String>> for Operations {
	type Error = anyhow::Error;
	fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
		let operations = vec
			.into_iter()
			.map(|op| op.parse())
			.collect::<Result<_>>()?;
		Ok(Self(operations))
	}
}
