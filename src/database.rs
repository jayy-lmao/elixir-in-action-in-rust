use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use serde_json::{from_str, to_string};
use std::fs::{self, File};
use std::io::{Read, Write};

use crate::list::TodoList;

pub struct TodoDatabase;

pub enum TodoDatabaseMessage {
    Store {
        key: String,
        list: TodoList,
    },
    Get {
        key: String,
        reply: RpcReplyPort<Option<TodoList>>,
    },
}

impl TodoDatabase {
    const DB_FOLDER: &'static str = "./persist";

    pub fn store(name: &str, todo_list: &TodoList) {
        fs::create_dir_all(Self::DB_FOLDER).unwrap();
        let path = format!("{}/{}.json", Self::DB_FOLDER, name);
        let data = to_string(todo_list).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    }

    pub fn get(name: &str) -> Option<TodoList> {
        let path = format!("{}/{}.json", Self::DB_FOLDER, name);
        if let Ok(mut file) = File::open(path) {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            from_str(&data).ok()
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl Actor for TodoDatabase {
    type Msg = TodoDatabaseMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Starting TodoDatabase");
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            TodoDatabaseMessage::Store { key, list } => {
                TodoDatabase::store(&key, &list);
            }
            TodoDatabaseMessage::Get { key, reply } => {
                let res = TodoDatabase::get(&key);
                let _ = reply.send(res);
            }
        }

        Ok(())
    }
}
