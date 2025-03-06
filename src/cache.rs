use std::collections::HashMap;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, SupervisionEvent};

use crate::{
    database::{TodoDatabase, TodoDatabaseMessage},
    server::{TodoServer, TodoServerMessage},
};

pub struct TodoCache;

#[derive(Clone)]
pub struct TodoCacheState {
    database_actor: ActorRef<TodoDatabaseMessage>,
    todo_servers: HashMap<String, ActorRef<TodoServerMessage>>,
}

pub enum TodoCacheMessage {
    ServerProcess {
        name: String,
        reply: RpcReplyPort<ActorRef<TodoServerMessage>>,
    },
}

#[async_trait::async_trait]
impl Actor for TodoCache {
    type Msg = TodoCacheMessage;
    type State = TodoCacheState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        let (database_actor, _) = Actor::spawn_linked(
            Some("database-actor".to_string()),
            TodoDatabase,
            (),
            myself.into(),
        )
        .await?;

        println!("Starting TodoCache");
        Ok(TodoCacheState {
            database_actor,
            todo_servers: HashMap::new(),
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            TodoCacheMessage::ServerProcess { name, reply } => {
                match state.todo_servers.get(&name) {
                    Some(server) => {
                        reply.send(server.clone()).unwrap();
                    }
                    None => {
                        let (server_actor, _) = Actor::spawn_linked(
                            Some(format!("server-actor:{name}")),
                            TodoServer,
                            (name.clone(), state.database_actor.clone()),
                            myself.into(),
                        )
                        .await?;
                        state
                            .todo_servers
                            .insert(name.clone(), server_actor.clone());

                        reply.send(server_actor).unwrap();
                    }
                }
            }
        };

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let mut name_to_delete = None;
        match message {
            SupervisionEvent::ActorTerminated(who, _, _reason) => {
                for (name, todo_server) in state.todo_servers.iter() {
                    if todo_server.get_id() == who.get_id() {
                        name_to_delete = Some(name.clone());
                    }
                }
            }
            SupervisionEvent::ActorFailed(who, _reason) => {
                for (name, todo_server) in state.todo_servers.iter() {
                    if todo_server.get_id() == who.get_id() {
                        name_to_delete = Some(name.clone());
                    }
                }
            }
            _ => {}
        }

        if let Some(name) = name_to_delete {
            state.todo_servers.remove(&name);
        }

        Ok(())
    }
}
