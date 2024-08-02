use crate::components::auth::AuthProvider;
use crate::components::{Signin, Signup};
use leptos::*;

#[component]
pub fn SignInPage() -> impl IntoView {
    view! {
      <AuthProvider unprotected=true>
        <section class="  uk-section">
          <div class="uk-container uk-container-xsmall">
            <ul class="uk-child-width-expand" uk-tab>
              <li>
                <a href="#">"Login"</a>
              </li>
              <a href="/signup" class="uk-button uk-button-text">
                "Sign up"
              </a>
            </ul>
            <div class="uk-margin">
              <Signin/>
            // <Signup/>
            </div>
          </div>
        </section>
      </AuthProvider>
    }
}

#[component]
pub fn SignUpPage() -> impl IntoView {
    view! {
      <AuthProvider unprotected=true>
        <section class="  uk-section">
          <div class="uk-container uk-container-xsmall">
            <ul class="uk-child-width-expand" uk-tab>
              <a href="/signin" class="uk-button uk-button-text">
                "Login"
              </a>
              <li>
                <a href="#">"Sign up"</a>
              </li>
            </ul>
            <div class="uk-margin">
              // <Signin/>
              <Signup/>
            </div>
          </div>
        </section>
      </AuthProvider>
    }
}
