use std::fmt::Debug;

pub mod accessors;
pub mod mutators;
pub mod structs;

pub trait BaseQuery: Debug + Clone + PartialEq + Eq + Send + Sync + 'static {}

impl<T: Debug + Clone + PartialEq + Eq + Send + Sync + 'static> BaseQuery for T {}

pub trait BasePath: Debug + Sized + Clone + PartialEq + Eq + Send + Sync + Unpin + 'static {}

impl<T: Debug + Sized + Clone + PartialEq + Eq + Send + Sync + Unpin + 'static> BasePath for T {}
