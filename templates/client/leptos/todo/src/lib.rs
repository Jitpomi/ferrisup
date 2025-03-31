use leptos::*;
use uuid::Uuid;

// Todo item model
#[derive(Debug, Clone, PartialEq, Eq)]
struct Todo {
    id: String,
    text: String,
    completed: bool,
}

// Filter options for todos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    fn matches(&self, todo: &Todo) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !todo.completed,
            Filter::Completed => todo.completed,
        }
    }
}

// Main app component
#[component]
pub fn App() -> impl IntoView {
    // Create reactive signals for todos and filter
    let (todos, set_todos) = create_signal(vec![
        Todo { id: Uuid::new_v4().to_string(), text: "Learn Leptos".to_string(), completed: false },
        Todo { id: Uuid::new_v4().to_string(), text: "Build a todo app".to_string(), completed: false },
        Todo { id: Uuid::new_v4().to_string(), text: "Profit!".to_string(), completed: false },
    ]);
    
    let (filter, set_filter) = create_signal(Filter::All);
    let (new_todo_text, set_new_todo_text) = create_signal(String::new());
    
    // Derived signal for filtered todos
    let filtered_todos = move || {
        todos.get()
            .iter()
            .filter(|todo| filter.get().matches(todo))
            .cloned()
            .collect::<Vec<_>>()
    };
    
    // Count of active todos
    let active_count = move || {
        todos.get()
            .iter()
            .filter(|todo| !todo.completed)
            .count()
    };
    
    // Add a new todo
    let add_todo = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let text = new_todo_text.get();
        if !text.is_empty() {
            set_todos.update(|todos| {
                todos.push(Todo {
                    id: Uuid::new_v4().to_string(),
                    text,
                    completed: false,
                });
            });
            set_new_todo_text.set(String::new());
        }
    };
    
    // Toggle a todo's completed status
    let toggle_todo = move |id: String| {
        set_todos.update(|todos| {
            if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
                todo.completed = !todo.completed;
            }
        });
    };
    
    // Delete a todo
    let delete_todo = move |id: String| {
        set_todos.update(|todos| {
            todos.retain(|t| t.id != id);
        });
    };
    
    // Toggle all todos
    let toggle_all = move |_| {
        let all_completed = todos.get().iter().all(|todo| todo.completed);
        set_todos.update(|todos| {
            for todo in todos.iter_mut() {
                todo.completed = !all_completed;
            }
        });
    };
    
    // Clear completed todos
    let clear_completed = move |_| {
        set_todos.update(|todos| {
            todos.retain(|todo| !todo.completed);
        });
    };
    
    view! {
        <div class="todo-app">
            <h1>"Leptos Todo App"</h1>
            
            <form on:submit=add_todo class="todo-form">
                <input
                    type="text"
                    placeholder="What needs to be done?"
                    prop:value=move || new_todo_text.get()
                    on:input=move |ev| set_new_todo_text.set(event_target_value(&ev))
                />
                <button type="submit">"Add"</button>
            </form>
            
            <div class="todo-controls">
                <button 
                    class="toggle-all"
                    on:click=toggle_all
                    disabled=move || todos.get().is_empty()
                >
                    "Toggle All"
                </button>
                
                <div class="filters">
                    <button 
                        class=move || if filter.get() == Filter::All { "active" } else { "" }
                        on:click=move |_| set_filter.set(Filter::All)
                    >
                        "All"
                    </button>
                    <button 
                        class=move || if filter.get() == Filter::Active { "active" } else { "" }
                        on:click=move |_| set_filter.set(Filter::Active)
                    >
                        "Active"
                    </button>
                    <button 
                        class=move || if filter.get() == Filter::Completed { "active" } else { "" }
                        on:click=move |_| set_filter.set(Filter::Completed)
                    >
                        "Completed"
                    </button>
                </div>
                
                <button 
                    class="clear-completed"
                    on:click=clear_completed
                    disabled=move || !todos.get().iter().any(|todo| todo.completed)
                >
                    "Clear completed"
                </button>
            </div>
            
            <ul class="todo-list">
                <For
                    each=filtered_todos
                    key=|todo| todo.id.clone()
                    children=move |todo| {
                        let todo_id_for_toggle = todo.id.clone();
                        let todo_id_for_delete = todo.id.clone();
                        
                        view! {
                            <li class=move || if todo.completed { "completed" } else { "" }>
                                <div class="todo-item">
                                    <input 
                                        type="checkbox" 
                                        prop:checked=todo.completed
                                        on:change=move |_| {
                                            toggle_todo(todo_id_for_toggle.clone())
                                        }
                                    />
                                    <span>{todo.text.clone()}</span>
                                    <button 
                                        class="delete"
                                        on:click=move |_| {
                                            delete_todo(todo_id_for_delete.clone())
                                        }
                                    >
                                        "×"
                                    </button>
                                </div>
                            </li>
                        }
                    }
                />
            </ul>
            
            <div class="todo-count">
                {move || {
                    let count = active_count();
                    format!("{} item{} left", count, if count == 1 { "" } else { "s" })
                }}
            </div>
        </div>
    }
}
