use leptos::*;

#[component]
pub fn AuthFormFields(legend: &'static str, button: &'static str) -> impl IntoView {
    #![cfg_attr(feature = "ssr", allow(unused))]
    let email_input = create_node_ref::<html::Input>();
    let password_input = create_node_ref::<html::Input>();

    view! {
        <div class="uk-flex uk-flex-center">
            <div class="uk-card uk-card-body uk-flex-column uk-text-center">
                <fieldset class="uk-fieldset uk-form-horizontal">
                    <legend class="uk-legend">{legend}</legend>
                    <div class="uk-margin">
                        <label for="email" class="uk-form-label uk-margin-top">
                            "Email"
                        </label>
                        <div class="uk-inline uk-margin-left uk-box-shadow-small">
                            <span uk-icon="mail" class="uk-form-icon uk-text-middle"></span>
                            <input
                                class="uk-input uk-form-blank"
                                node_ref=email_input
                                name="email"
                                type="email"
                                placeholder="user@email.com"
                                aria-label="Not clickable icon"
                                required
                            />
                        </div>
                    </div>
                    <div class="uk-margin">
                        <label for="password" class="uk-form-label uk-margin-top">
                            "Password"
                        </label>
                        <div class="uk-inline uk-margin-left uk-box-shadow-small">
                            <span uk-icon="lock" class="uk-form-icon uk-text-middle"></span>
                            <input
                                class="uk-input uk-form-blank"
                                node_ref=password_input
                                name="password"
                                type="password"
                                placeholder="**********"
                                aria-label="Not clickable icon"
                                minlength="8"
                                required
                            />
                        </div>
                    </div>
                    <button
                        type="submit"
                        class="uk-button uk-button-default uk-dark uk-margin-top uk-box-shadow-small uk-inline"
                    >
                        <span uk-icon="sign-in" class="uk-form-icon uk-form-icon-flip"></span>
                        {button}
                    </button>
                </fieldset>
            </div>
        </div>
    }
}
