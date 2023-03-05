mod operation;

pub use operation::Operation;

use anyhow::Result;
use derive_more::IntoIterator;

#[derive(Clone, IntoIterator)]
pub struct Operations(#[into_iterator(ref)] Vec<Operation>);

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
