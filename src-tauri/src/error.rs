use diesel;
use quick_error::quick_error;
use tiny_tokio_actor::ActorError;

use std::convert::From;

quick_error! {
    #[derive(Debug)]
    pub enum PantryError {
        LLMNotRunning {
            display("LLM Not Running")
        }
        ActorFailure (err: ActorError) {
            display("ActorError failure: {:?}", err)
            from()
        }
        OtherFailure(err: String) {
            display("Other Error: {:?}", err)
            from()
        }
        DatabaseError(err: diesel::result::Error) {
            display("Database Error: {:?}", err)
            from()
        }
    }
}

// #[derive(Debug)]
// pub enum PantryError {
//     LLMNotRunning,
// }

// impl Into<String> for PantryError {
//     fn into(self) -> String {
//         "test".into()
//     }

// }

// impl fmt::Display for PantryError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // write your error formatting logic here...
//         match self {
//             PantryError::LLMNotRunning => write!(f, "LLM isn't running"),
//             // Write match arms for your error variants here...
//             // For example, if you have a variant `Unexpected`, you might write:
//             // YourError::Unexpected => write!(f, "An unexpected error occurred"),
//         }
//     }
// }
