//! Shared notifier/toast component.

use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
}

impl ToastType {
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ToastNotificationProps {
    pub message: String,
    pub toast_type: ToastType,
    #[prop_or_default]
    pub on_dismiss: Callback<()>,
}

#[function_component(ToastNotification)]
pub fn toast_notification(props: &ToastNotificationProps) -> Html {
    let onclick = {
        let on_dismiss = props.on_dismiss.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            on_dismiss.emit(());
        })
    };
    html! {
        <div class={format!("toast show {}", props.toast_type.as_class())} onclick={onclick}>
            { &props.message }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToastContainerProps {
    pub children: Children,
}

#[function_component(ToastContainer)]
pub fn toast_container(props: &ToastContainerProps) -> Html {
    html! {
        <div class="toast-container">
            { for props.children.iter() }
        </div>
    }
}
