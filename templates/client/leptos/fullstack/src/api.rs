use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[server]
pub async fn get_todos() -> Result<Vec<Todo>, ServerFnError> {
    // In a real app, this would fetch from a database
    Ok(vec![
        Todo {
            id: Uuid::new_v4().to_string(),
            title: "Learn Leptos".to_string(),
            completed: false,
        },
        Todo {
            id: Uuid::new_v4().to_string(),
            title: "Build a fullstack app".to_string(),
            completed: false,
        },
        Todo {
            id: Uuid::new_v4().to_string(),
            title: "Enjoy Rust on the web".to_string(),
            completed: false,
        },
    ])
}

#[server]
pub async fn add_todo(title: String) -> Result<Todo, ServerFnError> {
    // In a real app, this would add to a database
    Ok(Todo {
        id: Uuid::new_v4().to_string(),
        title,
        completed: false,
    })
}

#[server]
pub async fn update_todo(id: String, completed: bool) -> Result<(), ServerFnError> {
    // In a real app, this would update the database
    log::info!("Updated todo {id} to completed={completed}");
    Ok(())
}

#[server]
pub async fn delete_todo(id: String) -> Result<(), ServerFnError> {
    // In a real app, this would delete from the database
    log::info!("Deleted todo {id}");
    Ok(())
}
