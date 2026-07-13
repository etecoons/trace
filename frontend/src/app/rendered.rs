use crate::app::App;
use crate::types::Msg;
use crate::utils::get_query_param;
use yew::prelude::*;

impl App {
    pub fn rendered_app(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(q) = get_query_param("lookup") {
                ctx.link().send_message(Msg::UpdateQuery(q));
                ctx.link().send_message(Msg::PerformLookup);
            }

            use wasm_bindgen::JsCast;
            let window = web_sys::window().unwrap();

            let link_online = ctx.link().clone();
            let on_online =
                wasm_bindgen::prelude::Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                    link_online.send_message(Msg::OnlineStatusChanged(true));
                });
            window
                .add_event_listener_with_callback("online", on_online.as_ref().unchecked_ref())
                .unwrap();
            on_online.forget();

            let link_offline = ctx.link().clone();
            let on_offline =
                wasm_bindgen::prelude::Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                    link_offline.send_message(Msg::OnlineStatusChanged(false));
                });
            window
                .add_event_listener_with_callback("offline", on_offline.as_ref().unchecked_ref())
                .unwrap();
            on_offline.forget();
        }
    }
}
