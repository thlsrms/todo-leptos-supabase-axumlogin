## TodoApp with leptos_supabase_axum-login

A simple leptos CRUD app with authentication and session management.

### Running:
Setup a new Supabase project and execute the **supabase.sql** query.

Install [cargo-leptos](https://github.com/leptos-rs/cargo-leptos?tab=readme-ov-file#getting-started) `cargo install --locked cargo-leptos` and optionally [just](https://github.com/casey/just) `cargo install just` 

Rename the **.env.example** to **.env** and set your supabase project env vars.

Run with `just run` 
or source your **.env** file and `cargo leptos watch --release`
