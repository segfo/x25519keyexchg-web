#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Initialize)]
    fn init();
}

use std::{borrow::Borrow, collections::HashMap, sync::Mutex};

use gloo::console::{console, console_dbg};
use gloo_storage::Storage;
use once_cell::sync::{Lazy, OnceCell};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, EventTarget, HtmlButtonElement, HtmlElement, HtmlInputElement};
use yew::*;

#[wasm_bindgen(start)]
pub fn rust_entry() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}

#[derive(PartialEq, Properties)]
pub struct NavigateProps {}

#[derive(PartialEq)]
pub struct NavigateState {
    selected_tab: AttrValue,
}
impl NavigateState {
    fn new() -> Self {
        NavigateState {
            selected_tab: "".to_owned().into(),
        }
    }
}
#[function_component]
pub fn Navigate(props: &NavigateProps) -> Html {
    let NavigateProps {} = props;
    let state = use_state_eq(|| NavigateState::new());
    let click = {
        let clone_state = state.clone();
        Callback::from(move |e: MouseEvent| {
            if let Some(target) = e.target().and_then(|t| t.dyn_into::<HtmlElement>().ok()) {
                let enabled = if let Some(parent) = target.parent_element() {
                    let name = parent.class_name();
                    if name.split_ascii_whitespace().find(|s| s == &"disabled") != None {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                };
                if enabled {
                    if let Some(attribute) = target.get_attribute("href") {
                        clone_state.set(NavigateState {
                            selected_tab: attribute.clone().into(),
                        });
                        gloo_storage::LocalStorage::set("selected_tab", attribute);
                    }
                }
            }
        })
    };
    let load = state.clone();
    use_effect_with((), move |_| {
        if let Ok(selected_tab) = gloo_storage::LocalStorage::get("selected_tab") {
            let selected_tab: String = selected_tab;
            load.set(NavigateState {
                selected_tab: selected_tab.into(),
            });
        }
    });
    html! {
          <>
          <nav class="nav-extended light-blue darken-1 ">
      <div class="nav-wrapper">
        <a href="#" class="brand-logo">{"ファイル用事前鍵交換サイト"}</a>
        <ul id="nav-mobile" class="right hide-on-med-and-down">
        //   <li><a href="sass.html">{"hogehoge"}</a></li>
        </ul>
      </div>
    </nav>
              // <nav class="nav-extended grey darken-4">
              //   <div class="nav-content">
              //       <ul class="tabs tabs-transparent">
              //       <li class="tab"><a href="#test1" onclick={click.clone()}>{"Test 1"}</a></li>
              //       </ul>
              //   </div>
              // </nav>
          </>
      }
}
#[derive(Default, PartialEq, Properties)]
pub struct AppProps {}
use yew::Callback;

use base64::prelude::*;
use rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};
fn gen_public_key() -> String {
    let pubkey = PublicKey::from(&*SECRET);
    BASE64_URL_SAFE_NO_PAD.encode(pubkey.as_bytes())
}

fn gen_shared_key(pubkey: &str) -> String {
    if let Ok(pubkey) = BASE64_URL_SAFE_NO_PAD.decode(pubkey) {
        if pubkey.len() == 32 {
            let pubkey: [u8; 32] = pubkey.into_iter().collect::<Vec<u8>>().try_into().unwrap();
            let pubkey = PublicKey::from(pubkey);

            let sharedkey = SECRET.diffie_hellman(&pubkey);
            BASE64_URL_SAFE_NO_PAD.encode(sharedkey)
        } else {
            return "入力された値は正しいx25519鍵交換用の公開鍵ではありません。".to_owned();
        }
    } else {
        return "入力された値は正しい公開鍵ではありません。（エンコードがBase64-NOPAD形式ではありません）".to_owned();
    }
}

static SECRET: Lazy<StaticSecret> = Lazy::new(|| StaticSecret::random_from_rng(OsRng));

