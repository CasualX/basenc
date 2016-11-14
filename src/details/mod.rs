/*!
Implementation details.
*/

use ::core::ops;

pub mod base64;
pub mod hex;

//----------------------------------------------------------------

pub trait Chunk: ops::Index<usize> {
	fn len(&self) -> usize;
}

fn id<T>(t: T) -> T { t }

//----------------------------------------------------------------

/// FlatMap an iterator over `Result`s while preserving the error.
///
/// Specifically, given a `Result<IterT, E>` where `IterT: IntoIterator` flat map to `Result<IterT::Item, E>`.
pub struct ResultFlatMap<IterT, E>(Result<IterT, Option<E>>);
impl<IterT: Iterator, E> ResultFlatMap<IterT, E> {
	pub fn new<T>(result: Result<T, E>) -> Self
		where T: IntoIterator<Item = IterT::Item, IntoIter = IterT>
	{
		ResultFlatMap(match result {
			Ok(ok) => Ok(ok.into_iter()),
			Err(err) => Err(Some(err)),
		})
	}
}
impl<IterT: Iterator, E> Iterator for ResultFlatMap<IterT, E> {
	type Item = Result<IterT::Item, E>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.0 {
			Ok(ref mut ok) => ok.next().map(Ok),
			Err(ref mut err) => err.take().map(Err),
		}
	}
}
