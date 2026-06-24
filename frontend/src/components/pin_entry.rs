use crate::i18n::Translations;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PinProps {
    pub translations: Translations,
    pub pin_length: usize,
    pub pin_input: String,
    pub error_message: Option<String>,
    pub on_input_change: Callback<String>,
    pub on_submit: Callback<()>,
}

#[function_component(PinEntry)]
pub fn pin_entry(props: &PinProps) -> Html {
    let tr = &props.translations;
    let pin_len = props.pin_length;
    let pin_input = &props.pin_input;
    let error_message = &props.error_message;
    let on_input_change = &props.on_input_change;
    let on_submit = &props.on_submit;

    let oninput = {
        let cb = on_input_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            cb.emit(input.value());
        })
    };

    let onsubmit = {
        let cb = on_submit.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            cb.emit(());
        })
    };

    html! {
        <div class="login-container">
            <div class="login-box">
                <div class="pin-header">
                    <h2 id="pin-description">
                        {tr.enter_pin}
                    </h2>
                </div>
                <form id="pin-form" {onsubmit}>
                    <div class="pin-wrapper">
                        <input
                            type="password"
                            class="pin-input-field"
                            value={pin_input.clone()}
                            {oninput}
                            placeholder={"• ".repeat(pin_len).trim().to_string()}
                            maxlength={pin_len.to_string()}
                            autofocus=true
                        />
                    </div>
                </form>
                <div class="pin-status">
                    if let Some(err) = error_message {
                        <p id="pin-error" class="pin-error" style="display: block;">
                            {if err == "Invalid PIN" { tr.invalid_pin.to_string() } else { err.clone() }}
                        </p>
                    }
                </div>
            </div>
        </div>
    }
}
