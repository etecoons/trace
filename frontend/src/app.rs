use crate::types::*;
use yew::prelude::*;

pub struct App {
    pub query: String,
    pub site_title: String,
    pub theme: String,
    pub language: Language,
    pub loading: bool,
    pub error: Option<String>,
    pub response: Option<LookupResponse>,
    pub toasts: Vec<Toast>,
    pub next_toast_id: usize,
    pub status_text: String,
    pub status_type: String,
    pub is_authenticated: bool,
    pub pin_required: bool,
    pub pin_length: usize,
    pub pin_input: String,
    pub error_message: Option<String>,
    pub enable_translation: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self::create_app(ctx)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.update_app(ctx, msg)
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        self.rendered_app(ctx, first_render)
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.view_app(ctx)
    }
}
