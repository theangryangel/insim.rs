pub use futures::{SinkExt, StreamExt}; // include StreamExt and SinkExt so the users dont have to

pub use crate::connection::{Connection, ConnectionBuilder, ConnectionTrait};