/// 入出力部分
///
use yew_hooks::prelude::*;
#[function_component]
pub fn KeyExchange(props: &AppProps) -> Html {
    let clipboard = use_clipboard();
    let input_pubkey = {
        Callback::from(|_| {
            let document = web_sys::window().unwrap().document().unwrap();
            let shared_key: HtmlElement = document
                .get_element_by_id("shared_key")
                .unwrap()
                .dyn_into()
                .unwrap();
            let shared_pubkey: HtmlInputElement = document
                .get_element_by_id("shared_pubkey")
                .unwrap()
                .dyn_into()
                .unwrap();
            let copy_button: HtmlButtonElement = document
                .get_element_by_id("copy_shared_key")
                .unwrap()
                .dyn_into()
                .unwrap();
            shared_key.set_inner_text(&gen_shared_key(&shared_pubkey.value()));
            copy_button.set_disabled(false);
        })
    };
    let save_clipboard = {
        let clipboard = clipboard.clone();
        Callback::from(move |_| {
            let shared_key: HtmlElement = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("shared_key")
                .unwrap()
                .dyn_into()
                .unwrap();
            clipboard.write_text(shared_key.inner_text());
        })
    };
    let generate_pubkey = {
        let clipboard = clipboard.clone();
        Callback::from(move |_| {
            let document = web_sys::window().unwrap().document().unwrap();
            let pubkey: HtmlInputElement = document
                .get_element_by_id("pubkey")
                .unwrap()
                .dyn_into()
                .unwrap();
            let query = QUERY_STRING;
            let key = gen_public_key();
            if let Some(_) = query.get("copy_url_pubkey") {
                clipboard.write_text(format!("{}?pubkey={}", &*BASE_URL, &key));
            } else {
                clipboard.write_text(key.clone());
            }
            if let Some(return_key) = document.get_element_by_id("return_key") {
                let return_key: HtmlElement = return_key.dyn_into().unwrap();
                return_key.set_inner_text(&key);
            }
            pubkey.set_value(&key);
        })
    };

    html! {
        <>
        <div class="input-field">
            <input id="pubkey" type="text" placeholder="ここをクリックすると公開鍵が生成され、クリップボードにコピーされます。" readonly=true onclick={generate_pubkey.clone()} />
            <label for="pubkey">{"送付用の鍵（生成済みの場合はクリックするとコピーされます。先方に送ってください）"}</label>
        </div>
        <div class="input-field">
            <input id="shared_pubkey" type="text" placeholder="先方から共有された公開鍵を入力してください" onkeyup={input_pubkey.clone()} />
            <label for="shared_pubkey">{"先方から送られた送付用の鍵"}</label>
        </div>
        <label class="shared_key_message_label">{"先方と共有したファイル開封用の鍵は以下の通りです"}</label><br/>
        <label id="shared_key" class="shared_key"></label><br/>
        <button id="copy_shared_key" class="waves-effect waves-light btn" disabled=true onclick={save_clipboard}>{"ファイル開封用秘密鍵をクリップボードにコピーする"}</button><br/>
        <KeyExchangeReceiver/>
        </>
    }
}

#[function_component]
pub fn KeyExchangeReceiver(props: &AppProps) -> Html {
    let clipboard = use_clipboard();
    let query = QUERY_STRING;
    let save_clipboard = {
        let clipboard = clipboard.clone();
        Callback::from(move |_| {
            let shared_key: HtmlElement = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("return_key")
                .unwrap()
                .dyn_into()
                .unwrap();
            clipboard.write_text(shared_key.inner_text());
        })
    };
    if query.contains_key("pubkey") {
        html! {
            <>
                <label class="shared_key_message_label">{"以下の値だけをURLの送信元に返送してください。"}</label><br/>
                <label id="return_key" class="shared_key"></label><br/>
                <button id="copy_return_key" class="waves-effect waves-light btn" onclick={save_clipboard}>{"送付用の鍵をクリップボードにコピーする"}</button><br/>
            </>
        }
    } else {
        html! {
            <></>
        }
    }
}

#[function_component]
pub fn App(props: &AppProps) -> Html {
    let AppProps {} = props;
    html! {
        <>
            <Navigate />
            <div id="test1" class="col s12">
                <KeyExchange />
            </div>
            <ScriptInit />
        </>
    }
}

#[derive(PartialEq, Properties)]
pub struct ScriptInitProps {}
impl ScriptInitProps {
    fn id(&self) -> u32 {
        0
    }
}
fn get_query() -> HashMap<String, String> {
    let location = gloo::utils::document().location().unwrap();
    let url = url::Url::parse(&location.href().unwrap()).unwrap();
    url.query_pairs().into_owned().collect()
}

const QUERY_STRING: Lazy<HashMap<String, String>> = Lazy::new(|| get_query());
const BASE_URL: Lazy<String> = Lazy::new(|| {
    let location = gloo::utils::document().location().unwrap().href().unwrap();
    let url = url::Url::parse(&location).unwrap();
    let port = if let Some(port) = url.port() {
        format!(":{}", port)
    } else {
        "".to_owned()
    };
    format!("{}://{}{}", url.scheme(), url.host_str().unwrap(), port)
});

#[function_component]
pub fn ScriptInit(props: &ScriptInitProps) -> Html {
    let ScriptInitProps {} = props;
    let sip = ScriptInitProps {};
    use_effect_with(sip.id(), move |_| {
        let query = QUERY_STRING;
        if let Some(pubkey) = query.get("pubkey") {
            let document = web_sys::window().unwrap().document().unwrap();
            let shared_pubkey: HtmlInputElement = document
                .get_element_by_id("shared_pubkey")
                .unwrap()
                .dyn_into()
                .unwrap();
            shared_pubkey.set_value(pubkey);
        }
        init();
    });
    html! {}
}
