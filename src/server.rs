use chrono::NaiveDate;
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

use crate::{database::TodoDatabaseMessage, entry::TodoEntry, list::TodoList};

pub struct TodoServer;

#[derive(Clone)]
pub struct TodoServerState {
    database_actor: ActorRef<TodoDatabaseMessage>,
    name: String,
    todo_list: TodoList,
}

pub enum TodoServerMessage {
    Crash,
    Post {
        entry: TodoEntry,
    },
    Get {
        date: NaiveDate,
        reply: RpcReplyPort<Vec<TodoEntry>>,
    },
}

#[async_trait::async_trait]
impl Actor for TodoServer {
    type Msg = TodoServerMessage;
    type State = TodoServerState;
    type Arguments = (String, ActorRef<TodoDatabaseMessage>);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        (name, database_ref): (String, ActorRef<TodoDatabaseMessage>),
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Starting TodoServer");
        let existing_list = database_ref
            .call(
                |reply| TodoDatabaseMessage::Get {
                    key: name.clone(),
                    reply,
                },
                None,
            )
            .await
            .expect("could not send")
            .expect("could not get db list");

        Ok(TodoServerState {
            database_actor: database_ref,
            name,
            todo_list: existing_list.unwrap_or(TodoList::new()),
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        println!("Server handling");
        match message {
            TodoServerMessage::Crash => {
                panic!("OH NOO!")
            }
            TodoServerMessage::Post { entry } => {
                state.todo_list.add_entry(entry);
                let _ = state.database_actor.cast(TodoDatabaseMessage::Store {
                    key: state.name.clone(),
                    list: state.todo_list.clone(),
                });
            }
            TodoServerMessage::Get { date, reply } => {
                let entries = state.todo_list.entries(date).into_iter().cloned().collect();
                let _ = reply.send(entries);
            }
        }

        Ok(())
    }
}
