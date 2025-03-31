use leptos::*;
use leptos::prelude::*;
use leptos::ev;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys;

// Local storage interaction
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = localStorage)]
    fn getItem(key: &str) -> Option<String>;
    
    #[wasm_bindgen(js_namespace = localStorage)]
    fn setItem(key: &str, val: &str);
}

// Todo item model
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Todo {
    id: String,
    text: String,
    completed: bool,
}

// Filter options for todos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
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
    // Load todos from local storage or use defaults
    let initial_todos = move || -> Vec<Todo> {
        if let Some(stored) = getItem("leptos-todos") {
            match serde_json::from_str(&stored) {
                Ok(todos) => todos,
                Err(_) => default_todos(),
            }
        } else {
            default_todos()
        }
    };

    // Create reactive signals using the newer API
    let todos = RwSignal::new(initial_todos());
    let filter = RwSignal::new(Filter::All);
    let new_todo_text = RwSignal::new(String::new());
    let editing = RwSignal::new(None::<String>);
    let edit_text = RwSignal::new(String::new());
    
    // Save todos to local storage whenever they change using Effect
    Effect::new(move |_| {
        let todos_json = serde_json::to_string(&todos.get()).unwrap_or_default();
        setItem("leptos-todos", &todos_json);
    });
    
    // Derived signal for filtered todos using Memo::new
    let filtered_todos = Memo::new(move |_| {
        todos.get()
            .into_iter()
            .filter(|todo| filter.get().matches(todo))
            .collect::<Vec<_>>()
    });
    
    // Derived signal for active todo count
    let active_count = Memo::new(move |_| {
        todos.get()
            .iter()
            .filter(|todo| !todo.completed)
            .count()
    });
    
    // Derived signal for completed todo count
    let completed_count = Memo::new(move |_| {
        todos.get()
            .iter()
            .filter(|todo| todo.completed)
            .count()
    });
    
    // Add a new todo
    let add_todo = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let text = new_todo_text.get().trim().to_string();
        if !text.is_empty() {
            let new_todo = Todo {
                id: Uuid::new_v4().to_string(),
                text,
                completed: false,
            };
            
            todos.update(|t| {
                t.push(new_todo);
            });
            
            new_todo_text.set(String::new());
        }
    };
    
    // Toggle a todo's completed status
    let toggle_todo = move |id: &str| {
        todos.update(|t| {
            if let Some(todo) = t.iter_mut().find(|t| t.id == id) {
                todo.completed = !todo.completed;
            }
        });
    };
    
    // Start editing a todo
    let start_editing = move |id: String, current_text: String| {
        editing.set(Some(id));
        edit_text.set(current_text);
    };
    
    // Save edits to a todo
    let save_edit = move |id: &str| {
        let trimmed_text = edit_text.get().trim().to_string();
        
        if trimmed_text.is_empty() {
            // If empty, remove the todo
            todos.update(|t| {
                t.retain(|todo| todo.id != id);
            });
        } else {
            // Otherwise update the text
            todos.update(|t| {
                if let Some(todo) = t.iter_mut().find(|t| t.id == id) {
                    todo.text = trimmed_text;
                }
            });
        }
        
        // Clear editing state
        editing.set(None);
    };
    
    // Cancel editing
    let cancel_edit = move |_| {
        editing.set(None);
    };
    
    // Delete a todo
    let delete_todo = move |id: &str| {
        todos.update(|t| {
            t.retain(|todo| todo.id != id);
        });
    };
    
    // Toggle all todos
    let toggle_all = move |_| {
        let all_completed = todos.get().iter().all(|todo| todo.completed);
        
        todos.update(|t| {
            for todo in t.iter_mut() {
                todo.completed = !all_completed;
            }
        });
    };
    
    // Clear completed todos
    let clear_completed = move |_| {
        todos.update(|t| {
            t.retain(|todo| !todo.completed);
        });
    };
    
    view! {
        <div class="todoapp">
            <header class="header">
                <h1>"todos"</h1>
                <form on:submit=add_todo>
                    <input 
                        class="new-todo"
                        placeholder="What needs to be done?"
                        autofocus
                        prop:value=move || new_todo_text.get()
                        on:input=move |ev| new_todo_text.set(event_target_value(&ev))
                    />
                </form>
            </header>
            
            <Show
                when=move || !todos.get().is_empty()
                fallback=move || view! { <div class="empty-state">"Add your first todo above!"</div> }
            >
                <section class="main">
                    <input 
                        id="toggle-all"
                        class="toggle-all"
                        type="checkbox"
                        prop:checked=move || todos.get().iter().all(|todo| todo.completed)
                        on:change=toggle_all
                    />
                    <label for="toggle-all">"Mark all as complete"</label>
                    
                    <ul class="todo-list">
                        <For
                            each=move || filtered_todos.get()
                            key=|todo| todo.id.clone()
                            children=move |todo| {
                                // Clone the ID once to avoid multiple clones in closures
                                let todo_id = todo.id.clone();
                                
                                let view_fn = move || view! {
                                    <li class="todo-item" 
                                        class:completed=move || todo.completed
                                        class:editing=move || editing.get() == Some(todo_id.clone())
                                    >
                                        <Show
                                            when=move || editing.get() == Some(todo_id.clone())
                                            fallback=move || view! {
                                                <div class="view">
                                                    <input 
                                                        class="toggle"
                                                        type="checkbox"
                                                        prop:checked=move || todo.completed
                                                        on:change=move |_| toggle_todo(&todo_id)
                                                    />
                                                    <label 
                                                        on:dblclick=move |_| start_editing(todo_id.clone(), todo.text.clone())
                                                    >
                                                        {todo.text.clone()}
                                                    </label>
                                                    <button 
                                                        class="destroy"
                                                        on:click=move |_| delete_todo(&todo_id)
                                                    />
                                                </div>
                                            }
                                        >
                                            <input 
                                                class="edit"
                                                prop:value=move || edit_text.get()
                                                on:input=move |ev| edit_text.set(event_target_value(&ev))
                                                on:blur=move |_: ev::FocusEvent| save_edit(&todo_id)
                                                on:keyup=move |ev: ev::KeyboardEvent| {
                                                    match ev.key().as_str() {
                                                        "Escape" => cancel_edit(()),
                                                        "Enter" => save_edit(&todo_id),
                                                        _ => {}
                                                    }
                                                }
                                                node_ref={
                                                    let input_ref = NodeRef::new();
                                                    
                                                    // Focus the input when it's rendered
                                                    Effect::new(move |_| {
                                                        if let Some(element) = input_ref.get() {
                                                            let input = wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlInputElement>(&element);
                                                            if let Some(input) = input {
                                                                let _ = input.focus();
                                                                let _ = input.select();
                                                            }
                                                        }
                                                    });
                                                    
                                                    input_ref
                                                }
                                            />
                                        </Show>
                                    </li>
                                };
                                view_fn()
                            }
                        />
                    </ul>
                </section>
                
                <footer class="footer">
                    <span class="todo-count">
                        <strong>{active_count}</strong>
                        {move || if active_count.get() == 1 { " item " } else { " items " }}
                        "left"
                    </span>
                    
                    <ul class="filters">
                        <li>
                            <a 
                                class="filter-btn"
                                class:selected=move || filter.get() == Filter::All
                                on:click=move |_| filter.set(Filter::All)
                            >
                                "All"
                            </a>
                        </li>
                        <li>
                            <a 
                                class="filter-btn"
                                class:selected=move || filter.get() == Filter::Active
                                on:click=move |_| filter.set(Filter::Active)
                            >
                                "Active"
                            </a>
                        </li>
                        <li>
                            <a 
                                class="filter-btn"
                                class:selected=move || filter.get() == Filter::Completed
                                on:click=move |_| filter.set(Filter::Completed)
                            >
                                "Completed"
                            </a>
                        </li>
                    </ul>
                    
                    <Show when=move || completed_count.get() != 0>
                        <button 
                            class="clear-completed"
                            on:click=clear_completed
                        >
                            "Clear completed"
                        </button>
                    </Show>
                </footer>
            </Show>
            
            <footer class="info">
                <p>"Double-click to edit a todo"</p>
                <p>"Created with " <a href="https://leptos.dev">"Leptos"</a></p>
                <p>"Part of " <a href="https://todomvc.com">"TodoMVC"</a></p>
            </footer>
        </div>
    }
}

// Default todos for new users
fn default_todos() -> Vec<Todo> {
    vec![
        Todo { id: Uuid::new_v4().to_string(), text: "Learn Leptos".to_string(), completed: true },
        Todo { id: Uuid::new_v4().to_string(), text: "Build an awesome app".to_string(), completed: false },
    ]
}
